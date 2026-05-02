//! Settings panel for TMDB, AI provider, theme, and rename template configuration.

use egui::{Color32, RichText, Ui};
use rusqlite::Connection;

use crate::config::settings::AppSettings;
use crate::db::settings as db_settings;

/// Settings panel state with editable fields for all configuration options.
pub struct SettingsPanel {
    pub tmdb_key: String,
    pub language: String,
    pub rename_template: String,
    pub is_dark: bool,
    pub saved: bool,
    pub ai_endpoint: String,
    pub ai_api_key: String,
    pub ai_model: String,
    pub ai_temperature: f32,
}

impl SettingsPanel {
    pub fn new(db: &Connection) -> Self {
        let settings = AppSettings::load_from_db(&|key| {
            db_settings::get_setting(db, key).map_err(|e| crate::util::error::AppError::Database(e))
        });

        Self {
            tmdb_key: settings.tmdb_api_key,
            language: settings.tmdb_language,
            rename_template: settings.rename_template,
            is_dark: settings.theme == "dark",
            saved: false,
            ai_endpoint: settings.ai_endpoint,
            ai_api_key: settings.ai_api_key,
            ai_model: settings.ai_model,
            ai_temperature: settings.ai_temperature,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, db: &Connection, is_dark: bool) {
        let text = crate::ui::theme::text_color(is_dark);
        let dim = crate::ui::theme::dim_color(is_dark);
        let primary = crate::ui::theme::primary_color();

        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        ui.horizontal(|ui| {
            crate::ui::icons::draw_icon(ui, "gear", 22.0, primary);
            ui.add_space(8.0);
            ui.heading(RichText::new("Settings / 设置").size(22.0).color(text));
        });
        ui.add_space(16.0);

        // TMDB API Key
        ui.label(RichText::new("TMDB API Key / TMDB 密钥").size(14.0).strong().color(text));
        ui.label(RichText::new("获取影片的元数据、海报和演职表。 / Fetch movie metadata, posters, and cast information.")
            .size(12.0).color(dim));
        let key_changed = ui.add(
            egui::TextEdit::singleline(&mut self.tmdb_key)
                .hint_text("Enter your TMDB API key / 输入 TMDB API Key...")
                .password(true)
                .desired_width(f32::INFINITY)
        ).changed();
        if key_changed { self.saved = false; }

        ui.add_space(16.0);

        // Language
        ui.label(RichText::new("Metadata Language / 元数据语言").size(14.0).strong().color(text));
        let lang_changed = egui::ComboBox::from_id_salt("lang")
            .selected_text(&self.language)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.language, "zh-CN".into(), "简体中文 / Chinese");
                ui.selectable_value(&mut self.language, "en-US".into(), "English / 英文");
            })
            .response.changed();
        if lang_changed { self.saved = false; }

        ui.add_space(16.0);

        // Rename template
        ui.label(RichText::new("File Rename Template / 文件重命名模板").size(14.0).strong().color(text));
        ui.label(RichText::new("Available variables / 可用变量: {title}, {year}, {resolution}, {source}, {codec}")
            .size(12.0).color(dim));
        let template_changed = ui.add(
            egui::TextEdit::singleline(&mut self.rename_template)
                .desired_width(f32::INFINITY)
        ).changed();
        if template_changed { self.saved = false; }

        ui.add_space(16.0);

        // ---- AI Settings ----
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            crate::ui::icons::draw_icon(ui, "sparkle", 20.0, primary);
            ui.add_space(6.0);
            ui.label(RichText::new("AI 设置 / AI Settings (OpenAI 兼容)").size(16.0).strong().color(text));
        });
        ui.label(
            RichText::new("兼容 OpenAI、Azure、Ollama、LM Studio 与其他 OpenAI 兼容接口。 / Works with OpenAI, Azure, Ollama, LM Studio, and other OpenAI-compatible APIs.")
                .size(11.0)
                .color(dim),
        );
        ui.add_space(8.0);

        // AI Endpoint
        ui.label(RichText::new("API Endpoint / 接口地址").size(13.0).strong().color(text));
        let ep_changed = ui.add(
            egui::TextEdit::singleline(&mut self.ai_endpoint)
                .hint_text("https://api.openai.com/v1")
                .desired_width(f32::INFINITY),
        ).changed();
        if ep_changed { self.saved = false; }
        ui.add_space(8.0);

        // AI API Key
        ui.label(RichText::new("API Key / 密钥").size(13.0).strong().color(text));
        let key_changed_ai = ui.add(
            egui::TextEdit::singleline(&mut self.ai_api_key)
                .hint_text("sk-...")
                .password(true)
                .desired_width(f32::INFINITY),
        ).changed();
        if key_changed_ai { self.saved = false; }
        ui.add_space(8.0);

        // AI Model
        ui.label(RichText::new("Model / 模型").size(13.0).strong().color(text));
        let model_changed = ui.add(
            egui::TextEdit::singleline(&mut self.ai_model)
                .hint_text("gpt-4o-mini")
                .desired_width(240.0),
        ).changed();
        if model_changed { self.saved = false; }
        ui.add_space(8.0);

        // Temperature
        ui.label(RichText::new("Temperature / 温度").size(13.0).strong().color(text));
        ui.label(RichText::new("0 = 更稳定精确，1 = 更发散灵感 / 0 = precise, 1 = creative").size(11.0).color(dim));
        let temp_changed = ui.add(
            egui::Slider::new(&mut self.ai_temperature, 0.0..=2.0)
                .step_by(0.1)
                .text(""),
        ).changed();
        if temp_changed { self.saved = false; }
        ui.add_space(8.0);

        // Provider quick-select
        ui.label(RichText::new("Quick Setup / 快速预设").size(12.0).color(dim));
        ui.horizontal_wrapped(|ui| {
            if ui.small_button("OpenAI").clicked() {
                self.ai_endpoint = "https://api.openai.com/v1".into();
                self.ai_model = "gpt-4o-mini".into();
                self.saved = false;
            }
            if ui.small_button("Ollama").clicked() {
                self.ai_endpoint = "http://localhost:11434/v1".into();
                self.ai_model = "llama3".into();
                self.saved = false;
            }
            if ui.small_button("LM Studio").clicked() {
                self.ai_endpoint = "http://localhost:1234/v1".into();
                self.ai_model = "local-model".into();
                self.saved = false;
            }
        });

        // Theme
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("Appearance / 外观").size(14.0).strong().color(text));
        let theme_changed = ui.horizontal(|ui| {
            let mut changed = false;
            changed |= ui.selectable_value(&mut self.is_dark, true, "🌙 深色 / Dark").clicked();
            changed |= ui.selectable_value(&mut self.is_dark, false, "☀️ 浅色 / Light").clicked();
            changed
        }).inner;
        if theme_changed { self.saved = false; }

        ui.add_space(24.0);

        // Save button
        if ui.button(RichText::new("Save Settings / 保存设置").size(14.0).color(Color32::WHITE)).clicked() {
            self.save(db);
        }

        if self.saved {
            ui.add_space(8.0);
            ui.label(RichText::new("✓ 设置已保存 / Settings saved").color(Color32::from_rgb(52, 211, 153)));
        }
        }); // close ScrollArea
    }

    fn save(&mut self, db: &Connection) {
        db_settings::set_setting(db, "tmdb_api_key", &self.tmdb_key).ok();
        db_settings::set_setting(db, "tmdb_language", &self.language).ok();
        db_settings::set_setting(db, "rename_template", &self.rename_template).ok();
        db_settings::set_setting(db, "theme", if self.is_dark { "dark" } else { "light" }).ok();
        db_settings::set_setting(db, "ai_endpoint", &self.ai_endpoint).ok();
        db_settings::set_setting(db, "ai_api_key", &self.ai_api_key).ok();
        db_settings::set_setting(db, "ai_model", &self.ai_model).ok();
        db_settings::set_setting(db, "ai_temperature", &self.ai_temperature.to_string()).ok();
        self.saved = true;
    }
}
