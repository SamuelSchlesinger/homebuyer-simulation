use crate::ui::app_state::AppState;
use crate::ui::theme::{format_currency, format_currency_full, format_percent};
use bevy_egui::egui::{self, Color32, Grid, RichText, Ui};

pub fn draw_summary_panel(ui: &mut Ui, state: &AppState) {
    if let Some(ref results) = state.results {
        ui.heading("Decision Snapshot");
        ui.add_space(8.0);

        if let Some(ref comparison) = state.comparison {
            let prob = comparison.buy_wins_probability;
            let verdict = if prob > 0.7 {
                "Buying looks financially favorable"
            } else if prob > 0.3 {
                "It is a close call"
            } else {
                "Renting looks financially favorable"
            };
            let verdict_color = if prob > 0.7 {
                Color32::from_rgb(100, 200, 100)
            } else if prob > 0.3 {
                Color32::from_rgb(255, 200, 100)
            } else {
                Color32::from_rgb(255, 100, 100)
            };

            egui::Frame::default()
                .fill(Color32::from_rgb(40, 40, 55))
                .corner_radius(8)
                .inner_margin(16)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Verdict:").strong());
                        ui.colored_label(verdict_color, verdict);
                    });
                    ui.add_space(4.0);
                    ui.label(format!(
                        "Buying is cheaper than renting in {} of scenarios.",
                        format_percent(prob)
                    ));
                    if comparison.monthly_savings_p50 > 0.0 {
                        ui.label(format!(
                            "Median monthly savings vs rent: {}",
                            format_currency(comparison.monthly_savings_p50)
                        ));
                    } else {
                        ui.label(format!(
                            "Median monthly extra cost vs rent: {}",
                            format_currency(-comparison.monthly_savings_p50)
                        ));
                    }
                });
        } else {
            ui.label("Run a simulation to see a buy vs rent recommendation.");
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        ui.heading("Simulation Summary");
        ui.add_space(8.0);

        ui.label(format!(
            "Based on {} Monte Carlo simulations over {} years",
            results.num_simulations,
            results.holding_months / 12
        ));
        ui.add_space(16.0);

        // Key metrics cards
        ui.horizontal(|ui| {
            draw_metric_card(
                ui,
                "Median Effective Monthly Cost",
                &format_currency(results.effective_monthly_stats.p50),
                Color32::from_rgb(255, 150, 100),
            );
            draw_metric_card(
                ui,
                "Median Final Equity",
                &format_currency(results.equity_stats.p50),
                Color32::from_rgb(100, 200, 100),
            );
            draw_metric_card(
                ui,
                "Rent Equivalent",
                &format_currency(state.config.base.rent_equivalent),
                Color32::from_rgb(100, 180, 255),
            );
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Detailed statistics tables
        ui.columns(2, |cols| {
            cols[0].heading("Effective Monthly Cost");
            draw_stats_table(&mut cols[0], "monthly", &results.effective_monthly_stats);

            cols[1].heading("Final Equity");
            draw_stats_table(&mut cols[1], "equity", &results.equity_stats);
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Risk analysis
        ui.heading("Risk Analysis");
        ui.add_space(8.0);

        let negative_equity_risk = results
            .equity_values
            .iter()
            .filter(|e| **e < 0.0)
            .count() as f64
            / results.num_simulations as f64;

        let rent = state.config.base.rent_equivalent;
        let worse_than_rent = results
            .effective_monthly_values
            .iter()
            .filter(|c| **c > rent)
            .count() as f64
            / results.num_simulations as f64;

        Grid::new("risk_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Probability of negative equity:");
                ui.colored_label(
                    risk_color(negative_equity_risk),
                    format_percent(negative_equity_risk),
                );
                ui.end_row();

                ui.label("Probability buying costs more than rent:");
                ui.colored_label(risk_color(worse_than_rent), format_percent(worse_than_rent));
                ui.end_row();

                ui.label("Worst case monthly cost (95th %ile):");
                ui.label(format_currency(results.effective_monthly_stats.p95));
                ui.end_row();

                ui.label("Worst case equity (5th %ile):");
                ui.label(format_currency_full(results.equity_stats.p5));
                ui.end_row();
            });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Run a simulation to see summary statistics");
        });
    }
}

fn draw_metric_card(ui: &mut Ui, label: &str, value: &str, color: Color32) {
    egui::Frame::default()
        .fill(Color32::from_rgb(40, 40, 55))
        .corner_radius(8)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(label).small());
                ui.label(RichText::new(value).heading().color(color));
            });
        });
}

fn draw_stats_table(ui: &mut Ui, id: &str, stats: &crate::simulation::PercentileStats) {
    Grid::new(format!("stats_{}", id))
        .num_columns(2)
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Mean:");
            ui.label(format_currency_full(stats.mean));
            ui.end_row();

            ui.label("Std Dev:");
            ui.label(format_currency_full(stats.std_dev));
            ui.end_row();

            ui.label("P5 (best):");
            ui.label(format_currency_full(stats.p5));
            ui.end_row();

            ui.label("Median:");
            ui.label(format_currency_full(stats.p50));
            ui.end_row();

            ui.label("P95 (worst):");
            ui.label(format_currency_full(stats.p95));
            ui.end_row();
        });
}

fn risk_color(probability: f64) -> Color32 {
    if probability < 0.05 {
        Color32::from_rgb(100, 200, 100) // Green - low risk
    } else if probability < 0.20 {
        Color32::from_rgb(255, 200, 100) // Yellow - moderate risk
    } else {
        Color32::from_rgb(255, 100, 100) // Red - high risk
    }
}
