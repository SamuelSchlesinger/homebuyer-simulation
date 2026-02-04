//! Monte Carlo simulation engine.

use super::mortgage::MortgageCalculator;
use super::parameters::{DistributionType, SimulationConfig, StochasticParameter};
use super::results::{MonthlySnapshot, SingleSimulationResult};
use rand::prelude::*;
use rand_distr::{Distribution, LogNormal, Normal, Triangular, Uniform};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Runs Monte Carlo simulations for a given configuration.
pub struct SimulationEngine {
    config: SimulationConfig,
}

impl SimulationEngine {
    pub fn new(config: SimulationConfig) -> Self {
        Self { config }
    }

    /// Run all simulations in parallel and return per-simulation results.
    ///
    /// `progress_callback` receives (done, total) counts for UI updates.
    pub fn run_parallel<F>(&self, progress_callback: F) -> Vec<SingleSimulationResult>
    where
        F: Fn(usize, usize) + Send + Sync,
    {
        let total = self.config.num_simulations;
        let completed = Arc::new(AtomicUsize::new(0));
        let callback = Arc::new(progress_callback);

        let results: Vec<SingleSimulationResult> = (0..total)
            .into_par_iter()
            .map(|i| {
                let seed = self.config.random_seed.unwrap_or(i as u64 * 31337) + i as u64;
                let result = self.run_single(seed);

                let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if done % 100 == 0 || done == total {
                    callback(done, total);
                }

                result
            })
            .collect();

        results
    }

    /// Run a single simulation with a deterministic RNG seed.
    pub fn run_single(&self, seed: u64) -> SingleSimulationResult {
        let mut rng = StdRng::seed_from_u64(seed);

        // Sample stochastic parameters
        let appreciation_rate =
            sample_parameter(&self.config.stochastic.home_appreciation, &mut rng);
        let tax_increase_rate =
            sample_parameter(&self.config.stochastic.property_tax_increase, &mut rng);
        let insurance_increase_rate =
            sample_parameter(&self.config.stochastic.insurance_increase, &mut rng);

        let base = &self.config.base;
        let mortgage = MortgageCalculator::from_params(base);
        let holding_months = self.config.holding_period_months();

        let mut snapshots = Vec::with_capacity(holding_months as usize);
        let mut home_value = base.house_value;

        // Initial costs: down payment + buying closing costs
        let buying_closing_costs = base.house_value * base.buying_closing_cost_rate;
        let mut cumulative_cost = base.down_payment() + buying_closing_costs;

        let mut current_property_tax_rate = base.property_tax_rate;
        let mut current_insurance_annual = base.insurance_annual;

        let monthly_appreciation = (1.0 + appreciation_rate).powf(1.0 / 12.0) - 1.0;

        for month in 1..=holding_months {
            // Update home value
            home_value *= 1.0 + monthly_appreciation;

            // Calculate loan balance (0 if past loan term)
            let loan_balance = if month <= base.loan_term_months {
                mortgage.balance_at_month(month)
            } else {
                0.0
            };

            // Mortgage payment (zero after loan is paid off)
            let mortgage_payment = if month <= base.loan_term_months {
                mortgage.monthly_payment
            } else {
                0.0
            };

            // Property tax (increases annually)
            if month > 1 && month % 12 == 1 {
                current_property_tax_rate *= 1.0 + tax_increase_rate;
                current_insurance_annual *= 1.0 + insurance_increase_rate;
            }
            let property_tax_monthly = home_value * current_property_tax_rate / 12.0;
            let insurance_monthly = current_insurance_annual / 12.0;

            // PMI (if applicable)
            let ltv = loan_balance / home_value;
            let pmi_monthly = if ltv > 0.80 {
                loan_balance * base.pmi_rate / 12.0
            } else {
                0.0
            };

            // Repairs (base + potential shock)
            let base_repairs = home_value * base.repair_rate / 12.0;
            let repair_shock = if rng.random::<f64>() < 0.02 {
                // 2% chance per month of a major repair shock (one-time cost)
                sample_parameter(&self.config.stochastic.repair_shock, &mut rng)
            } else {
                0.0
            };
            // Shock is a one-time cost, not amortized
            let repairs_monthly = base_repairs + repair_shock;

            // HOA
            let hoa = base.hoa_monthly;

            // Total monthly cost (all out-of-pocket payments)
            let total_monthly = mortgage_payment
                + property_tax_monthly
                + insurance_monthly
                + pmi_monthly
                + repairs_monthly
                + hoa;

            cumulative_cost += total_monthly;

            // Net equity = what you'd get if you sold now
            // = home value - loan balance - selling costs
            let selling_costs = home_value * base.selling_cost_rate;
            let equity = home_value - loan_balance - selling_costs;

            snapshots.push(MonthlySnapshot {
                month,
                equity,
                cumulative_cost,
            });
        }

        let final_snapshot = snapshots.last().unwrap();
        SingleSimulationResult::new(
            final_snapshot.equity,
            final_snapshot.cumulative_cost,
            holding_months,
            snapshots,
        )
    }
}

fn sample_parameter<R: RngCore>(param: &StochasticParameter, rng: &mut R) -> f64 {
    let raw_value = match param.distribution {
        DistributionType::Fixed => param.mean,
        DistributionType::Normal => {
            let dist = Normal::new(param.mean, param.std_dev).unwrap_or_else(|_| Normal::new(0.0, 1.0).unwrap());
            dist.sample(rng)
        }
        DistributionType::Uniform => {
            let dist = Uniform::new(param.min, param.max).unwrap();
            dist.sample(rng)
        }
        DistributionType::LogNormal => {
            if param.mean <= 0.0 || param.std_dev <= 0.0 {
                return param.mean.max(0.0);
            }
            // Convert mean/std to log-normal parameters
            let variance = param.std_dev.powi(2);
            let mu = (param.mean.powi(2) / (variance + param.mean.powi(2)).sqrt()).ln();
            let sigma = (1.0 + variance / param.mean.powi(2)).ln().sqrt();
            let dist = LogNormal::new(mu, sigma).unwrap_or_else(|_| LogNormal::new(0.0, 1.0).unwrap());
            dist.sample(rng)
        }
        DistributionType::Triangular => {
            let mode = param.mean;
            let dist = Triangular::new(param.min, param.max, mode)
                .unwrap_or_else(|_| Triangular::new(0.0, 1.0, 0.5).unwrap());
            dist.sample(rng)
        }
    };

    // Clamp to min/max
    raw_value.clamp(param.min, param.max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::parameters::SimulationConfig;

    #[test]
    fn test_single_simulation() {
        let config = SimulationConfig::default();
        let holding_months = config.holding_period_months();
        let engine = SimulationEngine::new(config);
        let result = engine.run_single(42);

        assert!(result.final_equity > 0.0);
        assert!(result.total_payments > 0.0);
        assert!(result.effective_monthly_cost > 0.0);
        assert_eq!(result.snapshots.len(), holding_months as usize);
    }

    #[test]
    fn test_parallel_simulation() {
        let mut config = SimulationConfig::default();
        config.num_simulations = 100;
        config.random_seed = Some(42);

        let engine = SimulationEngine::new(config);
        let results = engine.run_parallel(|_, _| {});

        assert_eq!(results.len(), 100);
    }
}
