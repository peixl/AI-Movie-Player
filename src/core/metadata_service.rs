//! Metadata enrichment orchestration: TMDB lookups, file hashing, and DB persistence.

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::PathBuf;

use crate::api::tmdb::TmdbClient;
use crate::core::filename_parser::{self, is_video_file};
use crate::db::models::*;
use crate::db::movies;
use crate::util::error::Result;

/// Orchestrates metadata enrichment: directory scanning, TMDB lookup, and DB storage.
pub struct MetadataService;

fn select_best_match<'a>(
    parsed: &ParsedFilename,
    results: &'a [TmdbSearchResult],
) -> Option<&'a TmdbSearchResult> {
    if results.is_empty() {
        return None;
    }

    if results.len() == 1 {
        return results.first();
    }

    results
        .iter()
        .find(|result| {
            result.title.to_lowercase() == parsed.title.to_lowercase()
                && parsed.year.is_none_or(|year| result.year == Some(year))
        })
        .or_else(|| results.first())
}

impl MetadataService {
    pub fn scan_directory(dir: &std::path::Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if is_video_file(&name) {
                        files.push(path);
                    }
                }
            }
        }
        files
    }

    pub fn compute_file_hash(path: &std::path::Path, sample_size: u64) -> std::io::Result<String> {
        let mut file = std::fs::File::open(path)?;
        let file_size = file.metadata()?.len();
        let read_size = std::cmp::min(sample_size, file_size);

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; read_size as usize];
        file.read_exact(&mut buffer)?;
        hasher.update(&buffer);

        if file_size > sample_size * 2 {
            // Also read tail
            let tail_start = file_size - sample_size;
            let mut file = std::fs::File::open(path)?;
            std::io::Seek::seek(
                &mut std::io::BufReader::new(&mut file),
                std::io::SeekFrom::Start(tail_start),
            )?;
            let mut tail_buf = vec![0u8; sample_size as usize];
            file.read_exact(&mut tail_buf)?;
            hasher.update(&tail_buf);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub async fn search_and_match(
        client: &TmdbClient,
        file_path: &std::path::Path,
        db: &Connection,
    ) -> Result<(TmdbSearchResult, TmdbMovieDetails)> {
        let filename =
            file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

        let parsed = filename_parser::parse_filename(&filename);

        let results = client.search_movies(&parsed.title, parsed.year).await?;

        if results.is_empty() {
            return Err(crate::util::error::AppError::MovieNotFound {
                query: parsed.title.clone(),
            });
        }

        let best = select_best_match(&parsed, &results).ok_or_else(|| {
            crate::util::error::AppError::MovieNotFound { query: parsed.title.clone() }
        })?;

        let details = client.get_movie_details(best.tmdb_id).await?;

        Ok((best.clone(), details))
    }

    pub async fn import_movie(
        client: &TmdbClient,
        file_path: &std::path::Path,
        details: &TmdbMovieDetails,
        db: &Connection,
        thumbnail_dir: &PathBuf,
    ) -> Result<Movie> {
        let filename =
            file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let parsed = filename_parser::parse_filename(&filename);
        let file_size = std::fs::metadata(file_path).ok().map(|m| m.len() as i64);
        let file_hash = Self::compute_file_hash(file_path, 64 * 1024).ok();

        let genres_json = serde_json::to_string(&details.genres).ok();
        let cast_json = serde_json::to_string(&details.cast_list).ok();
        let tmdb_json = serde_json::to_string(&details).ok();

        let poster_local = if let Some(ref poster_path) = details.poster_path {
            crate::thumbnail::cache::download_and_cache(
                client,
                poster_path,
                details.tmdb_id,
                thumbnail_dir,
            )
            .await
            .ok()
        } else {
            None
        };

        let movie = Movie {
            id: 0,
            tmdb_id: Some(details.tmdb_id),
            imdb_id: details.imdb_id.clone(),
            title: details.title.clone(),
            title_cn: details.title_cn.clone(),
            original_title: details.original_title.clone(),
            year: details.year,
            release_date: details.release_date.clone(),
            poster_path: details.poster_path.clone(),
            poster_local,
            backdrop_path: details.backdrop_path.clone(),
            backdrop_local: None,
            rating: details.rating,
            rating_count: details.rating_count,
            genres: genres_json,
            runtime: details.runtime,
            overview: details.overview.clone(),
            overview_cn: details.overview_cn.clone(),
            tagline: details.tagline.clone(),
            director: details.director.clone(),
            cast_list: cast_json,
            language: details.language.clone(),
            country: details.country.clone(),
            local_file_path: Some(file_path.to_string_lossy().to_string()),
            file_size,
            file_hash,
            resolution: parsed.resolution,
            source: parsed.source,
            codec: parsed.codec,
            audio_langs: None,
            added_date: String::new(),
            updated_date: String::new(),
            tmdb_data: tmdb_json,
        };

        let id = movies::insert_movie(db, &movie)?;

        Ok(Movie { id, ..movie })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    use super::{MetadataService, select_best_match};
    use crate::db::models::{ParsedFilename, TmdbSearchResult};

    fn parsed_filename(title: &str, year: Option<i32>) -> ParsedFilename {
        ParsedFilename {
            title: title.to_string(),
            year,
            resolution: None,
            source: None,
            codec: None,
            group: None,
            episode: None,
            is_tv: false,
        }
    }

    fn tmdb_result(id: i64, title: &str, year: Option<i32>) -> TmdbSearchResult {
        TmdbSearchResult {
            tmdb_id: id,
            title: title.to_string(),
            title_cn: None,
            year,
            poster_path: None,
            overview: None,
            rating: None,
        }
    }

    #[test]
    fn select_best_match_prefers_exact_title_and_year() {
        let parsed = parsed_filename("Dune", Some(2021));
        let results = vec![
            tmdb_result(1, "Dune", Some(1984)),
            tmdb_result(2, "Dune", Some(2021)),
            tmdb_result(3, "Dune Part Two", Some(2024)),
        ];

        let best = select_best_match(&parsed, &results).expect("expected best match");

        assert_eq!(best.tmdb_id, 2);
    }

    #[test]
    fn select_best_match_falls_back_to_first_when_no_exact_match() {
        let parsed = parsed_filename("Unknown Film", Some(2023));
        let results = vec![
            tmdb_result(10, "Possible Match", Some(2023)),
            tmdb_result(11, "Another Match", Some(2023)),
        ];

        let best = select_best_match(&parsed, &results).expect("expected fallback match");

        assert_eq!(best.tmdb_id, 10);
    }

    #[test]
    fn scan_directory_only_returns_top_level_video_files() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path();
        let nested_dir = root.join("nested");
        std::fs::create_dir_all(&nested_dir).expect("create nested dir");

        let top_level_video = root.join("movie.mkv");
        let top_level_text = root.join("readme.txt");
        let nested_video = nested_dir.join("nested.mp4");

        std::fs::write(&top_level_video, b"video").expect("write top level video");
        std::fs::write(&top_level_text, b"text").expect("write top level text");
        std::fs::write(&nested_video, b"video").expect("write nested video");

        let files = MetadataService::scan_directory(root);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], top_level_video);
    }

    #[test]
    fn compute_file_hash_reads_entire_small_file() {
        let temp = TempDir::new().expect("temp dir");
        let path = temp.path().join("sample.bin");
        let data = b"small-file-data";
        std::fs::write(&path, data).expect("write sample file");

        let hash = MetadataService::compute_file_hash(&path, 1024).expect("hash small file");
        let expected = format!("{:x}", Sha256::digest(data));

        assert_eq!(hash, expected);
    }

    #[test]
    fn compute_file_hash_uses_head_and_tail_for_large_files() {
        let temp = TempDir::new().expect("temp dir");
        let path = temp.path().join("large.bin");
        let sample_size = 8u64;

        let mut file = std::fs::File::create(&path).expect("create large file");
        file.write_all(b"ABCDEFGHmiddle-data12345678").expect("write large file");
        drop(file);

        let hash = MetadataService::compute_file_hash(&path, sample_size).expect("hash large file");

        let mut hasher = Sha256::new();
        hasher.update(b"ABCDEFGH");
        hasher.update(b"12345678");
        let expected = format!("{:x}", hasher.finalize());

        assert_eq!(hash, expected);
    }
}
