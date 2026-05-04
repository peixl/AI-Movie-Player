use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

use crate::db::models::{CastMember, TmdbMovieDetails, TmdbSearchResult};
use crate::util::error::{AppError, Result};

const BASE_URL: &str = "https://api.themoviedb.org/3";
const IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

/// TMDB API v3 client for movie metadata, posters, and cast information.
///
/// Includes rate limiting via semaphore and minimum request interval.
pub struct TmdbClient {
    api_key: String,
    language: String,
    client: reqwest::Client,
    rate_limiter: Arc<Semaphore>,
    last_request: Mutex<Instant>,
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
            client: reqwest::Client::new(),
            rate_limiter: Arc::new(Semaphore::new(40)),
            last_request: Mutex::new(Instant::now()),
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

        self.wait_rate_limit().await;

        let mut params = vec![
            ("api_key", self.api_key.as_str()),
            ("query", query),
            ("language", &self.language),
        ];
        let year_str;
        if let Some(y) = year {
            year_str = y.to_string();
            params.push(("year", &year_str));
        }

        let resp =
            self.client.get(format!("{}/search/movie", BASE_URL)).query(&params).send().await?;

        if !resp.status().is_success() {
            return Err(AppError::TmdbApi {
                code: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        let data: TmdbSearchResponse = resp.json().await?;
        let results = data
            .results
            .into_iter()
            .map(|r| TmdbSearchResult {
                tmdb_id: r.id,
                title: r.title.unwrap_or_else(|| r.name.unwrap_or_default()),
                title_cn: None, // Will be populated by get_details for matched movies
                year: r.release_date.or(r.first_air_date).and_then(|d| d[..4].parse().ok()),
                poster_path: r.poster_path,
                overview: r.overview,
                rating: r.vote_average,
            })
            .collect();

        Ok(results)
    }

    pub async fn get_movie_details(&self, tmdb_id: i64) -> Result<TmdbMovieDetails> {
        if !self.has_key() {
            return Err(AppError::TmdbKeyMissing);
        }

        self.wait_rate_limit().await;

        let resp = self
            .client
            .get(format!("{}/movie/{}", BASE_URL, tmdb_id))
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("language", &self.language),
                ("append_to_response", "credits"),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AppError::TmdbApi {
                code: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        let raw: TmdbMovieRaw = resp.json().await?;

        // Also fetch Chinese details in parallel
        let cn_details = if self.language != "zh-CN" {
            self.wait_rate_limit().await;
            let cn_resp = self
                .client
                .get(format!("{}/movie/{}", BASE_URL, tmdb_id))
                .query(&[("api_key", self.api_key.as_str()), ("language", "zh-CN")])
                .send()
                .await?;

            if cn_resp.status().is_success() {
                cn_resp.json::<TmdbMovieRaw>().await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let title_cn = cn_details.as_ref().and_then(|m| m.title.clone());

        let overview_cn = cn_details.as_ref().and_then(|m| m.overview.clone());

        let director = raw
            .credits
            .as_ref()
            .and_then(|c| c.crew.as_ref())
            .and_then(|crew| crew.iter().find(|p| p.job == "Director"))
            .map(|d| d.name.clone());

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

        Ok(TmdbMovieDetails {
            tmdb_id: raw.id,
            imdb_id: raw.imdb_id,
            title: raw.title.unwrap_or_default(),
            title_cn,
            original_title: raw.original_title,
            year: raw.release_date.as_ref().and_then(|d| d[..4].parse().ok()),
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
        })
    }

    pub fn poster_url(poster_path: &str, size: &str) -> String {
        format!("{}/{}{}", IMAGE_BASE, size, poster_path)
    }

    pub async fn download_poster(&self, poster_path: &str, size: &str) -> Result<Vec<u8>> {
        let url = Self::poster_url(poster_path, size);
        self.wait_rate_limit().await;
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(AppError::TmdbApi {
                code: resp.status().as_u16(),
                message: "Failed to download poster".into(),
            });
        }
        Ok(resp.bytes().await?.to_vec())
    }

    async fn wait_rate_limit(&self) {
        // Semaphore is never closed, so acquire only fails if poisoned
        let _permit = self
            .rate_limiter
            .acquire()
            .await
            .expect("TMDB rate limiter semaphore closed unexpectedly");
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();
        if elapsed < Duration::from_millis(25) {
            tokio::time::sleep(Duration::from_millis(25) - elapsed).await;
        }
        *last = Instant::now();
    }
}
