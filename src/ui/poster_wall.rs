//! Visual poster grid browsing with LRU texture cache.

use egui::{Color32, Context, RichText, Sense, TextureHandle, Ui, Vec2};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use crate::db::models::MovieSummary;
use crate::db::movies;
use crate::ui::Rounding;

/// Decoded RGBA pixels produced by a background decoder thread.
#[derive(Clone)]
struct DecodedImage {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

/// Maximum on-disk decode width; posters never need more for the grid.
const DECODE_MAX_WIDTH: u32 = 400;

const POSTER_WIDTH: f32 = 160.0;
const POSTER_HEIGHT: f32 = 240.0;
const GAP: f32 = 12.0;
const TEXTURE_CACHE_MAX: usize = 200;

/// Sort order for the poster wall grid.
#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    DateAdded,
    Title,
    Year,
    Rating,
}

/// Filters applied to the poster wall (genre, year range, rating, search, sort).
#[derive(Debug, Clone)]
pub struct PosterFilter {
    pub genre: Option<String>,
    pub year_min: Option<i32>,
    pub year_max: Option<i32>,
    pub rating_min: Option<f64>,
    pub search: Option<String>,
    pub sort: SortOrder,
    pub ascending: bool,
}

impl Default for PosterFilter {
    fn default() -> Self {
        Self {
            genre: None,
            year_min: None,
            year_max: None,
            rating_min: None,
            search: None,
            sort: SortOrder::DateAdded,
            ascending: false,
        }
    }
}

pub struct PosterWall {
    pub movies: Vec<MovieSummary>,
    pub selected_id: Option<i64>,
    pub filter: PosterFilter,
    texture_cache: HashMap<i64, TextureHandle>,
    texture_lru: VecDeque<i64>,
    filter_dirty: bool,
    /// Finished background decodes awaiting upload to GPU textures.
    decoded: Arc<Mutex<HashMap<i64, DecodedImage>>>,
    /// Ids with a decoder thread already running (or finished, failed or not).
    decode_inflight: HashSet<i64>,
    /// One-shot Ctrl+F request to focus the search box.
    pub focus_search_requested: bool,
}

#[derive(Clone, Copy)]
struct PosterColors {
    text: Color32,
    dim: Color32,
    background: Color32,
    skeleton: Color32,
}

impl PosterWall {
    pub fn new() -> Self {
        Self {
            movies: Vec::new(),
            selected_id: None,
            filter: PosterFilter::default(),
            texture_cache: HashMap::new(),
            texture_lru: VecDeque::new(),
            filter_dirty: true,
            decoded: Arc::new(Mutex::new(HashMap::new())),
            decode_inflight: HashSet::new(),
            focus_search_requested: false,
        }
    }

    pub fn refresh(&mut self, db: &Connection) {
        let sort_str = match self.filter.sort {
            SortOrder::DateAdded => "added_date",
            SortOrder::Title => "title",
            SortOrder::Year => "year",
            SortOrder::Rating => "rating",
        };

        let genre_filter = self.filter.genre.as_deref();
        let search_filter = self.filter.search.as_deref();

        if let Ok(movies) = movies::get_all_movie_summaries(
            db,
            sort_str,
            self.filter.ascending,
            genre_filter,
            search_filter,
        ) {
            self.movies = movies;
        }
        self.filter_dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.filter_dirty = true;
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &Context, db: &Connection, is_dark: bool) {
        if self.filter_dirty {
            self.refresh(db);
        }

        // Filter bar
        self.show_filter_bar(ui);

        ui.add_space(8.0);

        // Poster grid
        let available = ui.available_size();
        let columns = ((available.x / (POSTER_WIDTH + GAP)) as usize).clamp(2, 8);
        let actual_poster_width = (available.x - GAP * (columns as f32 - 1.0)) / columns as f32;
        let actual_poster_height = actual_poster_width * (POSTER_HEIGHT / POSTER_WIDTH);

        let colors = PosterColors {
            text: if is_dark {
                Color32::from_rgb(240, 240, 245)
            } else {
                Color32::from_rgb(15, 15, 25)
            },
            dim: if is_dark {
                Color32::from_rgb(150, 150, 165)
            } else {
                Color32::from_rgb(100, 100, 115)
            },
            background: if is_dark {
                Color32::from_rgb(40, 40, 55)
            } else {
                Color32::from_rgb(230, 230, 240)
            },
            skeleton: crate::ui::animation::skeleton_poster_color(ctx, is_dark),
        };

        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let mut row = 0;
            let mut col = 0;

            for index in 0..self.movies.len() {
                if col == 0 {
                    ui.horizontal(|ui| {
                        ui.add_space(GAP);
                    });
                }

                let x = GAP + col as f32 * (actual_poster_width + GAP);
                let y = row as f32 * (actual_poster_height + GAP + 40.0); // 40 for title
                let poster_rect = egui::Rect::from_min_size(
                    egui::pos2(x, y),
                    Vec2::new(actual_poster_width, actual_poster_height),
                );

                // Only render visible posters
                if ui.is_rect_visible(poster_rect) {
                    let movie = self.movies[index].clone();
                    self.render_poster_card(
                        ui,
                        ctx,
                        &movie,
                        actual_poster_width,
                        actual_poster_height,
                        colors,
                    );
                }

                col += 1;
                if col >= columns {
                    col = 0;
                    row += 1;
                    ui.end_row();
                }
            }

            // Empty state
            if self.movies.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    crate::ui::icons::icon_empty_library(ui, 80.0, colors.dim);
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("库中暂无影片 / No movies yet").size(18.0).color(colors.dim),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("点击“导入影片”开始 / Click Import Movies to start")
                            .size(14.0)
                            .color(colors.dim),
                    );
                });
            }
        });
    }

    fn render_poster_card(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        movie: &MovieSummary,
        width: f32,
        height: f32,
        colors: PosterColors,
    ) {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, height + 40.0), Sense::click());

        if !ui.is_rect_visible(rect) {
            return;
        }

        let is_selected = self.selected_id == Some(movie.id);
        let hovered = response.hovered();

        // Animate hover scale
        let scale = ctx.animate_bool(egui::Id::new(format!("hover_{}", movie.id)), hovered);
        let scale_factor = 1.0 + scale * 0.03;

        // Poster image area with scale applied
        let poster_img_rect = egui::Rect::from_center_size(
            rect.center() - egui::vec2(0.0, 20.0),
            Vec2::new(width * scale_factor, height * scale_factor),
        );

        // Hover shadow
        let rounding = Rounding::same(6.0);
        if scale > 0.1 {
            let shadow_rect = poster_img_rect.expand(4.0 + scale * 8.0);
            let shadow_color = Color32::from_rgba_premultiplied(99, 102, 241, (scale * 40.0) as u8);
            ui.painter().rect_filled(shadow_rect, Rounding::same(10.0), shadow_color);
        }

        if let Some(ref local_path) = movie.poster_local {
            if let Some(texture) = self.texture_cache.get(&movie.id) {
                // Cached texture — mark as recently used
                if let Some(pos) = self.texture_lru.iter().position(|&id| id == movie.id) {
                    self.texture_lru.remove(pos);
                }
                self.texture_lru.push_back(movie.id);

                let mut mesh = egui::Mesh::with_texture(texture.id());
                mesh.add_rect_with_uv(
                    poster_img_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                ui.painter().add(mesh);
            } else {
                // Show shimmer while the background decoder works.
                ui.painter().rect_filled(poster_img_rect, rounding, colors.skeleton);

                // A decoder thread finished: upload pixels to a GPU texture on
                // the UI thread (the only legal place to touch egui textures).
                let finished = self.decoded.lock().expect("decoded mutex").remove(&movie.id);
                if let Some(decoded) = finished {
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [decoded.width, decoded.height],
                        &decoded.rgba,
                    );
                    let texture = ctx.load_texture(
                        format!("poster_{}", movie.id),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );
                    self.evict_lru_if_needed();
                    self.texture_cache.insert(movie.id, texture.clone());
                    self.texture_lru.push_back(movie.id);

                    let mut mesh = egui::Mesh::with_texture(texture.id());
                    mesh.add_rect_with_uv(
                        poster_img_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    ui.painter().add(mesh);
                } else {
                    self.request_decode(movie.id, local_path);
                }
            }
        } else {
            ui.painter().rect_filled(poster_img_rect, rounding, colors.background);
            // Draw text placeholder
            let galley = ui.painter().layout_no_wrap(
                "无海报 / No poster".into(),
                egui::FontId::proportional(13.0),
                colors.dim,
            );
            let pos = poster_img_rect.center() - galley.size() / 2.0;
            ui.painter().galley(pos, galley, colors.dim);
        }

        // Selection border
        if is_selected {
            ui.painter().rect_stroke(
                poster_img_rect.expand(2.0),
                rounding,
                egui::Stroke::new(2.0, Color32::from_rgb(99, 102, 241)),
                egui::StrokeKind::Middle,
            );
        }

        // Title below poster (use layout with max_width for automatic truncation)
        let title_y = poster_img_rect.max.y + 4.0;
        let title =
            if let Some(ref cn) = movie.title_cn { cn.clone() } else { movie.title.clone() };
        let title_galley =
            ui.painter().layout(title, egui::FontId::proportional(12.0), colors.text, width);
        ui.painter().galley(egui::pos2(rect.min.x, title_y), title_galley, colors.text);

        // Year + Rating on second line
        let info_y = title_y + 16.0;
        let mut info = String::new();
        if let Some(y) = movie.year {
            info.push_str(&y.to_string());
        }
        if let Some(r) = movie.rating {
            info.push_str(&format!("  ★{:.1}", r));
        }
        if let Some(ref res) = movie.resolution {
            info.push_str(&format!("  {}", res));
        }
        let info_galley =
            ui.painter().layout_no_wrap(info, egui::FontId::proportional(11.0), colors.dim);
        ui.painter().galley(egui::pos2(rect.min.x, info_y), info_galley, colors.dim);

        // Handle click
        if response.clicked() {
            self.selected_id = Some(movie.id);
        }
    }

    fn evict_lru_if_needed(&mut self) {
        while self.texture_cache.len() >= TEXTURE_CACHE_MAX {
            if let Some(oldest) = self.texture_lru.pop_front() {
                self.texture_cache.remove(&oldest);
            } else {
                break;
            }
        }
    }

    /// Spawn (once per id) a background thread that decodes and downscales the
    /// poster off the UI thread. Results are collected through `decoded`.
    fn request_decode(&mut self, id: i64, path: &str) {
        if self.decode_inflight.contains(&id) {
            return;
        }
        self.decode_inflight.insert(id);

        let decoded = self.decoded.clone();
        let path = path.to_string();
        std::thread::spawn(move || {
            if let Ok(img) = image::open(&path) {
                let thumb = img.thumbnail(DECODE_MAX_WIDTH, u32::MAX).to_rgba8();
                let (width, height) = thumb.dimensions();
                let image = DecodedImage {
                    width: width as usize,
                    height: height as usize,
                    rgba: thumb.into_raw(),
                };
                decoded.lock().expect("decoded mutex").insert(id, image);
            }
        });
    }

    fn show_filter_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Search
            let search_id = ui.make_persistent_id("poster_search_box");
            if self.focus_search_requested {
                self.focus_search_requested = false;
                ui.ctx().memory_mut(|memory| memory.request_focus(search_id));
            }
            let mut search = self.filter.search.clone().unwrap_or_default();
            let search_response = ui.add(
                egui::TextEdit::singleline(&mut search)
                    .id(search_id)
                    .hint_text("Search / 搜索 (Ctrl+F)...")
                    .desired_width(200.0),
            );
            if search_response.changed() {
                self.filter.search = if search.is_empty() { None } else { Some(search) };
                self.filter_dirty = true;
            }

            ui.separator();

            // Sort
            ui.label("Sort / 排序:");
            let mut sort_changed = false;
            sort_changed |= ui
                .selectable_value(&mut self.filter.sort, SortOrder::DateAdded, "最近添加 / Recent")
                .clicked();
            sort_changed |= ui
                .selectable_value(&mut self.filter.sort, SortOrder::Title, "片名 / Title")
                .clicked();
            sort_changed |= ui
                .selectable_value(&mut self.filter.sort, SortOrder::Year, "年份 / Year")
                .clicked();
            sort_changed |= ui
                .selectable_value(&mut self.filter.sort, SortOrder::Rating, "评分 / Rating")
                .clicked();

            if sort_changed {
                self.filter_dirty = true;
            }

            let asc_label = if self.filter.ascending { "↑" } else { "↓" };
            if ui.selectable_label(false, asc_label).clicked() {
                self.filter.ascending = !self.filter.ascending;
                self.filter_dirty = true;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let count = self.movies.len();
                ui.label(format!("{} 部影片 / {} movies", count, count));
            });
        });
    }
}

impl Default for PosterWall {
    fn default() -> Self {
        Self::new()
    }
}
