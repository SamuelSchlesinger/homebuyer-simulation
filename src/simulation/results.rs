//! Aggregated simulation results and summary statistics.

use serde::{Deserialize, Serialize};

/// Monthly snapshot of a single simulation.
#[derive(Debug, Clone, Default)]
pub struct MonthlySnapshot {
    pub month: u32,
    pub equity: f64,
    pub cumulative_cost: f64,
}

/// Results from one simulation run.
#[derive(Debug, Clone)]
pub struct SingleSimulationResult {
    pub final_equity: f64,
    pub total_payments: f64,
    /// Effective cost = total payments - final equity (what you actually "lost")
    pub effective_cost: f64,
    /// Effective monthly cost = effective_cost / months
    pub effective_monthly_cost: f64,
    pub holding_months: u32,
    pub snapshots: Vec<MonthlySnapshot>,
}

impl SingleSimulationResult {
    pub fn new(final_equity: f64, total_payments: f64, holding_months: u32, snapshots: Vec<MonthlySnapshot>) -> Self {
        let effective_cost = total_payments - final_equity;
        let effective_monthly_cost = effective_cost / holding_months as f64;
        Self {
            final_equity,
            total_payments,
            effective_cost,
            effective_monthly_cost,
            holding_months,
            snapshots,
        }
    }
}

/// Percentile statistics for a distribution of values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PercentileStats {
    pub p5: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p95: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl PercentileStats {
    pub fn from_values(values: &mut [f64]) -> Self {
        if values.is_empty() {
            return Self::default();
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = values.len();
        let mean = values.iter().sum::<f64>() / n as f64;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;

        Self {
            p5: percentile(values, 0.05),
            p25: percentile(values, 0.25),
            p50: percentile(values, 0.50),
            p75: percentile(values, 0.75),
            p95: percentile(values, 0.95),
            mean,
            std_dev: variance.sqrt(),
            min: values[0],
            max: values[n - 1],
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let n = sorted_values.len();
    let idx = p * (n - 1) as f64;
    let lower = idx.floor() as usize;
    let upper = idx.ceil() as usize;
    let frac = idx - lower as f64;

    if lower == upper || upper >= n {
        sorted_values[lower.min(n - 1)]
    } else {
        sorted_values[lower] * (1.0 - frac) + sorted_values[upper] * frac
    }
}

/// Percentiles over time (for charts).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeSeriesPercentiles {
    pub months: Vec<u32>,
    pub p5: Vec<f64>,
    pub p25: Vec<f64>,
    pub p50: Vec<f64>,
    pub p75: Vec<f64>,
    pub p95: Vec<f64>,
}

impl TimeSeriesPercentiles {
    pub fn new(num_months: usize) -> Self {
        Self {
            months: (1..=num_months as u32).collect(),
            p5: Vec::with_capacity(num_months),
            p25: Vec::with_capacity(num_months),
            p50: Vec::with_capacity(num_months),
            p75: Vec::with_capacity(num_months),
            p95: Vec::with_capacity(num_months),
        }
    }
}

/// Aggregated results across all simulations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedResults {
    pub num_simulations: usize,
    pub holding_months: u32,
    /// Stats for effective monthly cost (the key metric)
    pub effective_monthly_stats: PercentileStats,
    /// Stats for total effective cost over holding period
    pub effective_cost_stats: PercentileStats,
    /// Stats for final equity
    pub equity_stats: PercentileStats,
    /// Equity over time percentiles
    pub equity_trajectory: TimeSeriesPercentiles,
    /// Effective cost over time percentiles
    pub effective_cost_trajectory: TimeSeriesPercentiles,
    /// Raw values for histograms
    pub effective_monthly_values: Vec<f64>,
    pub effective_cost_values: Vec<f64>,
    pub equity_values: Vec<f64>,
    #[serde(skip)]
    pub sample_trajectories: Vec<Vec<MonthlySnapshot>>,
}

impl AggregatedResults {
    pub fn from_simulations(
        results: Vec<SingleSimulationResult>,
        num_sample_trajectories: usize,
    ) -> Self {
        let num_simulations = results.len();
        if num_simulations == 0 {
            return Self::default();
        }

        let holding_months = results[0].holding_months;

        // Extract final values - focus on effective costs
        let effective_monthly_values: Vec<f64> = results.iter().map(|r| r.effective_monthly_cost).collect();
        let effective_cost_values: Vec<f64> = results.iter().map(|r| r.effective_cost).collect();
        let equity_values: Vec<f64> = results.iter().map(|r| r.final_equity).collect();

        let effective_monthly_stats = PercentileStats::from_values(&mut effective_monthly_values.clone());
        let effective_cost_stats = PercentileStats::from_values(&mut effective_cost_values.clone());
        let equity_stats = PercentileStats::from_values(&mut equity_values.clone());

        // Compute time series percentiles
        let num_months = results[0].snapshots.len();
        let mut equity_trajectory = TimeSeriesPercentiles::new(num_months);
        let mut effective_cost_trajectory = TimeSeriesPercentiles::new(num_months);

        for month_idx in 0..num_months {
            let mut month_equities: Vec<f64> = results
                .iter()
                .filter_map(|r| r.snapshots.get(month_idx).map(|s| s.equity))
                .collect();
            // Effective cost at each month = cumulative payments - equity at that point
            let mut month_effective_costs: Vec<f64> = results
                .iter()
                .filter_map(|r| r.snapshots.get(month_idx).map(|s| s.cumulative_cost - s.equity))
                .collect();

            let eq_stats = PercentileStats::from_values(&mut month_equities);
            equity_trajectory.p5.push(eq_stats.p5);
            equity_trajectory.p25.push(eq_stats.p25);
            equity_trajectory.p50.push(eq_stats.p50);
            equity_trajectory.p75.push(eq_stats.p75);
            equity_trajectory.p95.push(eq_stats.p95);

            let eff_stats = PercentileStats::from_values(&mut month_effective_costs);
            effective_cost_trajectory.p5.push(eff_stats.p5);
            effective_cost_trajectory.p25.push(eff_stats.p25);
            effective_cost_trajectory.p50.push(eff_stats.p50);
            effective_cost_trajectory.p75.push(eff_stats.p75);
            effective_cost_trajectory.p95.push(eff_stats.p95);
        }

        // Sample trajectories for visualization
        let step = if num_simulations > num_sample_trajectories {
            num_simulations / num_sample_trajectories
        } else {
            1
        };
        let sample_trajectories: Vec<Vec<MonthlySnapshot>> = results
            .iter()
            .step_by(step)
            .take(num_sample_trajectories)
            .map(|r| r.snapshots.clone())
            .collect();

        Self {
            num_simulations,
            holding_months,
            effective_monthly_stats,
            effective_cost_stats,
            equity_stats,
            equity_trajectory,
            effective_cost_trajectory,
            effective_monthly_values,
            effective_cost_values,
            equity_values,
            sample_trajectories,
        }
    }

    /// Probability that buying beats renting (effective cost < rent equivalent)
    pub fn buy_wins_probability(&self, monthly_rent: f64) -> f64 {
        let rent_total = monthly_rent * self.holding_months as f64;
        let count = self
            .effective_cost_values
            .iter()
            .filter(|&&eff| eff < rent_total)
            .count();
        count as f64 / self.num_simulations as f64
    }
}

/// Simple comparison: effective monthly cost of buying vs renting
///
/// Positive savings means buying is cheaper than renting.
#[derive(Debug, Clone, Default)]
pub struct RentComparison {
    /// Monthly rent
    pub monthly_rent: f64,
    /// Effective monthly cost of buying (median)
    pub buy_monthly_p50: f64,
    /// Effective monthly cost of buying (5th percentile - good outcome)
    pub buy_monthly_p5: f64,
    /// Effective monthly cost of buying (95th percentile - bad outcome)
    pub buy_monthly_p95: f64,
    /// Probability that buying is cheaper than renting
    pub buy_wins_probability: f64,
    /// Median monthly savings from buying (positive = buying saves money)
    pub monthly_savings_p50: f64,
}

impl RentComparison {
    pub fn calculate(results: &AggregatedResults, monthly_rent: f64) -> Self {
        let buy_monthly_p5 = results.effective_monthly_stats.p5;
        let buy_monthly_p50 = results.effective_monthly_stats.p50;
        let buy_monthly_p95 = results.effective_monthly_stats.p95;

        // Probability buying beats renting
        let count = results
            .effective_monthly_values
            .iter()
            .filter(|&&eff| eff < monthly_rent)
            .count();
        let buy_wins_probability = count as f64 / results.num_simulations as f64;

        let monthly_savings_p50 = monthly_rent - buy_monthly_p50;

        Self {
            monthly_rent,
            buy_monthly_p50,
            buy_monthly_p5,
            buy_monthly_p95,
            buy_wins_probability,
            monthly_savings_p50,
        }
    }
}
