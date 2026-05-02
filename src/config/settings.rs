//! Application settings model and persistence.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User-configurable application settings persisted in the database.
///
/// Covers TMDB API, AI provider, theme, subtitle languages, and library paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub tmdb_api_key: String,
    pub tmdb_language: String,
    pub subtitle_languages: Vec<String>,
    pub rename_template: String,
    pub theme: String,
    pub library_paths: Vec<PathBuf>,
    pub auto_scan: bool,
    pub ai_endpoint: String,
    pub ai_api_key: String,
    pub ai_model: String,
    pub ai_temperature: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            tmdb_api_key: String::new(),
            tmdb_language: "zh-CN".into(),
            subtitle_languages: vec!["zh".into(), "zh-CN".into(), "en".into()],
            rename_template: "{title} ({year})".into(),
            theme: "dark".into(),
            library_paths: vec![],
            auto_scan: false,
            ai_endpoint: "https://api.openai.com/v1".into(),
            ai_api_key: String::new(),
            ai_model: "gpt-4o-mini".into(),
            ai_temperature: 0.7,
        }
    }
}

impl AppSettings {
    pub fn load_from_db(
        get_setting: &dyn Fn(&str) -> Result<Option<String>, crate::util::error::AppError>,
    ) -> Self {
        let mut settings = Self::default();

        if let Ok(Some(v)) = get_setting("tmdb_api_key") { settings.tmdb_api_key = v; }
        if let Ok(Some(v)) = get_setting("tmdb_language") { settings.tmdb_language = v; }
        if let Ok(Some(v)) = get_setting("subtitle_languages") {
            if let Ok(langs) = serde_json::from_str::<Vec<String>>(&v) { settings.subtitle_languages = langs; }
        }
        if let Ok(Some(v)) = get_setting("rename_template") { settings.rename_template = v; }
        if let Ok(Some(v)) = get_setting("theme") { settings.theme = v; }
        if let Ok(Some(v)) = get_setting("library_paths") {
            if let Ok(paths) = serde_json::from_str::<Vec<PathBuf>>(&v) { settings.library_paths = paths; }
        }
        if let Ok(Some(v)) = get_setting("ai_endpoint") { settings.ai_endpoint = v; }
        if let Ok(Some(v)) = get_setting("ai_api_key") { settings.ai_api_key = v; }
        if let Ok(Some(v)) = get_setting("ai_model") { settings.ai_model = v; }
        if let Ok(Some(v)) = get_setting("ai_temperature") {
            if let Ok(t) = v.parse::<f32>() { settings.ai_temperature = t; }
        }

        settings
    }
}
