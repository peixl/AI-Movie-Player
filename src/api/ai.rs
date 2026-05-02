//! OpenAI-compatible AI client.
//! Supports OpenAI, Azure OpenAI, Ollama, LM Studio, and any OpenAI-compatible API.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::util::error::{AppError, Result};

const DEFAULT_ENDPOINT: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";

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
        !self.api_key.is_empty()
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

/// AI client with OpenAI-compatible API
pub struct AiClient {
    config: AiConfig,
    client: reqwest::Client,
    rate_limiter: Arc<Semaphore>,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        Self { config, client: reqwest::Client::new(), rate_limiter: Arc::new(Semaphore::new(10)) }
    }

    pub fn config(&self) -> &AiConfig {
        &self.config
    }

    pub fn is_ready(&self) -> bool {
        self.config.is_configured()
    }

    /// Send a chat completion request and get the full response.
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String> {
        if !self.is_ready() {
            return Err(AppError::Config("AI API key not configured / 未配置 AI API Key".into()));
        }

        let _permit = self.rate_limiter.acquire().await.expect("AI rate limiter semaphore closed");

        let req = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: None,
        };

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.config.endpoint.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Config(format!(
                "AI API error / AI 接口错误 ({}): {}",
                status,
                if body.len() > 200 { &body[..200] } else { &body }
            )));
        }

        let data: ChatResponse = resp.json().await?;
        let content = data
            .choices
            .first()
            .and_then(|c| c.message.content.as_deref())
            .unwrap_or_default()
            .to_string();

        Ok(content)
    }

    /// Stream a chat completion, yielding tokens as they arrive.
    pub async fn chat_stream(
        &self,
        messages: &[ChatMessage],
        mut on_token: impl FnMut(&str),
    ) -> Result<String> {
        if !self.is_ready() {
            return Err(AppError::Config("AI API key not configured / 未配置 AI API Key".into()));
        }

        let _permit = self.rate_limiter.acquire().await.expect("AI rate limiter semaphore closed");

        let req = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            stream: Some(true),
        };

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.config.endpoint.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Config(format!(
                "AI API error / AI 接口错误 ({})：{}",
                status, body
            )));
        }

        let mut full_response = String::new();
        let mut buffer = String::new();
        let mut stream = resp.bytes_stream();
        let mut saw_done = false;

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

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
    use super::collect_stream_tokens;

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
}
