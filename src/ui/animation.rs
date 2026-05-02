//! Animation utilities for egui immediate-mode UI.
//! Uses time-based animations since egui has no built-in animation timeline.

use egui::{Color32, Context, Id, Pos2};

/// Pulsing value (0.0 .. 1.0) that oscillates with given period in seconds.
/// Useful for skeleton loading, shimmer, attention indicators.
pub fn pulse(ctx: &Context, period: f32) -> f32 {
    let time = ctx.input(|i| i.time) as f32;
    let phase = (time % period) / period;
    if phase < 0.5 { (phase * 2.0).powf(0.7) } else { ((1.0 - phase) * 2.0).powf(0.7) }
}

/// Returns a color that shimmers between two colors for loading skeletons.
pub fn shimmer(base: Color32, highlight: Color32, ctx: &Context, period: f32) -> Color32 {
    let t = pulse(ctx, period);
    lerp_color(base, highlight, t)
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

/// Animate a boolean value with smooth interpolation.
/// Returns the current animated value between 0.0 and 1.0.
pub fn animated_bool(ctx: &Context, id: Id, value: bool, speed: f32) -> f32 {
    let time = ctx.input(|i| i.time);
    let key = id.with("anim_bool");

    let stored = ctx
        .data(|d| d.get_temp::<(f32, f64)>(key).unwrap_or((if value { 1.0 } else { 0.0 }, time)));

    let (current, last_time) = stored;
    let dt = ((time - last_time) as f32).min(0.1);
    let target: f32 = if value { 1.0 } else { 0.0 };

    let new = if (target - current).abs() < 0.001 {
        target
    } else {
        current + (target - current) * (dt * speed).min(1.0)
    };

    ctx.data_mut(|d| {
        d.insert_temp::<(f32, f64)>(key, (new, time));
    });

    new
}

/// Compute shimmer skeleton color for a poster placeholder.
/// Caller draws the rect using the returned color.
pub fn skeleton_poster_color(ctx: &Context, is_dark: bool) -> Color32 {
    let base =
        if is_dark { Color32::from_rgb(35, 35, 50) } else { Color32::from_rgb(225, 225, 235) };
    let highlight =
        if is_dark { Color32::from_rgb(50, 50, 65) } else { Color32::from_rgb(240, 240, 248) };
    shimmer(base, highlight, ctx, 1.5)
}

/// Draw a pulsing dot at the given center (for notifications).
pub fn pulsing_dot(painter: &egui::Painter, center: Pos2, color: Color32, ctx: &Context) {
    let t = pulse(ctx, 1.5);
    let radius = 4.0 + t * 6.0;
    let alpha = 0.3 + (1.0 - t) * 0.5;
    let c = Color32::from_rgba_premultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha.clamp(0.0, 1.0) * 255.0) as u8,
    );
    painter.circle_filled(center, radius, c);
    painter.circle_filled(center, 3.0, color);
}

/// Smooth hover transition — returns scale multiplier (0.97 .. 1.03).
pub fn hover_scale(ctx: &Context, id: Id, hovered: bool) -> f32 {
    let t = animated_bool(ctx, id, hovered, 6.0);
    0.97 + t * 0.06
}

/// Fade-in transition for panel content.
pub fn fade_alpha(ctx: &Context, id: Id, visible: bool) -> f32 {
    animated_bool(ctx, id, visible, 4.0)
}
