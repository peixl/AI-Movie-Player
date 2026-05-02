//! Extract movie metadata (title, year, resolution, codec, source) from filenames.
//!
//! Supports common naming conventions: standard scene names, episode-style,
//! parenthesized titles, and underscore-separated names.

use std::sync::LazyLock;

use regex::Regex;

use crate::db::models::ParsedFilename;

// Pre-compiled regex patterns using LazyLock for zero-cost static initialization
static RE_STANDARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(.+?)\.(\d{4})\.(\d{3,4}p|4K)\.(BluRay|WEB-DL|WEBRip|HDTV|DVDRip|BDRip|Remux)\.(x264|x265|H264|H\.?265|HEVC)(?:-(\w+))?$"
    ).expect("Invalid RE_STANDARD pattern")
});

static RE_EPISODE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^\[(\w+)\]\s+(.+?)\s*-\s*(\d+(?:v\d+)?)\s*\[(.*?)\]$"
    ).expect("Invalid RE_EPISODE pattern")
});

static RE_PARENTHESES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(.+?)\s*\((\d{4})\)\s*(?:(\d{3,4}p|4K)\s*)?(?:\.?(BluRay|WEB-DL|WEBRip|HDTV|DVDRip|BDRip|Remux)\s*)?(?:\.?(x264|x265|H264|H\.?265|HEVC)?)?$"
    ).expect("Invalid RE_PARENTHESES pattern")
});

static RE_UNDERSCORE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(.+?)_(\d{4})_(\d{3,4}p|4K)_(BluRay|WEB-DL|WEBRip|HDTV|Remux)$"
    ).expect("Invalid RE_UNDERSCORE pattern")
});

static RE_YEAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b((?:19|20)\d{2})\b").expect("Invalid RE_YEAR pattern")
});

static RE_RESOLUTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\d{3,4}p|4K)").expect("Invalid RE_RESOLUTION pattern")
});

/// Parse a video filename into structured metadata.
/// Supports 4 common naming conventions plus a generic fallback.
pub fn parse_filename(filename: &str) -> ParsedFilename {
    let basename = strip_extension(filename);

    // Pattern 1: "{title}.{year}.{resolution}.{source}.{codec}-{group}"
    if let Some(caps) = RE_STANDARD.captures(&basename) {
        return ParsedFilename {
            title: clean_title(&caps[1]),
            year: caps[2].parse().ok(),
            resolution: Some(caps[3].to_string()),
            source: Some(caps[4].to_string()),
            codec: Some(caps[5].to_string()),
            group: caps.get(6).map(|m| m.as_str().to_string()),
            episode: None,
            is_tv: false,
        };
    }

    // Pattern 2: "[Group] Title - Episode [Resolution]"
    if let Some(caps) = RE_EPISODE.captures(&basename) {
        return ParsedFilename {
            title: clean_title(&caps[2]),
            year: None,
            resolution: Some(caps[4].to_string()),
            source: None,
            codec: None,
            group: Some(caps[1].to_string()),
            episode: Some(caps[3].to_string()),
            is_tv: true,
        };
    }

    // Pattern 3: "{title} ({year}) {resolution} {source} {codec}"
    if let Some(caps) = RE_PARENTHESES.captures(&basename) {
        return ParsedFilename {
            title: clean_title(&caps[1]),
            year: caps[2].parse().ok(),
            resolution: caps.get(3).map(|m| m.as_str().to_string()),
            source: caps.get(4).map(|m| m.as_str().to_string()),
            codec: caps.get(5).map(|m| m.as_str().to_string()),
            group: None,
            episode: None,
            is_tv: false,
        };
    }

    // Pattern 4: "{title}_{year}_{resolution}_{source}"
    if let Some(caps) = RE_UNDERSCORE.captures(&basename) {
        return ParsedFilename {
            title: clean_title(&caps[1]),
            year: caps[2].parse().ok(),
            resolution: Some(caps[3].to_string()),
            source: Some(caps[4].to_string()),
            codec: None,
            group: None,
            episode: None,
            is_tv: false,
        };
    }

    // Fallback: extract year and resolution with generic patterns
    let year = RE_YEAR
        .captures(&basename)
        .and_then(|caps| caps[1].parse::<i32>().ok());

    let resolution = RE_RESOLUTION
        .captures(&basename)
        .map(|caps| caps[1].to_string());

    // Use everything before the year as title, or the whole basename
    let title = if let Some(y) = year {
        if let Some(idx) = basename.find(&y.to_string()) {
            clean_title(&basename[..idx])
        } else {
            clean_title(&basename)
        }
    } else {
        clean_title(&basename)
    };

    ParsedFilename {
        title,
        year,
        resolution,
        source: None,
        codec: None,
        group: None,
        episode: None,
        is_tv: false,
    }
}

fn strip_extension(filename: &str) -> String {
    let path = std::path::Path::new(filename);
    path.with_extension("").to_string_lossy().to_string()
}

fn clean_title(s: &str) -> String {
    s.trim()
        .trim_end_matches(|c: char| c == '-' || c == '_' || c == '.')
        .replace('.', " ")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn is_video_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    let ext = std::path::Path::new(&lower)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    matches!(
        ext.as_str(),
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv"
            | "webm" | "m4v" | "mpg" | "mpeg" | "ts"
            | "rmvb" | "divx" | "iso" | "vob"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Standard naming (dots) ---

    #[test]
    fn test_standard_naming() {
        let r = parse_filename("Avatar.2009.2160p.BluRay.x265-RARBG.mp4");
        assert_eq!(r.title, "Avatar");
        assert_eq!(r.year, Some(2009));
        assert_eq!(r.resolution.as_deref(), Some("2160p"));
        assert_eq!(r.source.as_deref(), Some("BluRay"));
        assert_eq!(r.codec.as_deref(), Some("x265"));
        assert_eq!(r.group.as_deref(), Some("RARBG"));
    }

    #[test]
    fn test_standard_naming_hevc() {
        let r = parse_filename("Dune.2021.1080p.BluRay.HEVC-RARBG.mkv");
        assert_eq!(r.title, "Dune");
        assert_eq!(r.year, Some(2021));
        assert_eq!(r.codec.as_deref(), Some("HEVC"));
    }

    #[test]
    fn test_standard_naming_no_group() {
        let r = parse_filename("Tenet.2020.2160p.WEB-DL.x264.mkv");
        assert_eq!(r.title, "Tenet");
        assert_eq!(r.source.as_deref(), Some("WEB-DL"));
        assert_eq!(r.group, None);
    }

    // --- Parentheses year ---

    #[test]
    fn test_parentheses_year_full() {
        let r = parse_filename("Inception (2010) 1080p BluRay x264.mp4");
        assert_eq!(r.title, "Inception");
        assert_eq!(r.year, Some(2010));
    }

    #[test]
    fn test_parentheses_year_minimal() {
        let r = parse_filename("Parasite (2019).mkv");
        assert_eq!(r.title, "Parasite");
        assert_eq!(r.year, Some(2019));
    }

    #[test]
    fn test_parentheses_year_with_codec() {
        let r = parse_filename("The Matrix (1999) 2160p BluRay HEVC.mkv");
        assert_eq!(r.title, "The Matrix");
        assert_eq!(r.year, Some(1999));
        assert_eq!(r.resolution.as_deref(), Some("2160p"));
        assert_eq!(r.codec.as_deref(), Some("HEVC"));
    }

    // --- Episode pattern ---

    #[test]
    fn test_episode_naming() {
        let r = parse_filename("[SubsPlease] Mushoku Tensei - 01v2 [1080p].mkv");
        assert_eq!(r.title, "Mushoku Tensei");
        assert_eq!(r.episode.as_deref(), Some("01v2"));
        assert!(r.is_tv);
        assert_eq!(r.group.as_deref(), Some("SubsPlease"));
    }

    // --- Underscore separator ---

    #[test]
    fn test_underscore_naming() {
        let r = parse_filename("Oppenheimer_2023_1080p_BluRay.mkv");
        assert_eq!(r.title, "Oppenheimer");
        assert_eq!(r.year, Some(2023));
        assert_eq!(r.resolution.as_deref(), Some("1080p"));
        assert_eq!(r.source.as_deref(), Some("BluRay"));
    }

    // --- Fallback ---

    #[test]
    fn test_fallback_bare() {
        let r = parse_filename("SomeRandomMovie.mkv");
        assert!(!r.title.is_empty());
        assert_eq!(r.year, None);
    }

    #[test]
    fn test_fallback_year_only() {
        let r = parse_filename("Arrival.2016.DVDRip.mkv");
        assert_eq!(r.year, Some(2016));
        assert!(!r.title.is_empty());
    }

    #[test]
    fn test_fallback_resolution_only() {
        let r = parse_filename("OldMovie.720p.x264.mkv");
        assert_eq!(r.resolution.as_deref(), Some("720p"));
    }

    #[test]
    fn test_fallback_with_4k() {
        let r = parse_filename("Documentary.4K.2022.mkv");
        assert_eq!(r.resolution.as_deref(), Some("4K"));
        assert_eq!(r.year, Some(2022));
    }

    // --- Title cleaning ---

    #[test]
    fn test_title_clean_dots() {
        let r = parse_filename("The.Dark.Knight.2008.1080p.BluRay.x264.mkv");
        assert_eq!(r.title, "The Dark Knight");
    }

    #[test]
    fn test_title_clean_underscores() {
        let r = parse_filename("Blade_Runner_2049_2017_2160p_Remux.mkv");
        assert_eq!(r.title, "Blade Runner 2049");
    }

    #[test]
    fn test_title_clean_trailing_dash() {
        let r = parse_filename("Interstellar.2014.1080p.BluRay.x264-YIFY.mp4");
        assert_eq!(r.title, "Interstellar");
    }

    // --- Video file detection ---

    #[test]
    fn test_video_detection() {
        for ext in &["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts", "iso"] {
            assert!(is_video_file(&format!("movie.{}", ext)), "should detect .{}", ext);
        }
    }

    #[test]
    fn test_non_video_rejected() {
        assert!(!is_video_file("subtitle.srt"));
        assert!(!is_video_file("readme.txt"));
        assert!(!is_video_file("cover.jpg"));
        assert!(!is_video_file("movie.nfo"));
    }

    // --- Edge cases ---

    #[test]
    fn test_empty_string() {
        let r = parse_filename("");
        assert!(r.title.is_empty());
    }

    #[test]
    fn test_only_extension() {
        let r = parse_filename(".mkv");
        assert!(r.title.is_empty());
    }

    #[test]
    fn test_chinese_title() {
        let r = parse_filename("流浪地球.The.Wandering.Earth.2019.1080p.BluRay.x265.mkv");
        assert!(r.title.contains("流浪地球") || r.title.contains("Wandering"));
    }

    #[test]
    fn test_mixed_case() {
        let r = parse_filename("GODZILLA.MINUS.ONE.2023.2160P.BLURAY.X265.mkv");
        assert_eq!(r.year, Some(2023));
        assert_eq!(r.resolution.as_deref(), Some("2160P"));
    }
}
