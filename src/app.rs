use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use egui::Color32;
use rusqlite::Connection;

use crate::api::ai::{AiClient, AiConfig};
use crate::api::tmdb::TmdbClient;
use crate::config::settings::AppSettings;
use crate::core::library_manager::{LibraryManager, ScanProgress};
use crate::core::metadata_service::MetadataService;
use crate::core::{playback, subtitle_finder::SubtitleFinder};
use crate::db::models::{Subtitle, SubtitleQuery};
use crate::db::{connection, models::Movie, movies, settings as db_settings, subtitles as sub_db};
use crate::ui::{
    Rounding,
    add_movie::{AddMovieWizard, ImportJob, ImportSummary, WizardState},
    ai_chat_panel::AiChatPanel,
    ai_recommend_panel::AiRecommendPanel,
    batch_ops::BatchOpsPanel,
    layout::{AppLayout, View},
    movie_detail::{DetailAction, MovieDetailPanel},
    poster_wall::PosterWall,
    settings_panel::SettingsPanel,
    subtitle_panel::{DownloadJob, SearchJob, SubtitleAction, SubtitlePanel},
    theme,
    watchlist_panel::WatchlistPanel,
};

/// Toast notification
struct Toast {
    message: String,
    kind: ToastKind,
    created: std::time::Instant,
    duration: Duration,
}

#[derive(Clone, Copy)]
enum ToastKind {
    Info,
    Error,
}

/// Central application state, implements `eframe::App` for the main event loop.
///
/// Manages the database connection, AI/TMDB clients, UI panels, and library cache.
/// All blocking/network work runs on the tokio runtime and reports through
/// shared status objects polled once per frame.
pub struct MovieBoxApp {
    // Core
    app_data_dir: PathBuf,
    db: Connection,
    settings: AppSettings,
    tmdb_client: Arc<TmdbClient>,
    ai_client: Option<Arc<AiClient>>,
    thumbnail_dir: PathBuf,
    is_dark: bool,
    runtime: tokio::runtime::Runtime,

    // UI state
    layout: AppLayout,
    poster_wall: PosterWall,
    add_wizard: AddMovieWizard,
    subtitle_panel: SubtitlePanel,
    batch_ops: BatchOpsPanel,
    settings_panel: Option<SettingsPanel>,
    ai_chat_panel: AiChatPanel,
    ai_recommend_panel: AiRecommendPanel,
    detail_panel: MovieDetailPanel,

    // Detail
    selected_movie: Option<Movie>,
    show_detail: bool,

    // Notifications
    toasts: Vec<Toast>,

    // Cached data (avoid per-frame DB queries)
    cached_movie_count: i64,
    cached_library: Vec<Movie>,
    library_dirty: bool,
}

impl MovieBoxApp {
    pub fn new(app_data_dir: PathBuf) -> Self {
        log::info!("Starting AI Movie Player v{} · ifq.ai", env!("CARGO_PKG_VERSION"));
        log::info!("Data directory: {}", app_data_dir.display());

        let db = connection::open_database(&app_data_dir).expect("Failed to open database");
        let thumbnail_dir = app_data_dir.join("thumbnails");
        std::fs::create_dir_all(&thumbnail_dir).ok();

        let settings = AppSettings::load_from_db(&|key| db_settings::get_setting(&db, key));

        log::info!("Theme: {}, Language: {}", settings.theme, settings.tmdb_language);

        let tmdb_client = Arc::new(TmdbClient::new(
            settings.tmdb_api_key.clone(),
            settings.tmdb_language.clone(),
        ));

        let ai_client = Self::build_ai_client(&settings);
        if ai_client.is_some() {
            log::info!("AI configured: endpoint and model are set");
        } else {
            log::info!("AI not configured (no API key)");
        }

        let is_dark = settings.theme == "dark";

        let mut poster_wall = PosterWall::new();
        poster_wall.refresh(&db);

        let movie_count = movies::get_movie_count(&db).unwrap_or(0);
        log::info!("Library: {} movies loaded", movie_count);

        let cached_library = movies::get_all_movies(&db).unwrap_or_default();
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        Self {
            app_data_dir,
            db,
            thumbnail_dir,
            is_dark,
            layout: AppLayout::new(),
            poster_wall,
            add_wizard: AddMovieWizard::new(),
            subtitle_panel: SubtitlePanel::new(),
            batch_ops: BatchOpsPanel::new(),
            settings_panel: None,
            ai_chat_panel: AiChatPanel::new(),
            ai_recommend_panel: AiRecommendPanel::new(),
            detail_panel: MovieDetailPanel::new(),
            selected_movie: None,
            show_detail: false,
            settings,
            tmdb_client,
            ai_client,
            runtime,
            toasts: Vec::new(),
            cached_movie_count: movie_count,
            cached_library,
            library_dirty: false,
        }
    }

    fn build_ai_client(settings: &AppSettings) -> Option<Arc<AiClient>> {
        if settings.ai_api_key.is_empty() {
            return None;
        }
        Some(Arc::new(AiClient::new(AiConfig {
            endpoint: settings.ai_endpoint.clone(),
            api_key: settings.ai_api_key.clone(),
            model: settings.ai_model.clone(),
            temperature: settings.ai_temperature,
            max_tokens: 2048,
        })))
    }

    fn refresh_library_cache(&mut self) {
        self.cached_movie_count = movies::get_movie_count(&self.db).unwrap_or(0);
        self.cached_library = movies::get_all_movies(&self.db).unwrap_or_default();
        self.library_dirty = false;
    }

    fn close_detail(&mut self) {
        self.show_detail = false;
        self.selected_movie = None;
        self.poster_wall.selected_id = None;
    }

    fn navigate_to(&mut self, view: View) {
        self.layout.active_view = view;
        self.close_detail();
    }

    fn open_detail(&mut self, movie: Movie) {
        self.layout.active_view = View::Library;
        self.selected_movie = Some(movie);
        self.show_detail = true;
    }

    fn display_title(movie: &Movie) -> String {
        movie.title_cn.clone().unwrap_or_else(|| movie.title.clone())
    }

    fn open_local_movie(&mut self, movie: &Movie) {
        let Some(file_path) =
            movie.local_file_path.as_deref().filter(|path| !path.trim().is_empty())
        else {
            self.push_toast(
                "No local file path for this movie / 这部电影没有本地文件路径",
                ToastKind::Error,
            );
            return;
        };

        let path = PathBuf::from(file_path);
        if !path.exists() {
            self.push_toast("Movie file not found / 未找到本地影片文件", ToastKind::Error);
            return;
        }

        // Detached spawn: the UI thread must never wait on the player.
        match playback::open_in_system_player(&path) {
            Ok(()) => {
                self.push_toast("Opening in system player / 正在调用系统播放器", ToastKind::Info)
            }
            Err(err) => self.push_toast(
                format!("Could not open movie / 无法打开影片: {err}"),
                ToastKind::Error,
            ),
        }
    }

    fn reveal_movie(&mut self, movie: &Movie) {
        let Some(file_path) = movie.local_file_path.as_deref() else {
            return;
        };
        if let Err(err) = playback::reveal_in_file_manager(Path::new(file_path)) {
            self.push_toast(
                format!("Could not reveal file / 无法在文件夹中显示: {err}"),
                ToastKind::Error,
            );
        }
    }

    /// Native "locate moved file" flow, then repoint the database record.
    fn relocate_movie(&mut self, movie: &Movie) {
        let picked = rfd::FileDialog::new()
            .set_title("Locate movie file / 重新定位影片文件")
            .add_filter(
                "Video / 视频",
                &[
                    "mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts", "iso", "rmvb",
                    "mpg", "mpeg", "m2ts",
                ],
            )
            .pick_file();

        let Some(new_path) = picked else { return };

        let size = std::fs::metadata(&new_path).map(|m| m.len() as i64).ok();
        let hash = MetadataService::compute_file_hash(&new_path, 64 * 1024).ok();
        let path_str = new_path.to_string_lossy().to_string();

        match movies::update_movie_file(&self.db, movie.id, &path_str, size, hash) {
            Ok(()) => {
                if let Ok(Some(updated)) = movies::get_movie_by_id(&self.db, movie.id) {
                    self.selected_movie = Some(updated);
                }
                self.library_dirty = true;
                self.poster_wall.mark_dirty();
                self.push_toast("File location updated / 已更新文件位置", ToastKind::Info);
            }
            Err(err) => self.push_toast(
                format!("Failed to update file / 更新文件失败: {err}"),
                ToastKind::Error,
            ),
        }
    }

    /// Kick off the bounded-concurrency import pipeline on a worker DB connection.
    fn start_import(&mut self, files: Vec<PathBuf>) {
        let count = files.len();
        let progress = Arc::new(Mutex::new(ScanProgress::default()));
        self.add_wizard.begin_job(progress.clone(), count);

        if count == 0 {
            *self.add_wizard.import_job.lock().expect("import job mutex") =
                ImportJob::Finished(ImportSummary::default());
            return;
        }

        let job_state = self.add_wizard.import_job.clone();
        let client = self.tmdb_client.clone();
        let data_dir = self.app_data_dir.clone();
        let thumbnail_dir = self.thumbnail_dir.clone();

        self.runtime.spawn(async move {
            // The rusqlite connection is `Send` but not `Sync`, so it cannot be
            // held across awaits on a multi-thread runtime. Run the orchestrator
            // on a current-thread runtime inside a blocking thread: worker tasks
            // stay `Send`, while the connection never leaves this thread.
            let join = tokio::task::spawn_blocking(move || {
                let local = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| crate::util::error::AppError::Parse(err.to_string()))?;
                local.block_on(async {
                    let conn = connection::open_database(&data_dir)?;
                    LibraryManager::batch_scan_and_import(
                        client,
                        &conn,
                        files,
                        thumbnail_dir,
                        progress,
                    )
                    .await
                })
            })
            .await;

            let outcome = match join {
                Ok(inner) => inner,
                Err(err) => Err(crate::util::error::AppError::Parse(err.to_string())),
            };

            let mut job = job_state.lock().expect("import job mutex");
            match outcome {
                Ok(report) => {
                    let summary = ImportSummary {
                        imported: report.imported.len(),
                        unmatched: report.unmatched,
                        skipped: report.skipped,
                        failures: report
                            .failed
                            .into_iter()
                            .map(|failure| (failure.file, failure.reason))
                            .collect(),
                        titles: report.imported.iter().map(MovieBoxApp::display_title).collect(),
                    };
                    *job = ImportJob::Finished(summary);
                }
                Err(err) => *job = ImportJob::Failed(err.to_string()),
            }
        });
    }

    fn handle_subtitle_search(&mut self) {
        let Some(movie_id) = self.subtitle_panel.movie_id else { return };
        let Ok(Some(movie)) = movies::get_movie_by_id(&self.db, movie_id) else { return };

        let query = SubtitleQuery {
            title: movie.title.clone(),
            year: movie.year,
            file_hash: movie.file_hash.clone(),
            languages: self.settings.subtitle_languages.clone(),
            imdb_id: movie.imdb_id.clone(),
        };
        let tmdb_id = movie.tmdb_id;
        let key = self.settings.opensubtitles_api_key.trim().to_string();
        let job = self.subtitle_panel.search_job.clone();

        if key.is_empty() {
            *job.lock().expect("search job mutex") = SearchJob::Failed(
                "OpenSubtitles API Key 未配置，请先到设置中填写 / Configure an API key in Settings."
                    .into(),
            );
            return;
        }

        self.subtitle_panel.set_search_running();
        self.runtime.spawn(async move {
            let result = SubtitleFinder::search_all_sources(&query, Some(&key), tmdb_id).await;
            *job.lock().expect("search job mutex") = match result {
                Ok(results) => SearchJob::Done(results),
                Err(err) => SearchJob::Failed(err.to_string()),
            };
        });
    }

    fn handle_subtitle_download(&mut self, file_id: i64, language: String) {
        let Some(movie_id) = self.subtitle_panel.movie_id else { return };
        let Ok(Some(movie)) = movies::get_movie_by_id(&self.db, movie_id) else { return };

        let meta = self
            .subtitle_panel
            .search_results
            .iter()
            .find(|result| result.file_id == Some(file_id))
            .cloned();
        let key = self.settings.opensubtitles_api_key.trim().to_string();
        let local_path = movie.local_file_path.clone();
        let data_dir = self.app_data_dir.clone();
        let job = self.subtitle_panel.download_job.clone();

        *job.lock().expect("download job mutex") = DownloadJob::Running { file_id };

        self.runtime.spawn(async move {
            let movie_path = local_path.map(PathBuf::from);
            let outcome = SubtitleFinder::download_by_file_id(
                &key,
                file_id,
                movie_path.as_deref(),
                language.clone(),
            )
            .await;

            match outcome {
                Ok((saved, final_language)) => {
                    let file_name =
                        saved.file_name().map(|name| name.to_string_lossy().to_string());
                    let extension = saved.extension().map(|ext| ext.to_string_lossy().to_string());
                    let file_size = tokio::fs::metadata(&saved).await.ok().map(|m| m.len() as i64);

                    let mut error = false;
                    if let Some(meta) = meta {
                        let record = Subtitle {
                            id: 0,
                            movie_id,
                            language: final_language.clone(),
                            language_label: Some(meta.language_label),
                            source: meta.source,
                            source_url: None,
                            file_name: file_name.clone(),
                            local_path: Some(saved.to_string_lossy().to_string()),
                            file_size,
                            rating: meta.rating,
                            download_count: meta.download_count,
                            is_ai: meta.is_ai,
                            is_hearing_imp: meta.is_hearing_imp,
                            format: extension,
                            encoding: Some("UTF-8".into()),
                            sync_status: None,
                            download_date: String::new(),
                        };

                        let persisted = tokio::task::spawn_blocking(move || {
                            let conn = connection::open_database(&data_dir)?;
                            sub_db::insert_subtitle(&conn, &record)?;
                            Ok::<_, crate::util::error::AppError>(())
                        })
                        .await;
                        error = !matches!(persisted, Ok(Ok(())));
                        if let Err(err) = &persisted {
                            log::warn!("Failed to record subtitle: {err}");
                        }
                    }

                    let message = if error {
                        format!(
                            "字幕已下载但记录失败 / Saved but record failed: {}",
                            saved.display()
                        )
                    } else {
                        format!("字幕已保存 / Subtitle saved: {}", saved.display())
                    };
                    *job.lock().expect("download job mutex") = DownloadJob::Finished {
                        file_id,
                        saved: message,
                        language: final_language,
                        error,
                    };
                }
                Err(err) => {
                    *job.lock().expect("download job mutex") = DownloadJob::Finished {
                        file_id,
                        saved: format!("下载失败 / Download failed: {err}"),
                        language,
                        error: true,
                    };
                }
            }
        });
    }

    /// Drain finished subtitle search/download jobs into panel state.
    fn poll_subtitle_jobs(&mut self) {
        if let Some(outcome) = self.subtitle_panel.poll_search_finished() {
            match outcome {
                Ok(results) => {
                    let count = results.len();
                    self.subtitle_panel.search_results = results;
                    self.subtitle_panel.message =
                        Some((format!("找到 {count} 条字幕 / Found {count} subtitles"), false));
                }
                Err(err) => self.subtitle_panel.message = Some((err, true)),
            }
        }
        if let Some((_, outcome)) = self.subtitle_panel.poll_download_finished() {
            match outcome {
                Ok((message, _)) => self.push_toast(message, ToastKind::Info),
                Err(message) => self.push_toast(message, ToastKind::Error),
            }
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        // Ctrl+1..8 to switch views
        if input.modifiers.ctrl {
            if input.key_pressed(egui::Key::Num1) {
                self.navigate_to(View::Library);
            }
            if input.key_pressed(egui::Key::Num2) {
                self.navigate_to(View::AddMovie);
            }
            if input.key_pressed(egui::Key::Num3) {
                self.navigate_to(View::SubtitleSearch);
            }
            if input.key_pressed(egui::Key::Num4) {
                self.navigate_to(View::BatchOps);
            }
            if input.key_pressed(egui::Key::Num5) {
                self.navigate_to(View::Watchlist);
            }
            if input.key_pressed(egui::Key::Num6) {
                self.navigate_to(View::Settings);
            }
            if input.key_pressed(egui::Key::Num7) {
                self.navigate_to(View::AiChat);
            }
            if input.key_pressed(egui::Key::Num8) {
                self.navigate_to(View::AiRecommend);
            }
            if input.key_pressed(egui::Key::F) {
                self.layout.active_view = View::Library;
                self.show_detail = false;
                self.poster_wall.focus_search_requested = true;
            }
        }

        // Escape to go back from detail or to library
        if input.key_pressed(egui::Key::Escape) {
            if self.show_detail {
                self.close_detail();
            } else {
                self.navigate_to(View::Library);
            }
        }
    }

    fn push_toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        self.toasts.push(Toast {
            message: message.into(),
            kind,
            created: std::time::Instant::now(),
            duration: Duration::from_secs(3),
        });
    }

    fn render_toasts(&mut self, ctx: &egui::Context) {
        let now = std::time::Instant::now();
        self.toasts.retain(|t| now.duration_since(t.created) < t.duration);

        if self.toasts.is_empty() {
            return;
        }

        let mut y_offset = 60.0;

        for (i, toast) in self.toasts.iter().enumerate() {
            let elapsed = now.duration_since(toast.created).as_secs_f32();
            let duration = toast.duration.as_secs_f32();
            let remaining = (duration - elapsed).max(0.0);

            // Fade in/out
            let fade = if elapsed < 0.2 {
                elapsed / 0.2
            } else if remaining < 0.5 {
                remaining / 0.5
            } else {
                1.0
            };

            let (bg, icon) = match toast.kind {
                ToastKind::Info => (theme::surface_light_color(self.is_dark), "ℹ"),
                ToastKind::Error => (Color32::from_rgb(127, 29, 29), "!"),
            };

            egui::Area::new(egui::Id::new(format!("toast_{}", i)))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, y_offset))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let text_color =
                        Color32::from_rgba_premultiplied(255, 255, 255, (fade * 255.0) as u8);
                    let bg_color = Color32::from_rgba_premultiplied(
                        bg.r(),
                        bg.g(),
                        bg.b(),
                        (fade.clamp(0.0, 0.95) * 255.0) as u8,
                    );

                    let galley = ui.painter().layout_no_wrap(
                        format!("{}  {}", icon, toast.message),
                        egui::FontId::proportional(13.0),
                        text_color,
                    );
                    let padding = egui::vec2(16.0, 8.0);
                    let size = galley.size() + padding * 2.0;
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(ui.max_rect().center().x - size.x / 2.0, y_offset),
                        size,
                    );
                    ui.painter().rect_filled(rect, Rounding::same(8.0), bg_color);
                    ui.painter().galley(rect.min + padding, galley, text_color);
                });

            y_offset += 44.0;
        }
    }

    /// Whether background work needs a fast repaint cadence.
    fn is_busy(&self) -> bool {
        if self.add_wizard.is_running() || !self.toasts.is_empty() {
            return true;
        }
        if matches!(
            *self.subtitle_panel.search_job.lock().expect("search job mutex"),
            SearchJob::Running
        ) {
            return true;
        }
        matches!(
            *self.subtitle_panel.download_job.lock().expect("download job mutex"),
            DownloadJob::Running { .. }
        )
    }
}

impl eframe::App for MovieBoxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_theme(ctx, self.is_dark);

        // --- Keyboard shortcuts ---
        self.handle_keyboard_shortcuts(ctx);

        // --- Poll AI chat stream ---
        self.ai_chat_panel.poll_stream();

        // --- Native folder chooser: consumed once per button click, so a
        // cancelled dialog cannot re-open every frame. ---
        if self.add_wizard.take_folder_pick_request() {
            self.add_wizard.state = WizardState::SelectingFolder;
            match rfd::FileDialog::new().pick_folder() {
                Some(path) => {
                    let files = self.add_wizard.scan_folder(path, &self.db);
                    self.start_import(files);
                }
                None => self.add_wizard.state = WizardState::Idle,
            }
        }

        // --- Import completion ---
        if let Some(outcome) = self.add_wizard.poll_completion() {
            self.library_dirty = true;
            self.poster_wall.mark_dirty();
            match outcome {
                Ok(summary) => self.push_toast(
                    format!(
                        "导入完成 / Imported {} movies ({} unmatched)",
                        summary.imported, summary.unmatched
                    ),
                    ToastKind::Info,
                ),
                Err(err) => {
                    self.push_toast(format!("导入失败 / Import failed: {err}"), ToastKind::Error)
                }
            }
        }

        // --- Subtitle background jobs ---
        self.poll_subtitle_jobs();

        // Handle poster wall selection → show detail
        let selected_movie = if self.layout.active_view == View::Library && !self.show_detail {
            self.poster_wall
                .selected_id
                .and_then(|movie_id| movies::get_movie_by_id(&self.db, movie_id).ok().flatten())
        } else {
            None
        };

        if let Some(movie) = selected_movie {
            self.open_detail(movie);
        }

        // --- Refresh cached data if dirty ---
        if self.library_dirty {
            self.refresh_library_cache();
        }

        // --- Sidebar ---
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(200.0)
            .min_width(180.0)
            .show(ctx, |ui| {
                if let Some(view) =
                    self.layout.show_sidebar(ui, ctx, self.is_dark, self.cached_movie_count)
                {
                    self.navigate_to(view);
                }
            });

        // --- Content ---
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.layout.active_view {
                View::Library => {
                    if self.show_detail {
                        if let Some(movie) = self.selected_movie.clone() {
                            ui.horizontal(|ui| {
                                if ui.button("返回 / Back").clicked() {
                                    self.close_detail();
                                }
                                ui.heading(&movie.title);
                            });
                            ui.add_space(4.0);
                            let detail_action =
                                self.detail_panel.show(ui, &movie, &self.db, self.is_dark);

                            match detail_action {
                                DetailAction::OpenFile => self.open_local_movie(&movie),
                                DetailAction::RevealInFolder => self.reveal_movie(&movie),
                                DetailAction::RelocateFile => self.relocate_movie(&movie),
                                DetailAction::AiAnalyze => {
                                    self.ai_chat_panel.select_movie(Some(movie.clone()));
                                    self.navigate_to(View::AiChat);
                                }
                                DetailAction::SearchSubtitles => {
                                    self.subtitle_panel.select_movie(movie.id);
                                    self.navigate_to(View::SubtitleSearch);
                                }
                                DetailAction::None => {}
                            }
                        }
                    } else {
                        self.poster_wall.show(ui, ctx, &self.db, self.is_dark);
                    }
                }

                View::AddMovie => {
                    self.add_wizard.show(ui, self.is_dark);
                }

                View::SubtitleSearch => {
                    let key_present = !self.settings.opensubtitles_api_key.trim().is_empty();
                    let action = self.subtitle_panel.show(ui, &self.db, self.is_dark, key_present);

                    if self.subtitle_panel.take_search_request() {
                        self.handle_subtitle_search();
                    }

                    match action {
                        SubtitleAction::Download { file_id, language } => {
                            self.handle_subtitle_download(file_id, language);
                        }
                        SubtitleAction::OpenSettings => {
                            self.navigate_to(View::Settings);
                        }
                        SubtitleAction::None => {}
                    }
                }

                View::BatchOps => {
                    self.batch_ops.show(ui, self.is_dark);
                }

                View::Watchlist => {
                    WatchlistPanel::show(ui, &self.db, self.is_dark);
                }

                View::Settings => {
                    if self.settings_panel.is_none() {
                        self.settings_panel = Some(SettingsPanel::new(&self.db));
                    }
                    if let Some(ref mut panel) = self.settings_panel {
                        let old_is_dark = self.is_dark;
                        panel.show(ui, &self.db, self.is_dark);

                        if panel.is_dark != old_is_dark {
                            self.is_dark = panel.is_dark;
                        }

                        if panel.saved {
                            panel.saved = false;

                            // Reload persisted settings wholesale.
                            self.settings = AppSettings::load_from_db(&|key| {
                                db_settings::get_setting(&self.db, key)
                            });
                            self.is_dark = self.settings.theme == "dark";
                            self.tmdb_client = Arc::new(TmdbClient::new(
                                self.settings.tmdb_api_key.clone(),
                                self.settings.tmdb_language.clone(),
                            ));
                            self.ai_client = Self::build_ai_client(&self.settings);

                            self.push_toast("设置已保存 / Settings saved", ToastKind::Info);
                        }
                    }
                }

                View::AiChat => {
                    if self.ai_chat_panel.show(
                        ui,
                        &self.db,
                        &self.ai_client,
                        &self.cached_library,
                        &self.runtime,
                        self.is_dark,
                    ) {
                        self.navigate_to(View::Settings);
                    }
                }

                View::AiRecommend => {
                    self.ai_recommend_panel.show(
                        ui,
                        &self.ai_client,
                        &self.cached_library,
                        &self.runtime,
                        self.is_dark,
                    );
                }
            }
        });

        // Render toast notifications
        self.render_toasts(ctx);

        // Footer status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let tmdb_status = if !self.settings.tmdb_api_key.is_empty() {
                    "TMDB ✓"
                } else {
                    "TMDB 未设置 / not set"
                };
                ui.label(format!(
                    "AI Movie Player | {} movies | {} | ifq.ai",
                    self.cached_movie_count, tmdb_status
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dim = if self.is_dark {
                        Color32::from_rgb(100, 100, 115)
                    } else {
                        Color32::from_rgb(140, 140, 155)
                    };
                    ui.label(
                        egui::RichText::new("Ctrl+1-8 切换视图 / switch views | Esc 返回 / back | Ctrl+F 搜索 / search")
                            .size(11.0)
                            .color(dim),
                    );
                });
            });
        });

        // Adaptive frame cadence: fast while work is in flight or toasts are
        // visible, otherwise idle at 2s to avoid burning CPU/GPU on every frame.
        let repaint_delay =
            if self.is_busy() { Duration::from_millis(150) } else { Duration::from_secs(2) };
        ctx.request_repaint_after(repaint_delay);
    }
}
