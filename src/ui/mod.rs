//! UI layer built on egui immediate-mode rendering.
//!
//! - [`layout`]: Sidebar navigation and view routing.
//! - [`theme`]: Color system, theme helpers, and dark/light mode.
//! - [`icons`]: Procedural hand-drawn icon system with organic wobble.
//! - [`animation`]: Hover, shimmer, toast, and pulse animations.
//! - [`poster_wall`]: Visual poster grid browsing with LRU texture cache.
//! - [`movie_detail`]: Movie detail panel with cached poster textures.
//! - [`ai_chat_panel`]: AI companion streaming chat interface.
//! - [`ai_recommend_panel`]: AI taste engine recommendation display.
//! - [`settings_panel`]: Settings and AI provider configuration.
//! - [`add_movie`]: Movie import workflow.
//! - [`subtitle_panel`]: Subtitle search and download interface.
//! - [`batch_ops`]: Batch operations for library management.
//! - [`watchlist_panel`]: Watchlist management interface.
//! - [`widgets`]: Reusable UI components.

pub(crate) struct Rounding;

impl Rounding {
    pub(crate) fn same(radius: f32) -> egui::CornerRadius {
        egui::CornerRadius::same(radius.round().clamp(0.0, u8::MAX as f32) as u8)
    }
}

pub mod add_movie;
pub mod ai_chat_panel;
pub mod ai_recommend_panel;
pub mod animation;
pub mod batch_ops;
pub mod icons;
pub mod layout;
pub mod movie_detail;
pub mod poster_wall;
pub mod settings_panel;
pub mod subtitle_panel;
pub mod theme;
pub mod watchlist_panel;
pub mod widgets;
