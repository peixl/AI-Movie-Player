//! Library scanning, import, and duplicate detection.

use rusqlite::Connection;
use std::path::PathBuf;

use crate::api::tmdb::TmdbClient;
use crate::core::{filename_parser, metadata_service::MetadataService};
use crate::db::{models::Movie, movies};
use crate::util::error::Result;

/// Stateless library management operations.
pub struct LibraryManager;

/// Progress tracking for library scan operations.
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
            scan_progress.current_file =
                Some(file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());

            // Skip already imported
            let file_path_str = file.to_string_lossy().to_string();
            if movies::movie_exists_by_path(db, &file_path_str).unwrap_or(false) {
                scan_progress.skipped += 1;
                progress_callback(scan_progress.clone());
                continue;
            }

            match MetadataService::search_and_match(client, file).await {
                Ok((_search_result, details)) => {
                    // In auto_confirm mode, import directly
                    // In manual mode, we'd show the match to the user
                    if auto_confirm {
                        match MetadataService::import_movie(
                            client,
                            file,
                            &details,
                            db,
                            thumbnail_dir,
                        )
                        .await
                        {
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
        let mut stmt = db.prepare(
            "SELECT COUNT(*) FROM movies
             WHERE resolution IS NOT NULL
             AND (
                 lower(resolution) LIKE '%4k%'
                 OR lower(resolution) LIKE '%2160p%'
             )",
        )?;
        let res_count: i64 = stmt.query_row([], |row| row.get(0))?;

        Ok(LibraryStats { total_movies: total, has_4k: res_count > 0 })
    }
}

#[derive(Debug, Clone)]
pub struct LibraryStats {
    pub total_movies: i64,
    pub has_4k: bool,
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::TempDir;

    use crate::db::{connection, models::Movie};

    use super::LibraryManager;

    fn setup_db() -> (Connection, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let conn =
            connection::open_database(&dir.path().to_path_buf()).expect("Failed to open test DB");
        (conn, dir)
    }

    fn sample_movie(title: &str, file_path: &str, resolution: Option<&str>) -> Movie {
        Movie {
            id: 0,
            tmdb_id: None,
            imdb_id: None,
            title: title.to_string(),
            title_cn: None,
            original_title: None,
            year: Some(2024),
            release_date: None,
            poster_path: None,
            poster_local: None,
            backdrop_path: None,
            backdrop_local: None,
            rating: None,
            rating_count: None,
            genres: None,
            runtime: None,
            overview: None,
            overview_cn: None,
            tagline: None,
            director: None,
            cast_list: None,
            language: None,
            country: None,
            local_file_path: Some(file_path.to_string()),
            file_size: None,
            file_hash: None,
            resolution: resolution.map(|value| value.to_string()),
            source: None,
            codec: None,
            audio_langs: None,
            added_date: "2026-05-03T00:00:00Z".to_string(),
            updated_date: "2026-05-03T00:00:00Z".to_string(),
            tmdb_data: None,
        }
    }

    #[test]
    fn scan_folder_collects_only_video_files_within_depth_limit() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path();
        let included_dir = root.join("a").join("b").join("c").join("d");
        let excluded_dir = included_dir.join("e").join("f");

        std::fs::create_dir_all(&included_dir).expect("create included dir");
        std::fs::create_dir_all(&excluded_dir).expect("create excluded dir");

        let top_level_video = root.join("movie.mp4");
        let nested_video = included_dir.join("feature.mkv");
        let deep_video = excluded_dir.join("too_deep.mkv");
        let non_video = root.join("notes.txt");

        std::fs::write(&top_level_video, b"video").expect("write top level video");
        std::fs::write(&nested_video, b"video").expect("write nested video");
        std::fs::write(&deep_video, b"video").expect("write deep video");
        std::fs::write(&non_video, b"notes").expect("write non video file");

        let files = LibraryManager::scan_folder(root);

        assert!(files.contains(&top_level_video));
        assert!(files.contains(&nested_video));
        assert!(!files.contains(&deep_video));
        assert!(!files.contains(&non_video));
    }

    #[test]
    fn get_library_stats_only_marks_4k_for_4k_or_2160p_entries() {
        let (conn, _dir) = setup_db();

        crate::db::movies::insert_movie(
            &conn,
            &sample_movie("HD", "/movies/hd.mkv", Some("1080p")),
        )
        .expect("insert hd movie");

        let stats = LibraryManager::get_library_stats(&conn).expect("stats for hd library");
        assert_eq!(stats.total_movies, 1);
        assert!(!stats.has_4k);

        crate::db::movies::insert_movie(
            &conn,
            &sample_movie("4K", "/movies/4k.mkv", Some("2160p")),
        )
        .expect("insert 4k movie");

        let stats = LibraryManager::get_library_stats(&conn).expect("stats for 4k library");
        assert_eq!(stats.total_movies, 2);
        assert!(stats.has_4k);
    }
}
