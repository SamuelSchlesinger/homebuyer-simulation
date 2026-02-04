//! Mortgage payment and amortization helpers.

use super::parameters::BaseParameters;

/// Computes mortgage payments and balances for a fixed-rate loan.
#[derive(Debug, Clone)]
pub struct MortgageCalculator {
    pub principal: f64,
    pub monthly_rate: f64,
    pub term_months: u32,
    pub monthly_payment: f64,
}

impl MortgageCalculator {
    /// Create a calculator from principal, annual rate (decimal), and term in months.
    pub fn new(principal: f64, annual_rate: f64, term_months: u32) -> Self {
        let monthly_rate = annual_rate / 12.0;
        let monthly_payment = Self::calculate_monthly_payment(principal, monthly_rate, term_months);

        Self {
            principal,
            monthly_rate,
            term_months,
            monthly_payment,
        }
    }

    /// Create a calculator from base parameters.
    pub fn from_params(params: &BaseParameters) -> Self {
        Self::new(
            params.loan_amount(),
            params.interest_rate,
            params.loan_term_months,
        )
    }

    fn calculate_monthly_payment(principal: f64, monthly_rate: f64, term_months: u32) -> f64 {
        if monthly_rate == 0.0 {
            return principal / term_months as f64;
        }

        let n = term_months as f64;
        let r = monthly_rate;
        principal * (r * (1.0 + r).powf(n)) / ((1.0 + r).powf(n) - 1.0)
    }

    /// Full amortization schedule (month-by-month principal/interest).
    pub fn amortization_schedule(&self) -> Vec<AmortizationEntry> {
        let mut schedule = Vec::with_capacity(self.term_months as usize);
        let mut balance = self.principal;

        for month in 1..=self.term_months {
            let interest_payment = balance * self.monthly_rate;
            let principal_payment = self.monthly_payment - interest_payment;
            balance -= principal_payment;

            // Handle floating point precision issues near the end
            if balance < 0.01 {
                balance = 0.0;
            }

            schedule.push(AmortizationEntry {
                month,
                payment: self.monthly_payment,
                principal: principal_payment,
                interest: interest_payment,
                remaining_balance: balance,
            });
        }

        schedule
    }

    /// Remaining balance after `month` payments.
    pub fn balance_at_month(&self, month: u32) -> f64 {
        if month >= self.term_months {
            return 0.0;
        }

        let n = self.term_months as f64;
        let m = month as f64;
        let r = self.monthly_rate;

        if r == 0.0 {
            return self.principal * (1.0 - m / n);
        }

        self.principal * ((1.0 + r).powf(n) - (1.0 + r).powf(m)) / ((1.0 + r).powf(n) - 1.0)
    }

    /// Total interest paid through `month`.
    pub fn interest_paid_to_month(&self, month: u32) -> f64 {
        let payments_made = self.monthly_payment * month as f64;
        let principal_paid = self.principal - self.balance_at_month(month);
        payments_made - principal_paid
    }

    /// Total principal paid through `month`.
    pub fn principal_paid_to_month(&self, month: u32) -> f64 {
        self.principal - self.balance_at_month(month)
    }
}

/// Amortization record for a single month.
#[derive(Debug, Clone)]
pub struct AmortizationEntry {
    pub month: u32,
    pub payment: f64,
    pub principal: f64,
    pub interest: f64,
    pub remaining_balance: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monthly_payment() {
        // $200,000 loan at 6% for 30 years should be about $1,199.10
        let calc = MortgageCalculator::new(200_000.0, 0.06, 360);
        assert!((calc.monthly_payment - 1199.10).abs() < 1.0);
    }

    #[test]
    fn test_amortization_balance() {
        let calc = MortgageCalculator::new(200_000.0, 0.06, 360);
        let schedule = calc.amortization_schedule();

        // Final balance should be zero
        assert!(schedule.last().unwrap().remaining_balance < 0.01);

        // Total payments should equal principal + interest
        let total_payments: f64 = schedule.iter().map(|e| e.payment).sum();
        let total_interest: f64 = schedule.iter().map(|e| e.interest).sum();
        let total_principal: f64 = schedule.iter().map(|e| e.principal).sum();

        assert!((total_principal - 200_000.0).abs() < 1.0);
        assert!((total_payments - total_principal - total_interest).abs() < 1.0);
    }
}
