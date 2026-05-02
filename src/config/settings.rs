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

        if let Ok(Some(v)) = get_setting("tmdb_api_key") {
            settings.tmdb_api_key = v;
        }
        if let Ok(Some(v)) = get_setting("tmdb_language") {
            settings.tmdb_language = v;
        }
        if let Ok(Some(v)) = get_setting("subtitle_languages") {
            if let Ok(langs) = serde_json::from_str::<Vec<String>>(&v) {
                settings.subtitle_languages = langs;
            }
        }
        if let Ok(Some(v)) = get_setting("rename_template") {
            settings.rename_template = v;
        }
        if let Ok(Some(v)) = get_setting("theme") {
            settings.theme = v;
        }
        if let Ok(Some(v)) = get_setting("library_paths") {
            if let Ok(paths) = serde_json::from_str::<Vec<PathBuf>>(&v) {
                settings.library_paths = paths;
            }
        }
        if let Ok(Some(v)) = get_setting("auto_scan") {
            if let Ok(enabled) = v.parse::<bool>() {
                settings.auto_scan = enabled;
            }
        }
        if let Ok(Some(v)) = get_setting("ai_endpoint") {
            settings.ai_endpoint = v;
        }
        if let Ok(Some(v)) = get_setting("ai_api_key") {
            settings.ai_api_key = v;
        }
        if let Ok(Some(v)) = get_setting("ai_model") {
            settings.ai_model = v;
        }
        if let Ok(Some(v)) = get_setting("ai_temperature") {
            if let Ok(t) = v.parse::<f32>() {
                settings.ai_temperature = t;
            }
        }

        settings
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::util::error::AppError;

    use super::AppSettings;

    fn load_with(entries: &[(&str, &str)]) -> AppSettings {
        let values: HashMap<String, String> =
            entries.iter().map(|(key, value)| ((*key).to_string(), (*value).to_string())).collect();

        AppSettings::load_from_db(&|key| Ok(values.get(key).cloned()))
    }

    #[test]
    fn load_from_db_reads_all_supported_fields_including_auto_scan() {
        let settings = load_with(&[
            ("tmdb_api_key", "tmdb-key"),
            ("tmdb_language", "en-US"),
            ("subtitle_languages", r#"["ja","en"]"#),
            ("rename_template", "{title} [{year}]"),
            ("theme", "light"),
            ("library_paths", r#"["C:\\Movies","D:\\Anime"]"#),
            ("auto_scan", "true"),
            ("ai_endpoint", "http://localhost:11434/v1"),
            ("ai_api_key", "local-key"),
            ("ai_model", "qwen3"),
            ("ai_temperature", "0.25"),
        ]);

        assert_eq!(settings.tmdb_api_key, "tmdb-key");
        assert_eq!(settings.tmdb_language, "en-US");
        assert_eq!(settings.subtitle_languages, vec!["ja", "en"]);
        assert_eq!(settings.rename_template, "{title} [{year}]");
        assert_eq!(settings.theme, "light");
        assert_eq!(settings.library_paths.len(), 2);
        assert!(settings.auto_scan);
        assert_eq!(settings.ai_endpoint, "http://localhost:11434/v1");
        assert_eq!(settings.ai_api_key, "local-key");
        assert_eq!(settings.ai_model, "qwen3");
        assert!((settings.ai_temperature - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn load_from_db_keeps_defaults_for_invalid_serialized_values() {
        let defaults = AppSettings::default();
        let settings = load_with(&[
            ("subtitle_languages", "not-json"),
            ("library_paths", "not-json"),
            ("auto_scan", "not-bool"),
            ("ai_temperature", "not-a-number"),
        ]);

        assert_eq!(settings.subtitle_languages, defaults.subtitle_languages);
        assert_eq!(settings.library_paths, defaults.library_paths);
        assert_eq!(settings.auto_scan, defaults.auto_scan);
        assert!((settings.ai_temperature - defaults.ai_temperature).abs() < f32::EPSILON);
    }

    #[test]
    fn load_from_db_ignores_storage_errors_and_uses_defaults() {
        let defaults = AppSettings::default();

        let settings =
            AppSettings::load_from_db(&|_| Err(AppError::Config("db unavailable".into())));

        assert_eq!(settings.tmdb_language, defaults.tmdb_language);
        assert_eq!(settings.rename_template, defaults.rename_template);
        assert_eq!(settings.auto_scan, defaults.auto_scan);
        assert_eq!(settings.ai_model, defaults.ai_model);
    }
}
