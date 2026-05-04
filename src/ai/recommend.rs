//! AI Recommendation Engine — personalized movie suggestions from your library.

use std::collections::BTreeMap;

use crate::ai::context::{
    display_file_size, display_genres, genre_items, increment_count, local_file_name,
    movie_technical_traits, top_counts,
};
use crate::api::ai::{AiClient, ChatMessage};
use crate::db::models::Movie;
use crate::util::error::Result;

fn push_count_line(ctx: &mut String, label: &str, counts: &BTreeMap<String, usize>) {
    let summary = top_counts(counts, 8);
    if !summary.is_empty() {
        ctx.push_str(&format!("- {}: {}\n", label, summary));
    }
}

fn build_library_signals(library: &[Movie]) -> String {
    let mut genre_counts = BTreeMap::new();
    let mut director_counts = BTreeMap::new();
    let mut resolution_counts = BTreeMap::new();
    let mut source_counts = BTreeMap::new();
    let mut codec_counts = BTreeMap::new();
    let mut four_k_count = 0usize;

    for movie in library {
        if let Some(genres) = movie.genres.as_deref() {
            for genre in genre_items(genres) {
                increment_count(&mut genre_counts, genre);
            }
        }
        if let Some(director) = movie.director.as_deref() {
            increment_count(&mut director_counts, director);
        }
        if let Some(resolution) = movie.resolution.as_deref() {
            let normalized = resolution.trim().to_lowercase();
            if normalized.contains("4k") || normalized.contains("2160p") {
                four_k_count += 1;
            }
            increment_count(&mut resolution_counts, resolution);
        }
        if let Some(source) = movie.source.as_deref() {
            increment_count(&mut source_counts, source);
        }
        if let Some(codec) = movie.codec.as_deref() {
            increment_count(&mut codec_counts, codec);
        }
    }

    let mut ctx = String::from("## Library Signals / 片库信号\n\n");
    ctx.push_str(&format!("- Total movies / 片库总量: {}\n", library.len()));
    ctx.push_str(&format!("- 4K or 2160p files / 4K 或 2160p 文件: {}\n", four_k_count));
    push_count_line(&mut ctx, "Dominant genres / 主要类型", &genre_counts);
    push_count_line(&mut ctx, "Frequent directors / 常见导演", &director_counts);
    push_count_line(&mut ctx, "Resolutions / 分辨率", &resolution_counts);
    push_count_line(&mut ctx, "Sources / 片源", &source_counts);
    push_count_line(&mut ctx, "Codecs / 编码", &codec_counts);
    ctx.push('\n');
    ctx
}

fn movie_library_line(movie: &Movie) -> String {
    let mut line = format!("- **{}**", movie.title);
    if let Some(year) = movie.year {
        line.push_str(&format!(" ({})", year));
    }

    let mut details = Vec::new();
    if let Some(rating) = movie.rating {
        details.push(format!("TMDB {:.1}/10", rating));
    }
    if let Some(genres) = movie.genres.as_deref() {
        details.push(display_genres(genres));
    }
    if let Some(director) = movie.director.as_deref().filter(|value| !value.trim().is_empty()) {
        details.push(format!("director {}", director.trim()));
    }
    if let Some(runtime) = movie.runtime {
        details.push(format!("{} min", runtime));
    }

    let technical_traits = movie_technical_traits(movie);
    if !technical_traits.is_empty() {
        details.push(technical_traits.join(", "));
    }
    if let Some(file_size) = movie.file_size.and_then(display_file_size) {
        details.push(file_size);
    }
    if let Some(file_name) = movie.local_file_path.as_deref().and_then(local_file_name) {
        details.push(format!("file {}", file_name));
    }

    if !details.is_empty() {
        line.push_str(" — ");
        line.push_str(&details.join(" · "));
    }

    line
}

/// Build a comprehensive system prompt from the user's library.
pub fn build_library_context(library: &[Movie], watchlist: &[String]) -> String {
    let mut ctx = String::from(
        "You are the taste intelligence inside AI Movie Player, created by ifq.ai. \
         Analyze the user's library and deliver recommendations that feel personal, elegant, and precise.\n\n\
         Response rules:\n\
         - Default to concise bilingual output with Chinese first and English second unless the user asks otherwise.\n\
         - Use concrete evidence from the library instead of generic genre labels.\n\
         - Treat local file names and technical media traits as private taste signals; never reveal full local paths or credentials.\n\
         - Recommend only real films. Never invent titles, directors, or years.\n\
         - Balance comfort picks, stretch picks, and one tasteful surprise.\n\n",
    );

    ctx.push_str(&build_library_signals(library));
    ctx.push_str("## User's Movie Library / 用户片库\n\n");
    for movie in library.iter().take(50) {
        ctx.push_str(&movie_library_line(movie));
        ctx.push('\n');
    }

    if library.len() > 50 {
        ctx.push_str(&format!(
            "\nOnly the newest 50 records are expanded above; use the aggregate signals for the remaining {} records. / 上方只展开最近 50 条，其余 {} 条请参考片库信号。\n",
            library.len() - 50,
            library.len() - 50
        ));
    }

    if !watchlist.is_empty() {
        ctx.push_str("\n## Watchlist / 想看清单\n\n");
        for item in watchlist {
            ctx.push_str(&format!("- {}\n", item));
        }
    }

    ctx
}

/// Generate personalized watch recommendations from the library.
pub async fn recommend_from_library(client: &AiClient, library: &[Movie]) -> Result<String> {
    let system = build_library_context(library, &[]);
    let messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(
            "请基于我的片库，用中英双语推荐 5 部我现在最该看的馆藏影片。\
             For each pick, include: why it matches my taste, the mood or moment it suits, and what kind of attention it rewards.\
             Finish with one wild card from my library and one blind spot I should add later.",
        ),
    ];
    client.chat(&messages).await
}

/// Get AI-curated discovery: movies NOT in the library that the user would love.
pub async fn discover_new(client: &AiClient, library: &[Movie]) -> Result<String> {
    let system = build_library_context(library, &[]);
    let messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(
            "请基于我的片库，用中英双语推荐 8 部我还没收藏、但大概率会喜欢的真实电影。\
             For each: title, year, director, why it fits me, and whether it is a comfort pick, stretch pick, or surprise pick.\
             Include both internationally loved classics and a few distinctive hidden gems.",
        ),
    ];
    client.chat(&messages).await
}

/// AI analyzes the library and gives a taste profile summary.
pub async fn taste_profile(client: &AiClient, library: &[Movie]) -> Result<String> {
    let system = build_library_context(library, &[]);
    let messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(
            "请基于我的片库，用中英双语输出一份高级但自然的观影品味画像。\
             Cover my dominant genres, directors, eras, emotional preferences, and blind spots.\
             End with a short 'cinephile signature' that feels specific to my collection, and cite actual films as evidence.",
        ),
    ];
    client.chat(&messages).await
}

#[cfg(test)]
mod tests {
    use super::build_library_context;
    use crate::db::models::Movie;

    fn sample_movie(index: usize) -> Movie {
        Movie {
            id: index as i64,
            tmdb_id: Some(1000 + index as i64),
            imdb_id: None,
            title: format!("Movie {}", index),
            title_cn: None,
            original_title: None,
            year: Some(2000 + index as i32),
            release_date: None,
            poster_path: None,
            poster_local: None,
            backdrop_path: None,
            backdrop_local: None,
            rating: Some(7.0 + index as f64 / 10.0),
            rating_count: None,
            genres: Some("[\"Drama\"]".to_string()),
            runtime: None,
            overview: None,
            overview_cn: None,
            tagline: None,
            director: None,
            cast_list: None,
            language: None,
            country: None,
            local_file_path: None,
            file_size: None,
            file_hash: None,
            resolution: None,
            source: None,
            codec: None,
            audio_langs: None,
            added_date: "2026-05-03T00:00:00Z".to_string(),
            updated_date: "2026-05-03T00:00:00Z".to_string(),
            tmdb_data: None,
        }
    }

    #[test]
    fn build_library_context_limits_embedded_movies_to_fifty() {
        let library: Vec<Movie> = (0..55).map(sample_movie).collect();

        let context = build_library_context(&library, &[]);

        assert!(context.contains("Movie 0"));
        assert!(context.contains("Movie 49"));
        assert!(!context.contains("Movie 50"));
        assert!(context.contains("Total movies / 片库总量: 55"));
        assert!(context.contains("remaining 5 records"));
    }

    #[test]
    fn build_library_context_includes_watchlist_section_only_when_present() {
        let library = vec![sample_movie(1)];
        let context_without_watchlist = build_library_context(&library, &[]);
        let context_with_watchlist = build_library_context(
            &library,
            &["In the Mood for Love".to_string(), "Yi Yi".to_string()],
        );

        assert!(!context_without_watchlist.contains("## Watchlist / 想看清单"));
        assert!(context_with_watchlist.contains("## Watchlist / 想看清单"));
        assert!(context_with_watchlist.contains("- In the Mood for Love"));
        assert!(context_with_watchlist.contains("- Yi Yi"));
    }

    #[test]
    fn build_library_context_uses_file_basename_and_technical_traits() {
        let mut movie = sample_movie(1);
        movie.local_file_path = Some("/Users/example/Private Movies/Movie 1.2160p.mkv".into());
        movie.file_size = Some(2_147_483_648);
        movie.resolution = Some("2160p".into());
        movie.source = Some("BluRay".into());
        movie.codec = Some("HEVC".into());
        movie.director = Some("A Director".into());

        let context = build_library_context(&[movie], &[]);

        assert!(context.contains("Movie 1.2160p.mkv"));
        assert!(context.contains("resolution 2160p"));
        assert!(context.contains("source BluRay"));
        assert!(context.contains("codec HEVC"));
        assert!(context.contains("A Director"));
        assert!(!context.contains("/Users/example"));
        assert!(!context.contains("Private Movies"));
        assert!(!context.contains(r#"["Drama"]"#));
    }
}
