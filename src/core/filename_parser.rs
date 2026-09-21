//! Extract movie metadata (title, year, resolution, codec, source) from filenames.
//!
//! Supports common naming conventions: standard scene names, episode-style,
//! parenthesized titles, and underscore-separated names. Anything not matched
//! by the strict anchored patterns falls through to a token-based generic
//! parser that recognizes resolutions, sources, codecs, audio/edition tags,
//! `SxxEyy` episodes, and release-group suffixes.

use std::sync::LazyLock;

use regex::Regex;

use crate::db::models::ParsedFilename;

// Pre-compiled regex patterns using LazyLock for zero-cost static initialization
static RE_STANDARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(.+?)\.(\d{4})\.(\d{3,4}p|4K|UHD)\.(BluRay|Blu-Ray|WEB-DL|WEBDL|WEBRip|WEB|HDTV|DVDRip|BDRip|BRRip|HDRip|Remux)\.(x264|x265|H264|H\.?265|HEVC|AV1|VP9|XviD|DivX)(?:-(\w+))?$"
    ).expect("Invalid RE_STANDARD pattern")
});

static RE_EPISODE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\[(\w+)\]\s+(.+?)\s*-\s*(\d+(?:v\d+)?)\s*\[(.*?)\]$")
        .expect("Invalid RE_EPISODE pattern")
});

static RE_PARENTHESES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(.+?)\s*\((\d{4})\)\s*(?:(\d{3,4}p|4K|UHD)\s*)?(?:\.?(BluRay|Blu-Ray|WEB-DL|WEBDL|WEBRip|WEB|HDTV|DVDRip|BDRip|BRRip|HDRip|Remux)\s*)?(?:\.?(x264|x265|H264|H\.?265|HEVC|AV1|VP9|XviD|DivX)?)?$"
    ).expect("Invalid RE_PARENTHESES pattern")
});

static RE_UNDERSCORE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(.+?)_(\d{4})_(\d{3,4}p|4K|UHD)_(BluRay|Blu-Ray|WEB-DL|WEBDL|WEBRip|WEB|HDTV|Remux)$")
        .expect("Invalid RE_UNDERSCORE pattern")
});

static RE_YEAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b((?:19|20)\d{2})\b").expect("Invalid RE_YEAR pattern"));

static RE_RESOLUTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\d{3,4}p|4K|UHD|8K)").expect("Invalid RE_RESOLUTION pattern")
});

static RE_SXXEYY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bS(\d{1,2})E(\d{1,3})(?:E(\d{1,3}))?\b").expect("Invalid RE_SXXEYY pattern")
});

static RE_BRACKET_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\[\(][^\[\]\(\)]*[\]\)]").expect("Invalid RE_BRACKET_BLOCK pattern")
});

static RE_TRAILING_GROUP: LazyLock<Regex> = LazyLock::new(|| {
    // Scene groups follow a hyphen (occasionally underscore); dots/spaces are
    // title word separators and must not steal words like "Two" or "Title".
    Regex::new(r"(?i)[-_]([a-z0-9]{2,12})$").expect("Invalid RE_TRAILING_GROUP pattern")
});

static RE_STRAY_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\[\]\(\)]").expect("Invalid RE_STRAY_BRACKET pattern"));

/// Source keywords ordered so longer/more-specific names match first.
const SOURCE_KEYWORDS: &[&str] = &[
    "WEB-DL", "WEBDL", "WEBRip", "Blu-Ray", "BluRay", "BDRip", "BRRip", "DVDRip", "DVDScr",
    "HDRip", "HDTC", "TELESYNC", "HDCAM", "CAMRip", "Remux", "HDTV", "DVDR", "WEB", "CAM", "TC",
    "TS",
];

/// Codec keywords, longest first.
const CODEC_KEYWORDS: &[&str] = &[
    "H.265", "H.264", "HEVC", "x265", "x264", "H265", "H264", "AV1", "VP9", "XviD", "DivX", "MPEG2",
];

/// Audio, container-quality, edition, and release-noise tokens that must never
/// survive into the title.
const NOISE_KEYWORDS: &[&str] = &[
    // Audio
    "DTS-HD",
    "DTSHD",
    "DTS",
    "TrueHD",
    "Dolby",
    "Atmos",
    "DD+",
    "DD5.1",
    "DD",
    "AC3",
    "EAC3",
    "AAC",
    "FLAC",
    "Opus",
    "MP3",
    "5.1",
    "7.1",
    "2.0",
    // HDR / quality
    "HDR10+",
    "HDR10",
    "HDR",
    "10bit",
    "8bit",
    "12bit",
    "SDR",
    "IMAX",
    "DoVi",
    "DV",
    "UHD",
    "BDRip",
    "BRRip",
    // Editions
    "Extended",
    "Unrated",
    "Directors",
    "Director's",
    "Theatrical",
    "Remastered",
    "Anniversary",
    "Criterion",
    "Proper",
    "Repack",
    "Internal",
    "MULTi",
    "MULTISUB",
    "DUAL",
    "DUBBED",
    "SUBBED",
    "3D",
    "MVC",
    // Common index/release-site tokens
    "YIFY",
    "YTS",
    "RARBG",
    "EZTV",
    "PAHE",
    "GANOOl",
    "PSA",
    "GalaxyRG",
    "NTG",
    "NTb",
];

/// Parse a video filename into structured metadata.
/// Supports 4 common naming conventions plus a generic fallback.
pub fn parse_filename(filename: &str) -> ParsedFilename {
    let basename = strip_extension(filename);

    // Pattern 1: "{title}.{year}.{resolution}.{source}.{codec}-{group}"
    if let Some(caps) = RE_STANDARD.captures(&basename) {
        return ParsedFilename {
            title: clean_title(&caps[1]),
            year: caps[2].parse().ok(),
            resolution: Some(normalize_resolution(&caps[3])),
            source: Some(caps[4].to_string()),
            codec: Some(normalize_codec(&caps[5])),
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
            resolution: caps.get(4).map(|m| normalize_resolution(m.as_str())),
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
            resolution: caps.get(3).map(|m| normalize_resolution(m.as_str())),
            source: caps.get(4).map(|m| m.as_str().to_string()),
            codec: caps.get(5).map(|m| normalize_codec(m.as_str())),
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
            resolution: Some(normalize_resolution(&caps[3])),
            source: Some(caps[4].to_string()),
            codec: None,
            group: None,
            episode: None,
            is_tv: false,
        };
    }

    parse_generic(&basename)
}

/// Token-based parser for names that do not match an anchored scene pattern.
fn parse_generic(basename: &str) -> ParsedFilename {
    let mut working = basename.to_string();

    // Episode markers (S01E02, S01E02E03).
    let (episode, is_tv) = if let Some(caps) = RE_SXXEYY.captures(&working) {
        let mut label = format!("S{}E{}", &caps[1], &caps[2]);
        if let Some(second) = caps.get(3) {
            label.push('E');
            label.push_str(second.as_str());
        }
        // Remove the marker (and surrounding separators) from the title text.
        if let Some(mat) = RE_SXXEYY.find(&working.clone()) {
            working.replace_range(mat.range(), " ");
        }
        (Some(label), true)
    } else {
        (None, false)
    };

    let year = RE_YEAR
        .captures(&working)
        .and_then(|caps| caps[1].parse::<i32>().ok())
        // Reject implausible years that are usually part of other tokens.
        .filter(|y| (1915..=2035).contains(y));

    let resolution = RE_RESOLUTION.captures(&working).map(|caps| normalize_resolution(&caps[1]));

    let source = find_keyword(&working, &RE_SOURCE_ANY, SOURCE_KEYWORDS);
    // Keep the matched surface form (e.g. "x264") for title stripping; the
    // canonical form (e.g. "H.264") is what we report in structured fields.
    let codec_surface = RE_CODEC_ANY
        .captures(&working)
        .and_then(|caps| caps.get(2).map(|m| m.as_str().to_string()));
    let codec = codec_surface.as_ref().map(|raw| normalize_codec(raw));

    // Title extraction: cut at the year when present.
    let mut title_working = if let Some(y) = year {
        if let Some(idx) = working.find(&y.to_string()) {
            working[..idx].to_string()
        } else {
            working.clone()
        }
    } else {
        working.clone()
    };

    // Remove bracketed blocks like [1080p], [YIFY], (Director's Cut).
    title_working = RE_BRACKET_BLOCK.replace_all(&title_working, " ").to_string();
    // A year cut can leave a dangling opener such as "Title (".
    title_working = RE_STRAY_BRACKET.replace_all(&title_working, " ").to_string();

    // Remove every recognized technical/noise token.
    title_working = remove_noise_tokens(&title_working);
    if let Some(res) = &resolution {
        title_working = remove_word(&title_working, res);
    }
    if let Some(src) = &source {
        title_working = remove_word(&title_working, src);
    }
    if let Some(surface) = &codec_surface {
        title_working = remove_word(&title_working, surface);
    }
    // Numeric tokens such as bare 1080p variants and bitrates.
    title_working = Regex::new(r"(?i)\b\d{3,4}p\b")
        .expect("resolution strip regex")
        .replace_all(&title_working, " ")
        .to_string();

    // Trailing release group, only when the name otherwise looks like a release
    // (has a year, resolution, or source) — protects hyphenated real titles.
    let looks_like_release =
        year.is_some() || resolution.is_some() || source.is_some() || codec.is_some() || is_tv;
    let group = if looks_like_release {
        if let Some(caps) =
            RE_TRAILING_GROUP.captures(title_working.trim_end_matches(['-', '_', '.', ' ']))
        {
            let candidate = caps[1].to_string();
            // Avoid treating a common short word as a group.
            if candidate.len() >= 3
                && candidate.chars().any(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
            {
                title_working = title_working
                    .trim_end_matches(['-', '_', '.', ' '])
                    .strip_suffix(&caps[0])
                    .unwrap_or(&title_working)
                    .to_string();
                Some(candidate)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let title = clean_title(&title_working);

    ParsedFilename { title, year, resolution, source, codec, group, episode, is_tv }
}

// The `regex` crate has no look-around, so word boundaries consume the
// surrounding separator via capture groups: group 1 = leading boundary
// (start or non-alphanumeric), group 2 = keyword, group 3 = trailing boundary.
static RE_SOURCE_ANY: LazyLock<Regex> = LazyLock::new(|| {
    let alternation =
        SOURCE_KEYWORDS.iter().map(|k| regex::escape(k)).collect::<Vec<_>>().join("|");
    Regex::new(&format!(r"(?i)(^|[^a-z0-9])({})([^a-z0-9]|$)", alternation))
        .expect("valid source keyword regex")
});

static RE_CODEC_ANY: LazyLock<Regex> = LazyLock::new(|| {
    let alternation = CODEC_KEYWORDS.iter().map(|k| regex::escape(k)).collect::<Vec<_>>().join("|");
    Regex::new(&format!(r"(?i)(^|[^a-z0-9])({})([^a-z0-9]|$)", alternation))
        .expect("valid codec keyword regex")
});

static RE_NOISE_ANY: LazyLock<Regex> = LazyLock::new(|| {
    let alternation = NOISE_KEYWORDS.iter().map(|k| regex::escape(k)).collect::<Vec<_>>().join("|");
    // `+` is a legitimate part of tokens such as HDR10+ / DD+.
    Regex::new(&format!(r"(?i)(^|[^a-z0-9])({})([^a-z0-9+]|$)", alternation))
        .expect("valid noise keyword regex")
});

/// Map a case-insensitive matched surface form back to its canonical keyword.
fn canonical_keyword(matched: &str, keywords: &[&str]) -> String {
    let matched_lower = matched.to_lowercase();
    keywords
        .iter()
        .find(|keyword| keyword.to_lowercase() == matched_lower)
        .copied()
        .unwrap_or(matched)
        .to_string()
}

/// Whole-word keyword search using the precompiled alternation.
fn find_keyword(text: &str, regex: &Regex, keywords: &[&str]) -> Option<String> {
    let caps = regex.captures(text)?;
    Some(canonical_keyword(caps.get(2)?.as_str(), keywords))
}

/// Remove all recognized technical/noise tokens. Adjacent tokens share the
/// separator consumed by the boundary groups, so repeat until stable.
fn remove_noise_tokens(text: &str) -> String {
    let mut current = text.to_string();
    for _ in 0..6 {
        let next = RE_NOISE_ANY.replace_all(&current, "${1} ${3}").to_string();
        if next == current {
            break;
        }
        current = next;
    }
    current
}

/// Remove a whole-word/term occurrence (case-insensitive) from text.
fn remove_word(text: &str, word: &str) -> String {
    let pattern = format!(r"(?i)(^|[^a-z0-9]){}([^a-z0-9+]|$)", regex::escape(word));
    match Regex::new(&pattern) {
        Ok(re) => {
            let mut current = text.to_string();
            for _ in 0..4 {
                let next = re.replace_all(&current, "${1} ${2}").to_string();
                if next == current {
                    break;
                }
                current = next;
            }
            current
        }
        Err(_) => text.to_string(),
    }
}

fn normalize_resolution(raw: &str) -> String {
    let upper = raw.trim().to_uppercase();
    if upper == "UHD" {
        "4K".to_string()
    } else if let Some(pixels) = upper.strip_suffix('P') {
        format!("{pixels}p")
    } else {
        upper
    }
}

fn normalize_codec(raw: &str) -> String {
    let cleaned = raw.replace('.', "");
    match cleaned.to_lowercase().as_str() {
        "h264" | "x264" => "H.264".to_string(),
        "h265" | "x265" | "hevc" => "H.265".to_string(),
        other => other.to_uppercase(),
    }
}

fn strip_extension(filename: &str) -> String {
    if filename.starts_with('.') && filename[1..].chars().filter(|c| *c == '.').count() == 0 {
        return String::new();
    }

    let path = std::path::Path::new(filename);
    path.with_extension("").to_string_lossy().to_string()
}

fn clean_title(s: &str) -> String {
    s.trim()
        .trim_end_matches(['-', '_', '.'])
        .replace(['.', '_'], " ")
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
        "mp4"
            | "mkv"
            | "avi"
            | "mov"
            | "wmv"
            | "flv"
            | "webm"
            | "m4v"
            | "mpg"
            | "mpeg"
            | "ts"
            | "m2ts"
            | "m2t"
            | "mts"
            | "rmvb"
            | "rm"
            | "divx"
            | "iso"
            | "vob"
            | "ogv"
            | "3gp"
            | "asf"
            | "f4v"
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
        assert_eq!(r.codec.as_deref(), Some("H.265"));
        assert_eq!(r.group.as_deref(), Some("RARBG"));
    }

    #[test]
    fn test_standard_naming_hevc() {
        let r = parse_filename("Dune.2021.1080p.BluRay.HEVC-RARBG.mkv");
        assert_eq!(r.title, "Dune");
        assert_eq!(r.year, Some(2021));
        assert_eq!(r.codec.as_deref(), Some("H.265"));
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
        assert_eq!(r.codec.as_deref(), Some("H.265"));
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
        for ext in &["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts", "iso", "m2ts"] {
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
        assert_eq!(r.resolution.as_deref(), Some("2160p"));
    }

    // --- Generic token parser ---

    #[test]
    fn generic_parser_strips_dense_scene_noise() {
        let r = parse_filename(
            "Dune.Part.Two.2024.2160p.UHD.BluRay.x265.10bit.HDR.DTS-HD.MA.5.1-GROUPNAME.mkv",
        );
        assert_eq!(r.year, Some(2024));
        assert_eq!(r.resolution.as_deref(), Some("2160p"));
        assert_eq!(r.source.as_deref(), Some("BluRay"));
        assert_eq!(r.codec.as_deref(), Some("H.265"));
        assert_eq!(r.title, "Dune Part Two");
        assert!(!r.title.to_lowercase().contains("hdr"));
        assert!(!r.title.to_lowercase().contains("dts"));
    }

    #[test]
    fn generic_parser_handles_web_release_with_group() {
        let r = parse_filename("Everything.Everywhere.All.at.Once.2022.1080p.WEB.H264-NTb.mkv");
        assert_eq!(r.year, Some(2022));
        assert_eq!(r.resolution.as_deref(), Some("1080p"));
        assert_eq!(r.source.as_deref(), Some("WEB"));
        assert_eq!(r.codec.as_deref(), Some("H.264"));
        assert_eq!(r.title, "Everything Everywhere All at Once");
        assert_eq!(r.group.as_deref(), Some("NTb"));
    }

    #[test]
    fn generic_parser_detects_sxxeyy_episodes() {
        let r = parse_filename("Breaking.Bad.S05E07.1080p.BluRay.x264-DEMAND.mkv");
        assert!(r.is_tv);
        assert_eq!(r.episode.as_deref(), Some("S05E07"));
        assert_eq!(r.title, "Breaking Bad");
    }

    #[test]
    fn generic_parser_keeps_hyphenated_titles_without_release_markers() {
        // No year/technical tags: do not steal a trailing word as a group.
        let r = parse_filename("Paris-Texas.mkv");
        assert_eq!(r.title, "Paris-Texas");
        assert_eq!(r.group, None);
    }

    #[test]
    fn generic_parser_strips_bracket_noise() {
        let r = parse_filename("Perfect Blue (1997) [1080p] [BluRay] [x265].mkv");
        assert_eq!(r.title, "Perfect Blue");
        assert_eq!(r.year, Some(1997));
        assert_eq!(r.resolution.as_deref(), Some("1080p"));
    }

    #[test]
    fn generic_parser_detects_av1_and_dolby_vision_noise() {
        let r = parse_filename("Movie.Title.2023.2160p.WEB-DL.DV.HDR10+.AV1.Opus-group.mkv");
        assert_eq!(r.year, Some(2023));
        assert_eq!(r.codec.as_deref(), Some("AV1"));
        assert_eq!(r.source.as_deref(), Some("WEB-DL"));
        assert_eq!(r.title, "Movie Title");
    }
}
