use crate::simulation::PercentileStats;
use crate::ui::theme::format_currency;
use bevy_egui::egui::{self, Color32, Id, Ui};
use egui_plot::{Bar, BarChart, Plot, Polygon, VLine};

#[derive(Clone, Copy, Default)]
struct RangeSelection {
    drag_start: Option<f64>,
    range: Option<(f64, f64)>,
    fixed_bound: Option<f64>,
}

pub fn draw_histogram(
    ui: &mut Ui,
    title: &str,
    values: &[f64],
    stats: &PercentileStats,
    _y_label: &str,
    color: Color32,
    reference_line: Option<f64>,
) {
    ui.heading(title);
    ui.add_space(8.0);

    if values.is_empty() {
        ui.label("No data available");
        return;
    }

    let min_val = stats.min;
    let max_val = stats.max;
    let range = max_val - min_val;

    if range <= 0.0 {
        ui.label("Insufficient data variance");
        return;
    }

    let num_bins = 50;
    let bin_width = range / num_bins as f64;
    let mut bins = vec![0usize; num_bins];

    for &value in values {
        let bin_idx = ((value - min_val) / bin_width) as usize;
        let bin_idx = bin_idx.min(num_bins - 1);
        bins[bin_idx] += 1;
    }

    // Store bin data for hover lookup
    let bin_data: Vec<(f64, f64, usize)> = bins
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let bin_start = min_val + i as f64 * bin_width;
            let bin_end = bin_start + bin_width;
            (bin_start, bin_end, count)
        })
        .collect();

    let bars: Vec<Bar> = bin_data
        .iter()
        .map(|(bin_start, bin_end, count)| {
            let x = (bin_start + bin_end) / 2.0;
            Bar::new(x, *count as f64)
                .width(bin_width * 0.9)
                .fill(color)
        })
        .collect();

    let chart = BarChart::new(title, bars);
    let total_values = values.len();

    let plot_id = format!("{}_histogram", title);
    let selection_id = Id::new(format!("{}_selection", plot_id));
    let reset_id = Id::new(format!("{}_reset_view", plot_id));

    let mut selection = ui
        .ctx()
        .data_mut(|data| data.get_temp::<RangeSelection>(selection_id))
        .unwrap_or_default();

    let mut plot_builder = Plot::new(plot_id)
        .height(300.0)
        .x_axis_label("Value")
        .y_axis_label("Frequency")
        .show_axes([true, true])
        .show_grid(true)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .allow_axis_zoom_drag(false)
        .allow_double_click_reset(true);

    let reset_requested = ui
        .ctx()
        .data_mut(|data| data.get_temp::<bool>(reset_id))
        .unwrap_or(false);
    if reset_requested {
        plot_builder = plot_builder.reset();
        ui.ctx().data_mut(|data| data.insert_temp(reset_id, false));
    }

    let plot_response = plot_builder.show(ui, |plot_ui| {
            // Selection band (drawn behind bars)
            if let Some((start, end)) = selection.range {
                let min_sel = start.min(end);
                let max_sel = start.max(end);
                let bounds = plot_ui.plot_bounds();
                if bounds.is_valid_y() {
                    let y_min = bounds.min()[1];
                    let y_max = bounds.max()[1];
                    let band_points = vec![
                        [min_sel, y_min],
                        [min_sel, y_max],
                        [max_sel, y_max],
                        [max_sel, y_min],
                    ];
                    plot_ui.polygon(
                        Polygon::new("Selection band", band_points)
                            .fill_color(Color32::from_rgba_unmultiplied(100, 150, 255, 40)),
                    );
                }
            }

            plot_ui.bar_chart(chart);

            // Reference line (e.g., rent)
            if let Some(ref_val) = reference_line {
                plot_ui.vline(
                    VLine::new("Rent", ref_val)
                        .color(Color32::from_rgb(100, 180, 255))
                        .width(2.0),
                );
            }

            // Median line
            plot_ui.vline(
                VLine::new("Median", stats.p50)
                    .color(Color32::from_rgb(255, 255, 100))
                    .style(egui_plot::LineStyle::Solid),
            );

            // Selection bounds
            if let Some((start, end)) = selection.range {
                let min_sel = start.min(end);
                let max_sel = start.max(end);
                plot_ui.vline(
                    VLine::new("Selection min", min_sel)
                        .color(Color32::from_rgb(180, 220, 255))
                        .style(egui_plot::LineStyle::dashed_dense()),
                );
                plot_ui.vline(
                    VLine::new("Selection max", max_sel)
                        .color(Color32::from_rgb(180, 220, 255))
                        .style(egui_plot::LineStyle::dashed_dense()),
                );
            }
        });

    // Update selection from drag gestures
    if plot_response.response.drag_started() {
        if let Some(pointer_pos) = plot_response.response.interact_pointer_pos() {
            let coord = plot_response.transform.value_from_position(pointer_pos);
            let is_shift = ui.ctx().input(|i| i.modifiers.shift);
            selection.drag_start = Some(coord.x);
            selection.fixed_bound = None;

            if is_shift {
                if let Some((start, end)) = selection.range {
                    let min_sel = start.min(end);
                    let max_sel = start.max(end);
                    let dist_min = (coord.x - min_sel).abs();
                    let dist_max = (coord.x - max_sel).abs();
                    let fixed = if dist_min <= dist_max { max_sel } else { min_sel };
                    selection.fixed_bound = Some(fixed);
                    selection.range = Some((fixed, coord.x));
                } else {
                    selection.range = None;
                }
            } else {
                selection.range = None;
            }
        }
    }

    if plot_response.response.dragged() {
        if let (Some(start), Some(pointer_pos)) = (
            selection.drag_start,
            plot_response.response.interact_pointer_pos(),
        ) {
            let coord = plot_response.transform.value_from_position(pointer_pos);
            if let Some(fixed) = selection.fixed_bound {
                selection.range = Some((fixed, coord.x));
            } else {
                selection.range = Some((start, coord.x));
            }
        }
    }

    if plot_response.response.drag_stopped() {
        selection.drag_start = None;
        selection.fixed_bound = None;
    }

    ui.ctx()
        .data_mut(|data| data.insert_temp(selection_id, selection));

    // Show tooltip at cursor if hovering over the plot
    let mut showed_bin_tooltip = false;
    if !plot_response.response.dragged() {
        if let Some(pointer_pos) = plot_response.response.hover_pos() {
            let coord = plot_response.transform.value_from_position(pointer_pos);
            let x = coord.x;
            // Find which bin this x falls into
            let bin_idx = ((x - min_val) / bin_width) as i32;
            if bin_idx >= 0 && (bin_idx as usize) < bin_data.len() {
                let (bin_start, bin_end, count) = bin_data[bin_idx as usize];
                // Check if we're actually over a bar (y > 0 and within bar height)
                if coord.y >= 0.0 && coord.y <= count as f64 && count > 0 {
                    let pct = (count as f64 / total_values as f64) * 100.0;
                    showed_bin_tooltip = true;
                    egui::show_tooltip_at_pointer(
                        ui.ctx(),
                        egui::LayerId::new(egui::Order::Tooltip, Id::new("histogram_tooltip")),
                        Id::new("histogram_tooltip_content"),
                        |ui: &mut Ui| {
                            ui.label(format!(
                                "{} - {}",
                                format_currency(bin_start),
                                format_currency(bin_end)
                            ));
                            ui.label(format!("{} simulations ({:.1}%)", count, pct));
                        },
                    );
                }
            }
        }
    }

    if selection.range.is_some()
        && (plot_response.response.dragged() || plot_response.response.hovered())
        && (!showed_bin_tooltip || plot_response.response.dragged())
    {
        let (min_sel, max_sel) = selection
            .range
            .map(|(start, end)| (start.min(end), start.max(end)))
            .unwrap();
        let count = values
            .iter()
            .filter(|&&value| value >= min_sel && value <= max_sel)
            .count();
        let pct = (count as f64 / total_values as f64) * 100.0;

        egui::show_tooltip_at_pointer(
            ui.ctx(),
            egui::LayerId::new(
                egui::Order::Tooltip,
                Id::new("histogram_selection_tooltip"),
            ),
            Id::new("histogram_selection_tooltip_content"),
            |ui: &mut Ui| {
                ui.label(format!(
                    "Selected: {} to {}",
                    format_currency(min_sel),
                    format_currency(max_sel)
                ));
                ui.label(format!("{} samples ({:.1}%)", count, pct));
            },
        );
    }

    // Stats summary
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(format!("Median: {}", format_currency(stats.p50)));
        ui.separator();
        ui.label(format!("Range: {} to {}", format_currency(stats.p5), format_currency(stats.p95)));
    });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if let Some((start, end)) = selection.range {
            let min_sel = start.min(end);
            let max_sel = start.max(end);
            let count = values
                .iter()
                .filter(|&&value| value >= min_sel && value <= max_sel)
                .count();
            let pct = (count as f64 / total_values as f64) * 100.0;
            ui.label(format!(
                "Selected: {} to {}",
                format_currency(min_sel),
                format_currency(max_sel)
            ));
            ui.separator();
            ui.label(format!("{} samples ({:.1}%)", count, pct));
            ui.separator();
            if ui.button("Clear selection").clicked() {
                ui.ctx()
                    .data_mut(|data| data.insert_temp(selection_id, RangeSelection::default()));
            }
            if ui.button("Reset view").clicked() {
                ui.ctx().data_mut(|data| data.insert_temp(reset_id, true));
            }
        } else {
            ui.label("Drag across the chart to measure a value range. Hold Shift to adjust the edges.");
            ui.separator();
            if ui.button("Reset view").clicked() {
                ui.ctx().data_mut(|data| data.insert_temp(reset_id, true));
            }
        }
    });
}
