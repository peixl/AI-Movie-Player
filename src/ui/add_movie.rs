//! Movie import wizard: folder scanning, TMDB matching, and batch import.
//!
//! Long-running work executes on the tokio runtime; this view only renders
//! shared progress state (`Arc<Mutex<..>>`), so the UI thread never blocks and
//! no work result is dropped on the floor.

use egui::{Color32, ProgressBar, RichText, Ui};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::core::library_manager::ScanProgress;
use crate::db::models::Movie;

/// Multi-step movie import wizard with folder scanning and TMDB matching.
pub struct AddMovieWizard {
    pub state: WizardState,
    selected_folder: Option<PathBuf>,
    found_files: Vec<PathBuf>,
    imported_movies: Vec<Movie>,
    /// Set by the "Choose folder" button and consumed once per frame by app.rs,
    /// so a cancelled native dialog never re-opens on the next frame.
    request_folder_pick: bool,
    /// Shared import job state polled every frame.
    pub import_job: Arc<Mutex<ImportJob>>,
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

/// Background import lifecycle, shared with the worker task.
#[derive(Clone)]
pub enum ImportJob {
    Idle,
    Running { progress: Arc<Mutex<ScanProgress>> },
    Finished(ImportSummary),
    Failed(String),
}

#[derive(Debug, Clone, Default)]
pub struct ImportSummary {
    pub imported: usize,
    pub unmatched: usize,
    pub skipped: usize,
    pub failures: Vec<(String, String)>,
    pub titles: Vec<String>,
}

impl AddMovieWizard {
    pub fn new() -> Self {
        Self {
            state: WizardState::Idle,
            selected_folder: None,
            found_files: Vec::new(),
            imported_movies: Vec::new(),
            request_folder_pick: false,
            import_job: Arc::new(Mutex::new(ImportJob::Idle)),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Button-side request for the native folder chooser.
    pub fn request_folder_picker(&mut self) {
        self.request_folder_pick = true;
    }

    /// App-side one-shot consumption of the chooser request.
    pub fn take_folder_pick_request(&mut self) -> bool {
        std::mem::take(&mut self.request_folder_pick)
    }

    pub fn begin_job(&mut self, progress: Arc<Mutex<ScanProgress>>, file_count: usize) {
        self.found_files = Vec::new();
        self.imported_movies = Vec::new();
        *self.import_job.lock().expect("import job mutex") = ImportJob::Running { progress };
        self.state = if file_count == 0 { WizardState::Done } else { WizardState::SearchingTMDB };
    }

    /// Returns a finished summary exactly once, so the app can refresh the
    /// library and surface a single toast.
    pub fn poll_completion(&mut self) -> Option<Result<ImportSummary, String>> {
        let mut job = self.import_job.lock().expect("import job mutex");
        match &*job {
            ImportJob::Finished(summary) => {
                let summary = summary.clone();
                *job = ImportJob::Idle;
                self.state = WizardState::Done;
                Some(Ok(summary))
            }
            ImportJob::Failed(err) => {
                let err = err.clone();
                *job = ImportJob::Idle;
                self.state = WizardState::Done;
                Some(Err(err))
            }
            _ => None,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, WizardState::SearchingTMDB | WizardState::Importing)
    }

    pub fn show(&mut self, ui: &mut Ui, is_dark: bool) {
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

        if let Some(folder) = &self.selected_folder {
            ui.label(RichText::new(format!("📂 {}", folder.display())).size(12.0).color(dim));
            ui.add_space(8.0);
        }

        match self.state {
            WizardState::Idle | WizardState::SelectingFolder => {
                ui.label(RichText::new("选择一个包含电影文件的文件夹进行导入。样片、预告片、隐藏目录会自动跳过。 / Select a folder that contains movie files. Samples, trailers and hidden folders are skipped automatically.").size(14.0).color(dim));
                ui.add_space(16.0);

                if ui.button(RichText::new("📁 Choose Folder / 选择文件夹").size(16.0)).clicked()
                {
                    self.request_folder_picker();
                }
            }

            WizardState::Scanning => {
                ui.label("正在扫描文件夹中的视频文件... / Scanning for video files...");
                ui.spinner();
            }

            WizardState::SearchingTMDB | WizardState::Importing => {
                let job = self.import_job.lock().expect("import job mutex");
                if let ImportJob::Running { progress } = &*job {
                    let p = progress.lock().expect("progress mutex");
                    ui.label(
                        RichText::new("正在匹配 TMDB 并导入（未匹配的影片也会保留为本地记录）... / Matching TMDB and importing (unmatched files are kept as local records)...")
                            .size(14.0)
                            .color(text),
                    );
                    ui.add_space(10.0);

                    let ratio = if p.total > 0 { p.processed as f32 / p.total as f32 } else { 0.0 };
                    ui.add(ProgressBar::new(ratio).text(format!(
                        "{}/{} · ✓ {} · ⊘ {} 跳过 · {} 未匹配",
                        p.processed, p.total, p.imported, p.skipped, p.unmatched
                    )));
                    if let Some(current) = &p.current_file {
                        ui.label(RichText::new(current).size(11.0).color(dim));
                    }
                    if p.failed > 0 {
                        ui.label(
                            RichText::new(format!("{} 个失败 / failed", p.failed))
                                .size(11.0)
                                .color(Color32::from_rgb(248, 113, 113)),
                        );
                    }
                } else {
                    ui.spinner();
                    ui.label("准备中... / Preparing...");
                }
            }

            WizardState::ShowResults => {}

            WizardState::Done => {
                let summary = {
                    let job = self.import_job.lock().expect("import job mutex");
                    match &*job {
                        ImportJob::Finished(summary) => Some(summary.clone()),
                        _ => None,
                    }
                };

                if let Some(summary) = summary {
                    self.render_summary(ui, &summary, text, dim);
                } else {
                    ui.label(
                        RichText::new("没有发现新的视频文件。 / No new video files found.")
                            .size(14.0)
                            .color(dim),
                    );
                }

                ui.add_space(16.0);
                if ui.button("Import More / 继续导入").clicked() {
                    self.reset();
                }
                ui.add_space(4.0);
                if ui.button("Back to Library / 返回片库").clicked() {
                    self.reset();
                }
            }
        }
    }

    fn render_summary(
        &mut self,
        ui: &mut Ui,
        summary: &ImportSummary,
        text: Color32,
        dim: Color32,
    ) {
        ui.label(
            RichText::new(format!(
                "✓ 导入完成 / Import complete: {} 部（{} 部未匹配已保留），{} 跳过",
                summary.imported, summary.unmatched, summary.skipped
            ))
            .size(15.0)
            .color(Color32::from_rgb(52, 211, 153)),
        );
        ui.add_space(8.0);

        if !summary.failures.is_empty() {
            ui.label(
                RichText::new(format!("失败 / Failed: {}", summary.failures.len()))
                    .color(Color32::from_rgb(248, 113, 113)),
            );
            egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                for (file, reason) in &summary.failures {
                    ui.label(RichText::new(format!("• {file} — {reason}")).size(11.0).color(dim));
                }
            });
            ui.add_space(8.0);
        }

        if !summary.titles.is_empty() {
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                for title in &summary.titles {
                    ui.label(RichText::new(format!("✓ {title}")).size(12.0).color(text));
                }
            });
        }
    }

    /// Called from app.rs with the selected folder; returns new video files.
    pub fn scan_folder(&mut self, path: PathBuf, db: &rusqlite::Connection) -> Vec<PathBuf> {
        self.selected_folder = Some(path.clone());
        let mut files = crate::core::library_manager::LibraryManager::scan_folder(&path);
        files.retain(|f| {
            let path_str = f.to_string_lossy().to_string();
            !crate::db::movies::movie_exists_by_path(db, &path_str).unwrap_or(false)
        });
        self.found_files = files.clone();
        self.state = WizardState::Scanning;
        files
    }

    pub fn get_found_files(&self) -> &[PathBuf] {
        &self.found_files
    }
}

impl Default for AddMovieWizard {
    fn default() -> Self {
        Self::new()
    }
}
