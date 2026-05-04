//! AI-Movie-Player: an AI-native movie player for people who care about cinema.
//!
//! The library crate owns the application modules so reusable movie, metadata,
//! subtitle, AI, and UI surfaces are checked as exported project APIs instead of
//! dead private code in the binary target.

pub mod ai;
pub mod api;
pub mod app;
pub mod config;
pub mod core;
pub mod db;
pub mod thumbnail;
pub mod ui;
pub mod util;
