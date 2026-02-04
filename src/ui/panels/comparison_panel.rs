use crate::ui::app_state::AppState;
use crate::ui::theme::{format_currency, format_percent};
use bevy_egui::egui::{self, Color32, Grid, RichText, Ui};

pub fn draw_comparison_panel(ui: &mut Ui, state: &AppState) {
    if let (Some(results), Some(comparison)) = (&state.results, &state.comparison) {
        ui.heading("Buy vs. Rent: Effective Monthly Cost");
        ui.add_space(8.0);

        let years = state.config.holding_period_years;
        ui.label(format!("Analysis over {} year holding period", years));
        ui.add_space(16.0);

        // Key comparison
        Grid::new("comparison_grid")
            .num_columns(2)
            .spacing([40.0, 12.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Monthly Rent:").strong());
                ui.label(RichText::new(format_currency(comparison.monthly_rent))
                    .color(Color32::from_rgb(100, 180, 255)));
                ui.end_row();

                ui.label(RichText::new("Buy: Median Monthly Cost:").strong());
                let buy_color = if comparison.buy_monthly_p50 < comparison.monthly_rent {
                    Color32::from_rgb(100, 200, 100)
                } else {
                    Color32::from_rgb(255, 150, 100)
                };
                ui.label(RichText::new(format_currency(comparison.buy_monthly_p50))
                    .color(buy_color));
                ui.end_row();

                ui.label("Buy: Best Case (5th %ile):");
                ui.label(format_currency(comparison.buy_monthly_p5));
                ui.end_row();

                ui.label("Buy: Worst Case (95th %ile):");
                ui.label(format_currency(comparison.buy_monthly_p95));
                ui.end_row();
            });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Probability
        let prob = comparison.buy_wins_probability;
        let prob_color = if prob > 0.5 {
            Color32::from_rgb(100, 200, 100)
        } else {
            Color32::from_rgb(255, 100, 100)
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new("Probability buying is cheaper:").strong());
            ui.label(RichText::new(format_percent(prob)).color(prob_color).strong());
        });

        ui.add_space(8.0);

        if comparison.monthly_savings_p50 > 0.0 {
            ui.label(format!(
                "Buying saves ~{}/month on average",
                format_currency(comparison.monthly_savings_p50)
            ));
        } else {
            ui.label(format!(
                "Renting saves ~{}/month on average",
                format_currency(-comparison.monthly_savings_p50)
            ));
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Interpretation
        egui::Frame::default()
            .fill(Color32::from_rgb(40, 40, 55))
            .corner_radius(8)
            .inner_margin(16)
            .show(ui, |ui| {
                ui.heading("What This Means");
                ui.add_space(4.0);

                ui.label("Effective monthly cost = (all payments + transaction costs − final equity) ÷ months");
                ui.add_space(8.0);
                ui.label("This estimates what you actually spend on housing each month after equity and selling costs.");
                ui.add_space(8.0);

                if prob > 0.7 {
                    ui.colored_label(
                        Color32::from_rgb(100, 200, 100),
                        "Buying looks favorable in most scenarios.",
                    );
                } else if prob > 0.3 {
                    ui.colored_label(
                        Color32::from_rgb(255, 200, 100),
                        "It's close - consider stability, lifestyle, and your risk tolerance.",
                    );
                } else {
                    ui.colored_label(
                        Color32::from_rgb(255, 100, 100),
                        "Renting looks favorable in most scenarios.",
                    );
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Includes your configured buying and selling costs.").small().weak());
            });

        // Stats table
        ui.add_space(16.0);
        ui.collapsing("Detailed Statistics", |ui| {
            Grid::new("stats_grid")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Simulations:");
                    ui.label(format!("{}", results.num_simulations));
                    ui.end_row();

                    ui.label("Holding period:");
                    ui.label(format!("{} months", results.holding_months));
                    ui.end_row();

                    ui.label("Effective cost mean:");
                    ui.label(format_currency(results.effective_monthly_stats.mean));
                    ui.end_row();

                    ui.label("Effective cost std dev:");
                    ui.label(format_currency(results.effective_monthly_stats.std_dev));
                    ui.end_row();

                    ui.label("Final equity median:");
                    ui.label(format_currency(results.equity_stats.p50));
                    ui.end_row();

                    ui.label("Final equity range:");
                    ui.label(format!(
                        "{} to {}",
                        format_currency(results.equity_stats.p5),
                        format_currency(results.equity_stats.p95)
                    ));
                    ui.end_row();
                });
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Run a simulation to see comparison");
        });
    }
}
