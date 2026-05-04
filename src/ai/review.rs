//! AI Review & Analysis Generator — one-click movie insights.

use crate::api::ai::{AiClient, ChatMessage};
use crate::db::models::Movie;
use crate::util::error::Result;

/// Generate a comprehensive AI review of a movie.
pub async fn generate_review(client: &AiClient, movie: &Movie) -> Result<String> {
    let system = build_review_system_prompt(movie);
    let messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(
            "请用中英双语写一篇结构清晰、判断鲜明的电影短评。包括：\n\
             1. 一句高概括结论 / one-line verdict\n\
             2. 影片最出色的地方 / what works\n\
             3. 不足与遗憾 / what does not fully work\n\
             4. 适合谁看、不适合谁 / audience fit\n\
             5. 最终结论：Watch / Consider / Skip\n\
             Keep it elegant, honest, and specific.",
        ),
    ];
    client.chat(&messages).await
}

/// Generate quick bullet-point pros/cons.
pub async fn quick_verdict(client: &AiClient, movie: &Movie) -> Result<String> {
    let system = build_review_system_prompt(movie);
    let messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(
            "请用中英双语给我一个快速判断，格式如下：\n\
             **One-liner / 一句话：** ...\n\
             **Pros / 优点：**\n- ...\n- ...\n- ...\n\
             **Cons / 不足：**\n- ...\n- ...\n\
             **Verdict / 结论：** ✅ Worth Watching / ⚠️ Consider / ❌ Skip",
        ),
    ];
    client.chat(&messages).await
}

/// Generate interesting trivia and behind-the-scenes facts.
pub async fn generate_trivia(client: &AiClient, movie: &Movie) -> Result<String> {
    let system = build_review_system_prompt(movie);
    let messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(
            "请用中英双语分享 8 条真正有信息密度的幕后细节、制作秘密、选角逸闻或彩蛋。\
             Number them, and avoid obvious trivia the user could get from a surface-level summary.",
        ),
    ];
    client.chat(&messages).await
}

/// Generate a comparison between two movies.
pub async fn compare_movies(client: &AiClient, movie_a: &Movie, movie_b: &Movie) -> Result<String> {
    let system = format!(
        "You are an expert film critic. Compare these two films:\n\n\
         **Movie A:** {} ({})\nDirector: {}\nRating: {:.1}/10\nGenres: {}\nSynopsis: {}\n\n\
         **Movie B:** {} ({})\nDirector: {}\nRating: {:.1}/10\nGenres: {}\nSynopsis: {}\n",
        movie_a.title,
        movie_a.year.unwrap_or(0),
        movie_a.director.as_deref().unwrap_or("Unknown"),
        movie_a.rating.unwrap_or(0.0),
        movie_a.genres.as_deref().unwrap_or("Unknown"),
        movie_a.overview.as_deref().unwrap_or("N/A"),
        movie_b.title,
        movie_b.year.unwrap_or(0),
        movie_b.director.as_deref().unwrap_or("Unknown"),
        movie_b.rating.unwrap_or(0.0),
        movie_b.genres.as_deref().unwrap_or("Unknown"),
        movie_b.overview.as_deref().unwrap_or("N/A"),
    );
    let messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(
            "Compare these two films. Which is better and why? \
             What do they have in common? How do they differ in style, theme, and execution? \
             If someone likes one, will they like the other? \
             Give a clear winner with reasoning.",
        ),
    ];
    client.chat(&messages).await
}

fn build_review_system_prompt(movie: &Movie) -> String {
    let mut prompt = String::from(
        "You are a professional film critic inside AI Movie Player, created by ifq.ai. \
         Default to a concise bilingual structure with Chinese first and English second unless the user asks otherwise. \
         Be insightful, honest, elegant, and specific. Use markdown formatting.\n\n",
    );
    prompt.push_str(&format!("**Film / 影片:** {} ({})\n", movie.title, movie.year.unwrap_or(0)));
    if let Some(ref dir) = movie.director {
        prompt.push_str(&format!("**Director / 导演:** {}\n", dir));
    }
    if let Some(ref genres) = movie.genres {
        prompt.push_str(&format!("**Genres / 类型:** {}\n", genres));
    }
    if let Some(r) = movie.rating {
        prompt.push_str(&format!("**TMDB Rating / 评分:** {:.1}/10\n", r));
    }
    if let Some(ref overview) = movie.overview {
        prompt.push_str(&format!("**Synopsis / 剧情简介:** {}\n", overview));
    }
    prompt
}
