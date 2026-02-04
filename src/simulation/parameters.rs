//! Simulation configuration and parameter definitions.

use serde::{Deserialize, Serialize};

/// Supported distribution types for stochastic parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DistributionType {
    Fixed,
    Normal,
    Uniform,
    LogNormal,
    Triangular,
}

impl Default for DistributionType {
    fn default() -> Self {
        Self::Normal
    }
}

/// Random input with a distribution and bounds.
///
/// All values are in the same units as the parameter itself.
/// Rates are expressed as decimals (e.g., 0.05 = 5%).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticParameter {
    pub name: String,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub distribution: DistributionType,
}

impl StochasticParameter {
    pub fn new(name: &str, mean: f64, std_dev: f64, min: f64, max: f64) -> Self {
        Self {
            name: name.to_string(),
            mean,
            std_dev,
            min,
            max,
            distribution: DistributionType::Normal,
        }
    }

    pub fn fixed(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            mean: value,
            std_dev: 0.0,
            min: value,
            max: value,
            distribution: DistributionType::Fixed,
        }
    }
}

/// Deterministic inputs used for every simulation.
///
/// - Rates are decimals (e.g., 0.065 = 6.5%).
/// - Monetary values are in USD.
/// - Monthly/annual values are indicated in the field name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseParameters {
    pub house_value: f64,
    pub down_payment_percent: f64,
    pub interest_rate: f64,
    pub loan_term_months: u32,
    pub hoa_monthly: f64,
    pub property_tax_rate: f64,
    pub insurance_annual: f64,
    pub pmi_rate: f64,
    pub repair_rate: f64,
    pub rent_equivalent: f64,
    /// Closing costs when buying (as fraction of purchase price)
    pub buying_closing_cost_rate: f64,
    /// Costs when selling (agent commission + closing, as fraction of sale price)
    pub selling_cost_rate: f64,
}

impl Default for BaseParameters {
    fn default() -> Self {
        Self {
            house_value: 925_000.0,
            down_payment_percent: 0.20,
            interest_rate: 0.065,
            loan_term_months: 360,
            hoa_monthly: 500.0,
            property_tax_rate: 0.0125,
            insurance_annual: 2400.0,
            pmi_rate: 0.005,
            repair_rate: 0.01,
            rent_equivalent: 3500.0,
            buying_closing_cost_rate: 0.03,  // 3% of purchase price
            selling_cost_rate: 0.07,          // 7% of sale price (6% agent + 1% other)
        }
    }
}

impl BaseParameters {
    pub fn down_payment(&self) -> f64 {
        self.house_value * self.down_payment_percent
    }

    pub fn loan_amount(&self) -> f64 {
        self.house_value - self.down_payment()
    }

    pub fn requires_pmi(&self) -> bool {
        self.down_payment_percent < 0.20
    }
}

/// Grouped stochastic parameters used by the simulation engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticParameters {
    pub home_appreciation: StochasticParameter,
    pub repair_shock: StochasticParameter,
    pub property_tax_increase: StochasticParameter,
    pub insurance_increase: StochasticParameter,
}

impl Default for StochasticParameters {
    fn default() -> Self {
        Self {
            home_appreciation: StochasticParameter::new(
                "Home Appreciation Rate",
                0.03,
                0.05,
                -0.20,
                0.30,
            ),
            repair_shock: StochasticParameter {
                name: "Repair Cost Shock".to_string(),
                mean: 0.0,
                std_dev: 5000.0,
                min: 0.0,
                max: 50000.0,
                distribution: DistributionType::LogNormal,
            },
            property_tax_increase: StochasticParameter::new(
                "Property Tax Increase",
                0.02,
                0.01,
                0.0,
                0.10,
            ),
            insurance_increase: StochasticParameter::new(
                "Insurance Increase",
                0.03,
                0.02,
                0.0,
                0.15,
            ),
        }
    }
}

/// Top-level simulation configuration.
///
/// `holding_period_years` is how long the property is held before selling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub base: BaseParameters,
    pub stochastic: StochasticParameters,
    pub num_simulations: usize,
    pub random_seed: Option<u64>,
    /// Number of years to hold the property before selling
    pub holding_period_years: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            base: BaseParameters::default(),
            stochastic: StochasticParameters::default(),
            num_simulations: 10_000,
            random_seed: None,
            holding_period_years: 10,
        }
    }
}

impl SimulationConfig {
    pub fn holding_period_months(&self) -> u32 {
        self.holding_period_years * 12
    }
}
