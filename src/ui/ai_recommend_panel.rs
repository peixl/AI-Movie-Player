//! AI Recommendation Panel — personalized suggestions, taste profile, discovery.

use std::sync::{Arc, Mutex};

use egui::{Color32, Frame, RichText, ScrollArea};

use crate::ai::recommend;
use crate::api::ai::AiClient;
use crate::db::models::Movie;
use crate::ui::Rounding;

#[derive(Clone, PartialEq)]
enum LoadState {
    Idle,
    Loading(String),
    Done(String),
    Error(String),
}

/// Each recommendation section holds its own `Arc<Mutex<LoadState>>` for safe async updates.
struct SectionState {
    content: Arc<Mutex<LoadState>>,
}

impl SectionState {
    fn new() -> Self {
        Self { content: Arc::new(Mutex::new(LoadState::Idle)) }
    }

    fn get(&self) -> LoadState {
        self.content.lock().unwrap().clone()
    }

    fn set(&self, state: LoadState) {
        *self.content.lock().unwrap() = state;
    }
}

pub struct AiRecommendPanel {
    library_recs: SectionState,
    discover_recs: SectionState,
    taste_profile: SectionState,
}

impl AiRecommendPanel {
    pub fn new() -> Self {
        Self {
            library_recs: SectionState::new(),
            discover_recs: SectionState::new(),
            taste_profile: SectionState::new(),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        client: &Option<Arc<AiClient>>,
        library: &[Movie],
        runtime: &tokio::runtime::Runtime,
        is_dark: bool,
    ) {
        let text =
            if is_dark { Color32::from_rgb(240, 240, 245) } else { Color32::from_rgb(15, 15, 25) };
        let dim = if is_dark {
            Color32::from_rgb(150, 150, 165)
        } else {
            Color32::from_rgb(100, 100, 115)
        };
        let primary = Color32::from_rgb(99, 102, 241);
        let bg =
            if is_dark { Color32::from_rgb(17, 17, 25) } else { Color32::from_rgb(250, 250, 253) };

        // Header
        ui.horizontal(|ui| {
            crate::ui::icons::draw_icon(ui, "sparkle", 22.0, primary);
            ui.add_space(8.0);
            ui.heading(RichText::new("AI Taste Engine / AI 推荐").size(22.0).color(text));
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new("Taste-aware cinematic guidance / 个性化观影引导 · by ifq.ai")
                .size(12.0)
                .color(dim),
        );
        ui.add_space(12.0);

        let is_ready = client.as_ref().map(|c| c.is_ready()).unwrap_or(false);
        if !is_ready {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                crate::ui::icons::draw_icon(ui, "sparkle", 48.0, dim.linear_multiply(0.3));
                ui.add_space(12.0);
                ui.label(
                    RichText::new("AI 未就绪 / Not configured")
                        .size(16.0)
                        .color(dim),
                );
                ui.label(
                    RichText::new("在设置中配置 OpenAI 兼容的 API 服务，开启 AI 推荐。 / Configure an OpenAI-compatible API in Settings to enable recommendations.")
                        .size(13.0)
                        .color(dim.linear_multiply(0.7)),
                );
            });
            return;
        }

        if library.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                crate::ui::icons::draw_icon(ui, "empty-library", 48.0, dim.linear_multiply(0.3));
                ui.add_space(12.0);
                ui.label(
                    RichText::new("片库为空 / Your library is empty")
                        .size(16.0)
                        .color(dim),
                );
                ui.label(
                    RichText::new("先导入影片，再获得个性化 AI 推荐与观影画像。 / Import movies first to unlock personalized AI guidance.")
                        .size(13.0)
                        .color(dim.linear_multiply(0.7)),
                );
            });
            return;
        }

        // Two-column layout: Library Recs + Taste Profile
        ui.columns(2, |cols| {
            self.render_section(
                &mut cols[0],
                "From Your Library / 从片库里选",
                "sparkle",
                "AI 从你的收藏里挑出 5 部此刻最值得看的影片",
                &self.library_recs,
                client,
                library,
                runtime,
                is_dark,
                |client, library| {
                    let system = recommend::build_library_context(&library, &[]);
                    let msg = "Based on my movie library, suggest 5 films I should watch next from my own collection. \
                              For each: explain why it fits my taste. Format: **Title** (Year) — Why watch it now.";
                    Box::pin(async move {
                        client.ask(&system, msg).await.map_err(|e| e.to_string())
                    })
                },
            );

            self.render_section(
                &mut cols[1],
                "Your Cinephile Profile / 你的观影画像",
                "heart",
                "AI 分析你的口味结构与观影人格",
                &self.taste_profile,
                client,
                library,
                runtime,
                is_dark,
                |client, library| {
                    let system = recommend::build_library_context(&library, &[]);
                    let msg = "Analyze my movie taste profile. What genres do I love? What directors? \
                              What era? What does my collection reveal about my personality? \
                              Be specific and reference actual films in my library. Make it fun.";
                    Box::pin(async move {
                        client.ask(&system, msg).await.map_err(|e| e.to_string())
                    })
                },
            );
        });

        ui.add_space(12.0);

        // Full-width: Discover New Films
        self.render_section(
            ui,
            "Discover New Films / 发现新电影",
            "search",
            "AI 推荐 8 部你还没收藏、但大概率会喜欢的电影",
            &self.discover_recs,
            client,
            library,
            runtime,
            is_dark,
            |client, library| {
                let system = recommend::build_library_context(&library, &[]);
                let msg = "Based on my library, recommend 8 real films I DON'T own but would absolutely love. \
                          Include a mix of classics and hidden gems. \
                          Format: **Title** (Year) — Director — Why you'll love it.";
                Box::pin(async move {
                    client.ask(&system, msg).await.map_err(|e| e.to_string())
                })
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_section(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        icon: &str,
        description: &str,
        state: &SectionState,
        client: &Option<Arc<AiClient>>,
        library: &[Movie],
        runtime: &tokio::runtime::Runtime,
        is_dark: bool,
        fetcher: impl Fn(
            Arc<AiClient>,
            Vec<Movie>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<String, String>> + Send>,
        > + Send
        + Sync
        + 'static,
    ) {
        let text =
            if is_dark { Color32::from_rgb(240, 240, 245) } else { Color32::from_rgb(15, 15, 25) };
        let dim = if is_dark {
            Color32::from_rgb(150, 150, 165)
        } else {
            Color32::from_rgb(100, 100, 115)
        };
        let primary = Color32::from_rgb(99, 102, 241);
        let bg =
            if is_dark { Color32::from_rgb(17, 17, 25) } else { Color32::from_rgb(250, 250, 253) };

        let current_state = state.get();

        Frame::NONE
            .fill(bg)
            .corner_radius(Rounding::same(10.0))
            .inner_margin(egui::Vec2::splat(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    crate::ui::icons::draw_icon(ui, icon, 18.0, primary);
                    ui.add_space(6.0);
                    ui.label(RichText::new(title).size(15.0).color(text).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        match &current_state {
                            LoadState::Idle => {
                                let btn = egui::Button::new(
                                    RichText::new("Generate / 生成")
                                        .size(12.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(primary)
                                .corner_radius(Rounding::same(6.0));
                                if ui.add(btn).clicked() {
                                    if let Some(client) = client.as_ref() {
                                        state.set(LoadState::Loading(
                                            "正在生成中 / Generating...".into(),
                                        ));
                                        let client = client.clone();
                                        let library = library.to_vec();
                                        let content_arc = state.content.clone();
                                        runtime.spawn(async move {
                                            match fetcher(client, library).await {
                                                Ok(content) => {
                                                    *content_arc.lock().unwrap() =
                                                        LoadState::Done(content);
                                                }
                                                Err(e) => {
                                                    *content_arc.lock().unwrap() =
                                                        LoadState::Error(e);
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                            LoadState::Loading(msg) => {
                                let shimmer = crate::ui::animation::shimmer(
                                    primary.linear_multiply(0.3),
                                    primary.linear_multiply(0.6),
                                    ui.ctx(),
                                    0.8,
                                );
                                ui.painter().circle_filled(
                                    ui.next_widget_position() + egui::vec2(6.0, 10.0),
                                    4.0,
                                    shimmer,
                                );
                                ui.add_space(12.0);
                                ui.label(RichText::new(msg).size(12.0).color(dim));
                                ui.ctx().request_repaint();
                            }
                            LoadState::Done(_) | LoadState::Error(_) => {
                                if ui
                                    .small_button(
                                        RichText::new("Regenerate / 重新生成")
                                            .size(11.0)
                                            .color(primary),
                                    )
                                    .clicked()
                                {
                                    state.set(LoadState::Idle);
                                }
                            }
                        }
                    });
                });
                ui.add_space(4.0);

                match &current_state {
                    LoadState::Idle => {
                        ui.label(RichText::new(description).size(12.0).color(dim));
                    }
                    LoadState::Loading(_) => {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "AI 正在分析你的片库... / AI is analyzing your library...",
                            )
                            .size(12.0)
                            .color(dim)
                            .italics(),
                        );
                    }
                    LoadState::Done(content) => {
                        ui.add_space(4.0);
                        ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                            for line in content.lines() {
                                if line.is_empty() {
                                    ui.add_space(4.0);
                                } else if line.starts_with('#') {
                                    ui.label(
                                        RichText::new(line.trim_start_matches('#'))
                                            .size(14.0)
                                            .color(text)
                                            .strong(),
                                    );
                                } else if line.starts_with("**") && line.contains("**") {
                                    ui.label(RichText::new(line).size(13.0).color(text).strong());
                                } else if line.starts_with('-') || line.starts_with('*') {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("•").size(12.0).color(primary));
                                        ui.label(
                                            RichText::new(line[1..].trim()).size(12.0).color(text),
                                        );
                                    });
                                } else {
                                    ui.label(RichText::new(line).size(12.0).color(text));
                                }
                            }
                        });
                    }
                    LoadState::Error(err) => {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("错误 / Error: {}", err))
                                .size(12.0)
                                .color(Color32::from_rgb(239, 68, 68)),
                        );
                    }
                }
            });
    }
}
