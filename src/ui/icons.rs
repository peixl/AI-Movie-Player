//! Hand-drawn procedural icons using egui's Shape API.
//! Each icon uses imperfect curves, varying stroke widths, and organic shapes
//! to achieve a warm, hand-sketched feel.

use egui::{pos2, vec2, Color32, Pos2, Rect, Rounding, Shape, Stroke, Ui};

/// Wobble offset for a point to create hand-drawn imperfection.
/// Returns offset that varies with position to avoid repeating patterns.
fn wobble(x: f32, y: f32, seed: f32) -> Pos2 {
    let wx = ((x * 3.7 + y * 1.3 + seed).sin() * 0.5
        + (x * 7.1 - y * 2.9 + seed * 1.7).sin() * 0.3
        + (x * 1.1 + y * 5.3 + seed * 3.1).sin() * 0.2)
        * 0.6;
    let wy = ((y * 3.3 + x * 1.7 + seed * 2.1).sin() * 0.5
        + (y * 6.7 - x * 3.1 + seed * 1.3).sin() * 0.3
        + (y * 1.9 + x * 4.7 + seed * 2.9).sin() * 0.2)
        * 0.6;
    pos2(x + wx, y + wy)
}

fn sketched_line(painter: &egui::Painter, p1: Pos2, p2: Pos2, color: Color32, width: f32, seed: f32) {
    let mid = pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
    // Add slight curve instead of straight line
    let offset = (p2 - p1).normalized().rot90() * (width * 0.3);
    let cp = mid + offset;
    let w1 = wobble(p1.x, p1.y, seed);
    let w2 = wobble(cp.x, cp.y, seed + 1.0);
    let w3 = wobble(p2.x, p2.y, seed + 2.0);

    let shape = Shape::CubicBezier(epaint::CubicBezierShape {
        points: [p1 + w1, cp + w2, cp + w2, p2 + w3],
        closed: false,
        fill: Color32::TRANSPARENT,
        stroke: Stroke::new(width, color),
    });
    painter.add(shape);
}

fn sketched_rect(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    stroke: f32,
    fill: Color32,
    seed: f32,
) {
    let r = 4.0; // corner radius
    let tl = rect.min;
    let tr = pos2(rect.max.x, rect.min.y);
    let br = rect.max;
    let bl = pos2(rect.min.x, rect.max.y);

    // Fill
    painter.rect_filled(rect, Rounding::same(r), fill);

    // Sketch each edge with slight curves
    let points = [
        wobble(tl.x, tl.y, seed),
        wobble(tr.x, tr.y, seed + 1.0),
        wobble(br.x, br.y, seed + 2.0),
        wobble(bl.x, bl.y, seed + 3.0),
    ];

    // Draw each side as a cubic bezier
    for i in 0..4 {
        let p1 = points[i];
        let p2 = points[(i + 1) % 4];
        let mid = pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
        let normal = (p2 - p1).normalized().rot90() * stroke;
        let cp = mid + normal;
        painter.add(Shape::CubicBezier(epaint::CubicBezierShape {
            points: [p1, cp, cp, p2],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(stroke, color),
        }));
    }
}

/// Film reel / movie icon
pub fn icon_film(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let r = size / 2.0 - 2.0;
    let seed = 42.0;

    // Outer rounded rectangle
    let rect = Rect::from_center_size(center, vec2(size * 0.8, size * 0.7));
    let points: Vec<Pos2> = (0..24)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 24.0;
            let rx = rect.width() / 2.0;
            let ry = rect.height() / 2.0;
            let p = pos2(center.x + angle.cos() * rx, center.y + angle.sin() * ry);
            wobble(p.x, p.y, seed + i as f32)
        })
        .collect();

    // Draw as polyline with rounded corners
    let stroke = Stroke::new(1.8, color);
    for i in 0..points.len() {
        let p1 = points[i];
        let p2 = points[(i + 1) % points.len()];
        let mid = pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
        painter.add(Shape::CubicBezier(epaint::CubicBezierShape {
            points: [p1, mid, mid, p2],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke,
        }));
    }

    // Film sprocket holes (left side)
    for i in 0..4 {
        let y = rect.min.y + rect.height() * (i as f32 + 0.5) / 4.0;
        let p = wobble(rect.min.x + 4.0, y, seed + 100.0 + i as f32);
        painter.circle_filled(p, 2.0, color.linear_multiply(0.5));
    }
    // Right side holes
    for i in 0..4 {
        let y = rect.min.y + rect.height() * (i as f32 + 0.5) / 4.0;
        let p = wobble(rect.max.x - 4.0, y, seed + 200.0 + i as f32);
        painter.circle_filled(p, 2.0, color.linear_multiply(0.5));
    }

    // Inner play triangle
    let tri_points = [
        wobble(center.x - 4.0, center.y - 6.0, seed + 300.0),
        wobble(center.x - 4.0, center.y + 6.0, seed + 301.0),
        wobble(center.x + 7.0, center.y, seed + 302.0),
    ];
    painter.add(Shape::convex_polygon(
        tri_points.to_vec(),
        color,
        Stroke::new(0.5, color),
    ));
}

/// Folder + plus icon (add movie)
pub fn icon_add_folder(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 73.0;

    // Folder body
    let folder_w = size * 0.65;
    let folder_h = size * 0.5;
    let left = center.x - folder_w / 2.0;
    let top = center.y - folder_h / 2.0 + 4.0;

    // Tab at top
    let tab_w = folder_w * 0.35;
    let tab_h = folder_h * 0.25;
    let tab_pts = [
        wobble(left, top + tab_h, seed),
        wobble(left + tab_w * 0.3, top + tab_h, seed + 1.0),
        wobble(left + tab_w * 0.5, top, seed + 2.0),
        wobble(left + tab_w, top, seed + 3.0),
        wobble(left + tab_w, top + tab_h, seed + 4.0),
    ];
    painter.add(Shape::convex_polygon(
        tab_pts.to_vec(),
        color.linear_multiply(0.15),
        Stroke::new(1.5, color),
    ));

    // Main folder body
    let body_pts = [
        wobble(left, top + tab_h * 0.5, seed + 5.0),
        wobble(left, center.y + folder_h / 2.0, seed + 6.0),
        wobble(center.x + folder_w / 2.0, center.y + folder_h / 2.0, seed + 7.0),
        wobble(center.x + folder_w / 2.0, top + tab_h * 0.5, seed + 8.0),
    ];
    let mut all_pts = tab_pts.to_vec();
    all_pts.extend_from_slice(&[
        body_pts[0], body_pts[1], body_pts[2], body_pts[3],
    ]);

    // Draw folder outline
    let rect = Rect::from_min_max(
        pos2(left, top),
        pos2(center.x + folder_w / 2.0, center.y + folder_h / 2.0),
    );
    painter.rect_filled(rect, Rounding::same(3.0), color.linear_multiply(0.08));
    for i in 0..all_pts.len() {
        let p1 = all_pts[i];
        let p2 = all_pts[(i + 1) % all_pts.len()];
        sketched_line(painter, p1, p2, color, 1.5, seed + i as f32 * 10.0);
    }

    // Plus sign
    let cx = center.x + 2.0;
    let cy = center.y + 6.0;
    let arm = 5.0;
    let plus_color = color.linear_multiply(1.2);
    sketched_line(painter, wobble(cx - arm, cy, seed + 50.0), wobble(cx + arm, cy, seed + 51.0), plus_color, 2.0, seed + 52.0);
    sketched_line(painter, wobble(cx, cy - arm, seed + 53.0), wobble(cx, cy + arm, seed + 54.0), plus_color, 2.0, seed + 55.0);
}

/// Subtitle / text bubble icon
pub fn icon_subtitle(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 107.0;

    // Speech bubble
    let bubble_w = size * 0.6;
    let bubble_h = size * 0.45;
    let rect = Rect::from_center_size(center - vec2(0.0, 2.0), vec2(bubble_w, bubble_h));
    let rounding = Rounding::same(6.0);
    painter.rect_filled(rect, rounding, color.linear_multiply(0.08));
    painter.rect_stroke(rect, rounding, Stroke::new(1.5, color));

    // Tail
    let tail_pts = [
        wobble(center.x - 4.0, rect.max.y, seed),
        wobble(center.x + 1.0, rect.max.y + 8.0, seed + 1.0),
        wobble(center.x + 6.0, rect.max.y, seed + 2.0),
    ];
    painter.add(Shape::convex_polygon(
        tail_pts.to_vec(),
        color.linear_multiply(0.08),
        Stroke::new(1.5, color),
    ));

    // Text lines inside bubble
    for i in 0..3 {
        let y = rect.min.y + 8.0 + i as f32 * 7.0;
        let line_w = if i == 2 { bubble_w * 0.5 } else { bubble_w * 0.7 };
        let lx = center.x - line_w / 2.0 + bubble_w * 0.05;
        let rx = lx + line_w;
        let p1 = wobble(lx, y, seed + 30.0 + i as f32);
        let p2 = wobble(rx, y, seed + 31.0 + i as f32);
        sketched_line(painter, p1, p2, color.linear_multiply(0.6), 1.2, seed + 32.0 + i as f32);
    }
}

/// Lightning bolt (batch operations)
pub fn icon_bolt(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 137.0;

    let pts = [
        wobble(center.x + 2.0, center.y - 9.0, seed),
        wobble(center.x - 4.0, center.y - 1.0, seed + 1.0),
        wobble(center.x - 1.0, center.y - 1.0, seed + 2.0),
        wobble(center.x - 3.0, center.y + 9.0, seed + 3.0),
        wobble(center.x + 3.0, center.y + 1.0, seed + 4.0),
        wobble(center.x, center.y + 1.0, seed + 5.0),
    ];

    // Fill
    let fill = color.linear_multiply(0.2);
    for i in 0..pts.len() - 2 {
        let tri = [pts[0], pts[i + 1], pts[i + 2]];
        painter.add(Shape::convex_polygon(tri.to_vec(), fill, Stroke::NONE));
    }
    // Outline
    for i in 0..pts.len() - 1 {
        sketched_line(painter, pts[i], pts[i + 1], color, 1.8, seed + i as f32 * 10.0);
    }
}

/// Bookmark / list icon (watchlist)
pub fn icon_bookmark(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 163.0;

    let w = size * 0.4;
    let h = size * 0.6;

    // Body
    let pts = [
        wobble(center.x - w, center.y - h, seed),
        wobble(center.x + w, center.y - h, seed + 1.0),
        wobble(center.x + w, center.y + h * 0.7, seed + 2.0),
        wobble(center.x, center.y + h * 0.2, seed + 3.0),
        wobble(center.x - w, center.y + h * 0.7, seed + 4.0),
    ];

    for i in 0..pts.len() {
        let p1 = pts[i];
        let p2 = pts[(i + 1) % pts.len()];
        sketched_line(painter, p1, p2, color, 1.8, seed + i as f32 * 7.0);
    }

    // Fill with light wash
    let center_low = pos2(center.x, center.y + h * 0.3);
    painter.circle_filled(center_low, w * 0.8, color.linear_multiply(0.08));

    // Check mark inside
    let check_x = center.x;
    let check_y = center.y - 2.0;
    sketched_line(
        painter,
        wobble(check_x - 4.0, check_y, seed + 50.0),
        wobble(check_x - 1.0, check_y + 3.0, seed + 51.0),
        color, 1.5, seed + 52.0,
    );
    sketched_line(
        painter,
        wobble(check_x - 1.0, check_y + 3.0, seed + 53.0),
        wobble(check_x + 5.0, check_y - 4.0, seed + 54.0),
        color, 1.5, seed + 55.0,
    );
}

/// Gear / settings icon
pub fn icon_gear(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 197.0;

    let outer_r = size * 0.3;
    let inner_r = size * 0.12;

    // Outer gear teeth
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let tooth_r = outer_r + 2.5;
        let (sx, sy) = (angle.cos(), angle.sin());
        let p1 = pos2(
            center.x + sx * (outer_r - 0.5),
            center.y + sy * (outer_r - 0.5),
        );
        let p2 = pos2(center.x + sx * tooth_r, center.y + sy * tooth_r);
        sketched_line(
            painter,
            wobble(p1.x, p1.y, seed + i as f32),
            wobble(p2.x, p2.y, seed + 10.0 + i as f32),
            color, 2.0, seed + 20.0 + i as f32,
        );
    }

    // Outer circle
    let circle_points: Vec<Pos2> = (0..20)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 20.0;
            let p = pos2(center.x + angle.cos() * outer_r, center.y + angle.sin() * outer_r);
            wobble(p.x, p.y, seed + 100.0 + i as f32)
        })
        .collect();
    for i in 0..circle_points.len() {
        let p1 = circle_points[i];
        let p2 = circle_points[(i + 1) % circle_points.len()];
        let mid = pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
        painter.add(Shape::CubicBezier(epaint::CubicBezierShape {
            points: [p1, mid, mid, p2],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.5, color),
        }));
    }

    // Inner circle
    painter.circle_filled(center, inner_r, color.linear_multiply(0.1));
    painter.circle_stroke(center, inner_r, Stroke::new(1.5, color));
}

/// Search / magnifying glass
pub fn icon_search(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 227.0;

    let glass_r = size * 0.28;
    let glass_center = center - vec2(2.0, 2.0);

    // Circle
    let circle_pts: Vec<Pos2> = (0..16)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 16.0;
            let p = pos2(glass_center.x + angle.cos() * glass_r, glass_center.y + angle.sin() * glass_r);
            wobble(p.x, p.y, seed + i as f32)
        })
        .collect();
    for i in 0..circle_pts.len() {
        let p1 = circle_pts[i];
        let p2 = circle_pts[(i + 1) % circle_pts.len()];
        let mid = pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
        painter.add(Shape::CubicBezier(epaint::CubicBezierShape {
            points: [p1, mid, mid, p2],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.8, color),
        }));
    }

    // Handle
    let handle_start = pos2(
        glass_center.x + glass_r * 0.7,
        glass_center.y + glass_r * 0.7,
    );
    let handle_end = pos2(handle_start.x + 6.0, handle_start.y + 6.0);
    sketched_line(painter, handle_start, handle_end, color, 2.0, seed + 50.0);
}

/// Star icon (for ratings)
pub fn icon_star(ui: &mut Ui, size: f32, color: Color32, filled: bool) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 257.0;
    let outer_r = size * 0.4;
    let inner_r = outer_r * 0.4;

    let pts: Vec<Pos2> = (0..10)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 10.0 - std::f32::consts::TAU / 4.0;
            let r = if i % 2 == 0 { outer_r } else { inner_r };
            let p = pos2(center.x + angle.cos() * r, center.y + angle.sin() * r);
            wobble(p.x, p.y, seed + i as f32)
        })
        .collect();

    if filled {
        painter.add(Shape::convex_polygon(
            pts.clone(),
            color,
            Stroke::new(0.5, color),
        ));
    } else {
        for i in 0..pts.len() {
            let p1 = pts[i];
            let p2 = pts[(i + 1) % pts.len()];
            sketched_line(painter, p1, p2, color, 1.0, seed + i as f32 * 3.0);
        }
    }
}

/// Download arrow icon
pub fn icon_download(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 283.0;

    // Arrow shaft going down
    let top = center.y - 8.0;
    let bottom = center.y + 4.0;
    sketched_line(
        painter,
        wobble(center.x, top, seed),
        wobble(center.x, bottom, seed + 1.0),
        color, 2.0, seed + 2.0,
    );

    // Arrow head
    sketched_line(
        painter,
        wobble(center.x - 5.0, bottom - 4.0, seed + 3.0),
        wobble(center.x, bottom + 2.0, seed + 4.0),
        color, 2.0, seed + 5.0,
    );
    sketched_line(
        painter,
        wobble(center.x, bottom + 2.0, seed + 6.0),
        wobble(center.x + 5.0, bottom - 4.0, seed + 7.0),
        color, 2.0, seed + 8.0,
    );

    // Top bar
    sketched_line(
        painter,
        wobble(center.x - 6.0, top, seed + 9.0),
        wobble(center.x + 6.0, top, seed + 10.0),
        color, 1.5, seed + 11.0,
    );
}

/// Heart icon
pub fn icon_heart(ui: &mut Ui, size: f32, color: Color32, filled: bool) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 317.0;
    let s = size * 0.35;

    // Heart shape using two arcs and a bottom point
    let left_center = pos2(center.x - s * 0.5, center.y - s * 0.3);
    let right_center = pos2(center.x + s * 0.5, center.y - s * 0.3);
    let bottom = pos2(center.x, center.y + s * 0.8);

    if filled {
        // Approximate filled heart with polygon
        let mut pts = Vec::new();
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::TAU / 8.0;
            let p = pos2(left_center.x + angle.cos() * s * 0.55, left_center.y + angle.sin() * s * 0.45);
            if p.x <= left_center.x {
                pts.push(wobble(p.x, p.y, seed + i as f32));
            }
        }
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::TAU / 8.0;
            let p = pos2(right_center.x + angle.cos() * s * 0.55, right_center.y + angle.sin() * s * 0.45);
            if p.x >= right_center.x {
                pts.push(wobble(p.x, p.y, seed + 100.0 + i as f32));
            }
        }
        pts.push(wobble(bottom.x, bottom.y, seed + 200.0));
        painter.add(Shape::convex_polygon(pts, color, Stroke::new(0.5, color)));
    } else {
        // Left arc
        let la = epaint::CubicBezierShape {
            points: [
                wobble(left_center.x, left_center.y + s * 0.45, seed),
                wobble(left_center.x - s * 0.7, left_center.y + s * 0.1, seed + 1.0),
                wobble(left_center.x - s * 0.5, left_center.y - s * 0.6, seed + 2.0),
                wobble(left_center.x, left_center.y - s * 0.3, seed + 3.0),
            ],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.5, color),
        };
        painter.add(Shape::CubicBezier(la));

        // Right arc
        let ra = epaint::CubicBezierShape {
            points: [
                wobble(right_center.x, right_center.y - s * 0.3, seed + 4.0),
                wobble(right_center.x + s * 0.5, right_center.y - s * 0.6, seed + 5.0),
                wobble(right_center.x + s * 0.7, right_center.y + s * 0.1, seed + 6.0),
                wobble(right_center.x, right_center.y + s * 0.45, seed + 7.0),
            ],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.5, color),
        };
        painter.add(Shape::CubicBezier(ra));

        // Bottom V
        sketched_line(painter, left_center + vec2(0.0, s * 0.45), bottom, color, 1.5, seed + 8.0);
        sketched_line(painter, right_center + vec2(0.0, s * 0.45), bottom, color, 1.5, seed + 9.0);
    }
}

/// Empty state icon — a film frame with a question mark
pub fn icon_empty_library(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 347.0;

    // Oversized film frame
    let frame = Rect::from_center_size(center, vec2(size * 0.7, size * 0.5));
    let stroke = Stroke::new(1.5, color.linear_multiply(0.4));
    painter.rect_stroke(frame, Rounding::same(4.0), stroke);

    // Question mark in center
    let qm_color = color.linear_multiply(0.5);
    // Top curve of ?
    let top_pts = [
        wobble(center.x, center.y - 7.0, seed),
        wobble(center.x + 5.0, center.y - 7.0, seed + 1.0),
        wobble(center.x + 5.0, center.y - 2.0, seed + 2.0),
        wobble(center.x, center.y + 1.0, seed + 3.0),
    ];
    for i in 0..top_pts.len() - 1 {
        sketched_line(painter, top_pts[i], top_pts[i + 1], qm_color, 1.5, seed + i as f32);
    }
    // Dot
    painter.circle_filled(wobble(center.x, center.y + 6.0, seed + 10.0), 1.5, qm_color);

    // Sprocket holes
    for i in 0..3 {
        let y = frame.min.y + frame.height() * (i as f32 + 0.5) / 3.0;
        painter.circle_filled(wobble(frame.min.x + 5.0, y, seed + 100.0 + i as f32), 2.0, color.linear_multiply(0.3));
        painter.circle_filled(wobble(frame.max.x - 5.0, y, seed + 200.0 + i as f32), 2.0, color.linear_multiply(0.3));
    }
}

/// Success checkmark
pub fn icon_checkmark(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 383.0;

    let s = size * 0.35;
    sketched_line(
        painter,
        wobble(center.x - s, center.y, seed),
        wobble(center.x - s * 0.3, center.y + s * 0.7, seed + 1.0),
        color, 2.5, seed + 2.0,
    );
    sketched_line(
        painter,
        wobble(center.x - s * 0.3, center.y + s * 0.7, seed + 3.0),
        wobble(center.x + s, center.y - s * 0.5, seed + 4.0),
        color, 2.5, seed + 5.0,
    );
}

pub fn icon_chat(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 420.0;

    // Speech bubble rounded rect
    let bw = size * 0.5;
    let bh = size * 0.38;
    let r = Rect::from_center_size(center - vec2(0.0, 1.0), vec2(bw, bh));
    painter.rect_filled(r, Rounding::same(5.0), color.linear_multiply(0.06));
    painter.rect_stroke(r, Rounding::same(5.0), Stroke::new(1.2, color));
    // Tail
    let tail = [
        wobble(center.x - 3.0, r.max.y, seed),
        wobble(center.x + 1.0, r.max.y + 5.0, seed + 1.0),
        wobble(center.x + 5.0, r.max.y, seed + 2.0),
    ];
    painter.add(Shape::convex_polygon(
        tail.to_vec(),
        color.linear_multiply(0.06),
        Stroke::new(1.2, color),
    ));
    // Three dots inside
    for i in 0..3 {
        let dx = (i as f32 - 1.0) * 5.0;
        painter.circle_filled(
            wobble(center.x + dx, center.y - 1.0, seed + 30.0 + i as f32),
            1.5,
            color.linear_multiply(0.5),
        );
    }
}

pub fn icon_sparkle(ui: &mut Ui, size: f32, color: Color32) {
    let painter = ui.painter();
    let center = ui.cursor().min + vec2(size / 2.0, size / 2.0);
    let seed = 560.0;
    let r = size * 0.38;

    // Four-pointed sparkle star
    for i in 0..4 {
        let a = i as f32 * std::f32::consts::TAU / 4.0;
        let a2 = a + std::f32::consts::TAU / 8.0;
        // Long point
        let p_outer = wobble(
            center.x + a.cos() * r,
            center.y + a.sin() * r,
            seed + i as f32,
        );
        // Short indent
        let p_inner1 = wobble(
            center.x + a2.cos() * r * 0.3,
            center.y + a2.sin() * r * 0.3,
            seed + 10.0 + i as f32,
        );
        let p_inner2 = wobble(
            center.x + (a2 + std::f32::consts::TAU / 4.0).cos() * r * 0.3,
            center.y + (a2 + std::f32::consts::TAU / 4.0).sin() * r * 0.3,
            seed + 11.0 + i as f32,
        );
        let next_a = a + std::f32::consts::TAU / 4.0;
        let p_next_outer = wobble(
            center.x + next_a.cos() * r,
            center.y + next_a.sin() * r,
            seed + (i + 1) as f32,
        );
        // Connect with lines: outer -> inner -> next outer
        sketched_line(painter, p_outer, p_inner1, color, 1.4, seed + 20.0 + i as f32);
        sketched_line(painter, p_inner1, p_next_outer, color, 1.4, seed + 21.0 + i as f32);
    }
    // Center dot
    painter.circle_filled(center, 2.2, color);
}

/// Draw any icon by name
pub fn draw_icon(ui: &mut Ui, name: &str, size: f32, color: Color32) {
    match name {
        "film" | "library" => icon_film(ui, size, color),
        "add" | "add-folder" => icon_add_folder(ui, size, color),
        "subtitle" | "subtitles" => icon_subtitle(ui, size, color),
        "bolt" | "batch" => icon_bolt(ui, size, color),
        "bookmark" | "watchlist" => icon_bookmark(ui, size, color),
        "gear" | "settings" => icon_gear(ui, size, color),
        "search" => icon_search(ui, size, color),
        "star" => icon_star(ui, size, color, true),
        "star-outline" => icon_star(ui, size, color, false),
        "download" => icon_download(ui, size, color),
        "heart" => icon_heart(ui, size, color, true),
        "heart-outline" => icon_heart(ui, size, color, false),
        "checkmark" | "success" => icon_checkmark(ui, size, color),
        "empty-library" => icon_empty_library(ui, size, color),
        "chat" => icon_chat(ui, size, color),
        "sparkle" | "ai" => icon_sparkle(ui, size, color),
        _ => {}
    }
}
