//! Subtitle search and download coordination.
//!
//! Uses the current OpenSubtitles REST API (`api.opensubtitles.com/api/v1`),
//! which replaces the retired `rest.opensubtitles.org` endpoint. Anonymous
//! search requires a free API key configured in Settings.
//!
//! Safety/quality features:
//! - shared timeout/retry HTTP client (no per-call client construction);
//! - results ranked by the user's language preference, rating, downloads, HD
//!   flag, with AI-translated and hearing-impaired variants deprioritized;
//! - zip extraction is path-traversal safe (zip-slip);
//! - text subtitles are normalized to UTF-8 (BOM stripped, GBK fallback for
//!   legacy Chinese releases);
//! - files are saved next to the movie using its basename so players such as
//!   IINA, VLC and MPC-HC auto-load them.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::api::http::{self, ClientProfile};
use crate::db::models::{SubtitleQuery, SubtitleResult};
use crate::util::error::{AppError, Result};

const API_BASE: &str = "https://api.opensubtitles.com/api/v1";
const SOURCE_NAME: &str = "OpenSubtitles";

/// Subtitle archive/text extensions in preference order (srt is best supported).
const SUBTITLE_EXTENSIONS: &[&str] = &["srt", "ass", "vtt", "sub", "ssa", "smi"];

/// Map application language codes to OpenSubtitles BCP-47 codes.
pub fn map_language_code(code: &str) -> &'static str {
    match code.to_lowercase().as_str() {
        "zh" | "zh-cn" | "chi" | "chs" | "sc" => "zh-CN",
        "zh-tw" | "zht" | "cht" | "hk" | "zh-hk" => "zh-TW",
        "en" | "eng" => "en",
        "ja" | "jpn" => "ja",
        "ko" | "kor" => "ko",
        "fr" | "fre" | "fra" => "fr",
        "de" | "ger" | "deu" => "de",
        "es" | "spa" => "es",
        "it" | "ita" => "it",
        "pt" | "por" => "pt-PT",
        "ru" | "rus" => "ru",
        "th" | "tha" => "th",
        "vi" | "vie" => "vi",
        _ => "en",
    }
}

/// Searches subtitle sources. `api_key` is an OpenSubtitles consumer key.
pub struct SubtitleFinder;

impl SubtitleFinder {
    pub async fn search_all_sources(
        query: &SubtitleQuery,
        api_key: Option<&str>,
        tmdb_id: Option<i64>,
    ) -> Result<Vec<SubtitleResult>> {
        let Some(api_key) = api_key.filter(|key| !key.trim().is_empty()) else {
            return Err(AppError::Config(
                "OpenSubtitles API key not configured / 未配置 OpenSubtitles API Key \
                 (Settings → Subtitles). Get a free key at opensubtitles.com."
                    .into(),
            ));
        };

        let client = http::build_client(ClientProfile::Api);
        let languages = query
            .languages
            .iter()
            .map(|l| map_language_code(l))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(",");

        let mut params: Vec<(&str, String)> = vec![("languages", languages)];
        if let Some(tmdb) = tmdb_id {
            params.push(("tmdb_id", tmdb.to_string()));
        } else if let Some(imdb) = query.imdb_id.as_deref() {
            params.push(("imdb_id", imdb.trim_start_matches("tt").to_string()));
        } else {
            params.push(("query", query.title.clone()));
            if let Some(year) = query.year {
                params.push(("year", year.to_string()));
            }
        }

        let resp = http::execute_with_retry(ClientProfile::Api, &client, |c| {
            c.get(format!("{API_BASE}/subtitles"))
                .header("Api-Key", api_key.trim())
                .header("Accept", "application/json")
                .query(&params)
        })
        .await?;

        let payload: SubtitleSearchResponse = resp.json().await?;
        let mut results = Vec::new();

        for item in payload.data {
            let attrs = item.attributes;
            let Some(file) = attrs.files.into_iter().max_by_key(|f| f.file_id) else {
                continue;
            };
            let language = attrs.language.unwrap_or_else(|| "en".to_string());
            let language_label = attrs
                .language_name
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| language.clone());
            let release_name = attrs.release_name.unwrap_or_else(|| file.file_name.clone());
            let format = std::path::Path::new(&file.file_name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .filter(|e| SUBTITLE_EXTENSIONS.contains(&e.as_str()))
                .unwrap_or_else(|| "srt".to_string());

            results.push(SubtitleResult {
                title: release_name.clone(),
                language: language.clone(),
                language_label,
                source: SOURCE_NAME.into(),
                source_url: String::new(),
                file_name: release_name,
                rating: parse_rating(&attrs.ratings),
                download_count: Some(attrs.download_count),
                is_ai: matches!(attrs.ai_translated.as_deref(), Some("true") | Some("1")),
                is_hearing_imp: attrs.hearing_impaired,
                format,
                file_id: Some(file.file_id),
                is_hd: attrs.hd,
            });
        }

        rank_results(&mut results, &query.languages);
        Ok(results)
    }

    /// Resolve a temporary download link for a provider file id.
    pub async fn resolve_download_link(
        client: &reqwest::Client,
        api_key: &str,
        file_id: i64,
    ) -> Result<String> {
        #[derive(serde::Serialize)]
        struct DownloadRequest {
            file_id: i64,
        }
        #[derive(Deserialize)]
        struct DownloadResponse {
            link: String,
        }

        let resp = http::execute_with_retry(ClientProfile::Api, client, |c| {
            c.post(format!("{API_BASE}/download"))
                .header("Api-Key", api_key.trim())
                .header("Accept", "application/json")
                .json(&DownloadRequest { file_id })
        })
        .await?;

        let payload: DownloadResponse = resp.json().await?;
        Ok(payload.link)
    }

    /// Download a subtitle by file id and install it next to the movie.
    pub async fn download_by_file_id(
        api_key: &str,
        file_id: i64,
        movie_path: Option<&Path>,
        language: String,
    ) -> Result<(PathBuf, String)> {
        let client = http::build_client(ClientProfile::Download);
        let link = Self::resolve_download_link(&client, api_key, file_id).await?;

        let resp = http::execute_with_retry(ClientProfile::Download, &client, |c| {
            c.get(&link).header("Accept", "*/*")
        })
        .await?;
        let bytes = resp.bytes().await?;

        let (dest_dir, base_name) = location_for_movie(movie_path);
        let install_language = language.clone();
        let saved = tokio::task::spawn_blocking(move || {
            install_subtitle_bytes(&bytes, &dest_dir, base_name.as_deref(), &install_language)
        })
        .await
        .map_err(|err| AppError::Parse(format!("subtitle install task failed: {err}")))??;

        Ok((saved, language))
    }
}

/// Rank subtitle results: preferred languages first, then quality signals.
pub fn rank_results(results: &mut [SubtitleResult], preferred_languages: &[String]) {
    let preference_rank = move |language: &str| -> usize {
        let mapped = map_language_code(language);
        preferred_languages
            .iter()
            .position(|pref| map_language_code(pref) == mapped)
            .unwrap_or(preferred_languages.len())
    };

    results.sort_by(|a, b| {
        let lang_a = preference_rank(&a.language);
        let lang_b = preference_rank(&b.language);
        lang_a.cmp(&lang_b).then_with(|| {
            score_result(b).partial_cmp(&score_result(a)).unwrap_or(std::cmp::Ordering::Equal)
        })
    });
}

fn score_result(result: &SubtitleResult) -> f64 {
    let mut score = 0.0;
    score += result.rating.unwrap_or(0.0) * 10.0;
    score += (result.download_count.unwrap_or(0) as f64).ln_1p() * 2.0;
    if result.is_hd {
        score += 3.0;
    }
    if result.is_ai {
        score -= 8.0;
    }
    if result.is_hearing_imp {
        score -= 2.0;
    }
    // Prefer broadly compatible formats.
    score += match result.format.as_str() {
        "srt" => 2.0,
        "ass" => 1.0,
        _ => 0.0,
    };
    score
}

fn parse_rating(raw: &Option<serde_json::Value>) -> Option<f64> {
    match raw {
        Some(serde_json::Value::String(s)) => s.parse().ok(),
        Some(serde_json::Value::Number(n)) => n.as_f64(),
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.get("rating")?.as_str()?.parse::<f64>().ok())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
        _ => None,
    }
}

fn location_for_movie(movie_path: Option<&Path>) -> (PathBuf, Option<String>) {
    if let Some(path) = movie_path {
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
        let base = path.file_stem().map(|s| s.to_string_lossy().to_string());
        (dir, base)
    } else {
        (std::env::temp_dir().join("ai-movie-player-subtitles"), None)
    }
}

/// Join an archive member name onto `dest` while preventing path traversal
/// (zip-slip). Returns `None` for absolute or escaping paths.
pub fn safe_join(dest: &Path, member_name: &str) -> Option<PathBuf> {
    // Backslashes are valid separators in zip entries from Windows.
    let member = member_name.replace('\\', "/");
    let mut candidate = PathBuf::new();
    for component in Path::new(&member).components() {
        match component {
            std::path::Component::Normal(part) => candidate.push(part),
            std::path::Component::CurDir => {}
            // Absolute paths, drive prefixes and `..` traversal are refused.
            _ => return None,
        }
    }

    if candidate.as_os_str().is_empty() {
        return None;
    }

    let joined = dest.join(&candidate);
    let normalized_dest = normalize_path(dest);
    let normalized_joined = normalize_path(&joined);
    if normalized_joined.starts_with(&normalized_dest) { Some(joined) } else { None }
}

/// Lexical path normalization (no filesystem access).
fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Choose the best subtitle entry from an archive using extension preference
/// and uncompressed size.
pub fn choose_best_entry<'a>(entries: &[(&'a str, u64)]) -> Option<&'a str> {
    entries
        .iter()
        .filter(|(name, _)| subtitle_extension(name).is_some())
        .max_by(|(name_a, size_a), (name_b, size_b)| {
            let ext_rank = |name: &str| {
                SUBTITLE_EXTENSIONS
                    .iter()
                    .position(|ext| Some(*ext) == subtitle_extension(name))
                    .unwrap_or(usize::MAX)
            };
            // Lower rank index means a more preferred extension: reverse the
            // comparison so `max_by` keeps the preferred one, then prefer the
            // larger entry among equal extensions.
            ext_rank(name_b).cmp(&ext_rank(name_a)).then_with(|| size_a.cmp(size_b))
        })
        .map(|(name, _)| *name)
}

fn subtitle_extension(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    SUBTITLE_EXTENSIONS.iter().copied().find(|ext| lower.ends_with(&format!(".{ext}")))
}

/// Normalize subtitle text bytes to UTF-8. Handles UTF-8/UTF-16 BOMs and
/// falls back to GBK for legacy Chinese subtitles that are not valid UTF-8.
pub fn decode_subtitle_text(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let units: Vec<u16> =
            rest.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
        return String::from_utf16_lossy(&units);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let units: Vec<u16> =
            rest.chunks_exact(2).map(|pair| u16::from_be_bytes([pair[0], pair[1]])).collect();
        return String::from_utf16_lossy(&units);
    }

    let stripped = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(stripped) {
        Ok(text) => text.to_string(),
        Err(_) => encoding_rs::GBK.decode(stripped).0.into_owned(),
    }
}

/// Extract/normalize downloaded bytes into a subtitle file on disk.
fn install_subtitle_bytes(
    bytes: &[u8],
    dest_dir: &Path,
    base_name: Option<&str>,
    language: &str,
) -> Result<PathBuf> {
    std::fs::create_dir_all(dest_dir)?;
    let safe_lang: String =
        language.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect();

    if bytes.starts_with(b"PK") {
        install_from_zip(bytes, dest_dir, base_name, &safe_lang)
    } else {
        let ext = guess_text_extension(bytes).unwrap_or("srt");
        let text = decode_subtitle_text(bytes);
        let dest = output_path(dest_dir, base_name, &safe_lang, ext);
        let mut file = std::fs::File::create(&dest)?;
        file.write_all(text.as_bytes())?;
        Ok(dest)
    }
}

fn install_from_zip(
    bytes: &[u8],
    dest_dir: &Path,
    base_name: Option<&str>,
    language: &str,
) -> Result<PathBuf> {
    let cursor = std::io::Cursor::new(bytes.to_vec());
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| AppError::Parse(format!("Invalid zip: {e}")))?;

    let mut entries: Vec<(String, u64)> = Vec::new();
    for i in 0..archive.len() {
        let file =
            archive.by_index(i).map_err(|e| AppError::Parse(format!("Zip entry error: {e}")))?;
        if file.is_file() {
            entries.push((file.name().to_string(), file.size()));
        }
    }

    let chosen_name =
        choose_best_entry(&entries.iter().map(|(n, s)| (n.as_str(), *s)).collect::<Vec<_>>())
            .ok_or_else(|| AppError::Parse("No subtitle file in archive".into()))?
            .to_string();

    // Reject archives whose chosen member tries to escape the destination.
    if safe_join(dest_dir, &chosen_name).is_none() {
        return Err(AppError::Parse(format!(
            "Refused unsafe archive path (zip-slip): {chosen_name}"
        )));
    }

    let index = (0..archive.len())
        .find(|&i| archive.by_index(i).is_ok_and(|f| f.name() == chosen_name))
        .expect("chosen entry exists");
    let mut member =
        archive.by_index(index).map_err(|e| AppError::Parse(format!("Zip entry error: {e}")))?;

    let mut payload = Vec::with_capacity(member.size() as usize);
    std::io::copy(&mut member, &mut payload)?;

    let ext = subtitle_extension(&chosen_name).unwrap_or("srt");
    let text = decode_subtitle_text(&payload);
    let dest = output_path(dest_dir, base_name, language, ext);
    let mut file = std::fs::File::create(&dest)?;
    file.write_all(text.as_bytes())?;

    Ok(dest)
}

fn output_path(dir: &Path, base_name: Option<&str>, language: &str, ext: &str) -> PathBuf {
    match base_name {
        Some(base) => dir.join(format!("{base}.{language}.{ext}")),
        None => dir.join(format!("subtitle_{language}.{ext}")),
    }
}

fn guess_text_extension(bytes: &[u8]) -> Option<&'static str> {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]);
    if head.contains("[Script Info]") {
        Some("ass")
    } else if head.contains("WEBVTT") {
        Some("vtt")
    } else if head.contains("-->") {
        Some("srt")
    } else {
        None
    }
}

// --- API response models (tolerant of upstream schema drift) ---

#[derive(Deserialize)]
struct SubtitleSearchResponse {
    data: Vec<SubtitleItem>,
}

#[derive(Deserialize)]
struct SubtitleItem {
    #[serde(default)]
    #[allow(dead_code)]
    id: i64,
    attributes: SubtitleAttributes,
}

#[derive(Deserialize)]
struct SubtitleAttributes {
    #[serde(default)]
    release_name: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(rename = "language_name", default)]
    language_name: Option<String>,
    #[serde(default)]
    download_count: i32,
    #[serde(default)]
    ratings: Option<serde_json::Value>,
    #[serde(default)]
    ai_translated: Option<String>,
    #[serde(default)]
    hearing_impaired: bool,
    #[serde(default)]
    hd: bool,
    #[serde(default)]
    files: Vec<SubtitleFile>,
}

#[derive(Deserialize)]
struct SubtitleFile {
    file_id: i64,
    #[serde(default)]
    file_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(language: &str, rating: Option<f64>, downloads: Option<i32>) -> SubtitleResult {
        SubtitleResult {
            title: "release".into(),
            language: language.into(),
            language_label: language.into(),
            source: SOURCE_NAME.into(),
            source_url: String::new(),
            file_name: "release.srt".into(),
            rating,
            download_count: downloads,
            is_ai: false,
            is_hearing_imp: false,
            format: "srt".into(),
            file_id: Some(1),
            is_hd: false,
        }
    }

    #[test]
    fn language_mapping_covers_chinese_variants_and_fallbacks() {
        assert_eq!(map_language_code("zh"), "zh-CN");
        assert_eq!(map_language_code("zh-CN"), "zh-CN");
        assert_eq!(map_language_code("chi"), "zh-CN");
        assert_eq!(map_language_code("zh-TW"), "zh-TW");
        assert_eq!(map_language_code("en"), "en");
        assert_eq!(map_language_code("xx-unknown"), "en");
    }

    #[test]
    fn ranking_prefers_language_then_rating_and_downloads() {
        let mut results = vec![
            result("en", Some(9.0), Some(1000)),
            result("zh-CN", Some(5.0), Some(10)),
            result("zh-CN", Some(8.0), Some(50)),
        ];
        rank_results(&mut results, &["zh".to_string(), "en".to_string()]);

        assert_eq!(results[0].language, "zh-CN");
        assert_eq!(results[0].rating, Some(8.0));
        assert_eq!(results[1].language, "zh-CN");
        assert_eq!(results[2].language, "en");
    }

    #[test]
    fn ai_translated_and_hearing_impaired_rank_lower() {
        let mut preferred = result("zh-CN", Some(7.0), Some(20));
        preferred.is_ai = false;
        let mut ai = result("zh-CN", Some(7.0), Some(20));
        ai.is_ai = true;
        let mut results = vec![ai.clone(), preferred.clone()];
        rank_results(&mut results, &["zh".to_string()]);
        assert!(!results[0].is_ai);

        let mut hi = result("en", Some(7.0), Some(20));
        hi.is_hearing_imp = true;
        let mut normal = result("en", Some(7.0), Some(20));
        normal.is_hearing_imp = false;
        let mut results = vec![hi, normal];
        rank_results(&mut results, &["en".to_string()]);
        assert!(!results[0].is_hearing_imp);
    }

    #[test]
    fn safe_join_blocks_path_traversal() {
        let dest = Path::new("/tmp/library/movie");
        assert!(safe_join(dest, "subs/movie.srt").is_some());
        assert!(safe_join(dest, "../escape.srt").is_none());
        assert!(safe_join(dest, "../../etc/passwd").is_none());
        assert!(safe_join(dest, "/etc/passwd").is_none());
        assert!(safe_join(dest, "subs\\..\\..\\escape.srt").is_none());
    }

    #[test]
    fn best_entry_prefers_srt_then_size() {
        let entries = [("movie.ass", 100u64), ("movie.srt", 50u64), ("readme.txt", 999u64)];
        assert_eq!(choose_best_entry(&entries.map(|(n, s)| (n, s))), Some("movie.srt"));

        let two_srt = [("a.srt", 10u64), ("b.srt", 100u64)];
        assert_eq!(choose_best_entry(&two_srt.map(|(n, s)| (n, s))), Some("b.srt"));

        let none = [("a.txt", 1u64)];
        assert!(choose_best_entry(&none.map(|(n, s)| (n, s))).is_none());
    }

    #[test]
    fn utf8_and_gbk_subtitles_decode_to_text() {
        let utf8 = "1\n00:00:01,000 --> 00:00:02,000\n你好\n".as_bytes();
        assert!(decode_subtitle_text(utf8).contains("你好"));

        let (gbk_bytes, _enc, had_errors) = encoding_rs::GBK.encode("简体中文字幕");
        assert!(!had_errors);
        let decoded = decode_subtitle_text(&gbk_bytes);
        assert!(decoded.contains("简体中文字幕"), "decoded: {decoded}");

        let with_bom = [&[0xEF, 0xBB, 0xBF], utf8].concat();
        assert!(decode_subtitle_text(&with_bom).starts_with('1'));
    }

    #[test]
    fn install_writes_same_basename_next_to_movie() {
        let temp = tempfile::tempdir().unwrap();
        let movie = temp.path().join("Dune.2021.mkv");
        std::fs::write(&movie, b"movie").unwrap();

        let bytes = "WEBVTT\n\n00:00.000 --> 00:01.000\nHi\n".as_bytes().to_vec();
        let saved =
            install_subtitle_bytes(&bytes, movie.parent().unwrap(), Some("Dune.2021"), "zh-CN")
                .expect("install");
        assert!(saved.file_name().unwrap().to_string_lossy().starts_with("Dune.2021.zh-CN."));
        assert!(saved.exists());
    }
}
