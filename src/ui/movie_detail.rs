//! Movie detail panel. Posters load asynchronously through egui_extras' image
//! loader, so opening a detail view never blocks the UI on image decoding.

use egui::{Color32, RichText, Ui, Vec2};
use rusqlite::Connection;

use crate::db::{
    models::{CastMember, Movie},
    subtitles as sub_db, watchlist,
};
use crate::ui::Rounding;

/// Actions triggered from the movie detail panel.
#[derive(PartialEq)]
pub enum DetailAction {
    None,
    OpenFile,
    RevealInFolder,
    RelocateFile,
    SearchSubtitles,
    AiAnalyze,
}

/// Stateless movie detail panel (poster textures are owned by the egui loader).
pub struct MovieDetailPanel;

impl MovieDetailPanel {
    pub fn new() -> Self {
        Self
    }

    /// Show movie detail. Returns the action the user wants to take.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        movie: &Movie,
        db: &Connection,
        is_dark: bool,
    ) -> DetailAction {
        let mut action = DetailAction::None;
        let text = crate::ui::theme::text_color(is_dark);
        let dim = crate::ui::theme::dim_color(is_dark);
        let primary = crate::ui::theme::primary_color();
        let bad = Color32::from_rgb(248, 113, 113);

        let local_path = movie.local_file_path.as_deref().filter(|path| !path.trim().is_empty());
        let file_exists = local_path.is_some_and(|p| std::path::Path::new(p).exists());

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Title + primary actions
            ui.horizontal(|ui| {
                ui.heading(RichText::new(&movie.title).size(22.0).color(text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let open_btn = egui::Button::new(
                        RichText::new("Open / 播放").size(13.0).color(Color32::WHITE),
                    )
                    .fill(primary)
                    .corner_radius(Rounding::same(6.0));
                    if ui.add_enabled(file_exists, open_btn).clicked() {
                        action = DetailAction::OpenFile;
                    }

                    let ai_btn = egui::Button::new(
                        RichText::new("AI Insight / AI 解析").size(13.0).color(Color32::WHITE),
                    )
                    .fill(crate::ui::theme::surface_light_color(is_dark))
                    .corner_radius(Rounding::same(6.0));
                    if ui.add(ai_btn).clicked() {
                        action = DetailAction::AiAnalyze;
                    }
                });
            });
            if let Some(ref cn) = movie.title_cn {
                ui.label(RichText::new(cn).size(16.0).color(dim));
            }

            if !file_exists {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("⚠ 本地文件丢失或已移动 / Local file is missing or was moved")
                        .size(13.0)
                        .color(bad),
                );
            }

            ui.add_space(12.0);

            // Poster + basic info
            ui.horizontal(|ui| {
                // Poster loads off the UI thread via the egui_extras image loader.
                if let Some(ref poster_path) = movie.poster_local {
                    let uri = format!("file://{}", poster_path);
                    ui.add(
                        egui::Image::new(uri)
                            .fit_to_fraction(Vec2::new(1.0, 1.0))
                            .max_size(Vec2::new(200.0, 300.0))
                            .corner_radius(Rounding::same(6.0)),
                    );
                }

                // Info
                ui.vertical(|ui| {
                    if let Some(rating) = movie.rating {
                        ui.horizontal(|ui| {
                            let color = super::theme::rating_color(rating);
                            ui.label(
                                RichText::new(format!("★ {:.1}/10", rating))
                                    .size(20.0)
                                    .color(color),
                            );
                            if let Some(count) = movie.rating_count {
                                ui.label(
                                    RichText::new(format!("({} votes / {} 票)", count, count))
                                        .size(12.0)
                                        .color(dim),
                                );
                            }
                        });
                    }
                    ui.add_space(4.0);

                    if let Some(ref genres) = movie.genres {
                        if let Ok(genre_list) = serde_json::from_str::<Vec<String>>(genres) {
                            ui.horizontal_wrapped(|ui| {
                                for g in &genre_list {
                                    super::widgets::badge(ui, g, primary);
                                    ui.add_space(4.0);
                                }
                            });
                        }
                    }

                    ui.add_space(8.0);

                    let mut info_items = Vec::new();
                    if let Some(y) = movie.year {
                        info_items.push(("Year / 年份", y.to_string()));
                    }
                    if let Some(r) = movie.runtime {
                        info_items.push(("Runtime / 时长", format!("{} min", r)));
                    }
                    if let Some(ref res) = movie.resolution {
                        info_items.push(("Resolution / 分辨率", res.clone()));
                    }
                    if let Some(ref src) = movie.source {
                        info_items.push(("Source / 来源", src.clone()));
                    }
                    if let Some(ref codec) = movie.codec {
                        info_items.push(("Codec / 编码", codec.clone()));
                    }
                    if let Some(ref lang) = movie.language {
                        info_items.push(("Language / 语言", lang.to_uppercase()));
                    }
                    if let Some(ref dir) = movie.director {
                        info_items.push(("Director / 导演", dir.clone()));
                    }

                    for (label, value) in &info_items {
                        super::widgets::info_row(ui, label, value);
                    }

                    if let Some(ref file) = movie.local_file_path {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("File / 文件: {file}")).size(11.0).color(dim),
                        );
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if file_exists
                            && ui
                                .button(
                                    RichText::new("📁 在文件夹中显示 / Show in Folder").size(12.0),
                                )
                                .clicked()
                        {
                            action = DetailAction::RevealInFolder;
                        }
                        if !file_exists
                            && ui
                                .button(RichText::new("🔎 重新定位文件 / Locate file…").size(12.0))
                                .clicked()
                        {
                            action = DetailAction::RelocateFile;
                        }
                    });
                });
            });

            ui.add_space(16.0);

            // Overview
            if let Some(ref overview) = movie.overview_cn {
                super::widgets::section_header(ui, "Synopsis / 剧情简介");
                ui.label(RichText::new(overview).size(13.0).color(text));
            } else if let Some(ref overview) = movie.overview {
                super::widgets::section_header(ui, "Synopsis / 剧情简介");
                ui.label(RichText::new(overview).size(13.0).color(text));
            }

            if let Ok(Some(item)) = watchlist::get_watchlist_item_for_movie(db, movie.id) {
                if let Some(workflow_note) =
                    item.notes.as_deref().and_then(watchlist::extract_workflow_summary)
                {
                    ui.add_space(16.0);
                    super::widgets::section_header(ui, "Workflow Snapshot / 工作流摘要");
                    ui.label(
                        RichText::new(format!(
                            "Saved in watchlist / 已保存到片单 · {}",
                            if item.status == "watched" {
                                "Watched / 已看"
                            } else {
                                "Want to Watch / 想看"
                            }
                        ))
                        .size(11.5)
                        .color(dim),
                    );
                    ui.add_space(4.0);
                    ui.label(RichText::new(workflow_note).size(12.5).color(text));
                }
            }

            // Cast
            if let Some(ref cast_json) = movie.cast_list {
                if let Ok(cast) = serde_json::from_str::<Vec<CastMember>>(cast_json) {
                    if !cast.is_empty() {
                        super::widgets::section_header(ui, "Cast / 演员阵容");
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for member in cast.iter().take(10) {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            RichText::new(&member.name).size(12.0).color(text),
                                        );
                                        ui.label(
                                            RichText::new(&member.character).size(11.0).color(dim),
                                        );
                                    });
                                    ui.add_space(20.0);
                                }
                            });
                        });
                    }
                }
            }

            // Subtitles section
            ui.add_space(16.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Subtitles / 字幕").size(16.0).color(text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Download Subtitles / 下载字幕").clicked() {
                        action = DetailAction::SearchSubtitles;
                    }
                });
            });

            if let Ok(subs) = sub_db::get_subtitles_for_movie(db, movie.id) {
                if subs.is_empty() {
                    ui.label(
                        RichText::new("暂无已下载字幕 / No subtitles downloaded yet")
                            .size(13.0)
                            .color(dim),
                    );
                } else {
                    for sub in &subs {
                        ui.horizontal(|ui| {
                            let lang_color = if sub.language.contains("zh") {
                                Color32::from_rgb(52, 211, 153)
                            } else {
                                primary
                            };
                            super::widgets::badge(ui, &sub.language, lang_color);
                            ui.label(&sub.source);
                            if let Some(ref format) = sub.format {
                                ui.label(format);
                            }
                            if let Some(rating) = sub.rating {
                                ui.label(format!("★{rating:.1}"));
                            }
                        });
                    }
                }
            }
        });

        action
    }
}

impl Default for MovieDetailPanel {
    fn default() -> Self {
        Self::new()
    }
}
