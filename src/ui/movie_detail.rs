use egui::{Color32, RichText, Rounding, Ui, Vec2};
use rusqlite::Connection;

use crate::db::{models::{CastMember, Movie}, subtitles as sub_db, watchlist};

#[derive(PartialEq)]
pub enum DetailAction {
    None,
    SearchSubtitles,
    AiAnalyze,
}

pub struct MovieDetailPanel;

impl MovieDetailPanel {
    /// Show movie detail. Returns the action the user wants to take.
    pub fn show(
        ui: &mut Ui,
        movie: &Movie,
        db: &Connection,
        is_dark: bool,
    ) -> DetailAction {
        let mut action = DetailAction::None;
        let text = if is_dark { Color32::from_rgb(240, 240, 245) }
            else { Color32::from_rgb(15, 15, 25) };
        let dim = if is_dark { Color32::from_rgb(150, 150, 165) }
            else { Color32::from_rgb(100, 100, 115) };
        let primary = Color32::from_rgb(99, 102, 241);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Title + AI Analyze button
            ui.horizontal(|ui| {
                ui.heading(RichText::new(&movie.title).size(22.0).color(text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let ai_btn = egui::Button::new(
                        RichText::new("AI Insight / AI 解析").size(13.0).color(Color32::WHITE),
                    )
                    .fill(primary)
                    .rounding(Rounding::same(6.0));
                    if ui.add(ai_btn).clicked() {
                        action = DetailAction::AiAnalyze;
                    }
                });
            });
            if let Some(ref cn) = movie.title_cn {
                ui.label(RichText::new(cn).size(16.0).color(dim));
            }

            ui.add_space(12.0);

            // Poster + basic info
            ui.horizontal(|ui| {
                // Poster
                if let Some(ref poster_path) = movie.poster_local {
                    if let Ok(img) = image::open(poster_path) {
                        let size = [img.width() as _, img.height() as _];
                        let rgba = img.to_rgba8();
                        let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());
                        let texture = ui.ctx().load_texture(
                            format!("detail_poster_{}", movie.id),
                            color_img,
                            egui::TextureOptions::LINEAR,
                        );
                        let max_size = Vec2::new(200.0, 300.0);
                        let aspect = size[0] as f32 / size[1] as f32;
                        let display_size = if aspect > 2.0/3.0 {
                            Vec2::new(max_size.x, max_size.x / aspect)
                        } else {
                            Vec2::new(max_size.y * aspect, max_size.y)
                        };
                        ui.image(egui::ImageSource::Texture(
                            egui::load::SizedTexture::new(texture.id(), display_size)
                        ));
                    }
                }

                // Info
                ui.vertical(|ui| {
                    if let Some(rating) = movie.rating {
                        ui.horizontal(|ui| {
                            let color = super::theme::rating_color(rating);
                            ui.label(RichText::new(format!("★ {:.1}/10", rating)).size(20.0).color(color));
                            if let Some(count) = movie.rating_count {
                                ui.label(RichText::new(format!("({} votes / {} 票)", count, count)).size(12.0).color(dim));
                            }
                        });
                    }
                    ui.add_space(4.0);

                    if let Some(ref genres) = movie.genres {
                        if let Ok(genre_list) = serde_json::from_str::<Vec<String>>(genres) {
                            ui.horizontal(|ui| {
                                for g in &genre_list {
                                    super::widgets::badge(ui, g, primary);
                                    ui.add_space(4.0);
                                }
                            });
                        }
                    }

                    ui.add_space(8.0);

                    let mut info_items = Vec::new();
                    if let Some(y) = movie.year { info_items.push(("Year / 年份", y.to_string())); }
                    if let Some(r) = movie.runtime { info_items.push(("Runtime / 时长", format!("{} min", r))); }
                    if let Some(ref res) = movie.resolution { info_items.push(("Resolution / 分辨率", res.clone())); }
                    if let Some(ref src) = movie.source { info_items.push(("Source / 来源", src.clone())); }
                    if let Some(ref codec) = movie.codec { info_items.push(("Codec / 编码", codec.clone())); }
                    if let Some(ref lang) = movie.language { info_items.push(("Language / 语言", lang.to_uppercase())); }
                    if let Some(ref dir) = movie.director { info_items.push(("Director / 导演", dir.clone())); }

                    for (label, value) in &info_items {
                        super::widgets::info_row(ui, label, value);
                    }

                    if let Some(ref file) = movie.local_file_path {
                        ui.add_space(4.0);
                        ui.label(RichText::new(format!("File / 文件: {}", file)).size(11.0).color(dim));
                    }
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
                if let Some(workflow_note) = item.notes.as_deref().and_then(watchlist::extract_workflow_summary) {
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
                                for member in &cast.iter().take(10) {
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(&member.name).size(12.0).color(text));
                                        ui.label(RichText::new(&member.character).size(11.0).color(dim));
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
                    ui.label(RichText::new("暂无已下载字幕 / No subtitles downloaded yet").size(13.0).color(dim));
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
                                ui.label(format!("★{:.1}", rating));
                            }
                        });
                    }
                }
            }
        });

        action
    }
}
