use crate::simulation::TimeSeriesPercentiles;
use crate::ui::theme::ChartColors;
use bevy_egui::egui::Ui;
use egui_plot::{Legend, Line, Plot, PlotPoints, Polygon};

pub fn draw_percentile_chart(
    ui: &mut Ui,
    title: &str,
    data: &TimeSeriesPercentiles,
    y_label: &str,
) {
    ui.heading(title);
    ui.add_space(8.0);

    if data.months.is_empty() {
        ui.label("No data available");
        return;
    }

    // Convert months to years for x-axis
    let to_years = |m: u32| m as f64 / 12.0;

    // Create polygon for P5-P95 band
    let p5_p95_points: Vec<[f64; 2]> = {
        let mut points: Vec<[f64; 2]> = data
            .months
            .iter()
            .zip(data.p5.iter())
            .map(|(&m, &v)| [to_years(m), v])
            .collect();

        // Add reverse path along P95
        let mut p95_rev: Vec<[f64; 2]> = data
            .months
            .iter()
            .zip(data.p95.iter())
            .map(|(&m, &v)| [to_years(m), v])
            .collect();
        p95_rev.reverse();
        points.extend(p95_rev);

        points
    };

    // Create polygon for P25-P75 band
    let p25_p75_points: Vec<[f64; 2]> = {
        let mut points: Vec<[f64; 2]> = data
            .months
            .iter()
            .zip(data.p25.iter())
            .map(|(&m, &v)| [to_years(m), v])
            .collect();

        let mut p75_rev: Vec<[f64; 2]> = data
            .months
            .iter()
            .zip(data.p75.iter())
            .map(|(&m, &v)| [to_years(m), v])
            .collect();
        p75_rev.reverse();
        points.extend(p75_rev);

        points
    };

    // Median line
    let median_points: PlotPoints = data
        .months
        .iter()
        .zip(data.p50.iter())
        .map(|(&m, &v)| [to_years(m), v])
        .collect();

    Plot::new(format!("{}_percentile", title))
        .height(300.0)
        .legend(Legend::default())
        .x_axis_label("Years")
        .y_axis_label(y_label)
        .show_axes([true, true])
        .show_grid(true)
        .allow_drag(false)
        .allow_zoom(false)
        .show(ui, |plot_ui| {
            // Draw bands from outer to inner
            plot_ui.polygon(
                Polygon::new("P5-P95", PlotPoints::new(p5_p95_points))
                    .fill_color(ChartColors::p5_p95()),
            );

            plot_ui.polygon(
                Polygon::new("P25-P75", PlotPoints::new(p25_p75_points))
                    .fill_color(ChartColors::p25_p75()),
            );

            // Median line
            plot_ui.line(
                Line::new("Median", median_points)
                    .color(ChartColors::median())
                    .width(2.0),
            );
        });

    // Legend
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.colored_label(ChartColors::p5_p95(), "P5-P95 Range");
        ui.separator();
        ui.colored_label(ChartColors::p25_p75(), "P25-P75 Range");
        ui.separator();
        ui.colored_label(ChartColors::median(), "Median");
    });
}
