use std::path::PathBuf;
use rusqlite::Connection;

use crate::api::tmdb::TmdbClient;
use crate::core::{filename_parser, metadata_service::MetadataService};
use crate::db::{movies, models::Movie};
use crate::util::error::Result;

pub struct LibraryManager;

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub total: usize,
    pub processed: usize,
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub current_file: Option<String>,
}

impl LibraryManager {
    pub fn scan_folder(path: &std::path::Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let walker = walkdir::WalkDir::new(path)
            .follow_links(false)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            if entry.file_type().is_file() {
                let name = entry.file_name().to_string_lossy();
                if filename_parser::is_video_file(&name) {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
        files
    }

    pub async fn batch_scan_and_import(
        client: &TmdbClient,
        db: &Connection,
        files: &[PathBuf],
        thumbnail_dir: &PathBuf,
        auto_confirm: bool,
        progress_callback: impl Fn(ScanProgress),
    ) -> Result<Vec<Movie>> {
        let total = files.len();
        let mut imported = Vec::new();
        let mut scan_progress = ScanProgress {
            total,
            processed: 0,
            imported: 0,
            skipped: 0,
            failed: 0,
            current_file: None,
        };

        for file in files {
            scan_progress.processed += 1;
            scan_progress.current_file = Some(
                file.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            );

            // Skip already imported
            let file_path_str = file.to_string_lossy().to_string();
            if movies::movie_exists_by_path(db, &file_path_str).unwrap_or(false) {
                scan_progress.skipped += 1;
                progress_callback(scan_progress.clone());
                continue;
            }

            match MetadataService::search_and_match(client, file, db).await {
                Ok((search_result, details)) => {
                    // In auto_confirm mode, import directly
                    // In manual mode, we'd show the match to the user
                    if auto_confirm {
                        match MetadataService::import_movie(
                            client, file, &details, db, thumbnail_dir,
                        ).await {
                            Ok(movie) => {
                                imported.push(movie);
                                scan_progress.imported += 1;
                            }
                            Err(e) => {
                                log::warn!("Failed to import {}: {}", file.display(), e);
                                scan_progress.failed += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Search failed for {}: {}", file.display(), e);
                    scan_progress.failed += 1;
                }
            }

            progress_callback(scan_progress.clone());
        }

        Ok(imported)
    }

    pub fn get_library_stats(db: &Connection) -> Result<LibraryStats> {
        let total = movies::get_movie_count(db)?;
        let mut stmt = db.prepare("SELECT COUNT(DISTINCT resolution) FROM movies WHERE resolution IS NOT NULL")?;
        let res_count: i64 = stmt.query_row([], |row| row.get(0))?;

        Ok(LibraryStats {
            total_movies: total,
            has_4k: res_count > 0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LibraryStats {
    pub total_movies: i64,
    pub has_4k: bool,
}
