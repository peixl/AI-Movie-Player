//! External API clients for AI and metadata services.
//!
//! - [`ai`]: OpenAI-compatible streaming chat client (supports OpenAI, Azure, Ollama, LM Studio).
//! - [`tmdb`]: TMDB API v3 client for movie metadata, posters, and cast information.

pub mod ai;
pub mod tmdb;
