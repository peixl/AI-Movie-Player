//! AI-Movie-Player: An AI-native local movie library companion for cinema lovers.
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
    let legacy = directories::ProjectDirs::from("ai", "ifq", "movie-box")
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

fn main() -> eframe::Result<()> {
    env_logger::init();

    let app_data_dir = resolve_app_data_dir();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title(format!("AI-Movie-Player v{} · ifq.ai", env!("CARGO_PKG_VERSION")))
            .with_icon(egui::IconData { rgba: vec![0; 32 * 32 * 4], width: 32, height: 32 }),
        ..Default::default()
    };

    eframe::run_native(
        "AI-Movie-Player",
        options,
        Box::new(|cc| {
            ai_movie_player::ui::fonts::install(&cc.egui_ctx);

            let app = MovieBoxApp::new(app_data_dir);
            Ok(Box::new(app))
        }),
    )
}
