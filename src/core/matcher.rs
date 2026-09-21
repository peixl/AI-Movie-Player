//! Fuzzy TMDB result matcher.
//!
//! The previous matcher accepted an exact title+year pair or, failing that,
//! blindly took the first search result — which silently attaches the wrong
//! metadata to remakes, sequels, and romanized titles. This module scores every
//! candidate with normalized title similarity (token Dice + containment +
//! character ratio), year distance, and TMDB popularity, and only returns a
//! match that clears a confidence threshold.

use std::collections::HashSet;

use crate::db::models::{ParsedFilename, TmdbSearchResult};

const LEADING_ARTICLES: &[&str] = &[
    "the", "a", "an", "la", "le", "les", "el", "los", "las", "il", "lo", "der", "die", "das", "det",
];

/// Normalize a title for comparison: lowercase, keep alphanumeric characters
/// (including CJK), drop punctuation, strip leading articles, collapse spaces.
pub fn normalize_title(input: &str) -> String {
    let mut chars: Vec<char> = Vec::with_capacity(input.len());
    let mut prev_space = true;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            chars.push(ch.to_ascii_lowercase());
            prev_space = false;
        } else if !prev_space {
            chars.push(' ');
            prev_space = true;
        }
    }
    let mut normalized: String = chars.iter().collect();
    normalized = normalized.trim().to_string();

    for article in LEADING_ARTICLES {
        if let Some(rest) = normalized.strip_prefix(&format!("{} ", article)) {
            normalized = rest.to_string();
            break;
        }
    }
    normalized
}

fn token_set(input: &str) -> HashSet<String> {
    input.split_whitespace().map(str::to_string).collect()
}

/// Sørensen–Dice coefficient over token sets.
fn token_dice(a: &str, b: &str) -> f64 {
    let a = token_set(a);
    let b = token_set(b);
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(&b).count() as f64;
    2.0 * intersection / (a.len() + b.len()) as f64
}

/// Damerau–Levenshtein similarity ratio over characters, range `[0.0, 1.0]`.
/// Unlike plain Levenshtein, an adjacent character transposition (a very common
/// typo, e.g. "Incpetion" vs "Inception") counts as a single edit.
fn char_ratio(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let mut prev_prev: Vec<usize> = vec![0; b.len() + 1];
    let mut prev: Vec<usize> = (0..=b.len()).collect();

    for i in 1..=a.len() {
        let mut cur = vec![0; b.len() + 1];
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (cur[j - 1] + 1) // insertion
                .min(prev[j] + 1) // deletion
                .min(prev[j - 1] + cost); // substitution / match

            // Adjacent transposition.
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                cur[j] = cur[j].min(prev_prev[j - 2] + 1);
            }
        }
        prev_prev = prev;
        prev = cur;
    }

    let distance = prev[b.len()] as f64;
    let longest = a.len().max(b.len()) as f64;
    1.0 - distance / longest
}

/// Composite title similarity in `[0.0, 1.0]`.
pub fn title_similarity(file_title: &str, candidate_title: &str) -> f64 {
    let a = normalize_title(file_title);
    let b = normalize_title(candidate_title);
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a == b {
        return 1.0;
    }

    let dice = token_dice(&a, &b);

    // Substring containment handles "Dune" vs "Dune Part Two" and CJK titles
    // that appear verbatim inside a bilingual release name.
    let shorter = a.len().min(b.len());
    let containment =
        if shorter >= 4 { if a.contains(&b) || b.contains(&a) { 0.9 } else { 0.0 } } else { 0.0 };

    // Compact character ratio helps with no-space CJK and small typos.
    let compact_a: String = a.chars().filter(|c| !c.is_whitespace()).collect();
    let compact_b: String = b.chars().filter(|c| !c.is_whitespace()).collect();
    let ratio = if compact_a.chars().count() <= 24 || compact_b.chars().count() <= 24 {
        char_ratio(&compact_a, &compact_b)
    } else {
        0.0
    };

    dice.max(containment).max(ratio)
}

fn year_score(parsed_year: Option<i32>, candidate_year: Option<i32>) -> f64 {
    match (parsed_year, candidate_year) {
        (Some(expected), Some(actual)) => match (expected - actual).unsigned_abs() {
            0 => 25.0,
            1 => 12.0,
            2 => 4.0,
            _ => -12.0,
        },
        _ => 0.0,
    }
}

fn popularity_score(popularity: Option<f64>) -> f64 {
    // TMDB popularity is a positive float typically in 0..500; compress it to a
    // small tiebreaker so it can never override title/year evidence.
    popularity.map(|p| ((1.0 + p.max(0.0)).ln() / (1.0_f64 + 500.0_f64).ln()) * 5.0).unwrap_or(0.0)
}

/// Confidence score for a candidate; higher is better. A negative or
/// below-threshold score means the candidate should not be auto-accepted.
pub fn score_candidate(parsed: &ParsedFilename, candidate: &TmdbSearchResult) -> f64 {
    let similarity = title_similarity(&parsed.title, &candidate.title);
    similarity * 70.0
        + year_score(parsed.year, candidate.year)
        + popularity_score(candidate.popularity)
}

/// Whether a score is high enough to trust given the year evidence.
fn clears_threshold(score: f64, similarity: f64, parsed: &ParsedFilename) -> bool {
    let has_year = parsed.year.is_some();
    let required_similarity = if has_year { 0.45 } else { 0.72 };
    similarity >= required_similarity && score >= if has_year { 55.0 } else { 62.0 }
}

/// Pick the best confident TMDB candidate, if any.
pub fn select_best_match<'a>(
    parsed: &ParsedFilename,
    results: &'a [TmdbSearchResult],
) -> Option<&'a TmdbSearchResult> {
    results
        .iter()
        .map(|candidate| {
            let similarity = title_similarity(&parsed.title, &candidate.title);
            let score = score_candidate(parsed, candidate);
            (candidate, score, similarity)
        })
        .filter(|(_, score, similarity)| clears_threshold(*score, *similarity, parsed))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(candidate, _, _)| candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::TmdbSearchResult;

    fn parsed(title: &str, year: Option<i32>) -> ParsedFilename {
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

    fn result(
        id: i64,
        title: &str,
        year: Option<i32>,
        popularity: Option<f64>,
    ) -> TmdbSearchResult {
        TmdbSearchResult {
            tmdb_id: id,
            title: title.to_string(),
            title_cn: None,
            year,
            poster_path: None,
            overview: None,
            rating: None,
            popularity,
        }
    }

    #[test]
    fn normalization_strips_noise_and_articles() {
        assert_eq!(normalize_title("The Dark Knight!"), "dark knight");
        assert_eq!(normalize_title("  dune:_part_two "), "dune part two");
        assert_eq!(normalize_title("流浪地球"), "流浪地球");
    }

    #[test]
    fn exact_title_and_year_wins_over_remake() {
        let parsed = parsed("Dune", Some(2021));
        let results = vec![
            result(1, "Dune", Some(1984), Some(40.0)),
            result(2, "Dune", Some(2021), Some(30.0)),
            result(3, "Dune: Part Two", Some(2024), Some(500.0)),
        ];
        assert_eq!(select_best_match(&parsed, &results).unwrap().tmdb_id, 2);
    }

    #[test]
    fn sequel_containment_with_matching_year_beats_original() {
        let parsed = parsed("Dune Part Two", Some(2024));
        let results = vec![
            result(1, "Dune", Some(2021), Some(400.0)),
            result(2, "Dune: Part Two", Some(2024), Some(400.0)),
        ];
        assert_eq!(select_best_match(&parsed, &results).unwrap().tmdb_id, 2);
    }

    #[test]
    fn unrelated_first_result_is_rejected() {
        let parsed = parsed("Unknown Film", Some(2023));
        let results = vec![
            result(10, "Possible Match", Some(2023), None),
            result(11, "Another Match", Some(2023), None),
        ];
        assert!(select_best_match(&parsed, &results).is_none());
    }

    #[test]
    fn punctuation_and_case_do_not_break_matching() {
        let parsed = parsed("Everything Everywhere All at Once", Some(2022));
        let results = vec![result(1, "Everything Everywhere All at Once", Some(2022), Some(120.0))];
        assert_eq!(select_best_match(&parsed, &results).unwrap().tmdb_id, 1);
    }

    #[test]
    fn cjk_titles_match_verbatim() {
        let sim = title_similarity("流浪地球", "流浪地球");
        assert_eq!(sim, 1.0);

        let parsed = parsed("流浪地球", Some(2019));
        let results = vec![
            result(1, "The Wandering Earth", Some(2019), Some(60.0)),
            result(2, "流浪地球", Some(2019), Some(60.0)),
        ];
        assert_eq!(select_best_match(&parsed, &results).unwrap().tmdb_id, 2);
    }

    #[test]
    fn typo_tolerant_matching() {
        // One transposed/typo character in a short title still scores high.
        let sim = title_similarity("Incpetion", "Inception");
        assert!(sim > 0.8, "expected high similarity, got {sim}");
    }

    #[test]
    fn popularity_only_breaks_ties_between_equally_good_titles() {
        let parsed = parsed("Hero Film", Some(2020));
        let results = vec![
            result(1, "Hero Film", Some(2020), Some(5.0)),
            result(2, "Hero Film", Some(2020), Some(400.0)),
        ];
        assert_eq!(select_best_match(&parsed, &results).unwrap().tmdb_id, 2);
    }
}
