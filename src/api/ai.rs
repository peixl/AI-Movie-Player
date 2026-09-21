//! OpenAI-compatible AI client.
//! Supports OpenAI, Azure OpenAI, Ollama, LM Studio, and any OpenAI-compatible API.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::Semaphore;

use crate::api::http;
use crate::util::error::{AppError, Result};

const DEFAULT_ENDPOINT: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const NON_STREAM_TIMEOUT: Duration = Duration::from_secs(120);
/// Maximum quiet period between streamed tokens before giving up. Long answers
/// are fine as long as tokens keep arriving; a stalled connection is not.
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_ATTEMPTS: u32 = 3;
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

/// Incremental UTF-8 decoder for byte streams.
///
/// Network chunks can split a multibyte character (common with CJK tokens).
/// Decoding each chunk independently with `from_utf8_lossy` injects permanent
/// replacement characters; this decoder carries the incomplete byte tail into
/// the next chunk and only substitutes genuinely invalid sequences.
#[derive(Default)]
struct Utf8Chunker {
    pending: Vec<u8>,
}

impl Utf8Chunker {
    fn new() -> Self {
        Self::default()
    }

    /// Append raw bytes and return the text that is now fully decodable.
    fn push(&mut self, bytes: &[u8]) -> String {
        self.pending.extend_from_slice(bytes);
        let mut decoded = String::new();

        loop {
            match std::str::from_utf8(&self.pending) {
                Ok(text) => {
                    decoded.push_str(text);
                    self.pending.clear();
                    break;
                }
                Err(error) => {
                    let valid_up_to = error.valid_up_to();
                    decoded.push_str(&String::from_utf8_lossy(&self.pending[..valid_up_to]));

                    match error.error_len() {
                        // Incomplete multibyte sequence at the boundary: keep
                        // it for the next chunk.
                        None => {
                            self.pending = self.pending[valid_up_to..].to_vec();
                            break;
                        }
                        // Genuinely invalid bytes: emit one replacement marker,
                        // drop the bad sequence, and keep decoding the valid tail
                        // within the same chunk so streams resynchronize at once.
                        Some(invalid_len) => {
                            decoded.push('\u{FFFD}');
                            let resume = valid_up_to + invalid_len;
                            self.pending = self.pending[resume.min(self.pending.len())..].to_vec();
                        }
                    }
                }
            }
        }

        decoded
    }
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
    /// Client with an overall request timeout, used for non-streaming calls.
    client: reqwest::Client,
    /// Client with only a connect timeout; stream liveness is enforced via an
    /// idle gap timeout so long generations never hit a total-time budget.
    stream_client: reqwest::Client,
    rate_limiter: Arc<Semaphore>,
}

fn build_client(overall_timeout: Option<Duration>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(4)
        .redirect(reqwest::redirect::Policy::limited(5))
        .gzip(true)
        .brotli(true)
        .user_agent(http::APP_USER_AGENT);
    if let Some(timeout) = overall_timeout {
        builder = builder.timeout(timeout);
    }
    builder.build().unwrap_or_else(|err| {
        log::warn!("Failed to build configured AI HTTP client: {err}");
        reqwest::Client::new()
    })
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        let client = build_client(Some(NON_STREAM_TIMEOUT));
        let stream_client = build_client(None);

        Self { config, client, stream_client, rate_limiter: Arc::new(Semaphore::new(10)) }
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

    fn build_request_body(&self, messages: &[ChatMessage], stream: Option<bool>) -> ChatRequest {
        ChatRequest {
            model: self.config.model.trim().to_string(),
            messages: messages.to_vec(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream,
        }
    }

    fn apply_headers(
        &self,
        request: reqwest::RequestBuilder,
        accept: &str,
    ) -> reqwest::RequestBuilder {
        request
            .header("Authorization", format!("Bearer {}", self.config.api_key.trim()))
            .header("Content-Type", "application/json")
            .header("Accept", accept)
    }

    /// Send a chat completion request and get the full response.
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String> {
        self.validate_ready()?;

        let _permit = self.rate_limiter.acquire().await.expect("AI rate limiter semaphore closed");

        let mut last_error: Option<AppError> = None;

        for attempt in 0..MAX_ATTEMPTS {
            let req = self.build_request_body(messages, None);
            let response = self
                .apply_headers(
                    self.client.post(self.chat_completions_url()).json(&req),
                    "application/json",
                )
                .send()
                .await;

            let resp = match response {
                Ok(resp) => resp,
                Err(err) => {
                    if http::is_retryable_network(&err) && attempt + 1 < MAX_ATTEMPTS {
                        let jitter = http::backoff_delay(attempt, None, attempt as f64 / 3.0);
                        tokio::time::sleep(jitter).await;
                        continue;
                    }
                    return Err(AppError::Network(err));
                }
            };

            let status = resp.status();
            if !status.is_success() {
                let code = status.as_u16();
                let body = resp.text().await.unwrap_or_default();
                let retryable = http::is_retryable_status(code);
                if retryable && attempt + 1 < MAX_ATTEMPTS {
                    last_error = Some(ai_api_error(code, &body));
                    let jitter = http::backoff_delay(attempt, None, attempt as f64 / 3.0);
                    tokio::time::sleep(jitter).await;
                    continue;
                }
                return Err(ai_api_error(code, &body));
            }

            let data: ChatResponse = resp.json().await?;
            return extract_chat_content(data);
        }

        Err(last_error.unwrap_or_else(|| {
            AppError::Config("AI request failed after retries / AI 请求重试后仍失败".into())
        }))
    }

    /// Stream a chat completion, yielding tokens as they arrive.
    pub async fn chat_stream(
        &self,
        messages: &[ChatMessage],
        mut on_token: impl FnMut(&str),
    ) -> Result<String> {
        self.validate_ready()?;

        let _permit = self.rate_limiter.acquire().await.expect("AI rate limiter semaphore closed");

        let req = self.build_request_body(messages, Some(true));

        let resp = self
            .apply_headers(
                self.stream_client.post(self.chat_completions_url()).json(&req),
                "text/event-stream, application/json",
            )
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
        let mut decoder = Utf8Chunker::new();

        use futures_util::StreamExt;
        loop {
            let next = tokio::time::timeout(STREAM_IDLE_TIMEOUT, stream.next()).await;
            let Some(chunk_result) = next.map_err(|_| {
                AppError::Timeout(format!(
                    "AI stream idle for over {}s / AI 流式响应空闲超时",
                    STREAM_IDLE_TIMEOUT.as_secs()
                ))
            })?
            else {
                break; // Stream ended.
            };

            let chunk = chunk_result?;
            let chunk_text = decoder.push(&chunk);
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

    #[test]
    fn utf8_chunker_reassembles_multibyte_chars_split_across_chunks() {
        // "你好" in UTF-8 is 6 bytes: E4 BD A0 E5 A5 BD
        let full = "你好，world";
        let bytes = full.as_bytes();
        let mut decoder = super::Utf8Chunker::new();
        let mut assembled = String::new();

        // Feed one byte at a time to guarantee every multibyte sequence is split.
        for byte in bytes {
            assembled.push_str(&decoder.push(std::slice::from_ref(byte)));
        }

        assert_eq!(assembled, full);
    }

    #[test]
    fn utf8_chunker_replaces_invalid_sequences_and_resynchronizes() {
        let mut decoder = super::Utf8Chunker::new();
        let mut out = String::new();
        out.push_str(&decoder.push(&[0xFF, b'o', b'k']));
        assert!(out.contains('\u{FFFD}'));
        assert!(out.ends_with("ok"));
    }

    #[test]
    fn multibyte_token_split_between_sse_chunks_still_parses() {
        let token_line = "data: {\"choices\":[{\"delta\":{\"content\":\"电影\"}}]}\n";
        let bytes = token_line.as_bytes();
        let mut decoder = super::Utf8Chunker::new();
        let mut buffer = String::new();
        let mut tokens = Vec::new();

        // Split at every possible byte offset.
        for split in 1..bytes.len() {
            let (left, right) = bytes.split_at(split);
            buffer.push_str(&decoder.push(left));
            let (parsed, _) = collect_stream_tokens(&mut buffer, false);
            tokens.extend(parsed);
            buffer.push_str(&decoder.push(right));
            let (parsed, done) = collect_stream_tokens(&mut buffer, true);
            tokens.extend(parsed);
            assert!(!done);
            decoder = super::Utf8Chunker::new();
            buffer.clear();
            tokens.clear();
        }

        // A straightforward split in the middle must preserve the CJK token.
        let mut decoder = super::Utf8Chunker::new();
        let mut buffer = String::new();
        let mid = bytes.len() / 2;
        buffer.push_str(&decoder.push(&bytes[..mid]));
        buffer.push_str(&decoder.push(&bytes[mid..]));
        let (tokens, _) = collect_stream_tokens(&mut buffer, true);
        assert_eq!(tokens, vec!["电影".to_string()]);
    }
}
