//! OpenAI-compatible AI client.
//! Supports OpenAI, Azure OpenAI, Ollama, LM Studio, and any OpenAI-compatible API.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::Semaphore;

use crate::util::error::{AppError, Result};

const DEFAULT_ENDPOINT: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_ERROR_BODY_CHARS: usize = 800;

static RE_BEARER_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)bearer\s+[A-Za-z0-9._~+/=-]{8,}").expect("valid bearer token regex")
});

static RE_COMMON_API_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:sk|tp)-[A-Za-z0-9_-]{8,}\b").expect("valid API key regex"));

/// AI provider configuration
#[derive(Debug, Clone)]
pub struct AiConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            endpoint: DEFAULT_ENDPOINT.into(),
            api_key: String::new(),
            model: DEFAULT_MODEL.into(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

impl AiConfig {
    pub fn is_configured(&self) -> bool {
        !self.endpoint.trim().is_empty()
            && !self.api_key.trim().is_empty()
            && !self.model.trim().is_empty()
    }
}

/// A single chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into() }
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamChunk {
    choices: Vec<ChatStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamChoice {
    delta: ChatStreamDelta,
}

#[derive(Debug, Deserialize)]
struct ChatStreamDelta {
    content: Option<String>,
}

enum StreamParseResult {
    Token(String),
    Done,
    Ignore,
}

fn parse_stream_line(line: &str) -> StreamParseResult {
    let line = line.trim();

    if line.is_empty() || !line.starts_with("data: ") {
        return StreamParseResult::Ignore;
    }

    let data = &line[6..];
    if data == "[DONE]" {
        return StreamParseResult::Done;
    }

    match serde_json::from_str::<ChatStreamChunk>(data) {
        Ok(parsed) => parsed
            .choices
            .first()
            .and_then(|choice| choice.delta.content.as_deref())
            .map(|token| StreamParseResult::Token(token.to_string()))
            .unwrap_or(StreamParseResult::Ignore),
        Err(_) => StreamParseResult::Ignore,
    }
}

fn collect_stream_tokens(buffer: &mut String, flush_partial: bool) -> (Vec<String>, bool) {
    let mut tokens = Vec::new();
    let mut saw_done = false;

    while let Some(newline_pos) = buffer.find('\n') {
        let line = buffer[..newline_pos].to_string();
        buffer.drain(..=newline_pos);

        match parse_stream_line(&line) {
            StreamParseResult::Token(token) => tokens.push(token),
            StreamParseResult::Done => {
                saw_done = true;
                buffer.clear();
                break;
            }
            StreamParseResult::Ignore => {}
        }
    }

    if flush_partial && !saw_done {
        let line = buffer.trim().to_string();
        buffer.clear();

        match parse_stream_line(&line) {
            StreamParseResult::Token(token) => tokens.push(token),
            StreamParseResult::Done => saw_done = true,
            StreamParseResult::Ignore => {}
        }
    }

    (tokens, saw_done)
}

fn truncate_for_display(value: &str, max_chars: usize) -> String {
    let mut chars = value.trim().chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();

    if chars.next().is_some() { format!("{}...", truncated.trim_end()) } else { truncated }
}

fn sanitize_provider_error_body(body: &str) -> String {
    let redacted = RE_COMMON_API_KEY.replace_all(body, "[REDACTED_API_KEY]");
    let redacted = RE_BEARER_TOKEN.replace_all(&redacted, "Bearer [REDACTED]");
    let body = truncate_for_display(&redacted, MAX_ERROR_BODY_CHARS);

    if body.is_empty() { "empty error body / 空错误内容".into() } else { body }
}

fn ai_api_error(status: u16, body: &str) -> AppError {
    AppError::Config(format!(
        "AI API error / AI 接口错误 ({}): {}",
        status,
        sanitize_provider_error_body(body)
    ))
}

fn extract_chat_content(data: ChatResponse) -> Result<String> {
    let content = data
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_deref())
        .unwrap_or_default()
        .to_string();

    if content.trim().is_empty() {
        return Err(AppError::Parse(
            "AI response did not contain text content / AI 响应没有文本内容".into(),
        ));
    }

    Ok(content)
}

fn parse_non_stream_chat_response(raw_response: &str) -> Option<String> {
    serde_json::from_str::<ChatResponse>(raw_response.trim())
        .ok()
        .and_then(|data| extract_chat_content(data).ok())
}

/// AI client with OpenAI-compatible API
pub struct AiClient {
    config: AiConfig,
    client: reqwest::Client,
    rate_limiter: Arc<Semaphore>,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(DEFAULT_REQUEST_TIMEOUT)
            .user_agent(format!("ai-movie-player/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_else(|err| {
                log::warn!("Failed to build configured AI HTTP client: {}", err);
                reqwest::Client::new()
            });

        Self { config, client, rate_limiter: Arc::new(Semaphore::new(10)) }
    }

    pub fn config(&self) -> &AiConfig {
        &self.config
    }

    pub fn is_ready(&self) -> bool {
        self.config.is_configured()
    }

    fn validate_ready(&self) -> Result<()> {
        if self.config.endpoint.trim().is_empty() {
            return Err(AppError::Config("AI endpoint not configured / 未配置 AI 接口地址".into()));
        }
        if self.config.api_key.trim().is_empty() {
            return Err(AppError::Config("AI API key not configured / 未配置 AI API Key".into()));
        }
        if self.config.model.trim().is_empty() {
            return Err(AppError::Config("AI model not configured / 未配置 AI 模型".into()));
        }

        Ok(())
    }

    fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.config.endpoint.trim().trim_end_matches('/'))
    }

    /// Send a chat completion request and get the full response.
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String> {
        self.validate_ready()?;

        let _permit = self.rate_limiter.acquire().await.expect("AI rate limiter semaphore closed");

        let req = ChatRequest {
            model: self.config.model.trim().to_string(),
            messages: messages.to_vec(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: None,
        };

        let resp = self
            .client
            .post(self.chat_completions_url())
            .header("Authorization", format!("Bearer {}", self.config.api_key.trim()))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(ai_api_error(status, &body));
        }

        let data: ChatResponse = resp.json().await?;
        extract_chat_content(data)
    }

    /// Stream a chat completion, yielding tokens as they arrive.
    pub async fn chat_stream(
        &self,
        messages: &[ChatMessage],
        mut on_token: impl FnMut(&str),
    ) -> Result<String> {
        self.validate_ready()?;

        let _permit = self.rate_limiter.acquire().await.expect("AI rate limiter semaphore closed");

        let req = ChatRequest {
            model: self.config.model.trim().to_string(),
            messages: messages.to_vec(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: Some(true),
        };

        let resp = self
            .client
            .post(self.chat_completions_url())
            .header("Authorization", format!("Bearer {}", self.config.api_key.trim()))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream, application/json")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(ai_api_error(status, &body));
        }

        let mut full_response = String::new();
        let mut raw_response = String::new();
        let mut buffer = String::new();
        let mut stream = resp.bytes_stream();
        let mut saw_done = false;

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_text = String::from_utf8_lossy(&chunk);
            raw_response.push_str(&chunk_text);
            buffer.push_str(&chunk_text);

            let (tokens, done) = collect_stream_tokens(&mut buffer, false);
            for token in tokens {
                full_response.push_str(&token);
                on_token(&token);
            }

            if done {
                saw_done = true;
                break;
            }
        }

        if !saw_done {
            let (tokens, _) = collect_stream_tokens(&mut buffer, true);
            for token in tokens {
                full_response.push_str(&token);
                on_token(&token);
            }
        }

        if full_response.trim().is_empty() {
            if let Some(content) = parse_non_stream_chat_response(&raw_response) {
                on_token(&content);
                return Ok(content);
            }

            return Err(AppError::Parse(
                "AI stream ended without text content / AI 流式响应没有文本内容".into(),
            ));
        }

        Ok(full_response)
    }

    /// Quick single-question helper — sends one user message, returns response.
    pub async fn ask(&self, system_prompt: &str, user_question: &str) -> Result<String> {
        let messages = vec![ChatMessage::system(system_prompt), ChatMessage::user(user_question)];
        self.chat(&messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AiConfig, ChatResponse, collect_stream_tokens, extract_chat_content,
        parse_non_stream_chat_response, sanitize_provider_error_body,
    };

    fn token_line(content: &str) -> String {
        format!("data: {{\"choices\":[{{\"delta\":{{\"content\":\"{}\"}}}}]}}", content)
    }

    #[test]
    fn stream_parser_handles_fragmented_chunks() {
        let mut buffer = String::from("data: {\"choices\":[{\"delta\":{\"content\":\"Hel");
        let (tokens, done) = collect_stream_tokens(&mut buffer, false);

        assert!(tokens.is_empty());
        assert!(!done);

        buffer.push_str("lo\"}}]}\n");
        buffer.push_str("data: [DONE]\n");

        let (tokens, done) = collect_stream_tokens(&mut buffer, false);
        assert_eq!(tokens, vec!["Hello"]);
        assert!(done);
        assert!(buffer.is_empty());
    }

    #[test]
    fn stream_parser_flushes_unterminated_tail_on_eof() {
        let mut buffer = token_line("TailToken");

        let (tokens, done) = collect_stream_tokens(&mut buffer, false);
        assert!(tokens.is_empty());
        assert!(!done);

        let (tokens, done) = collect_stream_tokens(&mut buffer, true);
        assert_eq!(tokens, vec!["TailToken"]);
        assert!(!done);
        assert!(buffer.is_empty());
    }

    #[test]
    fn stream_parser_ignores_non_data_lines_and_stops_after_done() {
        let mut buffer = String::from(
            "event: ping\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"A\"}}]}\ndata: [DONE]\ndata: {\"choices\":[{\"delta\":{\"content\":\"B\"}}]}\n",
        );

        let (tokens, done) = collect_stream_tokens(&mut buffer, false);
        assert_eq!(tokens, vec!["A"]);
        assert!(done);
        assert!(buffer.is_empty());
    }

    #[test]
    fn config_requires_endpoint_key_and_model() {
        let config = AiConfig {
            endpoint: "https://example.com/v1".into(),
            api_key: " ".into(),
            model: "model".into(),
            temperature: 0.2,
            max_tokens: 128,
        };

        assert!(!config.is_configured());
    }

    #[test]
    fn provider_error_body_is_redacted_and_truncated() {
        let sk_like = ["sk", "secret1234567890"].join("-");
        let tp_like = ["tp", "secret1234567890"].join("-");
        let body = format!("Authorization: Bearer {sk_like} token {tp_like} {}", "x".repeat(900));
        let sanitized = sanitize_provider_error_body(&body);

        assert!(sanitized.contains("[REDACTED"));
        assert!(!sanitized.contains(&sk_like));
        assert!(!sanitized.contains(&tp_like));
        assert!(sanitized.len() < body.len());
    }

    #[test]
    fn non_stream_json_response_can_backstop_streaming_clients() {
        let raw = r#"{"choices":[{"message":{"content":"Fallback content"}}]}"#;

        assert_eq!(parse_non_stream_chat_response(raw).as_deref(), Some("Fallback content"));
    }

    #[test]
    fn empty_chat_response_is_parse_error() {
        let response: ChatResponse = serde_json::from_str(r#"{"choices":[]}"#).unwrap();

        assert!(extract_chat_content(response).is_err());
    }
}
