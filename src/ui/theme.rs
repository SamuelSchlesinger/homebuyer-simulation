use bevy_egui::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, Style, Visuals};

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = Style::default();

    // Dark theme colors
    let bg_dark = Color32::from_rgb(25, 25, 35);
    let bg_medium = Color32::from_rgb(35, 35, 50);
    let bg_light = Color32::from_rgb(50, 50, 70);
    let accent = Color32::from_rgb(100, 150, 255);
    let accent_hover = Color32::from_rgb(130, 175, 255);
    let text_primary = Color32::from_rgb(230, 230, 240);
    let text_secondary = Color32::from_rgb(170, 170, 190);

    let mut visuals = Visuals::dark();

    visuals.window_fill = bg_dark;
    visuals.panel_fill = bg_dark;
    visuals.faint_bg_color = bg_medium;
    visuals.extreme_bg_color = bg_light;

    visuals.widgets.noninteractive.bg_fill = bg_medium;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_secondary);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

    visuals.widgets.inactive.bg_fill = bg_light;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

    visuals.widgets.hovered.bg_fill = accent;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

    visuals.widgets.active.bg_fill = accent_hover;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);

    visuals.selection.bg_fill = accent.gamma_multiply(0.5);
    visuals.selection.stroke = Stroke::new(1.0, accent);

    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0, bg_light);

    style.visuals = visuals;

    // Typography
    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(20.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        FontId::new(13.0, FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.slider_width = 200.0;

    ctx.set_style(style);
}

pub struct ChartColors;

impl ChartColors {
    pub fn equity() -> Color32 {
        Color32::from_rgb(100, 200, 100)
    }
    pub fn cost() -> Color32 {
        Color32::from_rgb(255, 150, 100)
    }
    pub fn waste() -> Color32 {
        Color32::from_rgb(255, 100, 100)
    }
    pub fn interest() -> Color32 {
        Color32::from_rgb(200, 150, 255)
    }
    pub fn rent() -> Color32 {
        Color32::from_rgb(100, 180, 255)
    }
    pub fn p5_p95() -> Color32 {
        Color32::from_rgba_premultiplied(100, 150, 255, 30)
    }
    pub fn p25_p75() -> Color32 {
        Color32::from_rgba_premultiplied(100, 150, 255, 60)
    }
    pub fn median() -> Color32 {
        Color32::from_rgb(100, 150, 255)
    }
}

pub fn format_currency(value: f64) -> String {
    if value.abs() >= 1_000_000.0 {
        format!("${:.2}M", value / 1_000_000.0)
    } else if value.abs() >= 1_000.0 {
        format!("${:.0}K", value / 1_000.0)
    } else {
        format!("${:.0}", value)
    }
}

pub fn format_currency_full(value: f64) -> String {
    if value < 0.0 {
        format!("-${}", format_number(-value))
    } else {
        format!("${}", format_number(value))
    }
}

fn format_number(value: f64) -> String {
    let value = value.round() as i64;
    let s = value.abs().to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

pub fn format_percent(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}
