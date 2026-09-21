//! External API clients for metadata, AI, and subtitle services.
//!
//! - [`http`]: shared client construction, timeouts, and retry/backoff.
//! - [`ai`]: OpenAI-compatible streaming chat client (supports OpenAI, Azure, Ollama, LM Studio).
//! - [`tmdb`]: TMDB API v3 client for movie metadata, posters, and cast information.

pub mod ai;
pub mod http;
pub mod tmdb;
