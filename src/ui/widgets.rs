use egui::{Color32, Response, RichText, Rounding, Ui, Vec2};

pub fn star_rating(ui: &mut Ui, rating: f64, max_stars: usize) -> Response {
    let star_size = 14.0;
    let full_stars = (rating / 2.0).floor() as usize;
    let half_star = (rating / 2.0) - full_stars as f64 >= 0.5;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(1.0);
        for i in 0..max_stars {
            let (star_char, color) = if i < full_stars {
                ("★", Color32::from_rgb(250, 176, 5))
            } else if i == full_stars && half_star {
                ("★", Color32::from_rgb(250, 176, 5).linear_multiply(0.6))
            } else {
                ("☆", Color32::from_rgb(80, 80, 95))
            };
            ui.label(RichText::new(star_char).size(star_size).color(color));
        }
    }).response
}

pub fn genre_chip(ui: &mut Ui, genre: &str, is_selected: bool, primary: Color32) -> Response {
    let text_color = if is_selected {
        Color32::WHITE
    } else {
        Color32::from_rgb(180, 180, 190)
    };
    let bg = if is_selected {
        primary
    } else {
        Color32::from_rgb(45, 45, 60)
    };

    let label = RichText::new(genre).size(12.0).color(text_color);
    let padding = egui::vec2(10.0, 4.0);
    let galley = ui.painter().layout_no_wrap(label.text().to_string(), egui::FontId::proportional(12.0), text_color);
    let size = galley.size() + padding * 2.0;

    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let rounding = Rounding::same(12.0);
        ui.painter().rect_filled(rect, rounding, bg);
        ui.painter().galley(rect.center() - galley.size() / 2.0, galley, text_color);
    }

    response
}

pub fn section_header(ui: &mut Ui, title: &str) {
    ui.add_space(16.0);
    ui.label(RichText::new(title).size(18.0).strong());
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(8.0);
}

pub fn info_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}:", label)).size(13.0).color(Color32::from_rgb(150, 150, 160)));
        ui.label(RichText::new(value).size(13.0));
    });
}

pub fn badge(ui: &mut Ui, text: &str, color: Color32) {
    let label = RichText::new(text).size(11.0).color(Color32::WHITE);
    let galley = ui.painter().layout_no_wrap(label.text().to_string(), egui::FontId::proportional(11.0), Color32::WHITE);
    let padding = egui::vec2(8.0, 3.0);
    let size = galley.size() + padding * 2.0;

    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, Rounding::same(4.0), color);
        ui.painter().galley(rect.center() - galley.size() / 2.0, galley, Color32::WHITE);
    }
}

/// Confirmation dialog that returns true when the user confirms.
/// Shows a modal-like overlay with message and Confirm/Cancel buttons.
pub fn confirm_dialog(ui: &mut Ui, title: &str, message: &str, confirm_label: &str) -> Option<bool> {
    let mut result = None;
    let dim = Color32::from_rgb(100, 100, 115);

    egui::Frame::none()
        .fill(Color32::from_rgba_premultiplied(0, 0, 0, 180))
        .rounding(Rounding::same(12.0))
        .inner_margin(egui::vec2(24.0, 20.0))
        .show(ui, |ui| {
            ui.set_width(320.0);
            ui.add_space(8.0);
            ui.label(RichText::new(title).size(16.0).strong());
            ui.add_space(8.0);
            ui.label(RichText::new(message).size(13.0).color(dim));
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new(confirm_label).color(Color32::WHITE)).clicked() {
                    result = Some(true);
                }
                if ui.button("Cancel / 取消").clicked() {
                    result = Some(false);
                }
            });
        });

    result
}

/// Simple modal overlay helper — use to wrap content that should appear on top
pub fn modal_overlay<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::none()
        .fill(Color32::from_rgba_premultiplied(0, 0, 0, 160))
        .inner_margin(egui::vec2(32.0, 24.0))
        .show(ui, |ui| {
            ui.set_max_width(400.0);
            content(ui)
        })
        .inner
}
