//! Database models for movies, subtitles, watchlist, and TMDB data.

use serde::{Deserialize, Serialize};

/// Full movie record with metadata, file info, and TMDB data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: i64,
    pub tmdb_id: Option<i64>,
    pub imdb_id: Option<String>,
    pub title: String,
    pub title_cn: Option<String>,
    pub original_title: Option<String>,
    pub year: Option<i32>,
    pub release_date: Option<String>,
    pub poster_path: Option<String>,
    pub poster_local: Option<String>,
    pub backdrop_path: Option<String>,
    pub backdrop_local: Option<String>,
    pub rating: Option<f64>,
    pub rating_count: Option<i32>,
    pub genres: Option<String>,
    pub runtime: Option<i32>,
    pub overview: Option<String>,
    pub overview_cn: Option<String>,
    pub tagline: Option<String>,
    pub director: Option<String>,
    pub cast_list: Option<String>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub local_file_path: Option<String>,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub resolution: Option<String>,
    pub source: Option<String>,
    pub codec: Option<String>,
    pub audio_langs: Option<String>,
    pub added_date: String,
    pub updated_date: String,
    pub tmdb_data: Option<String>,
}

/// Lightweight movie summary for list views (poster wall, search results).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieSummary {
    pub id: i64,
    pub title: String,
    pub title_cn: Option<String>,
    pub year: Option<i32>,
    pub poster_local: Option<String>,
    pub poster_path: Option<String>,
    pub rating: Option<f64>,
    pub genres: Option<String>,
    pub resolution: Option<String>,
    pub added_date: String,
}

/// Subtitle metadata associated with a movie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtitle {
    pub id: i64,
    pub movie_id: i64,
    pub language: String,
    pub language_label: Option<String>,
    pub source: String,
    pub source_url: Option<String>,
    pub file_name: Option<String>,
    pub local_path: Option<String>,
    pub file_size: Option<i64>,
    pub rating: Option<f64>,
    pub download_count: Option<i32>,
    pub is_ai: bool,
    pub is_hearing_imp: bool,
    pub format: Option<String>,
    pub encoding: Option<String>,
    pub sync_status: Option<String>,
    pub download_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: i64,
    pub movie_id: Option<i64>,
    pub tmdb_id: Option<i64>,
    pub status: String,
    pub user_rating: Option<f64>,
    pub notes: Option<String>,
    pub added_date: String,
    pub watched_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastMember {
    pub name: String,
    pub character: String,
    pub profile_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbSearchResult {
    pub tmdb_id: i64,
    pub title: String,
    pub title_cn: Option<String>,
    pub year: Option<i32>,
    pub poster_path: Option<String>,
    pub overview: Option<String>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbMovieDetails {
    pub tmdb_id: i64,
    pub imdb_id: Option<String>,
    pub title: String,
    pub title_cn: Option<String>,
    pub original_title: Option<String>,
    pub year: Option<i32>,
    pub release_date: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub rating: Option<f64>,
    pub rating_count: Option<i32>,
    pub genres: Vec<String>,
    pub runtime: Option<i32>,
    pub overview: Option<String>,
    pub overview_cn: Option<String>,
    pub tagline: Option<String>,
    pub director: Option<String>,
    pub cast_list: Vec<CastMember>,
    pub language: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFilename {
    pub title: String,
    pub year: Option<i32>,
    pub resolution: Option<String>,
    pub source: Option<String>,
    pub codec: Option<String>,
    pub group: Option<String>,
    pub episode: Option<String>,
    pub is_tv: bool,
}

#[derive(Debug, Clone)]
pub struct SubtitleQuery {
    pub title: String,
    pub year: Option<i32>,
    pub file_hash: Option<String>,
    pub languages: Vec<String>,
    pub imdb_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubtitleResult {
    pub title: String,
    pub language: String,
    pub language_label: String,
    pub source: String,
    pub source_url: String,
    pub file_name: String,
    pub rating: Option<f64>,
    pub download_count: Option<i32>,
    pub is_ai: bool,
    pub is_hearing_imp: bool,
    pub format: String,
}
