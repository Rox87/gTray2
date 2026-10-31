//! theme.rs - Sistema de temas (Dark Studio e Modern Light) e tokens de design para o gTray.

use eframe::egui::{Color32, CornerRadius, Stroke, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

impl ThemeMode {
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        if s.eq_ignore_ascii_case("light") {
            Self::Light
        } else {
            Self::Dark
        }
    }

    pub fn colors(&self) -> UIColors {
        match self {
            Self::Dark => UIColors {
                bg_panel: Color32::from_rgb(15, 23, 42),       // #0F172A
                bg_card: Color32::from_rgb(30, 41, 59),        // #1E293B
                bg_card_sub: Color32::from_rgb(15, 23, 42),    // #0F172A
                bg_input: Color32::from_rgb(11, 17, 32),       // #0B1120
                border: Color32::from_rgb(51, 65, 85),         // #334155
                border_subtle: Color32::from_rgb(30, 41, 59),  // #1E293B
                border_focus: Color32::from_rgb(99, 102, 241), // #6366F1
                text_primary: Color32::from_rgb(248, 250, 252),// #F8FAFC
                text_secondary: Color32::from_rgb(148, 163, 184), // #94A3B8
                text_muted: Color32::from_rgb(100, 116, 139),  // #64748B
                accent_blue: Color32::from_rgb(59, 130, 246),  // #3B82F6
                accent_blue_hover: Color32::from_rgb(96, 165, 250),
                accent_green: Color32::from_rgb(16, 185, 129), // #10B981
                accent_red: Color32::from_rgb(239, 68, 68),    // #EF4444
                accent_amber: Color32::from_rgb(245, 158, 11), // #F59E0B
                accent_purple: Color32::from_rgb(139, 92, 246),// #8B5CF6
                badge_active_bg: Color32::from_rgba_unmultiplied(16, 185, 129, 38),
                badge_active_fg: Color32::from_rgb(110, 231, 183),
                badge_paused_bg: Color32::from_rgba_unmultiplied(239, 68, 68, 38),
                badge_paused_fg: Color32::from_rgb(252, 165, 165),
                keycap_bg: Color32::from_rgb(15, 23, 42),
                keycap_border: Color32::from_rgb(71, 85, 105),
                keycap_text: Color32::from_rgb(56, 189, 248),
            },
            Self::Light => UIColors {
                bg_panel: Color32::from_rgb(248, 250, 252),    // #F8FAFC
                bg_card: Color32::from_rgb(255, 255, 255),     // #FFFFFF
                bg_card_sub: Color32::from_rgb(241, 245, 249), // #F1F5F9
                bg_input: Color32::from_rgb(255, 255, 255),
                border: Color32::from_rgb(226, 232, 240),      // #E2E8F0
                border_subtle: Color32::from_rgb(241, 245, 249),
                border_focus: Color32::from_rgb(59, 130, 246),
                text_primary: Color32::from_rgb(15, 23, 42),   // #0F172A
                text_secondary: Color32::from_rgb(71, 85, 105),// #475569
                text_muted: Color32::from_rgb(148, 163, 184),  // #94A3B8
                accent_blue: Color32::from_rgb(37, 99, 235),   // #2563EB
                accent_blue_hover: Color32::from_rgb(29, 78, 216),
                accent_green: Color32::from_rgb(5, 150, 105),  // #059669
                accent_red: Color32::from_rgb(220, 38, 38),    // #DC2626
                accent_amber: Color32::from_rgb(217, 119, 6),  // #D97706
                accent_purple: Color32::from_rgb(124, 58, 237),// #7C3AED
                badge_active_bg: Color32::from_rgba_unmultiplied(5, 150, 105, 25),
                badge_active_fg: Color32::from_rgb(5, 150, 105),
                badge_paused_bg: Color32::from_rgba_unmultiplied(220, 38, 38, 25),
                badge_paused_fg: Color32::from_rgb(220, 38, 38),
                keycap_bg: Color32::from_rgb(241, 245, 249),
                keycap_border: Color32::from_rgb(203, 213, 225),
                keycap_text: Color32::from_rgb(3, 105, 161),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UIColors {
    pub bg_panel: Color32,
    pub bg_card: Color32,
    pub bg_card_sub: Color32,
    pub bg_input: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub border_focus: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent_blue: Color32,
    pub accent_blue_hover: Color32,
    pub accent_green: Color32,
    pub accent_red: Color32,
    pub accent_amber: Color32,
    pub accent_purple: Color32,
    pub badge_active_bg: Color32,
    pub badge_active_fg: Color32,
    pub badge_paused_bg: Color32,
    pub badge_paused_fg: Color32,
    pub keycap_bg: Color32,
    pub keycap_border: Color32,
    pub keycap_text: Color32,
}

pub fn apply_theme(ctx: &eframe::egui::Context, theme: ThemeMode) {
    let colors = theme.colors();
    let mut visuals = if theme.is_dark() {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.panel_fill = colors.bg_panel;
    visuals.window_fill = colors.bg_card;
    visuals.window_stroke = Stroke::new(1.0, colors.border);
    visuals.window_corner_radius = CornerRadius::same(10);

    visuals.widgets.noninteractive.bg_fill = colors.bg_card_sub;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text_secondary);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

    visuals.widgets.inactive.bg_fill = colors.bg_card;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text_primary);
    visuals.widgets.inactive.weak_bg_fill = colors.bg_card_sub;
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = if theme.is_dark() {
        Color32::from_rgb(51, 65, 85)
    } else {
        Color32::from_rgb(226, 232, 240)
    };
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, colors.text_primary);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

    visuals.widgets.active.bg_fill = colors.accent_blue;
    visuals.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);

    visuals.widgets.open.bg_fill = colors.bg_card_sub;
    visuals.widgets.open.corner_radius = CornerRadius::same(6);

    visuals.selection.bg_fill = colors.accent_blue;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);

    ctx.set_visuals(visuals);
}
