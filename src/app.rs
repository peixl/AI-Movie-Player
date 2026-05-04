use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use egui::Color32;
use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::api::ai::{AiClient, AiConfig};
use crate::api::tmdb::TmdbClient;
use crate::config::settings::AppSettings;
use crate::db::{connection, models::Movie, movies, settings as db_settings};
use crate::ui::{
    Rounding,
    add_movie::{AddMovieWizard, WizardState},
    ai_chat_panel::AiChatPanel,
    ai_recommend_panel::AiRecommendPanel,
    batch_ops::BatchOpsPanel,
    layout::{AppLayout, View},
    movie_detail::MovieDetailPanel,
    poster_wall::PosterWall,
    settings_panel::SettingsPanel,
    subtitle_panel::SubtitlePanel,
    theme,
    watchlist_panel::WatchlistPanel,
};

/// Toast notification
struct Toast {
    message: String,
    kind: ToastKind,
    created: std::time::Instant,
    duration: std::time::Duration,
}

#[derive(Clone, Copy)]
enum ToastKind {
    Info,
    Error,
}

/// Central application state, implements `eframe::App` for the main event loop.
///
/// Manages the database connection, AI/TMDB clients, UI panels, and library cache.
/// Re-renders every frame via egui's immediate-mode paradigm.
pub struct MovieBoxApp {
    // Core
    db: Connection,
    settings: AppSettings,
    tmdb_client: Arc<Mutex<TmdbClient>>,
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

    // Deferred work
    pending_import: Option<Vec<PathBuf>>,

    // Notifications
    toasts: Vec<Toast>,
    last_view: View,

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

        let tmdb_client = Arc::new(Mutex::new(TmdbClient::new(
            settings.tmdb_api_key.clone(),
            settings.tmdb_language.clone(),
        )));

        let ai_client = if !settings.ai_api_key.is_empty() {
            log::info!(
                "AI configured: endpoint={}, model={}",
                settings.ai_endpoint,
                settings.ai_model
            );
            Some(Arc::new(AiClient::new(AiConfig {
                endpoint: settings.ai_endpoint.clone(),
                api_key: settings.ai_api_key.clone(),
                model: settings.ai_model.clone(),
                temperature: settings.ai_temperature,
                max_tokens: 2048,
            })))
        } else {
            log::info!("AI not configured (no API key)");
            None
        };

        let is_dark = settings.theme == "dark";

        let mut poster_wall = PosterWall::new();
        poster_wall.refresh(&db);

        let movie_count = movies::get_movie_count(&db).unwrap_or(0);
        log::info!("Library: {} movies loaded", movie_count);

        let cached_library = movies::get_all_movies(&db).unwrap_or_default();
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        Self {
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
            pending_import: None,
            settings,
            tmdb_client,
            ai_client,
            runtime,
            toasts: Vec::new(),
            last_view: View::Library,
            cached_movie_count: movie_count,
            cached_library,
            library_dirty: false,
        }
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

        match open_with_system_player(&path) {
            Ok(()) => {
                self.push_toast("Opening in system player / 正在调用系统播放器", ToastKind::Info)
            }
            Err(err) => self.push_toast(
                format!("Could not open movie / 无法打开影片: {}", err),
                ToastKind::Error,
            ),
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        // Ctrl+1..6 to switch views
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
                self.navigate_to(View::Library);
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
            duration: std::time::Duration::from_secs(3),
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
                ToastKind::Info => (crate::ui::theme::surface_light_color(self.is_dark), "ℹ"),
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
}

impl eframe::App for MovieBoxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_theme(ctx, self.is_dark);

        // --- Keyboard shortcuts ---
        self.handle_keyboard_shortcuts(ctx);

        // --- View change detection ---
        let current_view = self.layout.active_view;
        if current_view != self.last_view {
            self.last_view = current_view;
            let view_name = match current_view {
                View::Library => "片库 / Library",
                View::AddMovie => "导入影片 / Add Movies",
                View::SubtitleSearch => "字幕 / Subtitles",
                View::BatchOps => "批量操作 / Batch Operations",
                View::Watchlist => "片单 / Watchlist",
                View::Settings => "设置 / Settings",
                View::AiChat => "AI 对话 / AI Companion",
                View::AiRecommend => "AI 推荐 / AI Taste Engine",
            };
            self.push_toast(format!("已切换到 / Switched to {}", view_name), ToastKind::Info);
        }

        // --- Poll AI chat stream ---
        self.ai_chat_panel.poll_stream();

        // --- Handle pending async work ---

        // Pending import after folder selection
        if let Some(files) = self.pending_import.take() {
            self.add_wizard.state = WizardState::SearchingTMDB;

            let client = self.tmdb_client.clone();

            self.runtime.spawn(async move {
                let client_guard = client.lock().await;

                for file in &files {
                    let filename = file
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let parsed = crate::core::filename_parser::parse_filename(&filename);

                    if let Ok(results) =
                        client_guard.search_movies(&parsed.title, parsed.year).await
                    {
                        log::info!("TMDB search results for {}: {} found", filename, results.len());
                    } else {
                        log::warn!("TMDB search failed for {}", filename);
                    }
                }
                // Results are stored via a channel or mutable state;
                // for now we handle this through the wizard state machine
            });
        }

        // Handle poster wall selection → show detail
        let selected_movie = if self.layout.active_view == View::Library {
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
                            let detail_action = self.detail_panel.show(ui, &movie, &self.db, self.is_dark);

                            match detail_action {
                                crate::ui::movie_detail::DetailAction::OpenFile => {
                                    self.open_local_movie(&movie);
                                }
                                crate::ui::movie_detail::DetailAction::AiAnalyze => {
                                    self.ai_chat_panel.select_movie(Some(movie.clone()));
                                    self.navigate_to(View::AiChat);
                                }
                                crate::ui::movie_detail::DetailAction::SearchSubtitles => {
                                    self.subtitle_panel.select_movie(movie.id);
                                    self.navigate_to(View::SubtitleSearch);
                                }
                                _ => {}
                            }
                        }
                    } else {
                        self.poster_wall.show(ui, ctx, &self.db, self.is_dark);
                    }
                }

                View::AddMovie => {
                    // Folder picker
                    if self.add_wizard.state == WizardState::SelectingFolder {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.add_wizard.scan_folder(path, &self.db);
                            let files = self.add_wizard.get_found_files().to_vec();
                            if !files.is_empty() {
                                self.add_wizard.state = WizardState::SearchingTMDB;
                                // Import is handled synchronously via block_on
                                // for simplicity (a few seconds max for a folder)
                                let client = self.tmdb_client.clone();
                                self.runtime.spawn(async move {
                                    let client = client.lock().await;
                                    for file in &files {
                                        let filename = file
                                            .file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_default();
                                        let parsed =
                                            crate::core::filename_parser::parse_filename(&filename);
                                        client.search_movies(&parsed.title, parsed.year).await.ok();
                                    }
                                });
                            }
                        }
                    }

                    // Import trigger
                    if self.add_wizard.state == WizardState::Importing {
                        let file_count = self.add_wizard.get_found_files().len();
                        self.add_wizard.add_log(format!(
                            "正在导入 {} 个文件，完成后请刷新片库 / Importing {} files, refresh the library after completion",
                            file_count,
                            file_count
                        ));
                        self.add_wizard.state = WizardState::Done;
                        // Refresh poster wall
                        self.poster_wall.mark_dirty();
                        self.library_dirty = true;
                    }

                    self.add_wizard.show(
                        ui,
                        &self.db,
                        &self.tmdb_client,
                        &self.thumbnail_dir,
                        self.is_dark,
                    );
                }

                View::SubtitleSearch => {
                    self.subtitle_panel.show(ui, &self.db, self.is_dark);

                    // Handle subtitle search trigger
                    let subtitle_query = if self.subtitle_panel.searching {
                        self.subtitle_panel.movie_id.and_then(|movie_id| {
                            movies::get_movie_by_id(&self.db, movie_id)
                                .ok()
                                .flatten()
                                .map(|movie| crate::db::models::SubtitleQuery {
                                    title: movie.title.clone(),
                                    year: movie.year,
                                    file_hash: movie.file_hash.clone(),
                                    languages: self.settings.subtitle_languages.clone(),
                                    imdb_id: movie.imdb_id.clone(),
                                })
                        })
                    } else {
                        None
                    };

                    if let Some(query) = subtitle_query {
                        self.subtitle_panel.searching = false;

                        self.runtime.spawn(async move {
                            match crate::core::subtitle_finder::SubtitleFinder::search_all_sources(&query)
                                .await
                            {
                                Ok(_results) => {
                                    // Store results — for now, mark as done
                                    // Full integration requires channel back to UI
                                }
                                Err(e) => {
                                    log::warn!("Subtitle search failed: {}", e);
                                }
                            }
                        });

                        self.subtitle_panel.message = Some(
                            "正在搜索 OpenSubtitles、assrt.net 与 zimuku... / Searching subtitle sources...".into(),
                        );
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
                            let new_key = panel.tmdb_key.clone();
                            let new_lang = panel.language.clone();
                            let client = self.tmdb_client.clone();

                            let ai_endpoint = panel.ai_endpoint.clone();
                            let ai_api_key = panel.ai_api_key.clone();
                            let ai_model = panel.ai_model.clone();
                            let ai_temperature = panel.ai_temperature;

                            self.runtime.spawn(async move {
                                let mut c = client.lock().await;
                                *c = TmdbClient::new(new_key, new_lang);
                            });

                            self.ai_client = if !ai_api_key.is_empty() {
                                Some(Arc::new(AiClient::new(AiConfig {
                                    endpoint: ai_endpoint,
                                    api_key: ai_api_key,
                                    model: ai_model,
                                    temperature: ai_temperature,
                                    max_tokens: 2048,
                                })))
                            } else {
                                None
                            };
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

        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

fn open_with_system_player(path: &Path) -> std::io::Result<()> {
    let status = system_open_command(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("system open command exited with {}", status)))
    }
}

#[cfg(target_os = "macos")]
fn system_open_command(path: &Path) -> Command {
    let mut command = Command::new("open");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
fn system_open_command(path: &Path) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", ""]).arg(path);
    command
}

#[cfg(all(unix, not(target_os = "macos")))]
fn system_open_command(path: &Path) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(path);
    command
}
