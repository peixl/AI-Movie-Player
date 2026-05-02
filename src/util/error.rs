//! Unified error types for the application.

use thiserror::Error;

/// Application-wide error enum covering network, database, API, and parsing failures.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Network error / 网络错误: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error / 数据库错误: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error / 输入输出错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("TMDB API error / TMDB 接口错误 (code {code}): {message}")]
    TmdbApi { code: u16, message: String },

    #[error(
        "TMDB API key not configured / 未配置 TMDB API Key. Get one at https://www.themoviedb.org/settings/api"
    )]
    TmdbKeyMissing,

    #[error("Movie not found on TMDB / TMDB 未找到影片: {query}")]
    MovieNotFound { query: String },

    #[error("Multiple matches / 匹配结果过多 for '{query}': {count} results")]
    AmbiguousMatch { query: String, count: usize },

    #[error("Subtitle download failed / 字幕下载失败 ({source}): {reason}")]
    SubtitleError { source: String, reason: String },

    #[error("Parse error / 解析错误: {0}")]
    Parse(String),

    #[error("Invalid configuration / 配置无效: {0}")]
    Config(String),

    #[error("Image processing error / 图像处理错误: {0}")]
    Image(#[from] image::ImageError),
}

pub type Result<T> = std::result::Result<T, AppError>;
