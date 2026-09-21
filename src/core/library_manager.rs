//! Library scanning, import, and duplicate detection.
//!
//! The import pipeline is designed for large personal libraries:
//! - recursive walks skip hidden/metadata directories and non-feature files
//!   (samples, trailers, extras);
//! - content-hash dedup catches moved and renamed copies;
//! - TMDB enrichment runs with bounded concurrency;
//! - files that cannot be matched are still imported as honest minimal
//!   records instead of being silently dropped;
//! - progress is shared via a mutex so the UI can poll it without channels.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tokio::task::JoinSet;

use crate::api::tmdb::TmdbClient;
use crate::core::{filename_parser, metadata_service::MetadataService};
use crate::db::{models::Movie, movies};
use crate::util::error::Result;

/// Maximum number of files enriched against TMDB concurrently.
const IMPORT_CONCURRENCY: usize = 6;
const MAX_SCAN_DEPTH: usize = 5;

/// Tokens (folder or file names) that identify non-feature content.
const NON_FEATURE_TOKENS: &[&str] = &[
    "sample",
    "samples",
    "trailer",
    "trailers",
    "extras",
    "extra",
    "behindthescenes",
    "featurette",
    "interview",
    "interviews",
    "deleted",
    "deletedscenes",
    "short",
    "shorts",
    "advertisement",
    "advertisements",
    "etrg",
];

/// Stateless library management operations.
pub struct LibraryManager;

/// Progress tracking for library scan operations.
#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub total: usize,
    pub processed: usize,
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub unmatched: usize,
    pub current_file: Option<String>,
}

/// A file that could not be imported, with a human-readable reason.
#[derive(Debug, Clone)]
pub struct ImportFailure {
    pub file: String,
    pub reason: String,
}

/// Outcome of a batch import.
#[derive(Debug, Default)]
pub struct ImportReport {
    pub imported: Vec<Movie>,
    pub skipped: usize,
    pub unmatched: usize,
    pub failed: Vec<ImportFailure>,
}

impl ImportReport {
    pub fn total_imported(&self) -> usize {
        self.imported.len()
    }
}

/// Whether a path component denotes non-feature content (samples, extras...).
pub fn is_non_feature_path(path: &Path, scan_root: &Path) -> bool {
    let relative = path.strip_prefix(scan_root).unwrap_or(path);

    relative.components().filter_map(|c| c.as_os_str().to_str()).any(|raw| {
        // Hidden file/directory (NAS metadata, trash, thumbs).
        if raw.starts_with('.') && raw != "." && raw != ".." {
            return true;
        }

        let normalized: String = raw
            .chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { ' ' })
            .collect();
        normalized.split_whitespace().any(|token| NON_FEATURE_TOKENS.contains(&token))
    })
}

impl LibraryManager {
    /// Recursively collect feature video files under `path`.
    pub fn scan_folder(path: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let walker = walkdir::WalkDir::new(path)
            .follow_links(false)
            .max_depth(MAX_SCAN_DEPTH)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            if !entry.file_type().is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy();
            if !filename_parser::is_video_file(&name) {
                continue;
            }
            if is_non_feature_path(entry.path(), path) {
                continue;
            }
            files.push(entry.path().to_path_buf());
        }
        files
    }

    /// Import files with bounded-concurrency TMDB enrichment.
    ///
    /// Worker tasks perform only `Send`-safe work (hashing, network, poster
    /// download); the single SQLite connection stays on this task and records
    /// are inserted as workers finish.
    pub async fn batch_scan_and_import(
        client: Arc<TmdbClient>,
        db: &Connection,
        files: Vec<PathBuf>,
        thumbnail_dir: PathBuf,
        progress: Arc<Mutex<ScanProgress>>,
    ) -> Result<ImportReport> {
        {
            let mut p = progress.lock().expect("progress mutex");
            p.total = files.len();
        }

        let mut report = ImportReport::default();
        let mut tasks: JoinSet<(PathBuf, PreparedFile)> = JoinSet::new();
        let mut queued = files.into_iter();
        let mut in_flight = 0usize;

        loop {
            while in_flight < IMPORT_CONCURRENCY {
                if let Some(file) = queued.next() {
                    // Cheap sequential path dedup before spending network work.
                    let already =
                        movies::movie_exists_by_path(db, &file.to_string_lossy()).unwrap_or(false);
                    if already {
                        let mut p = progress.lock().expect("progress mutex");
                        p.skipped += 1;
                        report.skipped += 1;
                        continue;
                    }

                    let client = client.clone();
                    let thumbnail_dir = thumbnail_dir.clone();
                    tasks.spawn(async move {
                        let prepared = prepare_file(&client, &file, &thumbnail_dir).await;
                        (file, prepared)
                    });
                    in_flight += 1;
                } else {
                    break;
                }
            }

            let Some(joined) = tasks.join_next().await else { break };
            in_flight -= 1;
            let (file, prepared) = match joined {
                Ok(pair) => pair,
                Err(err) => {
                    log::error!("Import task panicked: {err}");
                    continue;
                }
            };

            let file_name = file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file.to_string_lossy().to_string());

            let mut p = progress.lock().expect("progress mutex");
            p.processed += 1;
            p.current_file = Some(file_name.clone());

            let outcome = persist_prepared(db, prepared);
            match outcome {
                FileOutcome::Imported(movie, matched) => {
                    p.imported += 1;
                    if !matched {
                        p.unmatched += 1;
                        report.unmatched += 1;
                    }
                    report.imported.push(*movie);
                }
                FileOutcome::Skipped => {
                    p.skipped += 1;
                    report.skipped += 1;
                }
                FileOutcome::Failed(reason) => {
                    p.failed += 1;
                    log::warn!("Failed to import {}: {}", file.display(), reason);
                    report.failed.push(ImportFailure { file: file_name, reason });
                }
            }
        }

        Ok(report)
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

enum FileOutcome {
    Imported(Box<Movie>, bool),
    Skipped,
    Failed(String),
}

/// Worker-side result: everything network/disk bound, no database access.
enum PreparedFile {
    Matched { movie: Movie },
    Unmatched { movie: Movie },
    Failed(String),
}

async fn prepare_file(client: &TmdbClient, file: &Path, thumbnail_dir: &Path) -> PreparedFile {
    match MetadataService::search_and_match(client, file).await {
        Ok((_search_result, details)) => {
            match MetadataService::prepare_matched_movie(client, file, &details, thumbnail_dir)
                .await
            {
                Ok(movie) => PreparedFile::Matched { movie },
                Err(err) => PreparedFile::Failed(err.to_string()),
            }
        }
        Err(crate::util::error::AppError::MovieNotFound { .. }) => {
            match MetadataService::build_unmatched_movie(file) {
                Ok(movie) => PreparedFile::Unmatched { movie },
                Err(err) => PreparedFile::Failed(err.to_string()),
            }
        }
        Err(err) => PreparedFile::Failed(err.to_string()),
    }
}

/// Sequential database persistence for a prepared file. Runs on the orchestrator
/// task so the non-`Sync` rusqlite connection never crosses an await boundary.
fn persist_prepared(db: &Connection, prepared: PreparedFile) -> FileOutcome {
    let (mut movie, matched) = match prepared {
        PreparedFile::Matched { movie } => (movie, true),
        PreparedFile::Unmatched { movie } => (movie, false),
        PreparedFile::Failed(reason) => return FileOutcome::Failed(reason),
    };

    // Content-hash dedup covers moved/renamed copies.
    if let Some(ref hash) = movie.file_hash {
        if movies::movie_exists_by_file_hash(db, hash).unwrap_or(false) {
            return FileOutcome::Skipped;
        }
    }

    match movies::insert_movie(db, &movie) {
        Ok(id) => {
            movie.id = id;
            FileOutcome::Imported(Box::new(movie), matched)
        }
        Err(err) => FileOutcome::Failed(err.to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct LibraryStats {
    pub total_movies: i64,
    pub has_4k: bool,
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use tempfile::TempDir;

    use crate::db::{connection, models::Movie};

    use super::{LibraryManager, is_non_feature_path};

    fn setup_db() -> (rusqlite::Connection, TempDir) {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let conn = connection::open_database(dir.path()).expect("Failed to open test DB");
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
    fn scan_folder_skips_samples_extras_and_hidden_dirs() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path();

        let real = root.join("Dune.2021.mkv");
        let sample = root.join("Sample").join("sample-dune.mkv");
        let trailer = root.join("Trailers").join("dune-trailer.mp4");
        let extras = root.join("Extras").join("behindthescenes.mkv");
        let hidden = root.join(".@__thumb").join("abc.mp4");
        std::fs::create_dir_all(sample.parent().unwrap()).unwrap();
        std::fs::create_dir_all(trailer.parent().unwrap()).unwrap();
        std::fs::create_dir_all(extras.parent().unwrap()).unwrap();
        std::fs::create_dir_all(hidden.parent().unwrap()).unwrap();
        for f in [&real, &sample, &trailer, &extras, &hidden] {
            std::fs::write(f, b"video").unwrap();
        }

        let files = LibraryManager::scan_folder(root);
        assert_eq!(files, vec![real.clone()]);
        assert!(!files.contains(&sample));
        assert!(!files.contains(&trailer));
        assert!(!files.contains(&extras));
        assert!(!files.contains(&hidden));
    }

    #[test]
    fn non_feature_detection_covers_common_layouts() {
        let root = Path::new("/library");
        assert!(is_non_feature_path(Path::new("/library/Sample/clip.mkv"), root));
        assert!(is_non_feature_path(Path::new("/library/Movie/extras/featurette.mkv"), root));
        assert!(is_non_feature_path(Path::new("/library/.trash/x.mp4"), root));
        assert!(!is_non_feature_path(Path::new("/library/Movie (2021)/movie.mkv"), root));
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
