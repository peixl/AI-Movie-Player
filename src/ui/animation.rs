//! Animation utilities for egui immediate-mode UI.
//! Uses time-based animations since egui has no built-in animation timeline.

use egui::{Color32, Context};

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

/// Compute shimmer skeleton color for a poster placeholder.
/// Caller draws the rect using the returned color.
pub fn skeleton_poster_color(ctx: &Context, is_dark: bool) -> Color32 {
    let base =
        if is_dark { Color32::from_rgb(35, 35, 50) } else { Color32::from_rgb(225, 225, 235) };
    let highlight =
        if is_dark { Color32::from_rgb(50, 50, 65) } else { Color32::from_rgb(240, 240, 248) };
    shimmer(base, highlight, ctx, 1.5)
}
