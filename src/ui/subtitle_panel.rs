use egui::{Color32, RichText, Ui};
use rusqlite::Connection;

use crate::db::models::SubtitleResult;
use crate::db::{movies, subtitles as sub_db};

pub struct SubtitlePanel {
    pub movie_id: Option<i64>,
    pub search_results: Vec<SubtitleResult>,
    pub searching: bool,
    pub downloading: bool,
    pub message: Option<String>,
}

impl SubtitlePanel {
    pub fn new() -> Self {
        Self {
            movie_id: None,
            search_results: Vec::new(),
            searching: false,
            downloading: false,
            message: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        db: &Connection,
        is_dark: bool,
    ) {
        let text = if is_dark { Color32::from_rgb(240, 240, 245) }
            else { Color32::from_rgb(15, 15, 25) };
        let dim = if is_dark { Color32::from_rgb(150, 150, 165) }
            else { Color32::from_rgb(100, 100, 115) };

        let primary = Color32::from_rgb(99, 102, 241);

        ui.horizontal(|ui| {
            crate::ui::icons::draw_icon(ui, "subtitle", 22.0, primary);
            ui.add_space(8.0);
            ui.heading(RichText::new("Subtitle Search / 字幕搜索").size(22.0).color(text));
        });
        ui.add_space(12.0);

        // Movie info
        if let Some(mid) = self.movie_id {
            if let Ok(Some(movie)) = movies::get_movie_by_id(db, mid) {
                ui.label(RichText::new(format!("Movie / 影片: {} ({})",
                    movie.title,
                    movie.year.map_or("?".into(), |y| y.to_string())
                )).size(14.0).color(text));

                // Show existing subtitles
                if let Ok(existing) = sub_db::get_subtitles_for_movie(db, mid) {
                    if !existing.is_empty() {
                        ui.add_space(8.0);
                        ui.label(RichText::new("Already downloaded / 已下载:").size(12.0).color(dim));
                        for sub in &existing {
                            ui.horizontal(|ui| {
                                ui.label(format!("  {} {} - {}",
                                    sub.language,
                                    sub.format.as_deref().unwrap_or("?"),
                                    sub.source));
                            });
                        }
                    }
                }
            }
        }

        ui.add_space(16.0);

        // Search trigger
        if ui.button(RichText::new("🔍 Search All Sources / 搜索全部来源").size(14.0)).clicked() {
            self.searching = true;
            self.search_results.clear();
        }

        if self.searching {
            ui.add_space(8.0);
            ui.spinner();
            ui.label("正在从多个来源搜索字幕... / Searching subtitles from multiple sources...");
        }

        ui.add_space(8.0);

        // Results
        if !self.search_results.is_empty() {
            ui.label(format!("找到 {} 条字幕结果 / Found {} subtitles:", self.search_results.len(), self.search_results.len()));
            ui.add_space(4.0);

            for result in &self.search_results {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&result.language_label).size(13.0).strong());
                        ui.label(format!("from / 来源 {}", result.source));
                        if let Some(rating) = result.rating {
                            ui.label(format!("★{:.1}", rating));
                        }
                        if result.is_ai {
                            ui.label(RichText::new("AI 优选").color(Color32::from_rgb(251, 146, 60)));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Download / 下载").clicked() {
                                self.downloading = true;
                            }
                        });
                    });
                    ui.label(format!("File / 文件: {} ({})", result.file_name, result.format));
                });
                ui.add_space(4.0);
            }
        }

        // Messages
        if let Some(ref msg) = self.message {
            ui.add_space(8.0);
            ui.label(RichText::new(msg).color(Color32::from_rgb(52, 211, 153)));
        }
    }

    pub fn select_movie(&mut self, movie_id: i64) {
        self.movie_id = Some(movie_id);
        self.search_results.clear();
        self.message = None;
    }
}
