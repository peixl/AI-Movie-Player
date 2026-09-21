//! Subtitle search and download interface.
//!
//! As with the import wizard, network work runs on the tokio runtime and
//! reports through shared status objects; this view only renders state and
//! emits one-shot actions for app.rs to handle.

use egui::{Color32, RichText, Ui};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::db::models::SubtitleResult;
use crate::db::{movies, subtitles as sub_db};

/// Background subtitle-search lifecycle.
#[derive(Clone)]
pub enum SearchJob {
    Idle,
    Running,
    Done(Vec<SubtitleResult>),
    Failed(String),
}

/// Background subtitle-download lifecycle, keyed by provider file id.
#[derive(Clone)]
pub enum DownloadJob {
    Idle,
    Running { file_id: i64 },
    Finished { file_id: i64, saved: String, language: String, error: bool },
}

/// Action emitted by the panel during a frame.
#[derive(Clone)]
pub enum SubtitleAction {
    None,
    Download { file_id: i64, language: String },
    OpenSettings,
}

/// Provider file id paired with Ok((saved path, language)) or an error message.
pub type DownloadOutcome = (i64, Result<(String, String), String>);

/// Subtitle panel state for search, results, and download tracking.
pub struct SubtitlePanel {
    pub movie_id: Option<i64>,
    pub search_results: Vec<SubtitleResult>,
    pub search_job: Arc<Mutex<SearchJob>>,
    pub download_job: Arc<Mutex<DownloadJob>>,
    search_requested: bool,
    pub message: Option<(String, bool)>,
}

impl SubtitlePanel {
    pub fn new() -> Self {
        Self {
            movie_id: None,
            search_results: Vec::new(),
            search_job: Arc::new(Mutex::new(SearchJob::Idle)),
            download_job: Arc::new(Mutex::new(DownloadJob::Idle)),
            search_requested: false,
            message: None,
        }
    }

    /// One-shot consumption of the "Search" button click.
    pub fn take_search_request(&mut self) -> bool {
        std::mem::take(&mut self.search_requested)
    }

    /// App-side: worker reports progress through this handle.
    pub fn set_search_running(&self) {
        *self.search_job.lock().expect("search job mutex") = SearchJob::Running;
    }

    /// App-side poll for finished searches (returns once per completion).
    pub fn poll_search_finished(&mut self) -> Option<Result<Vec<SubtitleResult>, String>> {
        let mut job = self.search_job.lock().expect("search job mutex");
        match std::mem::replace(&mut *job, SearchJob::Idle) {
            SearchJob::Done(results) => Some(Ok(results)),
            SearchJob::Failed(err) => Some(Err(err)),
            previous => {
                *job = previous;
                None
            }
        }
    }

    /// App-side poll for finished downloads (returns once per completion).
    pub fn poll_download_finished(&mut self) -> Option<DownloadOutcome> {
        let mut job = self.download_job.lock().expect("download job mutex");
        match &*job {
            DownloadJob::Finished { file_id, saved, language, error } => {
                let file_id = *file_id;
                let result =
                    if *error { Err(saved.clone()) } else { Ok((saved.clone(), language.clone())) };
                *job = DownloadJob::Idle;
                Some((file_id, result))
            }
            _ => None,
        }
    }

    fn downloading_id(&self) -> Option<i64> {
        match &*self.download_job.lock().expect("download job mutex") {
            DownloadJob::Running { file_id } => Some(*file_id),
            _ => None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        db: &Connection,
        is_dark: bool,
        api_key_present: bool,
    ) -> SubtitleAction {
        let mut action = SubtitleAction::None;
        let text =
            if is_dark { Color32::from_rgb(240, 240, 245) } else { Color32::from_rgb(15, 15, 25) };
        let dim = if is_dark {
            Color32::from_rgb(150, 150, 165)
        } else {
            Color32::from_rgb(100, 100, 115)
        };
        let primary = Color32::from_rgb(99, 102, 241);
        let warn = Color32::from_rgb(251, 146, 60);
        let good = Color32::from_rgb(52, 211, 153);
        let bad = Color32::from_rgb(248, 113, 113);

        ui.horizontal(|ui| {
            crate::ui::icons::draw_icon(ui, "subtitle", 22.0, primary);
            ui.add_space(8.0);
            ui.heading(RichText::new("Subtitle Search / 字幕搜索").size(22.0).color(text));
        });
        ui.add_space(12.0);

        // Movie info
        if let Some(mid) = self.movie_id {
            if let Ok(Some(movie)) = movies::get_movie_by_id(db, mid) {
                ui.label(
                    RichText::new(format!(
                        "Movie / 影片: {} ({})",
                        movie.title,
                        movie.year.map_or("?".into(), |y| y.to_string())
                    ))
                    .size(14.0)
                    .color(text),
                );

                let file_exists = movie
                    .local_file_path
                    .as_deref()
                    .is_some_and(|p| std::path::Path::new(p).exists());
                if !file_exists {
                    ui.label(
                        RichText::new("⚠ 本地文件不存在，字幕将下载到临时目录 / Local file missing; subtitle will go to a temp folder")
                            .size(11.0)
                            .color(warn),
                    );
                }

                // Show existing subtitles
                if let Ok(existing) = sub_db::get_subtitles_for_movie(db, mid) {
                    if !existing.is_empty() {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Already downloaded / 已下载:").size(12.0).color(dim),
                        );
                        for sub in &existing {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "  {} {} - {}",
                                    sub.language,
                                    sub.format.as_deref().unwrap_or("?"),
                                    sub.source
                                ));
                            });
                        }
                    }
                }
            }
        }

        ui.add_space(16.0);

        if !api_key_present {
            ui.label(
                RichText::new(
                    "需要 OpenSubtitles API Key（免费）/ A free OpenSubtitles API key is required.",
                )
                .color(warn),
            );
            if ui.button("前往设置 / Open Settings").clicked() {
                action = SubtitleAction::OpenSettings;
            }
            ui.add_space(8.0);
        }

        let searching =
            matches!(*self.search_job.lock().expect("search job mutex"), SearchJob::Running);
        let can_search = api_key_present && self.movie_id.is_some() && !searching;

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    can_search,
                    egui::Button::new(
                        RichText::new("🔍 Search OpenSubtitles / 搜索字幕").size(14.0),
                    ),
                )
                .clicked()
            {
                self.search_requested = true;
                self.message = None;
            }
            if searching {
                ui.spinner();
                ui.label(RichText::new("搜索中... / Searching...").color(dim));
            }
        });

        ui.add_space(8.0);

        // Results
        if !self.search_results.is_empty() {
            ui.label(format!(
                "找到 {} 条字幕结果 / Found {} subtitles:",
                self.search_results.len(),
                self.search_results.len()
            ));
            ui.add_space(4.0);

            let downloading = self.downloading_id();
            for result in &self.search_results {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&result.language_label).size(13.0).strong().color(text),
                        );
                        ui.label(
                            RichText::new(format!("来源 / {}", result.source))
                                .size(11.0)
                                .color(dim),
                        );
                        if let Some(rating) = result.rating {
                            ui.label(RichText::new(format!("★{rating:.1}")).size(11.0));
                        }
                        if let Some(downloads) = result.download_count {
                            ui.label(RichText::new(format!("⬇ {downloads}")).size(11.0).color(dim));
                        }
                        if result.is_hd {
                            ui.label(RichText::new("HD").size(10.0).color(good));
                        }
                        if result.is_ai {
                            ui.label(RichText::new("AI 翻译").size(10.0).color(warn));
                        }
                        if result.is_hearing_imp {
                            ui.label(RichText::new("听障 / HI").size(10.0).color(dim));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let busy = downloading == result.file_id;
                            let button = egui::Button::new(if busy {
                                "下载中... / Downloading"
                            } else {
                                "Download / 下载"
                            });
                            if ui
                                .add_enabled(
                                    result.file_id.is_some() && downloading.is_none(),
                                    button,
                                )
                                .clicked()
                            {
                                if let Some(file_id) = result.file_id {
                                    action = SubtitleAction::Download {
                                        file_id,
                                        language: result.language.clone(),
                                    };
                                }
                            }
                        });
                    });
                    ui.label(
                        RichText::new(format!(
                            "File / 文件: {} ({})",
                            result.file_name, result.format
                        ))
                        .size(11.0)
                        .color(dim),
                    );
                });
                ui.add_space(4.0);
            }
        }

        // Messages
        if let Some((msg, is_error)) = &self.message {
            ui.add_space(8.0);
            ui.label(RichText::new(msg).color(if *is_error { bad } else { good }));
        }

        action
    }

    pub fn select_movie(&mut self, movie_id: i64) {
        self.movie_id = Some(movie_id);
        self.search_results.clear();
        self.message = None;
        *self.search_job.lock().expect("search job mutex") = SearchJob::Idle;
        *self.download_job.lock().expect("download job mutex") = DownloadJob::Idle;
    }
}

impl Default for SubtitlePanel {
    fn default() -> Self {
        Self::new()
    }
}
