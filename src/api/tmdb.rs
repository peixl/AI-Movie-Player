//! TMDB API v3 client for movie metadata, posters, and cast information.
//!
//! Network hardening:
//! - Distinct HTTP clients for the JSON API and the image CDN, each with
//!   connect/request timeouts, connection pooling, compression and retries.
//! - Two independent rate limiters: the metadata API is throttled to a polite
//!   minimum interval, while poster downloads use the high-throughput CDN.
//! - Malformed release dates never panic; the year is extracted defensively.

use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::try_join;

use crate::api::http::{self, ClientProfile};
use crate::db::models::{CastMember, TmdbMovieDetails, TmdbSearchResult};
use crate::util::error::{AppError, Result};

const BASE_URL: &str = "https://api.themoviedb.org/3";
const IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

/// Minimum spacing between metadata API requests (~40 req/s ceiling).
const API_MIN_INTERVAL: Duration = Duration::from_millis(25);
/// Minimum spacing between image CDN requests (~100 req/s ceiling).
const IMAGE_MIN_INTERVAL: Duration = Duration::from_millis(10);
const API_MAX_CONCURRENCY: usize = 16;
const IMAGE_MAX_CONCURRENCY: usize = 24;

/// TMDB API v3 client for movie metadata, posters, and cast information.
pub struct TmdbClient {
    api_key: String,
    language: String,
    api: reqwest::Client,
    images: reqwest::Client,
    api_limiter: RateLimiter,
    image_limiter: RateLimiter,
}

/// Simple rate limiter combining a concurrency semaphore and a minimum interval
/// between request starts.
struct RateLimiter {
    semaphore: Arc<Semaphore>,
    last_start: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(max_concurrency: usize, min_interval: Duration) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            last_start: Mutex::new(Instant::now()),
            min_interval,
        }
    }

    async fn acquire(&self) {
        let _permit =
            self.semaphore.acquire().await.expect("rate limiter semaphore is never closed");
        let mut last = self.last_start.lock().await;
        let elapsed = last.elapsed();
        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }
        *last = Instant::now();
    }
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResponse {
    results: Vec<TmdbRawResult>,
}

#[derive(Debug, Deserialize)]
struct TmdbRawResult {
    id: i64,
    title: Option<String>,
    name: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    poster_path: Option<String>,
    overview: Option<String>,
    vote_average: Option<f64>,
    popularity: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TmdbMovieRaw {
    id: i64,
    imdb_id: Option<String>,
    title: Option<String>,
    original_title: Option<String>,
    original_language: Option<String>,
    release_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    vote_average: Option<f64>,
    vote_count: Option<i32>,
    genres: Option<Vec<TmdbGenre>>,
    runtime: Option<i32>,
    overview: Option<String>,
    tagline: Option<String>,
    credits: Option<TmdbCredits>,
    production_countries: Option<Vec<TmdbCountry>>,
}

#[derive(Debug, Deserialize)]
struct TmdbGenre {
    name: String,
}

#[derive(Debug, Deserialize)]
struct TmdbCredits {
    cast: Option<Vec<TmdbCast>>,
    crew: Option<Vec<TmdbCrew>>,
}

#[derive(Debug, Deserialize)]
struct TmdbCast {
    name: String,
    character: String,
    profile_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TmdbCrew {
    name: String,
    job: String,
}

#[derive(Debug, Deserialize)]
struct TmdbCountry {
    iso_3166_1: String,
}

impl TmdbClient {
    pub fn new(api_key: String, language: String) -> Self {
        Self {
            api_key,
            language,
            api: http::build_client(ClientProfile::Api),
            images: http::build_client(ClientProfile::Image),
            api_limiter: RateLimiter::new(API_MAX_CONCURRENCY, API_MIN_INTERVAL),
            image_limiter: RateLimiter::new(IMAGE_MAX_CONCURRENCY, IMAGE_MIN_INTERVAL),
        }
    }

    pub fn has_key(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub async fn search_movies(
        &self,
        query: &str,
        year: Option<i32>,
    ) -> Result<Vec<TmdbSearchResult>> {
        if !self.has_key() {
            return Err(AppError::TmdbKeyMissing);
        }

        let mut params = vec![
            ("api_key", self.api_key.as_str()),
            ("query", query),
            ("language", self.language.as_str()),
            ("include_adult", "false"),
        ];
        let year_str;
        if let Some(y) = year {
            year_str = y.to_string();
            params.push(("year", &year_str));
        }

        self.api_limiter.acquire().await;
        let resp = http::execute_with_retry(ClientProfile::Api, &self.api, |c| {
            c.get(format!("{}/search/movie", BASE_URL)).query(&params)
        })
        .await?;

        let data: TmdbSearchResponse = resp.json().await?;
        let results = data
            .results
            .into_iter()
            .map(|r| TmdbSearchResult {
                tmdb_id: r.id,
                title: r.title.unwrap_or_else(|| r.name.unwrap_or_default()),
                title_cn: None, // Populated by get_movie_details for the matched title.
                year: r.release_date.or(r.first_air_date).and_then(|d| year_from_release_date(&d)),
                poster_path: r.poster_path,
                overview: r.overview,
                rating: r.vote_average,
                popularity: r.popularity,
            })
            .collect();

        Ok(results)
    }

    pub async fn get_movie_details(&self, tmdb_id: i64) -> Result<TmdbMovieDetails> {
        if !self.has_key() {
            return Err(AppError::TmdbKeyMissing);
        }

        // Primary language and Chinese fallback are fetched concurrently so
        // bilingual users do not pay two serialized round trips.
        let primary = self.fetch_details_language(tmdb_id, &self.language);
        let chinese = if self.language != "zh-CN" {
            Some(self.fetch_details_language(tmdb_id, "zh-CN"))
        } else {
            None
        };

        let raw = match chinese {
            Some(chinese) => {
                let (primary, chinese) = try_join!(primary, chinese)?;
                TmdbDetailsBundle { primary, chinese: Some(chinese) }
            }
            None => TmdbDetailsBundle { primary: primary.await?, chinese: None },
        };

        Ok(raw.into_details())
    }

    async fn fetch_details_language(&self, tmdb_id: i64, language: &str) -> Result<TmdbMovieRaw> {
        let api_key = self.api_key.clone();
        self.api_limiter.acquire().await;
        let resp = http::execute_with_retry(ClientProfile::Api, &self.api, |c| {
            c.get(format!("{}/movie/{}", BASE_URL, tmdb_id)).query(&[
                ("api_key", api_key.as_str()),
                ("language", language),
                ("append_to_response", "credits"),
            ])
        })
        .await?;
        Ok(resp.json::<TmdbMovieRaw>().await?)
    }

    pub fn poster_url(poster_path: &str, size: &str) -> String {
        format!("{}/{}{}", IMAGE_BASE, size, poster_path)
    }

    pub async fn download_poster(&self, poster_path: &str, size: &str) -> Result<Vec<u8>> {
        let url = Self::poster_url(poster_path, size);
        self.image_limiter.acquire().await;
        let resp =
            http::execute_with_retry(ClientProfile::Image, &self.images, |c| c.get(&url)).await?;

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_lowercase();
        if !content_type.is_empty() && !content_type.starts_with("image/") {
            return Err(AppError::Parse(format!(
                "Poster response was not an image (content-type: {content_type})"
            )));
        }

        let bytes = resp.bytes().await?;
        if bytes.is_empty() {
            return Err(AppError::Parse("Poster download was empty".into()));
        }
        Ok(bytes.to_vec())
    }
}

/// Bundle of localized detail payloads, converted into the shared model.
struct TmdbDetailsBundle {
    primary: TmdbMovieRaw,
    chinese: Option<TmdbMovieRaw>,
}

impl TmdbDetailsBundle {
    fn into_details(self) -> TmdbMovieDetails {
        let TmdbDetailsBundle { primary: raw, chinese } = self;

        let title_cn = chinese.as_ref().and_then(|m| m.title.clone());
        let overview_cn = chinese.as_ref().and_then(|m| m.overview.clone());

        let director = raw.credits.as_ref().and_then(|c| c.crew.as_ref()).and_then(|crew| {
            crew.iter().find(|p| p.job.eq_ignore_ascii_case("director")).map(|d| d.name.clone())
        });

        let cast_list: Vec<CastMember> = raw
            .credits
            .as_ref()
            .and_then(|c| c.cast.as_ref())
            .map(|cast| {
                cast.iter()
                    .take(10)
                    .map(|c| CastMember {
                        name: c.name.clone(),
                        character: c.character.clone(),
                        profile_path: c.profile_path.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let genres: Vec<String> =
            raw.genres.unwrap_or_default().into_iter().map(|g| g.name).collect();

        let country =
            raw.production_countries.as_ref().and_then(|c| c.first()).map(|c| c.iso_3166_1.clone());

        TmdbMovieDetails {
            tmdb_id: raw.id,
            imdb_id: raw.imdb_id,
            title: raw.title.unwrap_or_default(),
            title_cn,
            original_title: raw.original_title,
            year: raw.release_date.as_deref().and_then(year_from_release_date),
            release_date: raw.release_date,
            poster_path: raw.poster_path,
            backdrop_path: raw.backdrop_path,
            rating: raw.vote_average,
            rating_count: raw.vote_count,
            genres,
            runtime: raw.runtime,
            overview: raw.overview,
            overview_cn,
            tagline: raw.tagline,
            director,
            cast_list,
            language: raw.original_language,
            country,
        }
    }
}

/// Extract a 4-digit year from a TMDB release-date string (`YYYY-MM-DD`).
///
/// Slicing `&date[..4]` directly panics on short or malformed values, so the
/// parse is fully bounds- and content-safe.
pub fn year_from_release_date(date: &str) -> Option<i32> {
    let date = date.trim();
    if date.len() < 4 {
        return None;
    }
    date.get(..4)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::year_from_release_date;

    #[test]
    fn year_extraction_handles_standard_and_malformed_dates() {
        assert_eq!(year_from_release_date("2021-10-22"), Some(2021));
        assert_eq!(year_from_release_date("1999-01"), Some(1999));
        assert_eq!(year_from_release_date("2024"), Some(2024));
        assert_eq!(year_from_release_date(""), None);
        assert_eq!(year_from_release_date("n/a"), None);
        assert_eq!(year_from_release_date("12"), None);
        assert_eq!(year_from_release_date("  2010-05-05 "), Some(2010));
    }
}
