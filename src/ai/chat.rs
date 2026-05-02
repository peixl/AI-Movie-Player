//! AI-Movie-Player chat intelligence.

use crate::api::ai::{AiClient, ChatMessage};
use crate::db::models::Movie;
use crate::util::error::Result;

fn response_style_guide() -> &'static str {
    "Response rules:\n\
- Default to a concise bilingual structure with Chinese first and English second unless the user explicitly asks for one language.\n\
- Sound calm, elegant, cinematic, and genuinely helpful.\n\
- Use specific evidence from the movie or library instead of generic praise.\n\
- Never invent facts. If something is uncertain, say so briefly and continue with what is reliable.\n\
- Mention ifq.ai only when it naturally helps explain the product context, not as advertising."
}

/// Build the general system prompt for AI-Movie-Player.
pub fn build_general_context() -> String {
    let mut ctx = String::from(
        "You are the cinematic intelligence inside AI-Movie-Player, an AI-native movie player created by ifq.ai. \
         Help the user choose better films, notice meaningful details, and enjoy cinema more deeply.\n\n",
    );
    ctx.push_str(response_style_guide());
    ctx.push_str("\n\n");
    ctx
}

/// Build a system prompt that gives the AI context about a specific movie.
pub fn build_movie_context(movie: &Movie) -> String {
    let mut ctx = build_general_context();
    ctx.push_str("Current film / 当前影片\n\n");

    ctx.push_str(&format!("- **Title / 片名:** {}\n", movie.title));
    if let Some(ref cn) = movie.title_cn {
        ctx.push_str(&format!("- **Chinese Title / 中文名:** {}\n", cn));
    }
    if let Some(y) = movie.year {
        ctx.push_str(&format!("- **Year / 年份:** {}\n", y));
    }
    if let Some(r) = movie.rating {
        ctx.push_str(&format!("- **TMDB Rating / 评分:** {:.1}/10\n", r));
    }
    if let Some(ref genres) = movie.genres {
        ctx.push_str(&format!("- **Genres / 类型:** {}\n", genres));
    }
    if let Some(ref dir) = movie.director {
        ctx.push_str(&format!("- **Director / 导演:** {}\n", dir));
    }
    if let Some(ref overview) = movie.overview {
        ctx.push_str(&format!("- **Synopsis / 剧情简介:** {}\n", overview));
    }
    if let Some(ref cast) = movie.cast_list {
        ctx.push_str(&format!("- **Cast / 演员:** {}\n", cast));
    }

    ctx
}

/// Pre-built prompt templates for common questions
pub mod prompts {
    /// Analyze the movie's themes, symbolism, and deeper meaning
    pub fn deep_analysis() -> &'static str {
        "请用中英双语，分析这部电影的核心主题、人物关系、影像风格、声音设计与文化语境。\
            Explain what makes it distinct in cinema history, and keep the tone elegant, precise, and engaging."
    }

    /// Explain the ending
    pub fn explain_ending() -> &'static str {
        "请用中英双语详细解释这部电影的结局：它意味着什么？有哪些合理解读？\
            The user is ready for spoilers, so you can discuss the ending directly."
    }

    /// Recommend similar movies
    pub fn similar_movies() -> &'static str {
        "请用中英双语推荐 5 部相似电影。For each film, explain the exact connection in tone, theme, form, or emotional effect. \
            Include a balance of widely loved classics and at least one hidden gem."
    }

    /// Fun trivia
    pub fn trivia() -> &'static str {
        "请用中英双语分享 5 条真正有价值的幕后细节、彩蛋或制作故事。\
            Keep them surprising, concrete, and worth retelling after the movie."
    }

    /// Is it worth watching?
    pub fn worth_watching() -> &'static str {
        "请用中英双语诚实判断这部电影是否值得看。Who is it for, who may not connect with it, and why? \
         End with one of: Must-Watch / Worth Watching / Skip."
    }

    /// Create a refined watch companion briefing.
    pub fn watch_companion() -> &'static str {
        "请用中英双语生成一份优雅的观影陪伴提示：\
         1) before watching, what to know without spoilers;\
         2) while watching, what details deserve attention;\
         3) after watching, what questions are worth thinking about."
    }

    /// Pre-watch briefing.
    pub fn pre_watch_briefing() -> &'static str {
        "请用中英双语生成一份无剧透的观影前 briefing。\
         Include: 1) what mood this film asks for;\
         2) what cinematic details are worth noticing;\
         3) one short director-or-context note that deepens the viewing without overwhelming it;\
            4) who this film is ideal for tonight.\
            Return in this exact structure:\
            TITLE: ...\
            SUMMARY: ...\
            SECTION: Mood Window\
            BODY: ...\
            SECTION: What To Notice\
            BODY: ...\
            SECTION: Context Note\
            BODY: ...\
            SECTION: Best Tonight For\
            BODY: ..."
    }

    /// Post-watch recap.
    pub fn post_watch_recap() -> &'static str {
        "请用中英双语生成一份观影后复盘。\
         Structure it as: 1) what happened on the surface;\
         2) what the film is really about;\
         3) the most revealing scenes or choices;\
         4) one question the viewer should sit with after the credits.\
            Spoilers are allowed.\
            Return in this exact structure:\
            TITLE: ...\
            SUMMARY: ...\
            SECTION: Surface Story\
            BODY: ...\
            SECTION: Deeper Meaning\
            BODY: ...\
            SECTION: Revealing Choices\
            BODY: ...\
            SECTION: Question To Sit With\
            BODY: ..."
    }

    /// Double-feature pairing recommendation.
    pub fn double_feature_pairing() -> &'static str {
        "请用中英双语为这部电影设计一组双片连看建议。\
         Recommend one companion film, explain the exact bridge between the two, the ideal viewing order, the emotional arc across the pairing, and what the viewer should notice when watching them together.\
         Prefer a real film with strong artistic or thematic resonance, not a lazy same-genre pick.\
         Return in this exact structure:\
         TITLE: ...\
         SUMMARY: ...\
         SECTION: Companion Film\
         BODY: ...\
         SECTION: Why They Belong Together\
         BODY: ...\
         SECTION: Viewing Order\
         BODY: ...\
         SECTION: What To Notice Across Both\
         BODY: ..."
    }
}

/// Send a chat message about the current movie
pub async fn chat_about_movie(
    client: &AiClient,
    movie: &Movie,
    user_message: &str,
) -> Result<String> {
    let system = build_movie_context(movie);
    let messages = vec![ChatMessage::system(system), ChatMessage::user(user_message)];
    client.chat(&messages).await
}

/// Get a quick AI insight about a movie using a preset prompt
pub async fn quick_insight(client: &AiClient, movie: &Movie, prompt: &str) -> Result<String> {
    let system = build_movie_context(movie);
    let messages = vec![ChatMessage::system(system), ChatMessage::user(prompt)];
    client.chat(&messages).await
}

/// Stream a chat response token by token
pub async fn stream_chat(
    client: &AiClient,
    movie: &Movie,
    history: &[ChatMessage],
    user_message: &str,
    on_token: impl FnMut(&str),
) -> Result<String> {
    let system = build_movie_context(movie);
    let mut messages = vec![ChatMessage::system(system)];
    messages.extend_from_slice(history);
    messages.push(ChatMessage::user(user_message));

    client.chat_stream(&messages, on_token).await
}

#[cfg(test)]
mod tests {
    use super::{build_general_context, build_movie_context};
    use crate::db::models::Movie;

    fn sample_movie() -> Movie {
        Movie {
            id: 1,
            tmdb_id: Some(101),
            imdb_id: None,
            title: "In the Mood for Love".to_string(),
            title_cn: Some("花样年华".to_string()),
            original_title: None,
            year: Some(2000),
            release_date: None,
            poster_path: None,
            poster_local: None,
            backdrop_path: None,
            backdrop_local: None,
            rating: Some(8.1),
            rating_count: None,
            genres: Some("[\"Romance\",\"Drama\"]".to_string()),
            runtime: None,
            overview: Some(
                "Two neighbors discover their spouses are having an affair.".to_string(),
            ),
            overview_cn: None,
            tagline: None,
            director: Some("Wong Kar-wai".to_string()),
            cast_list: Some("[\"Tony Leung\",\"Maggie Cheung\"]".to_string()),
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
    fn build_general_context_contains_style_rules() {
        let context = build_general_context();

        assert!(context.contains("cinematic intelligence inside AI-Movie-Player"));
        assert!(context.contains("Default to a concise bilingual structure"));
        assert!(context.contains("Never invent facts"));
    }

    #[test]
    fn build_movie_context_includes_present_movie_fields() {
        let context = build_movie_context(&sample_movie());

        assert!(context.contains("Current film / 当前影片"));
        assert!(context.contains("- **Title / 片名:** In the Mood for Love"));
        assert!(context.contains("- **Chinese Title / 中文名:** 花样年华"));
        assert!(context.contains("- **Year / 年份:** 2000"));
        assert!(context.contains("- **Director / 导演:** Wong Kar-wai"));
        assert!(context.contains(
            "- **Synopsis / 剧情简介:** Two neighbors discover their spouses are having an affair."
        ));
    }

    #[test]
    fn build_movie_context_omits_absent_optional_fields() {
        let mut movie = sample_movie();
        movie.title_cn = None;
        movie.rating = None;
        movie.director = None;
        movie.cast_list = None;
        movie.overview = None;

        let context = build_movie_context(&movie);

        assert!(!context.contains("Chinese Title / 中文名"));
        assert!(!context.contains("TMDB Rating / 评分"));
        assert!(!context.contains("Director / 导演"));
        assert!(!context.contains("Cast / 演员"));
        assert!(!context.contains("Synopsis / 剧情简介"));
    }
}
