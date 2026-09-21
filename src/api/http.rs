//! Shared HTTP infrastructure: client construction and resilient request execution.
//!
//! Every outbound request in the application goes through the profiles defined
//! here so that timeouts, connection pooling, compression, redirects, the user
//! agent, and transient-failure retries behave consistently across TMDB, the
//! image CDN, AI providers, and subtitle sources.

use std::future::Future;
use std::time::Duration;

use reqwest::{Client, RequestBuilder, StatusCode};

use crate::util::error::{AppError, Result};

/// Application-wide user agent. Some providers (notably subtitle services)
/// reject requests with generic or missing agents.
pub const APP_USER_AGENT: &str = concat!(
    "ai-movie-player/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/peixl/AI-Movie-Player)"
);

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const API_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const IMAGE_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const DOWNLOAD_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const RETRY_BASE: Duration = Duration::from_millis(400);
const RETRY_CAP: Duration = Duration::from_secs(8);

/// Kind of traffic a client is built for. Drives timeout and retry defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientProfile {
    /// JSON APIs (TMDB metadata, subtitle search): short timeouts, aggressive retries.
    Api,
    /// Image CDN traffic (posters, backdrops): larger timeout, modest retries.
    Image,
    /// File downloads (subtitle archives): the most generous timeout.
    Download,
}

impl ClientProfile {
    fn request_timeout(self) -> Duration {
        match self {
            ClientProfile::Api => API_REQUEST_TIMEOUT,
            ClientProfile::Image => IMAGE_REQUEST_TIMEOUT,
            ClientProfile::Download => DOWNLOAD_REQUEST_TIMEOUT,
        }
    }

    /// Default number of *attempts* (initial request + retries) for the profile.
    pub fn max_attempts(self) -> u32 {
        match self {
            ClientProfile::Api => 3,
            ClientProfile::Image | ClientProfile::Download => 2,
        }
    }
}

/// Build a [`reqwest::Client`] tuned for the given traffic profile.
///
/// Environment proxies (`HTTP_PROXY` / `HTTPS_PROXY`) are honored by reqwest's
/// default TLS backend, which matters for users behind corporate or regional
/// networks.
pub fn build_client(profile: ClientProfile) -> Client {
    let builder = Client::builder()
        .user_agent(APP_USER_AGENT)
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .timeout(profile.request_timeout())
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(8)
        .redirect(reqwest::redirect::Policy::limited(5))
        .gzip(true)
        .brotli(true);

    builder.build().unwrap_or_else(|err| {
        log::warn!("Failed to build HTTP client ({profile:?}): {err}; falling back to default");
        Client::new()
    })
}

/// Whether an HTTP status is worth retrying after a short backoff.
///
/// 429 (rate limited) and the 5xx family indicate transient server/network
/// conditions; other 4xx responses are deterministic failures.
pub fn is_retryable_status(status: u16) -> bool {
    matches!(status, 408 | 425 | 429 | 500 | 502 | 503 | 504)
}

/// Whether a transport-level failure may succeed on retry.
///
/// Connection establishment and timeouts are transient; request-body errors
/// happen before bytes leave the process and are also safe to repeat.
pub fn is_retryable_network(err: &reqwest::Error) -> bool {
    err.is_connect() || err.is_timeout() || err.is_request()
}

/// Exponential backoff with deterministic jitter, optionally overridden by a
/// server-provided `Retry-After` value (seconds).
///
/// `jitter_01` must be in `[0.0, 1.0]`; production callers pass a random value,
/// tests pass a fixed value for reproducible assertions.
pub fn backoff_delay(attempt: u32, retry_after_secs: Option<u64>, jitter_01: f64) -> Duration {
    if let Some(secs) = retry_after_secs {
        let advised = Duration::from_secs(secs.clamp(0, 30));
        return advised.max(RETRY_BASE.min(RETRY_CAP));
    }

    let exp = RETRY_BASE.saturating_mul(2u32.saturating_pow(attempt.min(8)));
    let capped = exp.min(RETRY_CAP);
    // Spread waits over +/-20% to avoid synchronized retry storms.
    let factor = 0.8 + 0.4 * jitter_01.clamp(0.0, 1.0);
    let millis = (capped.as_millis() as f64 * factor) as u64;
    Duration::from_millis(millis.max(1))
}

/// Parse a numeric `Retry-After` header (delta-seconds form). HTTP-date form is
/// intentionally not supported — providers in this ecosystem use seconds.
pub fn retry_after_secs(resp: &reqwest::Response) -> Option<u64> {
    resp.headers().get(reqwest::header::RETRY_AFTER)?.to_str().ok()?.trim().parse().ok()
}

/// Turn a non-success response into a structured API error, consuming the body
/// with a bounded read so error messages stay cheap and readable.
pub async fn error_for_status(resp: reqwest::Response, context: &'static str) -> AppError {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    let body = body.chars().take(800).collect::<String>();

    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return AppError::Config(format!(
            "{context} rejected credentials (HTTP {status}). Check the API key / token. Body: {body}"
        ));
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        return AppError::Config(format!(
            "{context} rate limited the request (HTTP 429). Please retry shortly. Body: {body}"
        ));
    }

    AppError::Config(format!("{context} returned HTTP {status}: {body}"))
}

/// Execute a request with retry/backoff for transient failures.
///
/// The `build` closure reconstructs the request for each attempt (request
/// builders are not reusable). Only the transport + response-head phase is
/// retried; once a success status returns, body consumption is the caller's
/// responsibility.
pub async fn execute_with_retry<F>(
    profile: ClientProfile,
    client: &Client,
    build: F,
) -> Result<reqwest::Response>
where
    F: Fn(&Client) -> RequestBuilder,
{
    let max_attempts = profile.max_attempts();
    let mut attempt = 0u32;

    loop {
        let request = build(client);
        match send_once(request).await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return Ok(resp);
                }
                if attempt + 1 >= max_attempts || !is_retryable_status(status.as_u16()) {
                    return Err(error_for_status(resp, "HTTP request / HTTP 请求").await);
                }
                let retry_after = retry_after_secs(&resp);
                drop(resp);
                sleep_backoff(attempt, retry_after).await;
            }
            Err(err) => {
                if attempt + 1 >= max_attempts || !is_retryable_network(&err) {
                    return Err(AppError::Network(err));
                }
                log::warn!("Transient HTTP failure (attempt {}): {err}", attempt + 1);
                sleep_backoff(attempt, None).await;
            }
        }
        attempt += 1;
    }
}

async fn send_once(
    request: RequestBuilder,
) -> std::result::Result<reqwest::Response, reqwest::Error> {
    request.send().await
}

async fn sleep_backoff(attempt: u32, retry_after: Option<u64>) {
    let jitter = random_jitter();
    let delay = backoff_delay(attempt, retry_after, jitter);
    tokio::time::sleep(delay).await;
}

#[cfg(not(test))]
fn random_jitter() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    (nanos % 1000) as f64 / 1000.0
}

#[cfg(test)]
fn random_jitter() -> f64 {
    0.5
}

/// Convenience: GET a URL with retries and return the decoded JSON body.
pub async fn get_json_with_retry<T: serde::de::DeserializeOwned>(
    profile: ClientProfile,
    client: &Client,
    url: impl Into<String>,
) -> Result<T> {
    let url = url.into();
    let resp = execute_with_retry(profile, client, |c| c.get(&url)).await?;
    Ok(resp.json::<T>().await?)
}

/// Retry a typed operation future produced by `make_attempt` up to `max_attempts`
/// times whenever `is_transient` indicates a retryable error. Used for AI
/// requests whose builder lives outside this module.
pub async fn retry_operation<T, E, Mk, Fut, Pred>(
    max_attempts: u32,
    make_attempt: Mk,
    is_transient: Pred,
) -> std::result::Result<T, E>
where
    Mk: Fn(u32) -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
    Pred: Fn(&E) -> bool,
{
    let mut attempt = 0u32;
    loop {
        match make_attempt(attempt).await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if attempt + 1 >= max_attempts || !is_transient(&err) {
                    return Err(err);
                }
                let jitter = random_jitter();
                tokio::time::sleep(backoff_delay(attempt, None, jitter)).await;
                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_status_classification() {
        for status in [408u16, 425, 429, 500, 502, 503, 504] {
            assert!(is_retryable_status(status), "{status} should be retryable");
        }
        for status in [400, 401, 403, 404, 409, 422, 451] {
            assert!(!is_retryable_status(status), "{status} should not be retryable");
        }
    }

    #[test]
    fn backoff_grows_exponentially_and_caps() {
        let first = backoff_delay(0, None, 0.5);
        let second = backoff_delay(1, None, 0.5);
        let third = backoff_delay(2, None, 0.5);
        let capped = backoff_delay(10, None, 0.5);

        assert!(second > first);
        assert!(third > second);
        assert_eq!(capped, RETRY_CAP);
    }

    #[test]
    fn retry_after_takes_precedence_and_is_clamped() {
        let delay = backoff_delay(0, Some(3), 0.5);
        assert_eq!(delay, Duration::from_secs(3));

        let huge = backoff_delay(0, Some(3600), 0.5);
        assert_eq!(huge, Duration::from_secs(30));
    }

    #[test]
    fn jitter_is_clamped_to_safe_range() {
        let a = backoff_delay(0, None, -5.0);
        let b = backoff_delay(0, None, 50.0);
        assert!(a >= RETRY_BASE.mul_f64(0.79));
        assert!(b <= RETRY_CAP);
    }

    #[test]
    fn clients_build_for_every_profile() {
        let _api = build_client(ClientProfile::Api);
        let _image = build_client(ClientProfile::Image);
        let _download = build_client(ClientProfile::Download);
    }
}
