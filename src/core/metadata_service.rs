use std::path::PathBuf;
use rusqlite::Connection;
use sha2::{Sha256, Digest};
use std::io::Read;

use crate::api::tmdb::TmdbClient;
use crate::core::filename_parser::{self, is_video_file};
use crate::db::models::*;
use crate::db::movies;
use crate::util::error::Result;

pub struct MetadataService;

impl MetadataService {
    pub fn scan_directory(dir: &std::path::Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name()
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
            std::io::Seek::seek(&mut std::io::BufReader::new(&mut file), std::io::SeekFrom::Start(tail_start))?;
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
        let filename = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let parsed = filename_parser::parse_filename(&filename);

        let results = client.search_movies(&parsed.title, parsed.year).await?;

        if results.is_empty() {
            return Err(crate::util::error::AppError::MovieNotFound {
                query: parsed.title.clone(),
            });
        }

        // Auto-match: if only 1 result, or exact title + year match
        let best = if results.len() == 1 {
            &results[0]
        } else {
            results.iter().find(|r| {
                r.title.to_lowercase() == parsed.title.to_lowercase()
                    && parsed.year.map_or(true, |y| r.year == Some(y))
            }).unwrap_or(&results[0])
        };

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
        let filename = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
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
            ).await.ok()
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
