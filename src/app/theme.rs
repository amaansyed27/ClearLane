use eframe::egui::{self, Color32, CornerRadius, Stroke, Vec2};

pub const SHELL_BG: Color32 = Color32::from_rgb(242, 243, 244);
pub const SIDEBAR_BG: Color32 = Color32::from_rgb(235, 237, 239);
pub const PAGE_BG: Color32 = Color32::from_rgb(255, 255, 255);
pub const OMNIBOX_BG: Color32 = Color32::from_rgb(252, 252, 252);
pub const TEXT: Color32 = Color32::from_rgb(31, 34, 38);
pub const MUTED: Color32 = Color32::from_rgb(105, 110, 118);
pub const FAINT: Color32 = Color32::from_rgb(146, 151, 158);
pub const BORDER: Color32 = Color32::from_rgb(213, 216, 220);
pub const HOVER: Color32 = Color32::from_rgb(222, 225, 229);
pub const ACTIVE: Color32 = Color32::from_rgb(211, 215, 220);
pub const ACCENT: Color32 = Color32::from_rgb(52, 108, 246);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(221, 230, 253);
pub const SAFE: Color32 = Color32::from_rgb(45, 122, 82);

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = SHELL_BG;
    visuals.window_fill = PAGE_BG;
    visuals.extreme_bg_color = OMNIBOX_BG;
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = ACCENT_SOFT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.corner_radius = CornerRadius::same(7);

    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.weak_bg_fill = HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.corner_radius = CornerRadius::same(7);

    visuals.widgets.active.bg_fill = ACTIVE;
    visuals.widgets.active.weak_bg_fill = ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.corner_radius = CornerRadius::same(7);

    visuals.widgets.open.bg_fill = ACTIVE;
    visuals.widgets.open.weak_bg_fill = ACTIVE;
    visuals.widgets.open.bg_stroke = Stroke::NONE;
    visuals.widgets.open.corner_radius = CornerRadius::same(7);

    style.visuals = visuals;
    style.spacing.item_spacing = Vec2::new(6.0, 6.0);
    style.spacing.button_padding = Vec2::new(8.0, 6.0);
    style.spacing.interact_size.y = 32.0;
    ctx.set_style(style);
}
