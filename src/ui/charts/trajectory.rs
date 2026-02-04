use crate::simulation::MonthlySnapshot;
use bevy_egui::egui::{Color32, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};

pub fn draw_trajectory_chart(
    ui: &mut Ui,
    title: &str,
    trajectories: &[Vec<MonthlySnapshot>],
    value_extractor: fn(&MonthlySnapshot) -> f64,
    y_label: &str,
    color: Color32,
) {
    ui.heading(title);
    ui.add_space(8.0);

    if trajectories.is_empty() {
        ui.label("No trajectory data available");
        return;
    }

    let to_years = |m: u32| m as f64 / 12.0;

    Plot::new(format!("{}_trajectory", title))
        .height(300.0)
        .legend(Legend::default())
        .x_axis_label("Years")
        .y_axis_label(y_label)
        .show_axes([true, true])
        .show_grid(true)
        .allow_drag(false)
        .allow_zoom(false)
        .show(ui, |plot_ui| {
            for (i, trajectory) in trajectories.iter().enumerate() {
                let points: PlotPoints = trajectory
                    .iter()
                    .map(|s| [to_years(s.month), value_extractor(s)])
                    .collect();

                // Alpha scales inversely with number of trajectories, but stays visible
                let alpha = (5000.0 / trajectories.len() as f32).clamp(30.0, 200.0) as u8;
                let line_color = Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    alpha,
                );

                let name = if i == 0 {
                    "Sample Trajectories"
                } else {
                    ""
                };

                plot_ui.line(
                    Line::new(name, points)
                        .color(line_color)
                        .width(1.0),
                );
            }
        });

    ui.add_space(4.0);
    ui.label(format!("Showing {} sample trajectories", trajectories.len()));
}
