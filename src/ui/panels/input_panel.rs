use crate::simulation::{DistributionType, MortgageCalculator, StochasticParameter};
use crate::ui::app_state::AppState;
use crate::ui::theme::{format_currency, format_currency_full, format_percent};
use bevy_egui::egui::{self, ComboBox, Grid, RichText, Ui};

pub fn draw_input_panel(ui: &mut Ui, state: &mut AppState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Set Up Your Scenario");
        ui.label("Start with the basics, then refine costs and uncertainty.");
        ui.add_space(8.0);

        section(
            ui,
            "Scenario Basics",
            "These are the minimum inputs for a clear buy vs. rent comparison.",
            |ui| draw_basics(ui, state),
        );

        ui.add_space(12.0);
        section(
            ui,
            "Monthly Ownership Costs",
            "Ongoing costs you pay while owning the home.",
            |ui| draw_monthly_costs(ui, state),
        );

        ui.add_space(12.0);
        section(
            ui,
            "Transaction Costs",
            "One-time costs at purchase and sale.",
            |ui| draw_transaction_costs(ui, state),
        );

        ui.add_space(12.0);
        section(
            ui,
            "Baseline Estimates",
            "Quick sanity-check based on your inputs (no randomness).",
            |ui| draw_estimates(ui, state),
        );

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        ui.heading("Uncertainty & Market Assumptions");
        ui.label("Only adjust these if you want to stress-test outcomes.");
        ui.add_space(8.0);

        ui.collapsing("Market uncertainty settings", |ui| {
            draw_stochastic_parameters(ui, state);
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        ui.heading("Simulation Settings");
        ui.label("Controls how many scenarios are simulated.");
        ui.add_space(8.0);

        draw_simulation_settings(ui, state);
    });
}

fn section<F: FnOnce(&mut Ui)>(ui: &mut Ui, title: &str, subtitle: &str, contents: F) {
    ui.label(RichText::new(title).strong());
    if !subtitle.is_empty() {
        ui.label(RichText::new(subtitle).small().weak());
    }
    ui.add_space(4.0);
    ui.group(|ui| {
        ui.add_space(4.0);
        contents(ui);
        ui.add_space(4.0);
    });
}

fn draw_basics(ui: &mut Ui, state: &mut AppState) {
    Grid::new("basics_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            let base = &mut state.config.base;

            ui.label("Home price ($):")
                .on_hover_text("The purchase price of the home.");
            ui.add(
                egui::DragValue::new(&mut base.house_value)
                    .speed(1000.0)
                    .range(100_000.0..=5_000_000.0)
                    .prefix("$"),
            );
            ui.end_row();

            ui.label("Down payment (%):")
                .on_hover_text("Percent of the home price you will pay upfront.");
            let mut down_pct = base.down_payment_percent * 100.0;
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut down_pct, 3.0..=50.0)
                        .suffix("%")
                        .fixed_decimals(1),
                );
                let down_payment = base.house_value * (down_pct / 100.0);
                ui.label(format_currency_full(down_payment));
            });
            base.down_payment_percent = down_pct / 100.0;
            ui.end_row();

            ui.label("Mortgage rate (APR %):")
                .on_hover_text("Annual interest rate for your mortgage.");
            let mut rate_pct = base.interest_rate * 100.0;
            ui.add(
                egui::Slider::new(&mut rate_pct, 1.0..=15.0)
                    .suffix("%")
                    .fixed_decimals(2),
            );
            base.interest_rate = rate_pct / 100.0;
            ui.end_row();

            ui.label("Loan term (years):");
            let mut term_years = (base.loan_term_months / 12).max(1);
            ui.add(egui::Slider::new(&mut term_years, 5..=40).suffix(" years"));
            base.loan_term_months = term_years * 12;
            ui.end_row();

            ui.label("Holding period (years):")
                .on_hover_text("How long you expect to own the home before selling.");
            ui.add(
                egui::Slider::new(&mut state.config.holding_period_years, 1..=40)
                    .suffix(" years"),
            );
            ui.end_row();

            ui.label("Rent equivalent (monthly $):")
                .on_hover_text("Your realistic alternative monthly rent for a comparable home.");
            ui.add(
                egui::DragValue::new(&mut base.rent_equivalent)
                    .speed(50.0)
                    .range(500.0..=20_000.0)
                    .prefix("$"),
            );
            ui.end_row();
        });
}

fn draw_monthly_costs(ui: &mut Ui, state: &mut AppState) {
    Grid::new("monthly_costs_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            let base = &mut state.config.base;

            ui.label("HOA (monthly $):");
            ui.add(
                egui::DragValue::new(&mut base.hoa_monthly)
                    .speed(10.0)
                    .range(0.0..=2000.0)
                    .prefix("$"),
            );
            ui.end_row();

            ui.label("Property tax rate (%/year):")
                .on_hover_text("Annual tax rate applied to the home value.");
            let mut tax_pct = base.property_tax_rate * 100.0;
            ui.add(
                egui::Slider::new(&mut tax_pct, 0.0..=5.0)
                    .suffix("%")
                    .fixed_decimals(3),
            );
            base.property_tax_rate = tax_pct / 100.0;
            ui.end_row();

            ui.label("Home insurance (annual $):");
            ui.add(
                egui::DragValue::new(&mut base.insurance_annual)
                    .speed(100.0)
                    .range(0.0..=20_000.0)
                    .prefix("$"),
            );
            ui.end_row();

            ui.label("PMI rate (%/year):")
                .on_hover_text("Private mortgage insurance rate, if down payment is under 20%.");
            let mut pmi_pct = base.pmi_rate * 100.0;
            ui.add(
                egui::Slider::new(&mut pmi_pct, 0.0..=2.0)
                    .suffix("%")
                    .fixed_decimals(3),
            );
            base.pmi_rate = pmi_pct / 100.0;
            ui.end_row();

            ui.label("Repairs & maintenance (%/year):")
                .on_hover_text("Ongoing maintenance as a percent of home value.");
            let mut repair_pct = base.repair_rate * 100.0;
            ui.add(
                egui::Slider::new(&mut repair_pct, 0.0..=5.0)
                    .suffix("%")
                    .fixed_decimals(2),
            );
            base.repair_rate = repair_pct / 100.0;
            ui.end_row();
        });
}

fn draw_transaction_costs(ui: &mut Ui, state: &mut AppState) {
    Grid::new("transaction_costs_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            let base = &mut state.config.base;

            ui.label("Buying closing costs (%):")
                .on_hover_text("Lender fees, title, escrow, and other one-time costs.");
            let mut buying_pct = base.buying_closing_cost_rate * 100.0;
            ui.add(
                egui::Slider::new(&mut buying_pct, 0.0..=6.0)
                    .suffix("%")
                    .fixed_decimals(1),
            );
            base.buying_closing_cost_rate = buying_pct / 100.0;
            ui.end_row();

            ui.label("Selling costs (%):")
                .on_hover_text("Agent commission and closing costs when you sell.");
            let mut selling_pct = base.selling_cost_rate * 100.0;
            ui.add(
                egui::Slider::new(&mut selling_pct, 0.0..=10.0)
                    .suffix("%")
                    .fixed_decimals(1),
            );
            base.selling_cost_rate = selling_pct / 100.0;
            ui.end_row();
        });
}

fn draw_estimates(ui: &mut Ui, state: &AppState) {
    let base = &state.config.base;
    let mortgage = MortgageCalculator::from_params(base);

    let property_tax_monthly = base.house_value * base.property_tax_rate / 12.0;
    let insurance_monthly = base.insurance_annual / 12.0;
    let repairs_monthly = base.house_value * base.repair_rate / 12.0;
    let pmi_monthly = if base.requires_pmi() {
        mortgage.principal * base.pmi_rate / 12.0
    } else {
        0.0
    };
    let all_in_monthly = mortgage.monthly_payment
        + property_tax_monthly
        + insurance_monthly
        + base.hoa_monthly
        + repairs_monthly
        + pmi_monthly;

    let cash_to_close = base.down_payment() + base.house_value * base.buying_closing_cost_rate;
    let ltv = if base.house_value > 0.0 {
        base.loan_amount() / base.house_value
    } else {
        0.0
    };

    Grid::new("baseline_estimates_grid")
        .num_columns(2)
        .spacing([20.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Est. mortgage payment (P&I):");
            ui.label(format_currency(mortgage.monthly_payment));
            ui.end_row();

            ui.label("Est. all-in monthly cost:");
            ui.label(format_currency(all_in_monthly));
            ui.end_row();

            ui.label("Upfront cash to close:");
            ui.label(format_currency_full(cash_to_close));
            ui.end_row();

            ui.label("Initial loan amount:");
            ui.label(format_currency_full(base.loan_amount()));
            ui.end_row();

            ui.label("Loan-to-value (LTV):");
            ui.label(format_percent(ltv));
            ui.end_row();

            ui.label("Estimated PMI (monthly):");
            if base.requires_pmi() {
                ui.label(format_currency(pmi_monthly));
            } else {
                ui.label("No PMI expected");
            }
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.label(RichText::new("These estimates use your current inputs before any random variation.").small().weak());
}

fn draw_stochastic_parameters(ui: &mut Ui, state: &mut AppState) {
    let stochastic = &mut state.config.stochastic;

    ui.collapsing("Home Appreciation", |ui| {
        draw_stochastic_param_editor(ui, "appreciation", &mut stochastic.home_appreciation);
    });

    ui.collapsing("Repair Shocks", |ui| {
        draw_stochastic_param_editor(ui, "repair", &mut stochastic.repair_shock);
    });

    ui.collapsing("Property Tax Increases", |ui| {
        draw_stochastic_param_editor(ui, "tax", &mut stochastic.property_tax_increase);
    });

    ui.collapsing("Insurance Increases", |ui| {
        draw_stochastic_param_editor(ui, "insurance", &mut stochastic.insurance_increase);
    });
}

fn draw_stochastic_param_editor(ui: &mut Ui, id: &str, param: &mut StochasticParameter) {
    Grid::new(format!("stochastic_{}", id))
        .num_columns(2)
        .spacing([20.0, 6.0])
        .show(ui, |ui| {
            ui.label("Distribution:");
            ComboBox::from_id_salt(format!("dist_{}", id))
                .selected_text(format!("{:?}", param.distribution))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut param.distribution, DistributionType::Fixed, "Fixed");
                    ui.selectable_value(&mut param.distribution, DistributionType::Normal, "Normal");
                    ui.selectable_value(&mut param.distribution, DistributionType::Uniform, "Uniform");
                    ui.selectable_value(&mut param.distribution, DistributionType::LogNormal, "LogNormal");
                    ui.selectable_value(&mut param.distribution, DistributionType::Triangular, "Triangular");
                });
            ui.end_row();

            let is_rate = param.name.contains("Rate") || param.name.contains("Increase") || param.name.contains("Appreciation");

            if is_rate {
                ui.label("Mean (%):");
                let mut mean_pct = param.mean * 100.0;
                ui.add(egui::DragValue::new(&mut mean_pct).speed(0.1).suffix("%"));
                param.mean = mean_pct / 100.0;
                ui.end_row();

                ui.label("Std Dev (%):");
                let mut std_pct = param.std_dev * 100.0;
                ui.add(egui::DragValue::new(&mut std_pct).speed(0.1).suffix("%"));
                param.std_dev = std_pct / 100.0;
                ui.end_row();

                ui.label("Min (%):");
                let mut min_pct = param.min * 100.0;
                ui.add(egui::DragValue::new(&mut min_pct).speed(0.1).suffix("%"));
                param.min = min_pct / 100.0;
                ui.end_row();

                ui.label("Max (%):");
                let mut max_pct = param.max * 100.0;
                ui.add(egui::DragValue::new(&mut max_pct).speed(0.1).suffix("%"));
                param.max = max_pct / 100.0;
                ui.end_row();
            } else {
                ui.label("Mean ($):");
                ui.add(egui::DragValue::new(&mut param.mean).speed(100.0).prefix("$"));
                ui.end_row();

                ui.label("Std Dev ($):");
                ui.add(egui::DragValue::new(&mut param.std_dev).speed(100.0).prefix("$"));
                ui.end_row();

                ui.label("Min ($):");
                ui.add(egui::DragValue::new(&mut param.min).speed(100.0).prefix("$"));
                ui.end_row();

                ui.label("Max ($):");
                ui.add(egui::DragValue::new(&mut param.max).speed(100.0).prefix("$"));
                ui.end_row();
            }
        });
}

fn draw_simulation_settings(ui: &mut Ui, state: &mut AppState) {
    Grid::new("sim_settings_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .show(ui, |ui| {
            ui.label("Number of Simulations:");
            ui.add(
                egui::DragValue::new(&mut state.config.num_simulations)
                    .speed(100)
                    .range(100..=1_000_000),
            );
            ui.end_row();

            ui.label("Random Seed:");
            let mut use_seed = state.config.random_seed.is_some();
            ui.horizontal(|ui| {
                ui.checkbox(&mut use_seed, "Use fixed seed");
                if use_seed {
                    let mut seed = state.config.random_seed.unwrap_or(42);
                    ui.add(egui::DragValue::new(&mut seed).speed(1));
                    state.config.random_seed = Some(seed);
                } else {
                    state.config.random_seed = None;
                }
            });
            ui.end_row();
        });
}
