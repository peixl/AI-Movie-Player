//! Ignored live smoke tests for OpenAI-compatible providers.
//!
//! These tests intentionally read provider details from environment variables so
//! maintainers can validate real endpoints without committing secrets or vendor
//! model names into the repository.

use std::{env, path::Path};

use ai_movie_player::{
    ai::chat,
    api::ai::{AiClient, AiConfig, ChatMessage},
    core::{filename_parser, metadata_service::MetadataService},
    db::models::Movie,
};

fn env_value(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn live_config() -> Option<(String, String, String, String)> {
    let endpoint = env_value("AI_MOVIE_PLAYER_LIVE_ENDPOINT")?;
    let api_key = env_value("AI_MOVIE_PLAYER_LIVE_API_KEY")?;
    let model = env_value("AI_MOVIE_PLAYER_LIVE_MODEL")?;
    let video_path = env_value("AI_MOVIE_PLAYER_LIVE_VIDEO_PATH")?;
    Some((endpoint, api_key, model, video_path))
}

fn movie_from_video_path(video_path: &Path) -> Movie {
    let filename = video_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "Local Test Video.mp4".into());
    let parsed = filename_parser::parse_filename(&filename);
    let title =
        if parsed.title.trim().is_empty() { "Local Test Video".to_string() } else { parsed.title };

    Movie {
        id: 0,
        tmdb_id: None,
        imdb_id: None,
        title,
        title_cn: None,
        original_title: None,
        year: parsed.year,
        release_date: None,
        poster_path: None,
        poster_local: None,
        backdrop_path: None,
        backdrop_local: None,
        rating: None,
        rating_count: None,
        genres: None,
        runtime: None,
        overview: Some(
            "Metadata is intentionally sparse; infer only from the local filename and media traits."
                .into(),
        ),
        overview_cn: None,
        tagline: None,
        director: None,
        cast_list: None,
        language: None,
        country: None,
        local_file_path: Some(video_path.to_string_lossy().to_string()),
        file_size: std::fs::metadata(video_path).ok().map(|metadata| metadata.len() as i64),
        file_hash: MetadataService::compute_file_hash(video_path, 64 * 1024).ok(),
        resolution: parsed.resolution,
        source: parsed.source,
        codec: parsed.codec,
        audio_langs: None,
        added_date: String::new(),
        updated_date: String::new(),
        tmdb_data: None,
    }
}

#[tokio::test]
#[ignore = "requires live endpoint credentials in AI_MOVIE_PLAYER_LIVE_* environment variables"]
async fn live_openai_compatible_movie_prompt_streams() {
    let Some((endpoint, api_key, model, video_path)) = live_config() else {
        eprintln!("skipping live AI smoke test; AI_MOVIE_PLAYER_LIVE_* variables are not all set");
        return;
    };

    let video_path = Path::new(&video_path);
    assert!(video_path.exists(), "configured live video path does not exist");

    let movie = movie_from_video_path(video_path);
    let client =
        AiClient::new(AiConfig { endpoint, api_key, model, temperature: 0.2, max_tokens: 512 });

    let messages = vec![
        ChatMessage::system(chat::build_movie_context(&movie)),
        ChatMessage::user(
            "用中英双语各一句话回答：这个本地视频文件在片库里应该如何被 AI 用作观影上下文？不要输出本地完整路径。",
        ),
    ];

    let mut streamed = String::new();
    let response = client.chat_stream(&messages, |token| streamed.push_str(token)).await.unwrap();

    assert!(!response.trim().is_empty());
    assert_eq!(response, streamed);

    if let Some(parent) = video_path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        assert!(!response.contains(parent.to_string_lossy().as_ref()));
    }
}
