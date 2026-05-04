//! Shared prompt context helpers for movie and library AI workflows.

use std::{collections::BTreeMap, path::Path};

use crate::db::models::{CastMember, Movie};

const DEFAULT_TEXT_LIMIT: usize = 900;

pub(crate) fn clipped_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    let mut chars = trimmed.chars();
    let clipped: String = chars.by_ref().take(max_chars).collect();

    if chars.next().is_some() { format!("{}...", clipped.trim_end()) } else { clipped }
}

pub(crate) fn push_prompt_line(output: &mut String, label: &str, value: impl AsRef<str>) {
    let value = value.as_ref().trim();
    if !value.is_empty() {
        output.push_str(&format!("- **{}:** {}\n", label, clipped_text(value, DEFAULT_TEXT_LIMIT)));
    }
}

pub(crate) fn genre_items(raw_genres: &str) -> Vec<String> {
    if let Ok(genres) = serde_json::from_str::<Vec<String>>(raw_genres) {
        return genres
            .into_iter()
            .map(|genre| genre.trim().to_string())
            .filter(|genre| !genre.is_empty())
            .collect();
    }

    raw_genres
        .split([',', '/', '|'])
        .map(|genre| genre.trim().trim_matches(['[', ']', '"']).to_string())
        .filter(|genre| !genre.is_empty())
        .collect()
}

pub(crate) fn display_genres(raw_genres: &str) -> String {
    let genres = genre_items(raw_genres);
    if genres.is_empty() { raw_genres.trim().to_string() } else { genres.join(", ") }
}

pub(crate) fn display_cast(raw_cast: &str, limit: usize) -> String {
    if let Ok(cast_members) = serde_json::from_str::<Vec<CastMember>>(raw_cast) {
        let cast = cast_members
            .into_iter()
            .take(limit)
            .map(|member| {
                if member.character.trim().is_empty() {
                    member.name
                } else {
                    format!("{} as {}", member.name, member.character)
                }
            })
            .collect::<Vec<_>>();

        if !cast.is_empty() {
            return cast.join(", ");
        }
    }

    clipped_text(raw_cast, DEFAULT_TEXT_LIMIT)
}

pub(crate) fn local_file_name(local_file_path: &str) -> Option<String> {
    Path::new(local_file_path)
        .file_name()
        .map(|name| name.to_string_lossy().trim().to_string())
        .filter(|name| !name.is_empty())
}

pub(crate) fn display_file_size(bytes: i64) -> Option<String> {
    if bytes < 0 {
        return None;
    }

    let size = bytes as f64;
    let gib = 1024.0 * 1024.0 * 1024.0;
    let mib = 1024.0 * 1024.0;

    if size >= gib {
        Some(format!("{:.1} GB", size / gib))
    } else if size >= mib {
        Some(format!("{:.1} MB", size / mib))
    } else {
        Some(format!("{} bytes", bytes))
    }
}

pub(crate) fn movie_technical_traits(movie: &Movie) -> Vec<String> {
    let mut traits = Vec::new();

    if let Some(resolution) = movie.resolution.as_deref().filter(|value| !value.trim().is_empty()) {
        traits.push(format!("resolution {}", resolution.trim()));
    }
    if let Some(source) = movie.source.as_deref().filter(|value| !value.trim().is_empty()) {
        traits.push(format!("source {}", source.trim()));
    }
    if let Some(codec) = movie.codec.as_deref().filter(|value| !value.trim().is_empty()) {
        traits.push(format!("codec {}", codec.trim()));
    }
    if let Some(language) = movie.language.as_deref().filter(|value| !value.trim().is_empty()) {
        traits.push(format!("language {}", language.trim()));
    }

    traits
}

pub(crate) fn increment_count(counts: &mut BTreeMap<String, usize>, value: impl AsRef<str>) {
    let value = value.as_ref().trim();
    if !value.is_empty() {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
}

pub(crate) fn top_counts(counts: &BTreeMap<String, usize>, limit: usize) -> String {
    let mut entries = counts.iter().collect::<Vec<_>>();
    entries.sort_by(|(left_name, left_count), (right_name, right_count)| {
        right_count.cmp(left_count).then_with(|| left_name.cmp(right_name))
    });

    entries
        .into_iter()
        .take(limit)
        .map(|(name, count)| format!("{} ({})", name, count))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::{display_cast, display_file_size, display_genres, local_file_name};

    #[test]
    fn local_file_name_keeps_only_basename() {
        let name = local_file_name("/Users/example/Movies/Secret Folder/Dune.2021.mkv");

        assert_eq!(name.as_deref(), Some("Dune.2021.mkv"));
    }

    #[test]
    fn display_genres_turns_json_array_into_readable_text() {
        assert_eq!(display_genres(r#"["Drama","Sci-Fi"]"#), "Drama, Sci-Fi");
    }

    #[test]
    fn display_cast_turns_structured_cast_into_readable_text() {
        let cast = r#"[{"name":"Performer","character":"Lead","profile_path":null}]"#;

        assert_eq!(display_cast(cast, 4), "Performer as Lead");
    }

    #[test]
    fn display_file_size_uses_human_units() {
        assert_eq!(display_file_size(1_610_612_736).as_deref(), Some("1.5 GB"));
    }
}
