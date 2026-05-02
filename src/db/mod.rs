//! Database layer: SQLite with WAL mode and FTS5 full-text search.
//!
//! - [`connection`]: Database connection setup and WAL configuration.
//! - [`migrations`]: Schema versioning and migration logic.
//! - [`models`]: Shared database model types.
//! - [`movies`]: Movie CRUD operations and search.
//! - [`settings`]: Key-value settings store.
//! - [`subtitles`]: Subtitle metadata storage.
//! - [`watchlist`]: Watchlist management and workflow cards.

pub mod connection;
pub mod migrations;
pub mod models;
pub mod movies;
pub mod settings;
pub mod subtitles;
pub mod watchlist;

#[cfg(test)]
mod tests;
