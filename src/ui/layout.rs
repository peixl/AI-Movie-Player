use egui::{Color32, Context, Rect, RichText, Sense, Stroke, Ui, pos2};

use crate::ui::Rounding;

/// Application views navigable from the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Library,
    AddMovie,
    SubtitleSearch,
    BatchOps,
    Watchlist,
    Settings,
    AiChat,
    AiRecommend,
}

/// Sidebar navigation state and view routing.
pub struct AppLayout {
    pub active_view: View,
}

#[derive(Clone, Copy)]
struct NavColors {
    text: Color32,
    dim: Color32,
    primary: Color32,
}

struct NavItem<'a> {
    view: View,
    icon: &'a str,
    label: &'a str,
    badge: &'a str,
    styled_badge: bool,
}

impl AppLayout {
    pub fn new() -> Self {
        Self { active_view: View::Library }
    }

    fn render_nav_item(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        is_dark: bool,
        item: NavItem<'_>,
        colors: NavColors,
    ) -> bool {
        let is_active = self.active_view == item.view;
        let label_color = if is_active { Color32::WHITE } else { colors.text };

        let btn_size = egui::vec2(ui.available_width() - 8.0, 36.0);
        let (rect, response) = ui.allocate_exact_size(btn_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let hover_t = ctx.animate_bool(
                egui::Id::new(format!("nav_hover_{}", item.label)),
                response.hovered(),
            );

            if is_active {
                ui.painter().rect_filled(rect, Rounding::same(6.0), colors.primary);
            } else if hover_t > 0.01 {
                let hover_alpha = (hover_t * 10.0).clamp(0.0, 1.0);
                let hover_bg = if is_dark {
                    Color32::from_rgba_premultiplied(255, 255, 255, (hover_alpha * 8.0) as u8)
                } else {
                    Color32::from_rgba_premultiplied(0, 0, 0, (hover_alpha * 8.0) as u8)
                };
                ui.painter().rect_filled(rect, Rounding::same(6.0), hover_bg);
            }

            let icon_center = pos2(rect.min.x + 20.0, rect.center().y);
            let icon_color = if is_active { Color32::WHITE } else { colors.primary };
            draw_icon_at(ui.painter(), item.icon, 20.0, icon_center, icon_color);

            let label_galley = ui.painter().layout_no_wrap(
                item.label.to_string(),
                egui::FontId::proportional(14.0),
                label_color,
            );
            let label_pos = pos2(rect.min.x + 42.0, rect.center().y - label_galley.size().y / 2.0);
            ui.painter().galley(label_pos, label_galley, label_color);

            if !item.badge.is_empty() && !is_active {
                if item.styled_badge {
                    let badge_color = colors.primary.linear_multiply(0.8);
                    let badge_galley = ui.painter().layout_no_wrap(
                        item.badge.to_string(),
                        egui::FontId::proportional(10.0),
                        badge_color,
                    );
                    let badge_bg = if is_dark {
                        Color32::from_rgba_premultiplied(99, 102, 241, 30)
                    } else {
                        Color32::from_rgba_premultiplied(99, 102, 241, 20)
                    };
                    let badge_rect = egui::Rect::from_min_size(
                        pos2(
                            rect.max.x - badge_galley.size().x - 16.0,
                            rect.center().y - badge_galley.size().y / 2.0 - 2.0,
                        ),
                        badge_galley.size() + egui::vec2(8.0, 4.0),
                    );
                    ui.painter().rect_filled(badge_rect, Rounding::same(4.0), badge_bg);
                    ui.painter().galley(
                        pos2(
                            badge_rect.min.x + 4.0,
                            badge_rect.center().y - badge_galley.size().y / 2.0,
                        ),
                        badge_galley,
                        badge_color,
                    );
                } else {
                    let badge_galley = ui.painter().layout_no_wrap(
                        item.badge.to_string(),
                        egui::FontId::proportional(11.0),
                        colors.dim,
                    );
                    let badge_pos = pos2(
                        rect.max.x - badge_galley.size().x - 12.0,
                        rect.center().y - badge_galley.size().y / 2.0,
                    );
                    ui.painter().galley(badge_pos, badge_galley, colors.dim);
                }
            }
        }

        response.clicked()
    }

    pub fn show_sidebar(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        is_dark: bool,
        movie_count: i64,
    ) -> Option<View> {
        let mut selected_view = None;

        let text =
            if is_dark { Color32::from_rgb(240, 240, 245) } else { Color32::from_rgb(15, 15, 25) };
        let dim = if is_dark {
            Color32::from_rgb(150, 150, 165)
        } else {
            Color32::from_rgb(100, 100, 115)
        };
        let primary = Color32::from_rgb(99, 102, 241);
        let colors = NavColors { text, dim, primary };

        // App branding with hand-drawn film icon
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                super::icons::draw_icon(ui, "film", 24.0, primary);
                ui.add_space(6.0);
                ui.label(RichText::new("AI-Movie-Player").size(18.0).color(text).strong());
            });
        });
        ui.add_space(16.0);

        // Nav items with hand-drawn icons
        let nav_items = vec![
            (View::Library, "film", "片库", format!("{} 部", movie_count)),
            (View::AddMovie, "add-folder", "导入影片", String::new()),
            (View::SubtitleSearch, "subtitle", "字幕", String::new()),
            (View::BatchOps, "bolt", "批量操作", String::new()),
            (View::Watchlist, "bookmark", "片单", String::new()),
            (View::Settings, "gear", "设置", String::new()),
        ];

        // Separator before AI items
        ui.add_space(6.0);
        ui.label(RichText::new("AI Features / AI 功能").size(11.0).color(dim).strong());
        ui.add_space(2.0);

        let ai_nav_items = vec![
            (View::AiChat, "chat", "AI 对话", "ifq.ai"),
            (View::AiRecommend, "sparkle", "AI 推荐", "Taste"),
        ];

        for (view, icon, label, badge) in &nav_items {
            if self.render_nav_item(
                ui,
                ctx,
                is_dark,
                NavItem { view: *view, icon, label, badge: badge.as_str(), styled_badge: false },
                colors,
            ) {
                selected_view = Some(*view);
            }
        }

        // Render AI nav items
        for (view, icon, label, badge) in &ai_nav_items {
            if self.render_nav_item(
                ui,
                ctx,
                is_dark,
                NavItem { view: *view, icon, label, badge, styled_badge: true },
                colors,
            ) {
                selected_view = Some(*view);
            }
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Stats
        ui.label(RichText::new("Library Stats / 片库统计").size(12.0).color(dim).strong());
        ui.label(RichText::new(format!("Total / 总计: {}", movie_count)).size(12.0).color(dim));

        selected_view
    }
}

impl Default for AppLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a hand-drawn icon at a specific screen position (used in custom layout)
fn draw_icon_at(
    painter: &egui::Painter,
    name: &str,
    size: f32,
    center: egui::Pos2,
    color: Color32,
) {
    let seed_base = center.x * 7.0 + center.y * 13.0;

    match name {
        "film" | "library" => draw_film_icon(painter, center, size, color, seed_base),
        "add-folder" => draw_add_folder_icon(painter, center, size, color, seed_base),
        "subtitle" => draw_subtitle_icon(painter, center, size, color, seed_base),
        "bolt" => draw_bolt_icon(painter, center, size, color, seed_base),
        "bookmark" => draw_bookmark_icon(painter, center, size, color, seed_base),
        "gear" => draw_gear_icon(painter, center, size, color, seed_base),
        "search" => draw_search_icon(painter, center, size, color, seed_base),
        "chat" => draw_chat_icon(painter, center, size, color, seed_base),
        "sparkle" => draw_sparkle_icon(painter, center, size, color, seed_base),
        _ => {}
    }
}

use super::icons::wobble;

fn draw_film_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: Color32,
    seed: f32,
) {
    let rect = Rect::from_center_size(center, egui::vec2(size * 0.75, size * 0.6));
    let stroke = Stroke::new(1.6, color);
    // Rounded rect outline
    let pts: Vec<egui::Pos2> = (0..16)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 16.0;
            let rx = rect.width() / 2.0;
            let ry = rect.height() / 2.0;
            wobble(center.x + a.cos() * rx, center.y + a.sin() * ry, seed + i as f32)
        })
        .collect();
    for i in 0..pts.len() {
        let p1 = pts[i];
        let p2 = pts[(i + 1) % pts.len()];
        let mid = pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
            points: [p1, mid, mid, p2],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: stroke.into(),
        }));
    }
    // Sprocket holes
    for i in 0..3 {
        let y = rect.min.y + rect.height() * (i as f32 + 0.5) / 3.0;
        painter.circle_filled(
            wobble(rect.min.x + 3.0, y, seed + 100.0 + i as f32),
            1.5,
            color.linear_multiply(0.5),
        );
        painter.circle_filled(
            wobble(rect.max.x - 3.0, y, seed + 200.0 + i as f32),
            1.5,
            color.linear_multiply(0.5),
        );
    }
    // Play triangle
    let tri = [
        wobble(center.x - 3.0, center.y - 5.0, seed + 300.0),
        wobble(center.x - 3.0, center.y + 5.0, seed + 301.0),
        wobble(center.x + 6.0, center.y, seed + 302.0),
    ];
    painter.add(egui::Shape::convex_polygon(tri.to_vec(), color, Stroke::new(0.5, color)));
}

fn draw_add_folder_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: Color32,
    seed: f32,
) {
    let fw = size * 0.55;
    let fh = size * 0.4;
    let l = center.x - fw / 2.0;
    let t = center.y - fh / 2.0 + 3.0;
    // Tab
    let tw = fw * 0.3;
    let th = fh * 0.2;
    let tab = [
        wobble(l, t + th, seed),
        wobble(l + tw * 0.3, t + th, seed + 1.0),
        wobble(l + tw * 0.5, t, seed + 2.0),
        wobble(l + tw, t, seed + 3.0),
        wobble(l + tw, t + th, seed + 4.0),
    ];
    painter.add(egui::Shape::convex_polygon(
        tab.to_vec(),
        color.linear_multiply(0.12),
        Stroke::new(1.2, color),
    ));
    // Body rect
    let body =
        Rect::from_min_max(pos2(l, t + th * 0.3), pos2(center.x + fw / 2.0, center.y + fh / 2.0));
    painter.rect_filled(body, Rounding::same(3.0), color.linear_multiply(0.06));
    painter.rect_stroke(
        body,
        Rounding::same(3.0),
        Stroke::new(1.2, color),
        egui::StrokeKind::Middle,
    );
    // Plus
    let cx = center.x + 1.0;
    let cy = center.y + 4.0;
    let arm = 4.0;
    let pc = color.linear_multiply(1.3);
    let h1 = [wobble(cx - arm, cy, seed + 50.0), wobble(cx + arm, cy, seed + 51.0)];
    let h2 = [wobble(cx, cy - arm, seed + 53.0), wobble(cx, cy + arm, seed + 54.0)];
    painter.add(egui::Shape::line_segment(h1, Stroke::new(1.8, pc)));
    painter.add(egui::Shape::line_segment(h2, Stroke::new(1.8, pc)));
}

fn draw_subtitle_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: Color32,
    seed: f32,
) {
    let bw = size * 0.5;
    let bh = size * 0.38;
    let r = Rect::from_center_size(center - egui::vec2(0.0, 1.0), egui::vec2(bw, bh));
    painter.rect_filled(r, Rounding::same(5.0), color.linear_multiply(0.06));
    painter.rect_stroke(r, Rounding::same(5.0), Stroke::new(1.2, color), egui::StrokeKind::Middle);
    // Tail
    let tail = [
        wobble(center.x - 3.0, r.max.y, seed),
        wobble(center.x + 1.0, r.max.y + 6.0, seed + 1.0),
        wobble(center.x + 5.0, r.max.y, seed + 2.0),
    ];
    painter.add(egui::Shape::convex_polygon(
        tail.to_vec(),
        color.linear_multiply(0.06),
        Stroke::new(1.2, color),
    ));
    // Text lines
    for i in 0..2 {
        let y = r.min.y + 7.0 + i as f32 * 6.0;
        let lw = if i == 1 { bw * 0.45 } else { bw * 0.65 };
        let p1 = wobble(center.x - lw / 2.0 + bw * 0.03, y, seed + 30.0 + i as f32);
        let p2 = wobble(p1.x + lw, y, seed + 31.0 + i as f32);
        painter
            .add(egui::Shape::line_segment([p1, p2], Stroke::new(1.0, color.linear_multiply(0.5))));
    }
}

fn draw_bolt_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    _size: f32,
    color: Color32,
    seed: f32,
) {
    let pts = [
        wobble(center.x + 1.0, center.y - 7.0, seed),
        wobble(center.x - 3.0, center.y - 1.0, seed + 1.0),
        wobble(center.x - 1.0, center.y - 1.0, seed + 2.0),
        wobble(center.x - 2.0, center.y + 7.0, seed + 3.0),
        wobble(center.x + 2.0, center.y + 1.0, seed + 4.0),
        wobble(center.x, center.y + 1.0, seed + 5.0),
    ];
    let fill = color.linear_multiply(0.18);
    for i in 0..pts.len() - 2 {
        painter.add(egui::Shape::convex_polygon(
            [pts[0], pts[i + 1], pts[i + 2]].to_vec(),
            fill,
            Stroke::NONE,
        ));
    }
    painter.add(egui::Shape::line_segment([pts[0], pts[1]], Stroke::new(1.5, color)));
    painter.add(egui::Shape::line_segment([pts[1], pts[2]], Stroke::new(1.5, color)));
    painter.add(egui::Shape::line_segment([pts[2], pts[3]], Stroke::new(1.5, color)));
    painter.add(egui::Shape::line_segment([pts[3], pts[4]], Stroke::new(1.5, color)));
    painter.add(egui::Shape::line_segment([pts[4], pts[5]], Stroke::new(1.5, color)));
}

fn draw_bookmark_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: Color32,
    seed: f32,
) {
    let w = size * 0.32;
    let h = size * 0.5;
    let pts = [
        wobble(center.x - w, center.y - h, seed),
        wobble(center.x + w, center.y - h, seed + 1.0),
        wobble(center.x + w, center.y + h * 0.7, seed + 2.0),
        wobble(center.x, center.y + h * 0.2, seed + 3.0),
        wobble(center.x - w, center.y + h * 0.7, seed + 4.0),
    ];
    for i in 0..pts.len() {
        painter.add(egui::Shape::line_segment(
            [pts[i], pts[(i + 1) % pts.len()]],
            Stroke::new(1.5, color),
        ));
    }
    painter.circle_filled(
        pos2(center.x, center.y + h * 0.25),
        w * 0.7,
        color.linear_multiply(0.06),
    );
    // Checkmark
    let ck = [
        wobble(center.x - 3.0, center.y, seed + 50.0),
        wobble(center.x - 0.5, center.y + 2.5, seed + 51.0),
        wobble(center.x + 4.0, center.y - 3.0, seed + 52.0),
    ];
    painter.add(egui::Shape::line_segment([ck[0], ck[1]], Stroke::new(1.3, color)));
    painter.add(egui::Shape::line_segment([ck[1], ck[2]], Stroke::new(1.3, color)));
}

fn draw_gear_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: Color32,
    seed: f32,
) {
    let or = size * 0.25;
    let ir = size * 0.1;
    // Teeth
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let p1 = pos2(center.x + a.cos() * (or - 0.5), center.y + a.sin() * (or - 0.5));
        let p2 = pos2(center.x + a.cos() * (or + 2.0), center.y + a.sin() * (or + 2.0));
        painter.add(egui::Shape::line_segment(
            [wobble(p1.x, p1.y, seed + i as f32), wobble(p2.x, p2.y, seed + 10.0 + i as f32)],
            Stroke::new(1.5, color),
        ));
    }
    // Outer circle
    let cpts: Vec<egui::Pos2> = (0..14)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 14.0;
            wobble(center.x + a.cos() * or, center.y + a.sin() * or, seed + 100.0 + i as f32)
        })
        .collect();
    for i in 0..cpts.len() {
        let p1 = cpts[i];
        let p2 = cpts[(i + 1) % cpts.len()];
        let m = pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
            points: [p1, m, m, p2],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.2, color).into(),
        }));
    }
    painter.circle_filled(center, ir, color.linear_multiply(0.08));
    painter.circle_stroke(center, ir, Stroke::new(1.2, color));
}

fn draw_search_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: Color32,
    seed: f32,
) {
    let gr = size * 0.24;
    let gc = center - egui::vec2(1.5, 1.5);
    let cpts: Vec<egui::Pos2> = (0..12)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 12.0;
            wobble(gc.x + a.cos() * gr, gc.y + a.sin() * gr, seed + i as f32)
        })
        .collect();
    for i in 0..cpts.len() {
        let p1 = cpts[i];
        let p2 = cpts[(i + 1) % cpts.len()];
        let m = pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
        painter.add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
            points: [p1, m, m, p2],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.5, color).into(),
        }));
    }
    let hs = pos2(gc.x + gr * 0.7, gc.y + gr * 0.7);
    let he = pos2(hs.x + 5.0, hs.y + 5.0);
    painter.add(egui::Shape::line_segment([hs, he], Stroke::new(1.8, color)));
}

fn draw_chat_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: Color32,
    seed: f32,
) {
    let bw = size * 0.5;
    let bh = size * 0.38;
    let r = Rect::from_center_size(center - egui::vec2(0.0, 1.0), egui::vec2(bw, bh));
    painter.rect_filled(r, Rounding::same(5.0), color.linear_multiply(0.06));
    painter.rect_stroke(r, Rounding::same(5.0), Stroke::new(1.2, color), egui::StrokeKind::Middle);
    let tail = [
        wobble(center.x - 3.0, r.max.y, seed),
        wobble(center.x + 1.0, r.max.y + 5.0, seed + 1.0),
        wobble(center.x + 5.0, r.max.y, seed + 2.0),
    ];
    painter.add(egui::Shape::convex_polygon(
        tail.to_vec(),
        color.linear_multiply(0.06),
        Stroke::new(1.2, color),
    ));
    for i in 0..3 {
        let dx = (i as f32 - 1.0) * 5.0;
        painter.circle_filled(
            wobble(center.x + dx, center.y - 1.0, seed + 30.0 + i as f32),
            1.5,
            color.linear_multiply(0.5),
        );
    }
}

fn draw_sparkle_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    color: Color32,
    seed: f32,
) {
    let r = size * 0.38;
    for i in 0..4 {
        let a = i as f32 * std::f32::consts::TAU / 4.0;
        let a2 = a + std::f32::consts::TAU / 8.0;
        let p_outer = wobble(center.x + a.cos() * r, center.y + a.sin() * r, seed + i as f32);
        let p_inner1 = wobble(
            center.x + a2.cos() * r * 0.3,
            center.y + a2.sin() * r * 0.3,
            seed + 10.0 + i as f32,
        );
        let next_a = a + std::f32::consts::TAU / 4.0;
        let p_next =
            wobble(center.x + next_a.cos() * r, center.y + next_a.sin() * r, seed + (i + 1) as f32);
        painter.add(egui::Shape::line_segment([p_outer, p_inner1], Stroke::new(1.4, color)));
        painter.add(egui::Shape::line_segment([p_inner1, p_next], Stroke::new(1.4, color)));
    }
    painter.circle_filled(center, 2.2, color);
}
