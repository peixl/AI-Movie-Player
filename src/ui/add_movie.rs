//! Movie import wizard: folder scanning, TMDB matching, and batch import.

use egui::{Color32, RichText, Ui};
use rusqlite::Connection;
use std::path::PathBuf;

use crate::api::tmdb::TmdbClient;
use crate::core::library_manager::LibraryManager;
use crate::db::models::{Movie, TmdbSearchResult};

/// Multi-step movie import wizard with folder scanning and TMDB matching.
pub struct AddMovieWizard {
    pub state: WizardState,
    selected_folder: Option<PathBuf>,
    found_files: Vec<PathBuf>,
    search_results: Vec<(PathBuf, Vec<TmdbSearchResult>)>,
    found_count: usize,
    imported_movies: Vec<Movie>,
    log_messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WizardState {
    Idle,
    SelectingFolder,
    Scanning,
    SearchingTMDB,
    ShowResults,
    Importing,
    Done,
}

impl AddMovieWizard {
    pub fn new() -> Self {
        Self {
            state: WizardState::Idle,
            selected_folder: None,
            found_files: Vec::new(),
            search_results: Vec::new(),
            found_count: 0,
            imported_movies: Vec::new(),
            log_messages: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        _db: &Connection,
        _client: &std::sync::Arc<tokio::sync::Mutex<TmdbClient>>,
        _thumbnail_dir: &PathBuf,
        is_dark: bool,
    ) {
        let text =
            if is_dark { Color32::from_rgb(240, 240, 245) } else { Color32::from_rgb(15, 15, 25) };
        let dim = if is_dark {
            Color32::from_rgb(150, 150, 165)
        } else {
            Color32::from_rgb(100, 100, 115)
        };
        let primary = Color32::from_rgb(99, 102, 241);

        ui.horizontal(|ui| {
            crate::ui::icons::draw_icon(ui, "add-folder", 22.0, primary);
            ui.add_space(8.0);
            ui.heading(RichText::new("Import Movies / 导入影片").size(22.0).color(text));
        });
        ui.add_space(12.0);

        match self.state {
            WizardState::Idle => {
                ui.label(RichText::new("选择一个包含电影文件的文件夹进行导入。 / Select a folder that contains movie files.").size(14.0).color(dim));
                ui.add_space(16.0);

                if ui.button(RichText::new("📁 Choose Folder / 选择文件夹").size(16.0)).clicked()
                {
                    self.state = WizardState::SelectingFolder;
                }
            }

            WizardState::SelectingFolder => {
                ui.label("正在打开文件夹选择器 / Opening folder chooser...");
                // Folder picking is handled by app.rs via rfd
                // This state is just a visual transition
            }

            WizardState::Scanning => {
                ui.label("正在扫描文件夹中的视频文件... / Scanning for video files...");
                ui.add_space(8.0);
                ui.label(format!(
                    "发现 {} 个视频文件 / Found {} video files",
                    self.found_files.len(),
                    self.found_files.len()
                ));

                if !self.found_files.is_empty() {
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for file in &self.found_files {
                            ui.label(
                                file.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default(),
                            );
                        }
                    });
                }
            }

            WizardState::SearchingTMDB => {
                ui.label("正在从 TMDB 匹配影片信息... / Searching TMDB for matches...");
                ui.add_space(8.0);
                ui.spinner();
            }

            WizardState::ShowResults => {
                ui.label(format!(
                    "找到 {} 个待确认结果 / Review the matches below:",
                    self.found_files.len()
                ));
                ui.add_space(12.0);

                for (file, results) in &self.search_results {
                    let filename = file
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    ui.group(|ui| {
                        ui.label(RichText::new(&filename).strong());
                        ui.add_space(4.0);

                        if results.is_empty() {
                            ui.label(
                                RichText::new("未找到匹配结果 / No matching results")
                                    .color(Color32::from_rgb(248, 113, 113)),
                            );
                        } else {
                            for result in results.iter().take(5) {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "{} ({})",
                                        result.title,
                                        result.year.map_or("?".into(), |y| y.to_string())
                                    ));
                                    if let Some(r) = result.rating {
                                        ui.label(format!("★{:.1}", r));
                                    }
                                });
                            }
                        }
                    });
                    ui.add_space(4.0);
                }

                ui.add_space(16.0);
                if ui
                    .button(RichText::new("Import All / 全部导入").size(14.0).color(Color32::WHITE))
                    .clicked()
                {
                    self.state = WizardState::Importing;
                }
            }

            WizardState::Importing => {
                ui.label("正在导入影片... / Importing movies...");
                ui.add_space(8.0);
                ui.spinner();
                ui.add_space(8.0);

                // Show log
                for msg in &self.log_messages {
                    ui.label(RichText::new(msg).size(12.0).color(dim));
                }
            }

            WizardState::Done => {
                ui.label(
                    RichText::new(format!(
                        "导入完成：{} 部影片 / Import complete: {} movies",
                        self.imported_movies.len(),
                        self.imported_movies.len()
                    ))
                    .size(16.0)
                    .color(Color32::from_rgb(52, 211, 153)),
                );
                ui.add_space(12.0);

                if !self.imported_movies.is_empty() {
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        for movie in &self.imported_movies {
                            ui.horizontal(|ui| {
                                ui.label(format!("✓ {}", movie.title));
                                if let Some(y) = movie.year {
                                    ui.label(format!("({})", y));
                                }
                            });
                        }
                    });
                }

                ui.add_space(16.0);
                if ui.button("Back to Library / 返回片库").clicked() {
                    self.reset();
                }
            }
        }
    }

    // Called from app.rs with the selected folder
    pub fn scan_folder(&mut self, path: PathBuf, db: &Connection) {
        self.selected_folder = Some(path.clone());
        self.found_files = LibraryManager::scan_folder(&path);
        self.found_files.retain(|f| {
            let path_str = f.to_string_lossy().to_string();
            !crate::db::movies::movie_exists_by_path(db, &path_str).unwrap_or(false)
        });
        self.found_count = self.found_files.len();
        self.state = WizardState::Scanning;
    }

    pub fn set_found(&mut self, count: usize) {
        self.found_count = count;
    }

    pub fn get_found_files(&self) -> &[PathBuf] {
        &self.found_files
    }

    pub fn set_search_results(&mut self, results: Vec<(PathBuf, Vec<TmdbSearchResult>)>) {
        self.search_results = results;
        self.state = WizardState::ShowResults;
    }

    pub fn add_log(&mut self, msg: String) {
        self.log_messages.push(msg);
    }

    pub fn set_import_done(&mut self, movies: Vec<Movie>) {
        self.imported_movies = movies;
        self.state = WizardState::Done;
    }
}
