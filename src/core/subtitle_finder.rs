//! Subtitle search coordination across multiple sources.

use crate::db::models::{SubtitleQuery, SubtitleResult};
use crate::util::error::{AppError, Result};

/// Searches multiple subtitle sources (OpenSubtitles, assrt.net, zimuku).
pub struct SubtitleFinder;

impl SubtitleFinder {
    pub async fn search_all_sources(query: &SubtitleQuery) -> Result<Vec<SubtitleResult>> {
        let mut all_results = Vec::new();

        // Try OpenSubtitles
        match Self::search_opensubtitles(query).await {
            Ok(mut results) => all_results.append(&mut results),
            Err(e) => log::warn!("OpenSubtitles search failed: {}", e),
        }

        // assrt.net and zimuku are best-effort for now;
        // they require more complex scraping that will be implemented
        // when the user explicitly requests Chinese subtitles.

        // Sort by rating (highest first)
        all_results
            .sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));

        Ok(all_results)
    }

    async fn search_opensubtitles(query: &SubtitleQuery) -> Result<Vec<SubtitleResult>> {
        use reqwest::Client;
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        struct OsSubtitle {
            #[serde(rename = "SubFileName")]
            file_name: Option<String>,
            #[serde(rename = "SubDownloadsCnt")]
            downloads: Option<String>,
            #[serde(rename = "SubRating")]
            rating: Option<String>,
            #[serde(rename = "LanguageName")]
            language_name: Option<String>,
            #[serde(rename = "SubFormat")]
            format: Option<String>,
            #[serde(rename = "SubDownloadLink")]
            download_link: Option<String>,
            #[serde(rename = "ISO639")]
            iso639: Option<String>,
            #[serde(rename = "SubHearingImpaired")]
            hearing_imp: Option<String>,
        }

        let client = Client::new();

        // Build search URL
        let lang_ids = if query.languages.is_empty() {
            "all".to_string()
        } else {
            // Map language codes to OpenSubtitles language IDs
            query
                .languages
                .iter()
                .map(|l| match l.as_str() {
                    "zh" | "zh-CN" | "chi" => "chi".to_string(),
                    "zh-TW" => "zht".to_string(),
                    "en" => "eng".to_string(),
                    "ja" => "jpn".to_string(),
                    "ko" => "kor".to_string(),
                    "fr" => "fre".to_string(),
                    "de" => "ger".to_string(),
                    "es" => "spa".to_string(),
                    _ => "eng".to_string(),
                })
                .collect::<Vec<_>>()
                .join(",")
        };

        let mut url = if let Some(ref imdb) = query.imdb_id {
            format!(
                "https://rest.opensubtitles.org/search/imdbid-{}/sublanguageid-{}",
                imdb.trim_start_matches("tt"),
                lang_ids
            )
        } else {
            let encoded =
                url::form_urlencoded::byte_serialize(query.title.as_bytes()).collect::<String>();
            format!(
                "https://rest.opensubtitles.org/search/query-{}/sublanguageid-{}",
                encoded, lang_ids
            )
        };

        if let Some(year) = query.year {
            url.push_str(&format!("/season-{}/episode-0", year));
        }

        let resp = client
            .get(&url)
            .header("User-Agent", "TemporaryUserAgent")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(AppError::Network)?;

        if !resp.status().is_success() {
            return Err(AppError::SubtitleError {
                source_name: "OpenSubtitles".into(),
                reason: format!("HTTP {}", resp.status()),
            });
        }

        let raw: Vec<OsSubtitle> =
            resp.json().await.map_err(|e| AppError::Parse(format!("OpenSubtitles JSON: {}", e)))?;

        let mut results: Vec<SubtitleResult> = raw
            .into_iter()
            .map(|s| {
                let lang_code = s.iso639.unwrap_or_else(|| "en".into());
                SubtitleResult {
                    title: s.file_name.clone().unwrap_or_default(),
                    language: lang_code.clone(),
                    language_label: s.language_name.unwrap_or_else(|| lang_code.clone()),
                    source: "OpenSubtitles".into(),
                    source_url: s.download_link.unwrap_or_default(),
                    file_name: s.file_name.unwrap_or_default(),
                    rating: s.rating.and_then(|r| r.parse().ok()),
                    download_count: s.downloads.and_then(|d| d.parse().ok()),
                    is_ai: false,
                    is_hearing_imp: s.hearing_imp.as_deref() == Some("1"),
                    format: s.format.unwrap_or_else(|| "srt".into()),
                }
            })
            .collect();

        // Filter by desired languages
        if !query.languages.is_empty() {
            results.retain(|r| {
                let lang_lower = r.language.to_lowercase();
                query.languages.iter().any(|q| {
                    q.to_lowercase().starts_with(&lang_lower)
                        || lang_lower == q.to_lowercase()
                        || (q == "zh-CN" && lang_lower == "chi")
                        || (q == "zh" && lang_lower == "chi")
                        || (q == "en" && lang_lower == "eng")
                })
            });
        }

        Ok(results)
    }

    pub async fn download_subtitle(
        url: &str,
        dest_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf> {
        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .header("User-Agent", "AI-Movie-Player/0.2 (ifq.ai)")
            .send()
            .await
            .map_err(AppError::Network)?;

        let bytes = resp.bytes().await.map_err(AppError::Network)?;

        // Check if it's a zip
        if url.ends_with(".zip") || bytes.starts_with(b"PK") {
            let tmp = tempfile::tempdir()?;
            let zip_path = tmp.path().join("subtitle.zip");
            std::fs::write(&zip_path, &bytes)?;

            let file = std::fs::File::open(&zip_path)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| AppError::Parse(format!("Invalid zip: {}", e)))?;

            let mut extracted_path = None;
            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| AppError::Parse(format!("Zip entry error: {}", e)))?;
                let name = entry.name().to_string();

                if name.ends_with(".srt") || name.ends_with(".ass") || name.ends_with(".vtt") {
                    let dest = dest_dir.join(&name);
                    let mut out = std::fs::File::create(&dest)?;
                    std::io::copy(&mut entry, &mut out)?;
                    extracted_path = Some(dest);
                }
            }

            extracted_path.ok_or_else(|| AppError::Parse("No subtitle file in archive".into()))
        } else {
            // Direct download
            let ext = if url.ends_with(".ass") {
                "ass"
            } else if url.ends_with(".vtt") {
                "vtt"
            } else {
                "srt"
            };

            let filename = format!("subtitle.{}", ext);
            let dest = dest_dir.join(&filename);
            std::fs::write(&dest, &bytes)?;
            Ok(dest)
        }
    }
}
