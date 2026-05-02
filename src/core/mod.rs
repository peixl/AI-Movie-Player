//! Core domain logic: models, parsers, and library management.
//!
//! - [`filename_parser`]: Extract movie metadata (title, year, resolution, codec) from filenames.
//! - [`file_organizer`]: File organization and renaming utilities.
//! - [`library_manager`]: Library scanning, import, and duplicate detection.
//! - [`metadata_service`]: Metadata enrichment orchestration (TMDB lookups).
//! - [`subtitle_finder`]: Subtitle search and download coordination.

pub mod file_organizer;
pub mod filename_parser;
pub mod library_manager;
pub mod metadata_service;
pub mod subtitle_finder;
