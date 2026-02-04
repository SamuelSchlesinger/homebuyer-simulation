use crate::simulation::MonthlySnapshot;
use crate::ui::app_state::{AppState, ChartType};
use crate::ui::charts::{
    histogram::draw_histogram,
    percentile::draw_percentile_chart,
    trajectory::draw_trajectory_chart,
};
use crate::ui::theme::{format_currency, format_percent, ChartColors};
use bevy_egui::egui::{Color32, RichText, Ui};

pub fn draw_results_panel(ui: &mut Ui, state: &mut AppState) {
    if let Some(ref results) = state.results {
        // Key metric summary at top
        ui.horizontal(|ui| {
            ui.heading("Effective Monthly Cost:");
            let median = results.effective_monthly_stats.p50;
            let rent = state.config.base.rent_equivalent;
            let color = if median < rent {
                Color32::from_rgb(100, 200, 100) // green - buying wins
            } else {
                Color32::from_rgb(255, 150, 100) // orange - renting wins
            };
            ui.label(RichText::new(format_currency(median)).color(color).strong());
            ui.label(format!("vs {} rent", format_currency(rent)));
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(format!(
                "Range: {} (5th) to {} (95th)",
                format_currency(results.effective_monthly_stats.p5),
                format_currency(results.effective_monthly_stats.p95)
            ));
        });

        if let Some(ref comparison) = state.comparison {
            ui.add_space(4.0);
            ui.label(format!(
                "Buying is cheaper than renting in {} of scenarios",
                format_percent(comparison.buy_wins_probability)
            ));
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        // Chart selector
        ui.horizontal(|ui| {
            ui.label("Chart:");
            ui.selectable_value(&mut state.chart_type, ChartType::EffectiveMonthlyCost, "Monthly Cost");
            ui.selectable_value(&mut state.chart_type, ChartType::EquityHistogram, "Final Equity");
            ui.selectable_value(&mut state.chart_type, ChartType::EquityTrajectory, "Equity Over Time");
            ui.selectable_value(&mut state.chart_type, ChartType::EffectiveCostTrajectory, "Net Cost Over Time");
            ui.selectable_value(&mut state.chart_type, ChartType::AllEquityTrajectories, "All Trajectories");
        });

        ui.add_space(8.0);

        match state.chart_type {
            ChartType::EffectiveMonthlyCost => {
                draw_histogram(
                    ui,
                    "Effective Monthly Cost Distribution",
                    &results.effective_monthly_values,
                    &results.effective_monthly_stats,
                    "Monthly Cost ($)",
                    ChartColors::cost(),
                    Some(state.config.base.rent_equivalent), // Show rent line
                );
            }
            ChartType::EquityHistogram => {
                draw_histogram(
                    ui,
                    "Final Equity Distribution",
                    &results.equity_values,
                    &results.equity_stats,
                    "Equity ($)",
                    ChartColors::equity(),
                    None,
                );
            }
            ChartType::EquityTrajectory => {
                draw_percentile_chart(
                    ui,
                    "Equity Over Time",
                    &results.equity_trajectory,
                    "Equity ($)",
                );
            }
            ChartType::EffectiveCostTrajectory => {
                draw_percentile_chart(
                    ui,
                    "Cumulative Net Cost Over Time (Payments - Equity)",
                    &results.effective_cost_trajectory,
                    "Net Cost ($)",
                );
            }
            ChartType::AllEquityTrajectories => {
                draw_trajectory_chart(
                    ui,
                    "All Simulation Equity Trajectories",
                    &results.sample_trajectories,
                    |s: &MonthlySnapshot| s.equity,
                    "Equity ($)",
                    ChartColors::equity(),
                );
            }
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("Run a simulation to see results");
        });
    }
}
