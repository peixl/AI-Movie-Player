//! AI Movie Player: An AI-native local movie library companion for cinema lovers.
//!
//! Built with Rust and egui, combining local library management, TMDB metadata,
//! subtitles, system-player launch, poster-wall browsing, and OpenAI-compatible AI features.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ai_movie_player::app::MovieBoxApp;

fn resolve_app_data_dir() -> std::path::PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".into());

    let preferred = directories::ProjectDirs::from("ai", "ifq", "ai-movie-player")
        .map(|dirs| dirs.data_dir().to_path_buf());
    let legacy = directories::ProjectDirs::from("ai", "if", "movie-box")
        .map(|dirs| dirs.data_dir().to_path_buf());

    if let Some(path) = legacy.clone().filter(|path| path.exists()) {
        return path;
    }

    let legacy_fallback = std::path::PathBuf::from(&home).join(".ai-movie-box");
    if legacy_fallback.exists() {
        return legacy_fallback;
    }

    preferred.or(legacy).unwrap_or_else(|| std::path::PathBuf::from(home).join(".ai-movie-player"))
}

/// Build the application icon procedurally: a dark rounded tile with an indigo
/// film strip and a play triangle. Replaces the previous fully-transparent icon.
fn app_icon() -> egui::IconData {
    const S: usize = 256;
    let mut rgba = vec![0u8; S * S * 4];

    let dark = [16u8, 18, 34];
    let indigo = [99u8, 102, 241];
    let white = [245u8, 246, 255];

    let put = |rgba: &mut [u8], x: usize, y: usize, color: [u8; 3], alpha: u8| {
        if x >= S || y >= S {
            return;
        }
        let i = (y * S + x) * 4;
        rgba[i] = color[0];
        rgba[i + 1] = color[1];
        rgba[i + 2] = color[2];
        rgba[i + 3] = alpha;
    };

    let in_rounded = |x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32| -> bool {
        if x < x0 || y < y0 || x > x1 || y > y1 {
            return false;
        }
        let inner_w = (x1 - x0 - 2.0 * r).max(0.0);
        let inner_h = (y1 - y0 - 2.0 * r).max(0.0);
        let nearest_x = x0 + r + (x - (x0 + r)).clamp(0.0, inner_w);
        let nearest_y = y0 + r + (y - (y0 + r)).clamp(0.0, inner_h);
        let dx = x - nearest_x;
        let dy = y - nearest_y;
        dx * dx + dy * dy <= r * r
    };

    let in_triangle = |px: f32, py: f32, a: (f32, f32), b: (f32, f32), c: (f32, f32)| -> bool {
        let sign = |p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)| {
            (p1.0 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 - p3.1)
        };
        let d1 = sign((px, py), a, b);
        let d2 = sign((px, py), b, c);
        let d3 = sign((px, py), c, a);
        let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
        let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
        !(has_neg && has_pos)
    };

    for y in 0..S {
        for x in 0..S {
            let (fx, fy) = (x as f32, y as f32);
            // Background tile.
            if in_rounded(fx, fy, 4.0, 4.0, 252.0, 252.0, 52.0) {
                put(&mut rgba, x, y, dark, 255);
            }
            // Film strip.
            if in_rounded(fx, fy, 56.0, 30.0, 200.0, 226.0, 18.0) {
                put(&mut rgba, x, y, indigo, 255);
            }
            // Sprocket holes (two columns).
            for row in 0..7 {
                let hy = 44.0 + row as f32 * 28.0;
                for hx in [66.0f32, 182.0] {
                    if in_rounded(fx, fy, hx, hy, hx + 8.0, hy + 14.0, 3.0) {
                        put(&mut rgba, x, y, dark, 255);
                    }
                }
            }
            // Play triangle.
            if in_triangle(fx + 0.5, fy + 0.5, (108.0, 92.0), (108.0, 164.0), (170.0, 128.0))
                && in_rounded(fx, fy, 56.0, 30.0, 200.0, 226.0, 18.0)
            {
                put(&mut rgba, x, y, white, 255);
            }
        }
    }

    egui::IconData { rgba, width: S as u32, height: S as u32 }
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let app_data_dir = resolve_app_data_dir();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title(format!("AI Movie Player v{} · ifq.ai", env!("CARGO_PKG_VERSION")))
            .with_icon(app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "AI Movie Player",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            ai_movie_player::ui::fonts::install(&cc.egui_ctx);

            let app = MovieBoxApp::new(app_data_dir);
            Ok(Box::new(app))
        }),
    )
}
