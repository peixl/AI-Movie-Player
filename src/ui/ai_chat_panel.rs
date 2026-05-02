//! AI-Movie-Player chat panel with streaming, presets, and multi-turn memory.

use std::sync::{Arc, Mutex};

use egui::{Color32, Frame, RichText, Rounding, ScrollArea, Sense, Stroke};
use rusqlite::Connection;

use crate::ai::chat;
use crate::api::ai::{AiClient, ChatMessage};
use crate::db::{models::Movie, watchlist};

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkflowKind {
    PreWatchBriefing,
    PostWatchRecap,
    DoubleFeaturePairing,
}

struct WorkflowPreset {
    kind: WorkflowKind,
    title: &'static str,
    description: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkflowTemplate {
    Lean,
    Balanced,
    DeepDive,
}

#[derive(Clone)]
struct WorkflowSection {
    title: String,
    body: String,
}

#[derive(Clone)]
struct WorkflowCard {
    kind: WorkflowKind,
    template: WorkflowTemplate,
    title: String,
    summary: String,
    generated_for: String,
    sections: Vec<WorkflowSection>,
}

#[derive(Clone)]
struct WorkflowNotice {
    message: String,
    is_error: bool,
}

#[derive(Clone)]
enum WorkflowStatus {
    Idle,
    Loading(WorkflowKind),
    Done(WorkflowCard),
    Error(Option<WorkflowKind>, String),
}

#[derive(Clone, PartialEq)]
enum StreamStatus {
    Idle,
    Waiting,
    Streaming(String),
    Done(String),
    Error(String),
}

struct ChatEntry {
    role: String,
    content: String,
    is_streaming: bool,
}

pub struct AiChatPanel {
    history: Vec<ChatEntry>,
    input: String,
    stream_status: Arc<Mutex<StreamStatus>>,
    workflow_status: Arc<Mutex<WorkflowStatus>>,
    stream_idx: Option<usize>,
    selected_movie: Option<Movie>,
    movie_search_input: String,
    presets_visible: bool,
    selected_template: WorkflowTemplate,
    workflow_notice: Option<WorkflowNotice>,
    workflow_cards: Vec<WorkflowCard>,
}

const VIEWING_WORKFLOWS: &[WorkflowPreset] = &[
    WorkflowPreset {
        kind: WorkflowKind::PreWatchBriefing,
        title: "Pre-Watch Briefing / 观影前导览",
        description: "A spoiler-light primer for mood, context, and what to notice before pressing play.",
    },
    WorkflowPreset {
        kind: WorkflowKind::PostWatchRecap,
        title: "Post-Watch Recap / 观影后复盘",
        description: "A structured reflection on what the film did, meant, and left behind after the credits.",
    },
    WorkflowPreset {
        kind: WorkflowKind::DoubleFeaturePairing,
        title: "Double Feature Pairing / 双片连看建议",
        description: "A second film that meaningfully amplifies this one instead of repeating it mechanically.",
    },
];

impl WorkflowKind {
    fn prompt(self) -> &'static str {
        match self {
            WorkflowKind::PreWatchBriefing => chat::prompts::pre_watch_briefing(),
            WorkflowKind::PostWatchRecap => chat::prompts::post_watch_recap(),
            WorkflowKind::DoubleFeaturePairing => chat::prompts::double_feature_pairing(),
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            WorkflowKind::PreWatchBriefing => "Pre-Watch Briefing / 观影前导览",
            WorkflowKind::PostWatchRecap => "Post-Watch Recap / 观影后复盘",
            WorkflowKind::DoubleFeaturePairing => "Double Feature Pairing / 双片连看",
        }
    }

    fn default_watchlist_status(self) -> &'static str {
        match self {
            WorkflowKind::PostWatchRecap => "watched",
            WorkflowKind::PreWatchBriefing | WorkflowKind::DoubleFeaturePairing => "want_to_watch",
        }
    }
}

impl WorkflowTemplate {
    fn label(self) -> &'static str {
        match self {
            WorkflowTemplate::Lean => "Lean / 轻量",
            WorkflowTemplate::Balanced => "Balanced / 平衡",
            WorkflowTemplate::DeepDive => "Deep Dive / 深入",
        }
    }

    fn description(self) -> &'static str {
        match self {
            WorkflowTemplate::Lean => "Fast, compact, and easy to act on before or after a film.",
            WorkflowTemplate::Balanced => "The default studio output: clear, elegant, and practical.",
            WorkflowTemplate::DeepDive => "Longer sections with more layered interpretation and viewing cues.",
        }
    }

    fn instructions(self) -> &'static str {
        match self {
            WorkflowTemplate::Lean => "Template mode: Lean / 轻量版. Keep SUMMARY to one compact bilingual paragraph. Keep each BODY concise, usually 1-2 sentences. Prioritize clarity and immediate usefulness.",
            WorkflowTemplate::Balanced => "Template mode: Balanced / 平衡版. Keep SUMMARY concise but textured. Keep each BODY to 2-3 sentences with a calm, elegant cadence.",
            WorkflowTemplate::DeepDive => "Template mode: Deep Dive / 深入版. Keep the exact structure, but make each BODY more layered and observant, usually 3-5 sentences without becoming bloated.",
        }
    }
}

const WORKFLOW_TEMPLATES: [WorkflowTemplate; 3] = [
    WorkflowTemplate::Lean,
    WorkflowTemplate::Balanced,
    WorkflowTemplate::DeepDive,
];

impl WorkflowCard {
    fn export_summary_text(&self) -> String {
        format!(
            "{}\n{}\nTemplate / 模板: {}\n{}",
            self.title,
            self.generated_for,
            self.template.label(),
            self.summary
        )
    }

    fn export_full_text(&self) -> String {
        let mut output = self.export_summary_text();
        for section in &self.sections {
            output.push_str("\n\n");
            output.push_str(&section.title);
            output.push_str("\n");
            output.push_str(&section.body);
        }
        output
    }
}

impl AiChatPanel {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            input: String::new(),
            stream_status: Arc::new(Mutex::new(StreamStatus::Idle)),
            workflow_status: Arc::new(Mutex::new(WorkflowStatus::Idle)),
            stream_idx: None,
            selected_movie: None,
            movie_search_input: String::new(),
            presets_visible: false,
            selected_template: WorkflowTemplate::Balanced,
            workflow_notice: None,
            workflow_cards: Vec::new(),
        }
    }

    pub fn select_movie(&mut self, movie: Option<Movie>) {
        let changed_movie = self.selected_movie.as_ref().map(|m| m.id) != movie.as_ref().map(|m| m.id);
        self.selected_movie = movie;
        if changed_movie {
            self.workflow_cards.clear();
            self.workflow_notice = None;
            *self.workflow_status.lock().unwrap() = WorkflowStatus::Idle;
        }
        if let Some(ref m) = self.selected_movie {
            self.history.push(ChatEntry {
                role: "system".into(),
                content: format!("当前影片 / Current film: **{}** ({})", m.title, m.year.unwrap_or(0)),
                is_streaming: false,
            });
        }
    }

    fn build_conversation_history(&self) -> Vec<ChatMessage> {
        self.history
            .iter()
            .filter_map(|entry| match entry.role.as_str() {
                "user" => Some(ChatMessage::user(entry.content.clone())),
                "assistant" if !entry.is_streaming && !entry.content.trim().is_empty() => {
                    Some(ChatMessage::assistant(entry.content.clone()))
                }
                _ => None,
            })
            .collect()
    }

    /// Poll the stream status and update history. Call every frame from app.rs.
    pub fn poll_stream(&mut self) {
        let status = self.stream_status.lock().unwrap().clone();
        match status {
            StreamStatus::Streaming(ref content) => {
                if let Some(idx) = self.stream_idx {
                    if let Some(entry) = self.history.get_mut(idx) {
                        entry.content = content.clone();
                    }
                }
            }
            StreamStatus::Done(ref content) => {
                if let Some(idx) = self.stream_idx {
                    if let Some(entry) = self.history.get_mut(idx) {
                        entry.content = content.clone();
                        entry.is_streaming = false;
                    }
                }
                *self.stream_status.lock().unwrap() = StreamStatus::Idle;
                self.stream_idx = None;
            }
            StreamStatus::Error(ref _err) => {
                if let Some(idx) = self.stream_idx {
                    if let Some(entry) = self.history.get_mut(idx) {
                        entry.is_streaming = false;
                    }
                }
                *self.stream_status.lock().unwrap() = StreamStatus::Idle;
                self.stream_idx = None;
            }
            _ => {}
        }

        let workflow_status = self.workflow_status.lock().unwrap().clone();
        if let WorkflowStatus::Done(card) = workflow_status {
            self.upsert_workflow_card(card);
            *self.workflow_status.lock().unwrap() = WorkflowStatus::Idle;
        }
    }

    /// Returns true if user wants to navigate to settings.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        db: &Connection,
        client: &Option<Arc<AiClient>>,
        library: &[Movie],
        runtime: &tokio::runtime::Runtime,
        is_dark: bool,
    ) -> bool {
        let mut navigate_settings = false;
        let text = if is_dark {
            Color32::from_rgb(240, 240, 245)
        } else {
            Color32::from_rgb(15, 15, 25)
        };
        let dim = if is_dark {
            Color32::from_rgb(150, 150, 165)
        } else {
            Color32::from_rgb(100, 100, 115)
        };
        let primary = Color32::from_rgb(99, 102, 241);
        let bg = if is_dark {
            Color32::from_rgb(17, 17, 25)
        } else {
            Color32::from_rgb(250, 250, 253)
        };

        // Header
        ui.horizontal(|ui| {
            crate::ui::icons::draw_icon(ui, "chat", 22.0, primary);
            ui.add_space(8.0);
            ui.heading(RichText::new("AI Companion / AI 电影对话").size(22.0).color(text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(RichText::new("Clear / 清空").size(12.0).color(dim))
                    .clicked()
                {
                    self.history.clear();
                    self.workflow_notice = None;
                    if let Some(ref m) = self.selected_movie {
                        self.history.push(ChatEntry {
                            role: "system".into(),
                            content: format!(
                                "当前影片 / Current film: **{}** ({})",
                                m.title,
                                m.year.unwrap_or(0)
                            ),
                            is_streaming: false,
                        });
                    }
                }
            });
        });
        ui.add_space(4.0);

        // AI status indicator
        let is_ready = client.as_ref().map(|c| c.is_ready()).unwrap_or(false);
        ui.horizontal(|ui| {
            let dot_color = if is_ready {
                Color32::from_rgb(52, 211, 153)
            } else {
                Color32::from_rgb(239, 68, 68)
            };
            ui.painter()
                .circle_filled(ui.next_widget_position() + egui::vec2(6.0, 6.0), 4.0, dot_color);
            ui.add_space(10.0);
            ui.label(
                RichText::new(if is_ready {
                    "AI 已就绪 / Ready to assist · ifq.ai"
                } else {
                    "AI 未就绪 · 在设置中配置 / Configure AI in Settings"
                })
                .size(12.0)
                .color(dim),
            );
            if !is_ready {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn = egui::Button::new(
                        RichText::new("Open Settings / 打开设置").size(12.0).color(Color32::WHITE),
                    )
                    .fill(primary)
                    .rounding(Rounding::same(6.0));
                    if ui.add(btn).clicked() {
                        navigate_settings = true;
                    }
                });
            }
        });
        ui.add_space(8.0);
        ui.separator();

        // Movie context selector
        ui.horizontal(|ui| {
            ui.label(RichText::new("Context / 上下文:").size(13.0).color(dim));
            if let Some(ref m) = self.selected_movie {
                ui.label(
                    RichText::new(format!("{} ({})", m.title, m.year.unwrap_or(0)))
                        .size(13.0)
                        .color(primary),
                );
                if ui
                    .small_button(RichText::new("✕").size(11.0).color(dim))
                    .clicked()
                {
                    self.selected_movie = None;
                    self.history.push(ChatEntry {
                        role: "system".into(),
                        content: "已切换为通用电影对话 / Switched to general movie chat".into(),
                        is_streaming: false,
                    });
                }
            } else {
                ui.label(
                    RichText::new("通用模式 / General mode")
                        .size(13.0)
                        .color(dim),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(RichText::new("Pick / 选择影片").size(12.0).color(primary))
                    .clicked()
                {
                    self.presets_visible = !self.presets_visible;
                }
            });
        });

        // Movie picker
        if self.presets_visible {
            ui.add_space(4.0);
            Frame::none()
                .fill(bg)
                .rounding(Rounding::same(8.0))
                .inner_margin(egui::Vec2::splat(8.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.movie_search_input)
                            .hint_text("Search library / 搜索片库...")
                            .desired_width(200.0),
                    );
                    ui.add_space(4.0);
                    ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                        for movie in library.iter().filter(|m| {
                            self.movie_search_input.is_empty()
                                || m.title
                                    .to_lowercase()
                                    .contains(&self.movie_search_input.to_lowercase())
                        }) {
                            let label = format!("{} ({})", movie.title, movie.year.unwrap_or(0));
                            if ui
                                .selectable_label(
                                    self.selected_movie
                                        .as_ref()
                                        .map(|s| s.id == movie.id)
                                        .unwrap_or(false),
                                    RichText::new(&label).size(13.0),
                                )
                                .clicked()
                            {
                                self.select_movie(Some(movie.clone()));
                                self.presets_visible = false;
                            }
                        }
                    });
                });
            ui.add_space(4.0);
        }

        ui.add_space(8.0);
        self.show_workflow_panel(ui, db, client, runtime, is_dark, text, dim, primary, bg);

        ui.add_space(8.0);

        // Chat message area
        let chat_frame = Frame::none()
            .fill(bg)
            .rounding(Rounding::same(10.0))
            .inner_margin(egui::Vec2::splat(10.0));

        chat_frame.show(ui, |ui| {
            let available_height = ui.available_height() - 120.0;
            ScrollArea::vertical()
                .max_height(available_height.max(200.0))
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if self.history.is_empty() {
                        ui.add_space(20.0);
                        ui.vertical_centered(|ui| {
                            crate::ui::icons::draw_icon(
                                ui,
                                "chat",
                                48.0,
                                dim.linear_multiply(0.3),
                            );
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new("AI-Movie-Player · Cinematic Intelligence")
                                    .size(16.0)
                                    .color(dim),
                            );
                            ui.label(
                                RichText::new(
                                    "选择一部影片开始深入对话，或直接进入通用电影模式。\n\
                                     Ask for analysis, recommendations, endings, trivia, a pre-watch briefing, a post-watch recap, or a refined watch companion.",
                                )
                                .size(12.0)
                                .color(dim.linear_multiply(0.7)),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("OpenAI-compatible · OpenAI / Azure / Ollama / LM Studio · subtly crafted by ifq.ai")
                                    .size(11.0)
                                    .color(dim.linear_multiply(0.5)),
                            );
                        });
                        ui.add_space(20.0);
                    }

                    for (i, entry) in self.history.iter().enumerate() {
                        match entry.role.as_str() {
                            "system" => {
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new(&entry.content)
                                            .size(11.0)
                                            .color(dim)
                                            .italics(),
                                    );
                                });
                                ui.add_space(4.0);
                            }
                            _ => {
                                let is_user = entry.role == "user";
                                let align = if is_user {
                                    egui::Align::RIGHT
                                } else {
                                    egui::Align::LEFT
                                };

                                ui.with_layout(
                                    egui::Layout::top_down_justified(align),
                                    |ui| {
                                        let max_width = ui.available_width() * 0.75;
                                        let bubble_bg = if is_user {
                                            primary
                                        } else if is_dark {
                                            Color32::from_rgb(40, 40, 55)
                                        } else {
                                            Color32::from_rgb(240, 240, 246)
                                        };
                                        let bubble_text = if is_user {
                                            Color32::WHITE
                                        } else {
                                            text
                                        };

                                        let role_label =
                                            if is_user { "You" } else { "AI" };
                                        let role_color = if is_user {
                                            Color32::WHITE.linear_multiply(0.6)
                                        } else {
                                            primary
                                        };

                                        ui.horizontal(|ui| {
                                            if !is_user {
                                                ui.label(
                                                    RichText::new(role_label)
                                                        .size(10.0)
                                                        .color(role_color)
                                                        .strong(),
                                                );
                                            }
                                            let content = if entry.is_streaming {
                                                format!("{} ●", entry.content)
                                            } else {
                                                entry.content.clone()
                                            };
                                            let galley = ui.painter().layout(
                                                content,
                                                egui::FontId::proportional(13.0),
                                                bubble_text,
                                                max_width,
                                            );
                                            let padding = egui::vec2(12.0, 8.0);
                                            let bubble_size = galley.size() + padding * 2.0;
                                            let bubble_pos = ui.next_widget_position();

                                            ui.painter().rect_filled(
                                                egui::Rect::from_min_size(
                                                    bubble_pos, bubble_size,
                                                ),
                                                Rounding::same(12.0),
                                                bubble_bg,
                                            );
                                            ui.painter().galley(
                                                bubble_pos + padding,
                                                galley,
                                                bubble_text,
                                            );
                                            ui.allocate_exact_size(
                                                bubble_size,
                                                Sense::hover(),
                                            );
                                            if is_user {
                                                ui.label(
                                                    RichText::new(format!(" {}", role_label))
                                                        .size(10.0)
                                                        .color(role_color)
                                                        .strong(),
                                                );
                                            }
                                        });
                                    },
                                );
                                ui.add_space(6.0);
                            }
                        }
                    }

                    // Loading indicator
                    let status = self.stream_status.lock().unwrap().clone();
                    if status == StreamStatus::Waiting {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let shimmer = crate::ui::animation::shimmer(
                                primary.linear_multiply(0.3),
                                primary.linear_multiply(0.6),
                                ui.ctx(),
                                0.8,
                            );
                            ui.painter().circle_filled(
                                ui.next_widget_position() + egui::vec2(12.0, 8.0),
                                5.0,
                                shimmer,
                            );
                            ui.add_space(14.0);
                            ui.label(
                                RichText::new("AI 生成中 / Generating...")
                                    .size(12.0)
                                    .color(dim),
                            );
                        });
                        ui.add_space(4.0);
                        ui.ctx().request_repaint();
                    }

                    if let StreamStatus::Error(ref err) = status {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("生成出错 / Generation error: {}", err))
                                .size(12.0)
                                .color(Color32::from_rgb(239, 68, 68)),
                        );
                    }
                });
        });

        ui.add_space(8.0);

        // Preset quick questions
        Frame::none()
            .fill(bg.linear_multiply(0.3))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Vec2::splat(8.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Quick Prompts / 快捷灵感")
                        .size(12.0)
                        .color(dim)
                        .strong(),
                );
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    let presets: &[(&str, &str)] = if self.selected_movie.is_some() {
                        &[
                            ("Deep Analysis / 深度解析", chat::prompts::deep_analysis()),
                            ("Ending Decode / 结局解读", chat::prompts::explain_ending()),
                            ("Similar Gems / 相似佳作", chat::prompts::similar_movies()),
                            ("Trivia / 幕后趣闻", chat::prompts::trivia()),
                            ("Watch Companion / 观影陪伴", chat::prompts::watch_companion()),
                            ("Worth Watching? / 值得看吗", chat::prompts::worth_watching()),
                        ]
                    } else {
                        &[
                            (
                                "Tonight's Pick / 今晚看什么",
                                "请用中英双语，根据经典电影与普适审美，推荐今晚适合看的 3 部电影，并说明适合的情绪与观看时机。",
                            ),
                            (
                                "Great Directors / 导演入门",
                                "请用中英双语介绍最值得入门的 5 位世界级导演，以及他们各自最适合作为第一部看的代表作。",
                            ),
                            (
                                "Film History / 电影史一页",
                                "请用中英双语介绍电影史上一个重要流派或运动，并解释它为何至今仍有影响。",
                            ),
                            (
                                "Hidden Gems / 冷门佳片",
                                "请用中英双语推荐 5 部被低估的电影，每部都说明它真正值得被重新发现的原因。",
                            ),
                        ]
                    };

                    for (label, prompt) in presets {
                        let btn = egui::Button::new(RichText::new(label).size(11.0))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, primary.linear_multiply(0.4)))
                            .rounding(Rounding::same(14.0));
                        if ui.add(btn).clicked() {
                            self.send_message(
                                client,
                                prompt.to_string(),
                                self.selected_movie.clone(),
                                runtime,
                            );
                        }
                    }
                });
            });

        ui.add_space(6.0);

        // Input bar
        ui.horizontal(|ui| {
            let input_field = egui::TextEdit::multiline(&mut self.input)
                .hint_text(if self.selected_movie.is_some() {
                    "Ask anything about this film / 问这部电影的任何问题..."
                } else {
                    "Ask anything about cinema / 问电影、导演、流派、历史..."
                })
                .desired_width(f32::INFINITY)
                .desired_rows(1);

            let response = ui.add(input_field);

            let is_idle = *self.stream_status.lock().unwrap() == StreamStatus::Idle;
            let can_send = !self.input.trim().is_empty()
                && client.as_ref().map(|c| c.is_ready()).unwrap_or(false)
                && is_idle;

            let send_btn = egui::Button::new(
                RichText::new("Send / 发送").size(13.0).color(Color32::WHITE),
            )
            .fill(if can_send {
                primary
            } else {
                primary.linear_multiply(0.3)
            })
            .rounding(Rounding::same(6.0));

            let send_clicked = ui.add_enabled(can_send, send_btn).clicked();
            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if (send_clicked || enter_pressed) && can_send {
                let message = std::mem::take(&mut self.input);
                self.send_message(client, message, self.selected_movie.clone(), runtime);
            }
        });

        navigate_settings
    }

    fn send_message(
        &mut self,
        client: &Option<Arc<AiClient>>,
        message: String,
        movie: Option<Movie>,
        runtime: &tokio::runtime::Runtime,
    ) {
        let Some(client) = client.as_ref() else { return };
        if !client.is_ready() {
            return;
        }

        let conversation_history = self.build_conversation_history();

        self.history.push(ChatEntry {
            role: "user".into(),
            content: message.clone(),
            is_streaming: false,
        });

        *self.stream_status.lock().unwrap() = StreamStatus::Waiting;

        let stream_idx = self.history.len();
        self.stream_idx = Some(stream_idx);
        self.history.push(ChatEntry {
            role: "assistant".into(),
            content: String::new(),
            is_streaming: true,
        });

        let client = client.clone();
        let status = self.stream_status.clone();
        let movie_clone = movie.clone();

        runtime.spawn(async move {
            let mut full = String::new();
            let mut last_update_len: usize = 0;
            let result = if let Some(ref m) = movie_clone {
                chat::stream_chat(&client, m, &conversation_history, &message, |token| {
                    full.push_str(token);
                    // Only update shared state every ~80 chars to reduce clone overhead
                    if full.len() - last_update_len >= 80 {
                        last_update_len = full.len();
                        let mut s = status.lock().unwrap();
                        *s = StreamStatus::Streaming(full.clone());
                    }
                })
                .await
            } else {
                let mut messages = vec![ChatMessage::system(chat::build_general_context())];
                messages.extend(conversation_history);
                messages.push(ChatMessage::user(&message));

                client
                    .chat_stream(&messages, |token| {
                        full.push_str(token);
                        if full.len() - last_update_len >= 80 {
                            last_update_len = full.len();
                            let mut s = status.lock().unwrap();
                            *s = StreamStatus::Streaming(full.clone());
                        }
                    })
                    .await
            };

            // Always send the final accumulated content
            let final_content = match result {
                Ok(resp) => resp,
                Err(e) => {
                    let mut s = status.lock().unwrap();
                    *s = StreamStatus::Error(format!("{}", e));
                    return;
                }
            };
            let mut s = status.lock().unwrap();
            *s = StreamStatus::Done(final_content);
        });
    }

    fn show_workflow_panel(
        &mut self,
        ui: &mut egui::Ui,
        db: &Connection,
        client: &Option<Arc<AiClient>>,
        runtime: &tokio::runtime::Runtime,
        is_dark: bool,
        text: Color32,
        dim: Color32,
        primary: Color32,
        bg: Color32,
    ) {
        let workflow_status = self.workflow_status.lock().unwrap().clone();

        Frame::none()
            .fill(bg.linear_multiply(0.35))
            .rounding(Rounding::same(10.0))
            .inner_margin(egui::Vec2::splat(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    crate::ui::icons::draw_icon(ui, "sparkle", 18.0, primary);
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Workflow Studio / 工作流面板")
                            .size(13.0)
                            .color(text)
                            .strong(),
                    );
                });
                ui.add_space(4.0);

                if let Some(movie) = self.selected_movie.as_ref() {
                    ui.label(
                        RichText::new(format!(
                            "为 {} 生成结构化观影结果，并将导览、复盘与连看建议保留为可复用卡片。 / Structured viewing outputs for {} that stay reusable after generation.",
                            movie.title,
                            movie.title
                        ))
                        .size(11.0)
                        .color(dim.linear_multiply(0.85)),
                    );
                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new("Output Template / 输出模板:")
                                .size(11.0)
                                .color(dim),
                        );
                        for template in WORKFLOW_TEMPLATES {
                            if ui
                                .selectable_label(
                                    self.selected_template == template,
                                    RichText::new(template.label()).size(11.0),
                                )
                                .clicked()
                            {
                                self.selected_template = template;
                            }
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(self.selected_template.description())
                            .size(10.5)
                            .color(dim),
                    );
                    ui.add_space(10.0);

                    ui.columns(3, |columns| {
                        for (column, workflow) in columns.iter_mut().zip(VIEWING_WORKFLOWS.iter()) {
                            let is_loading = matches!(workflow_status, WorkflowStatus::Loading(kind) if kind == workflow.kind);
                            let button = egui::Button::new(
                                RichText::new(workflow.title).size(11.2).color(text),
                            )
                            .min_size(egui::vec2(column.available_width(), 42.0))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, primary.linear_multiply(0.35)))
                            .rounding(Rounding::same(10.0));

                            if column.add_enabled(!is_loading, button).clicked() {
                                self.run_workflow(workflow.kind, client, runtime);
                            }

                            column.add_space(4.0);
                            column.label(
                                RichText::new(workflow.description)
                                    .size(10.5)
                                    .color(dim),
                            );

                            if is_loading {
                                column.add_space(4.0);
                                column.label(
                                    RichText::new("生成中 / Generating...")
                                        .size(10.5)
                                        .color(primary),
                                );
                            }
                        }
                    });

                    ui.add_space(12.0);

                    match &workflow_status {
                        WorkflowStatus::Error(kind, err) => {
                            let label = kind.map(|k| k.short_label()).unwrap_or("Workflow / 工作流");
                            ui.label(
                                RichText::new(format!("{} 生成失败 / {} generation failed: {}", label, label, err))
                                    .size(11.5)
                                    .color(Color32::from_rgb(239, 68, 68)),
                            );
                            ui.add_space(8.0);
                        }
                        WorkflowStatus::Loading(kind) if self.workflow_cards.is_empty() => {
                            ui.label(
                                RichText::new(format!(
                                    "{} 正在生成结构化结果，请稍候。 / Building a structured result card...",
                                    kind.short_label()
                                ))
                                .size(11.5)
                                .color(dim),
                            );
                            ui.add_space(8.0);
                        }
                        _ => {}
                    }

                    if let Some(notice) = &self.workflow_notice {
                        ui.label(
                            RichText::new(&notice.message)
                                .size(11.0)
                                .color(if notice.is_error {
                                    Color32::from_rgb(239, 68, 68)
                                } else {
                                    Color32::from_rgb(52, 211, 153)
                                }),
                        );
                        ui.add_space(8.0);
                    }

                    if self.workflow_cards.is_empty() {
                        ui.label(
                            RichText::new("暂无工作流卡片 / No workflow cards yet\n生成一张卡片，保存可复用的导览、复盘或连看建议 / Generate one to keep reusable viewing guidance.")
                                .size(11.0)
                                .color(dim),
                        );
                    } else {
                        let mut pending_copy_summary: Option<String> = None;
                        let mut pending_copy_full: Option<String> = None;
                        let mut pending_save: Option<WorkflowCard> = None;

                        for card in self.workflow_cards.clone() {
                            Frame::none()
                                .fill(if is_dark {
                                    Color32::from_rgb(24, 24, 34)
                                } else {
                                    Color32::from_rgb(245, 246, 250)
                                })
                                .rounding(Rounding::same(10.0))
                                .inner_margin(egui::Vec2::splat(12.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(&card.title)
                                                .size(13.0)
                                                .color(text)
                                                .strong(),
                                        );
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(
                                                RichText::new(&card.generated_for)
                                                    .size(10.5)
                                                    .color(dim),
                                            );
                                        });
                                    });
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(format!("Template / 模板: {}", card.template.label()))
                                            .size(10.5)
                                            .color(primary),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(&card.summary)
                                            .size(11.5)
                                            .color(dim),
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal_wrapped(|ui| {
                                        if ui.small_button("Copy Summary / 复制摘要").clicked() {
                                            pending_copy_summary = Some(card.export_summary_text());
                                        }
                                        if ui.small_button("Copy Card / 复制卡片").clicked() {
                                            pending_copy_full = Some(card.export_full_text());
                                        }
                                        if ui.small_button("Save to Watchlist / 保存到片单").clicked() {
                                            pending_save = Some(card.clone());
                                        }
                                    });
                                    ui.add_space(8.0);

                                    for section in &card.sections {
                                        Frame::none()
                                            .fill(Color32::TRANSPARENT)
                                            .stroke(Stroke::new(1.0, primary.linear_multiply(0.18)))
                                            .rounding(Rounding::same(8.0))
                                            .inner_margin(egui::Vec2::splat(10.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    RichText::new(&section.title)
                                                        .size(11.5)
                                                        .color(primary)
                                                        .strong(),
                                                );
                                                ui.add_space(4.0);
                                                ui.label(
                                                    RichText::new(&section.body)
                                                        .size(11.5)
                                                        .color(text),
                                                );
                                            });
                                        ui.add_space(6.0);
                                    }
                                });
                            ui.add_space(8.0);
                        }

                        if let Some(text_to_copy) = pending_copy_summary {
                            ui.ctx().copy_text(text_to_copy);
                            self.workflow_notice = Some(WorkflowNotice {
                                message: "已复制摘要 / Summary copied".into(),
                                is_error: false,
                            });
                        }

                        if let Some(text_to_copy) = pending_copy_full {
                            ui.ctx().copy_text(text_to_copy);
                            self.workflow_notice = Some(WorkflowNotice {
                                message: "已复制完整卡片 / Full workflow card copied".into(),
                                is_error: false,
                            });
                        }

                        if let Some(card) = pending_save {
                            self.save_workflow_card(db, &card);
                        }
                    }
                } else {
                    ui.label(
                        RichText::new("Pick a film first to unlock pre-watch briefing, post-watch recap, and double-feature pairing. / 先选择影片，再开启观影前导览、观影后复盘与双片连看建议。")
                            .size(11.0)
                            .color(dim.linear_multiply(0.85)),
                    );
                }
            });
    }

    fn run_workflow(
        &mut self,
        kind: WorkflowKind,
        client: &Option<Arc<AiClient>>,
        runtime: &tokio::runtime::Runtime,
    ) {
        let Some(client) = client.as_ref() else { return };
        let Some(movie) = self.selected_movie.clone() else { return };
        if !client.is_ready() {
            return;
        }

        self.workflow_notice = None;
        *self.workflow_status.lock().unwrap() = WorkflowStatus::Loading(kind);

        let client = client.clone();
        let workflow_status = self.workflow_status.clone();
        let template = self.selected_template;

        runtime.spawn(async move {
            let prompt = format!("{}\n\n{}", kind.prompt(), template.instructions());
            match chat::quick_insight(&client, &movie, &prompt).await {
                Ok(content) => {
                    let card = parse_workflow_card(kind, template, &movie, &content);
                    *workflow_status.lock().unwrap() = WorkflowStatus::Done(card);
                }
                Err(err) => {
                    *workflow_status.lock().unwrap() = WorkflowStatus::Error(Some(kind), format!("{}", err));
                }
            }
        });
    }

    fn upsert_workflow_card(&mut self, card: WorkflowCard) {
        if let Some(existing) = self.workflow_cards.iter_mut().find(|existing| existing.kind == card.kind) {
            *existing = card;
        } else {
            self.workflow_cards.push(card);
        }
    }

    fn save_workflow_card(&mut self, db: &Connection, card: &WorkflowCard) {
        let Some(movie) = self.selected_movie.as_ref() else { return };

        match watchlist::upsert_workflow_summary(
            db,
            movie,
            card.kind.default_watchlist_status(),
            &card.export_full_text(),
        ) {
            Ok(()) => {
                self.workflow_notice = Some(WorkflowNotice {
                    message: "工作流已保存到片单备注，并会出现在详情页 / Workflow saved to watchlist notes and detail view".into(),
                    is_error: false,
                });
            }
            Err(err) => {
                self.workflow_notice = Some(WorkflowNotice {
                    message: format!("工作流保存失败 / Workflow save failed: {}", err),
                    is_error: true,
                });
            }
        }
    }
}

fn parse_workflow_card(
    kind: WorkflowKind,
    template: WorkflowTemplate,
    movie: &Movie,
    content: &str,
) -> WorkflowCard {
    let mut title = kind.short_label().to_string();
    let mut summary = String::new();
    let mut sections = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_body: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(value) = trimmed.strip_prefix("TITLE:") {
            if let Some(section_title) = current_title.take() {
                sections.push(WorkflowSection {
                    title: section_title,
                    body: current_body.join("\n").trim().to_string(),
                });
                current_body.clear();
            }
            title = value.trim().to_string();
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("SUMMARY:") {
            if let Some(section_title) = current_title.take() {
                sections.push(WorkflowSection {
                    title: section_title,
                    body: current_body.join("\n").trim().to_string(),
                });
                current_body.clear();
            }
            summary = value.trim().to_string();
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("SECTION:") {
            if let Some(section_title) = current_title.take() {
                sections.push(WorkflowSection {
                    title: section_title,
                    body: current_body.join("\n").trim().to_string(),
                });
                current_body.clear();
            }
            current_title = Some(value.trim().to_string());
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("BODY:") {
            if !value.trim().is_empty() {
                current_body.push(value.trim().to_string());
            }
            continue;
        }

        if current_title.is_some() {
            current_body.push(trimmed.to_string());
        }
    }

    if let Some(section_title) = current_title.take() {
        sections.push(WorkflowSection {
            title: section_title,
            body: current_body.join("\n").trim().to_string(),
        });
    }

    if summary.is_empty() {
        summary = format!(
            "Structured workflow output for {} / 为 {} 生成的结构化观影结果",
            movie.title,
            movie.title
        );
    }

    if sections.is_empty() {
        sections.push(WorkflowSection {
            title: "Notes / 要点".into(),
            body: content.trim().to_string(),
        });
    }

    WorkflowCard {
        kind,
        template,
        title,
        summary,
        generated_for: format!("{} ({})", movie.title, movie.year.unwrap_or(0)),
        sections,
    }
}
