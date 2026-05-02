//! Watchlist management panel with workflow cards.

use egui::{Color32, RichText, Ui};
use rusqlite::Connection;

use crate::db::watchlist;

/// Watchlist panel for managing movies to watch and completed viewings.
pub struct WatchlistPanel;

impl WatchlistPanel {
    pub fn show(ui: &mut Ui, db: &Connection, is_dark: bool) {
        let text = if is_dark { Color32::from_rgb(240, 240, 245) }
            else { Color32::from_rgb(15, 15, 25) };
        let dim = if is_dark { Color32::from_rgb(150, 150, 165) }
            else { Color32::from_rgb(100, 100, 115) };

        let primary = Color32::from_rgb(99, 102, 241);

        ui.horizontal(|ui| {
            crate::ui::icons::draw_icon(ui, "bookmark", 22.0, primary);
            ui.add_space(8.0);
            ui.heading(RichText::new("Watchlist / 片单").size(22.0).color(text));
        });
        ui.add_space(8.0);

        ui.label(
            RichText::new("Saved workflow notes appear here alongside your watchlist status. / 已保存的 AI 工作流摘要会和片单状态一起显示在这里。")
                .size(12.0)
                .color(dim),
        );
        ui.add_space(12.0);

        if let Ok(items) = watchlist::get_watchlist(db, None) {
            if items.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    crate::ui::icons::icon_bookmark(ui, 64.0, dim);
                    ui.add_space(12.0);
                    ui.label(RichText::new("你的片单还是空的 / Your watchlist is empty").size(14.0).color(dim));
                    ui.label(RichText::new("从片库中加入影片，管理接下来真正想看的内容。 / Add films from your library to track what to watch next.").size(12.0).color(dim));
                });
            } else {
                for (status_key, status_title) in [
                    ("want_to_watch", "Want to Watch / 想看"),
                    ("watched", "Watched / 已看"),
                ] {
                    let section_items: Vec<_> = items.iter().filter(|item| item.status == status_key).collect();
                    if section_items.is_empty() {
                        continue;
                    }

                    ui.label(RichText::new(status_title).size(15.0).color(text).strong());
                    ui.add_space(6.0);

                    for item in section_items {
                        ui.group(|ui| {
                            if let Some(mid) = item.movie_id {
                                if let Ok(Some(movie)) = crate::db::movies::get_movie_by_id(db, mid) {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&movie.title).strong());
                                        if let Some(y) = movie.year {
                                            ui.label(format!("({})", y));
                                        }
                                        if let Some(r) = item.user_rating {
                                            ui.label(format!("★{}", r));
                                        }
                                    });
                                }
                            }
                            ui.label(format!("Added / 添加于: {}", &item.added_date[..10]));

                            if let Some(workflow_note) = item.notes.as_deref().and_then(watchlist::extract_workflow_summary) {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("Workflow Snapshot / 工作流摘要")
                                        .size(11.5)
                                        .color(primary)
                                        .strong(),
                                );
                                ui.label(
                                    RichText::new(workflow_preview(&workflow_note))
                                        .size(11.5)
                                        .color(dim),
                                );
                            }
                        });
                        ui.add_space(8.0);
                    }
                }
            }
        }
    }
}

fn workflow_preview(text: &str) -> String {
    let preview_lines: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).take(4).collect();
    preview_lines.join("\n")
}
