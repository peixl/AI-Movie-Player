//! Batch operations panel for bulk library management.

use egui::{Color32, ProgressBar, RichText, Ui};

/// Batch operations panel state tracking scan progress and import results.
pub struct BatchOpsPanel {
    pub scanning: bool,
    pub total_files: usize,
    pub processed: usize,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub current_file: Option<String>,
    pub message: Option<String>,
}

impl BatchOpsPanel {
    pub fn new() -> Self {
        Self {
            scanning: false,
            total_files: 0,
            processed: 0,
            imported_count: 0,
            skipped_count: 0,
            failed_count: 0,
            current_file: None,
            message: None,
        }
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
            crate::ui::icons::draw_icon(ui, "bolt", 22.0, primary);
            ui.add_space(8.0);
            ui.heading(RichText::new("Batch Operations / 批量整理").size(22.0).color(text));
        });
        ui.add_space(12.0);

        if !self.scanning {
            ui.vertical_centered(|ui| {
                crate::ui::icons::icon_bolt(ui, 48.0, primary.linear_multiply(0.5));
                ui.add_space(12.0);
            });
            ui.label("扫描整个媒体库，一次导入所有影片。 / Scan your entire media library and import all movies at once.");
            ui.add_space(8.0);
            ui.label(RichText::new("This will / 这会执行:").size(13.0).color(dim));
            ui.label("  • 递归扫描所选文件夹 / Scan selected folder recursively");
            ui.label("  • 找到全部视频文件 / Find all video files");
            ui.label("  • 自动匹配 TMDB / Match against TMDB automatically");
            ui.label("  • 下载海报并保存元数据 / Download posters and save metadata");
            ui.add_space(16.0);
            if ui
                .button(
                    RichText::new("Start Batch Scan / 开始批量扫描")
                        .size(14.0)
                        .fill(Color32::WHITE),
                )
                .clicked()
            {
                self.scanning = true;
                self.message = Some("请选择要扫描的文件夹 / Select a folder to scan".into());
            }
        } else {
            ui.label(format!(
                "扫描中：已处理 {} / {} 个文件 / Scanning: {} of {} files",
                self.processed, self.total_files, self.processed, self.total_files
            ));
        }

        // Progress
        if self.scanning && self.total_files > 0 {
            ui.add_space(16.0);
            let ratio = if self.total_files > 0 {
                self.processed as f32 / self.total_files as f32
            } else {
                0.0
            };
            ui.add(ProgressBar::new(ratio).desired_width(f32::INFINITY).fill(primary));
            ui.add_space(8.0);
            ui.label(format!(
                "已导入 {} | 已跳过 {} | 错误 {} / Imported {} | Skipped {} | Errors {}",
                self.imported_count,
                self.skipped_count,
                self.failed_count,
                self.imported_count,
                self.skipped_count,
                self.failed_count
            ));
            if let Some(ref file) = self.current_file {
                ui.label(
                    RichText::new(format!("当前文件 / Current file: {}", file))
                        .size(12.0)
                        .color(dim),
                );
            }
        }

        // Message
        if let Some(ref msg) = self.message {
            ui.add_space(12.0);
            ui.label(RichText::new(msg).color(Color32::from_rgb(52, 211, 153)));
        }
    }
}
