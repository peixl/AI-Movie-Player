use egui::{Color32, Rounding, Stroke, Style, Visuals};

/// Color palette for dark and light themes.
pub struct AppTheme {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_light: Color32,
    pub primary: Color32,
    pub accent: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_muted: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
}

pub const DARK_THEME: AppTheme = AppTheme {
    bg: Color32::from_rgb(10, 10, 14),
    surface: Color32::from_rgb(22, 22, 30),
    surface_light: Color32::from_rgb(35, 35, 48),
    primary: Color32::from_rgb(99, 102, 241),
    accent: Color32::from_rgb(250, 176, 5),
    text: Color32::from_rgb(240, 240, 245),
    text_dim: Color32::from_rgb(180, 180, 190),
    text_muted: Color32::from_rgb(120, 120, 130),
    success: Color32::from_rgb(52, 211, 153),
    warning: Color32::from_rgb(251, 191, 36),
    error: Color32::from_rgb(248, 113, 113),
};

pub const LIGHT_THEME: AppTheme = AppTheme {
    bg: Color32::from_rgb(248, 248, 252),
    surface: Color32::from_rgb(255, 255, 255),
    surface_light: Color32::from_rgb(243, 244, 250),
    primary: Color32::from_rgb(79, 70, 229),
    accent: Color32::from_rgb(245, 158, 11),
    text: Color32::from_rgb(15, 15, 25),
    text_dim: Color32::from_rgb(80, 80, 95),
    text_muted: Color32::from_rgb(140, 140, 155),
    success: Color32::from_rgb(16, 185, 129),
    warning: Color32::from_rgb(245, 158, 11),
    error: Color32::from_rgb(239, 68, 68),
};

pub fn apply_theme(ctx: &egui::Context, is_dark: bool) {
    let theme = if is_dark { &DARK_THEME } else { &LIGHT_THEME };

    let mut visuals = if is_dark { Visuals::dark() } else { Visuals::light() };

    let egui_style = Style {
        visuals: Visuals {
            window_rounding: Rounding::same(8.0),
            window_shadow: epaint::Shadow {
                offset: [0, 2].into(),
                blur: 12,
                spread: 0,
                color: Color32::from_black_alpha(if is_dark { 80 } else { 30 }),
            },
            panel_fill: theme.bg,
            window_fill: theme.surface,
            widget_open: theme.surface_light,
            widget_hover: theme.surface_light,
            widget_active: theme.surface_light,
            faint_bg_color: theme.surface,
            extreme_bg_color: theme.bg,
            selection: egui::style::Selection {
                bg_fill: theme.primary.linear_multiply(0.3),
                stroke: Stroke::new(1.0, theme.primary),
            },
            hyperlink_color: theme.primary,
            ..visuals
        },
        ..Default::default()
    };

    ctx.set_style(egui_style);
}

/// Get the current theme based on dark mode flag.
pub fn current_theme(is_dark: bool) -> &'static AppTheme {
    if is_dark { &DARK_THEME } else { &LIGHT_THEME }
}

pub fn text_color(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(240, 240, 245) } else { Color32::from_rgb(15, 15, 25) }
}

pub fn dim_color(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(150, 150, 165) } else { Color32::from_rgb(100, 100, 115) }
}

pub fn muted_color(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(120, 120, 130) } else { Color32::from_rgb(140, 140, 155) }
}

pub fn primary_color() -> Color32 {
    Color32::from_rgb(99, 102, 241)
}

pub fn bg_color(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(17, 17, 25) } else { Color32::from_rgb(250, 250, 253) }
}

pub fn success_color(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(6, 78, 59) } else { Color32::from_rgb(209, 250, 229) }
}

pub fn error_color(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(127, 29, 29) } else { Color32::from_rgb(254, 226, 226) }
}

pub fn surface_light_color(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(224, 231, 255) }
}

pub fn rating_color(rating: f64) -> Color32 {
    match rating {
        r if r >= 8.0 => Color32::from_rgb(52, 211, 153),
        r if r >= 6.0 => Color32::from_rgb(250, 176, 5),
        r if r >= 4.0 => Color32::from_rgb(251, 146, 60),
        _ => Color32::from_rgb(248, 113, 113),
    }
}
