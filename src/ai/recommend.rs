//! AI Recommendation Engine — personalized movie suggestions from your library.

use crate::api::ai::{AiClient, ChatMessage};
use crate::db::models::Movie;
use crate::util::error::Result;

/// Build a comprehensive system prompt from the user's library.
pub fn build_library_context(library: &[Movie], watchlist: &[String]) -> String {
    let mut ctx = String::from(
        "You are the taste intelligence inside AI-Movie-Player, created by ifq.ai. \
         Analyze the user's library and deliver recommendations that feel personal, elegant, and precise.\n\n\
         Response rules:\n\
         - Default to concise bilingual output with Chinese first and English second unless the user asks otherwise.\n\
         - Use concrete evidence from the library instead of generic genre labels.\n\
         - Recommend only real films. Never invent titles, directors, or years.\n\
         - Balance comfort picks, stretch picks, and one tasteful surprise.\n\n",
    );

    ctx.push_str("## User's Movie Library / 用户片库\n\n");
    for movie in library.iter().take(50) {
        ctx.push_str(&format!("- **{}**", movie.title));
        if let Some(y) = movie.year {
            ctx.push_str(&format!(" ({})", y));
        }
        if let Some(r) = movie.rating {
            ctx.push_str(&format!(" — ★{:.1}", r));
        }
        if let Some(ref genres) = movie.genres {
            ctx.push_str(&format!(" [{}]", genres));
        }
        ctx.push('\n');
    }

    if !watchlist.is_empty() {
        ctx.push_str("\n## Watchlist / 想看清单\n\n");
        for item in watchlist {
            ctx.push_str(&format!("- {}\n", item));
        }
    }

    ctx.push_str(&format!("\nTotal movies in library / 片库总量: {}\n", library.len()));
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
        assert!(context.contains("Total movies in library / 片库总量: 55"));
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
}
