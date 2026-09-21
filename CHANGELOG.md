# Changelog

English | [简体中文](changelog-cn.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Shared resilient HTTP layer (`src/api/http.rs`) with per-profile timeouts (API/image/download), connection pooling, gzip/brotli, redirect limits, a branded user agent, exponential backoff with jitter and `Retry-After` support, and retryable-status/network classification.
- Fuzzy TMDB matcher (`src/core/matcher.rs`) combining normalized title similarity (token Dice, containment, Damerau–Levenshtein), year distance, and a popularity tiebreaker, with explicit confidence thresholds so wrong first-result matches are rejected.
- Honest fallback import: files that cannot be matched to TMDB are still added as minimal records instead of being silently dropped.
- Content-hash duplicate detection (head/tail file hash) in addition to path-based dedup, with bounded-concurrency batch import (6 workers) and a live progress report (imported / skipped / unmatched / failed).
- Sample, trailer, extras, featurette, interview, deleted-scene, and hidden-directory filtering during folder scans, plus a bounded scan depth and additional video extensions.
- OpenSubtitles official REST API (`api.opensubtitles.com/api/v1`) integration with API-key configuration, tmdb/imdb/query search, BCP-47 language mapping, quality ranking (language preference, rating, downloads, HD; AI-translated/HI deprioritized), and per-file download.
- Subtitle archive safety: zip-slip path traversal protection, best-entry selection (srt > ass > vtt …), UTF-8/UTF-16 BOM and GBK decoding, and smart same-basename installation next to the movie for player auto-loading.
- Detached cross-platform playback (`src/core/playback.rs`): "play with default player" and "reveal in file manager" never block the UI, with Linux `xdg-open`/`gio` fallback.
- Movie detail actions to reveal a file in its folder and relocate a missing file; the UI now warns when the stored file is gone and disables playback.
- Working import wizard with live progress bar, per-file status, and a completion summary; folder pickers open once per request instead of looping after cancel.
- Working subtitle panel: search results (language, source, rating, downloads, HD/AI/HI badges), inline downloads with re-entry protection, and a settings shortcut when no API key is configured.
- Async poster loading: wall decoding runs on background threads with an LRU texture cache and skeleton placeholders; detail posters use the egui image loader; thumbnail writes are atomic (temp + rename).
- Adaptive repaint scheduling (fast while work is in flight, idle otherwise), Ctrl+F/Cmd+F search focus, a real 256px procedural app icon, and SQLite `busy_timeout`/WAL-friendly pragmas.
- Quick demo path in README to show the import, metadata, movie detail, AI briefing, and double-feature flow.
- AI grounding examples that explain which movie and library context each AI workflow should use.
- Starter issue pack for `good first issue` and `help wanted` roadmap tasks.

### Changed
- TMDB client uses separate HTTP clients and rate limiters for API calls vs image CDN traffic, fetches primary-language and zh-CN details concurrently, and validates poster responses (content type, empty body).
- AI streaming uses a connect-only timeout plus a per-chunk idle timeout, so long generations are no longer killed by a fixed total timeout; non-streaming chat retries with backoff.
- SSE stream decoding uses an incremental UTF-8 decoder instead of per-chunk `from_utf8_lossy`, eliminating spurious replacement characters when multibyte tokens are split across chunks.
- TMDB client is shared via `Arc<TmdbClient>` instead of a global mutex that serialized all requests.
- Settings changes rebuild network clients in place; subtitle downloads record rows through an isolated database connection.
- Updated launch documentation to treat screenshots, demo media, and visible starter issues as the remaining community-readiness gaps after v0.2.1.
- Clarified README release wording now that native v0.2.1 assets and checksums are available.

### Fixed
- Import wizard never completed: TMDB search results and subtitle search results spawned in background tasks were discarded, leaving the wizard stuck on the searching state.
- Short or missing release dates could panic the TMDB parser (`&date[..4]`); date parsing is now bounds-safe.
- Folder dialog could re-open every frame after being cancelled.
- Poster image decoding on the UI thread could stutter large libraries; it is now off-thread.
- Removed per-view-switch toast noise and the permanent 500 ms repaint wakeup.

### Security
- Fixed a zip-slip vulnerability in subtitle archive extraction that allowed path traversal via crafted member names.
- Subtitle downloads require a user-configured OpenSubtitles API key; no third-party key is bundled, and provider error bodies are redacted/truncated before display.

## [0.2.1] - 2026-05-04

### Added
- Movie detail action to open a stored local file with the operating system's default player.
- GitHub Packages publishing workflow for the GHCR package `ghcr.io/peixl/ai-movie-player`.
- README status note that clarifies the current playback model and beta maturity.
- Guided AI viewing workflows for pre-watch briefing, post-watch recap, and double-feature pairing.
- GitHub issue templates for bugs, feature requests, and AI workflow ideas.
- GitHub launch kit documentation covering repo description, topics, release copy, social preview direction, and screenshot planning.
- Dedicated Chinese companion files for contribution and changelog documentation.
- Security policy (SECURITY.md).
- PR template with checklist.
- Auto-labeler workflow for PRs based on file paths.
- Stale issue/PR management workflow.
- CODEOWNERS file for code ownership definitions.
- Linux release packaging with SHA256 checksums.
- Documentation build verification in CI.
- `rustfmt.toml` and `clippy.toml` for consistent code style.
- Architecture diagram in README (mermaid).
- Tech Stack section in README.
- Acknowledgments section in README.

### Changed
- Grouped future Dependabot updates to reduce PR churn and keep maintenance signals cleaner.
- Repositioned public copy around a local movie library companion with system-player launch while embedded playback remains a roadmap item.
- Clarified that release binaries depend on published GitHub release assets and source builds remain the current fallback.
- Expanded security notes with a system credential storage roadmap and safer key-handling guidance.
- Expanded the README with screenshot planning, feature comparison, roadmap, FAQ, and stronger homepage-facing structure.
- Refined the repository documentation strategy so top-level reader docs are split by language instead of mixed into one file.
- Improved project structure tree with file descriptions.
- Enhanced CONTRIBUTING.md with architecture overview, commit convention, and branch naming.
- CI workflow now includes concurrency groups, documentation build, and dependency ordering.
- Release workflow now produces SHA256 checksums for all artifacts.
- Cargo.toml now includes full metadata: homepage, repository, license, keywords, categories, authors.
- Upgraded the grouped Cargo dependency set, including eframe/egui, reqwest, rusqlite, rfd, zip, scraper, toml, and sha2.

### Fixed
- Chinese UI labels could render as square glyphs when egui's default fonts lacked CJK coverage.
- Compatibility fixes for reqwest 0.13 query support and sha2 0.11 digest formatting.

## [0.2.0] - 2026-05-02

### Added
- AI Companion for streaming chat and deeper film analysis around a selected movie.
- AI recommendation flows for library picks, discovery, and taste profiling.
- AI review entry points from the detail page.
- OpenAI-compatible API support for OpenAI, Azure, Ollama, LM Studio, and similar providers.
- Procedural icon system.
- Animation utilities including hover, shadow, toast, pulse, and shimmer.
- Keyboard shortcuts for core navigation.
- Toast notifications.
- Confirmation dialog component.
- Database and filename parser test coverage.
- Cross-platform GitHub Actions CI.

### Changed
- Rebranded the project to AI Movie Player with subtle ifq.ai attribution.
- Updated core UI copy to a bilingual Chinese-English style where it improves usability.
- Upgraded AI prompts for more natural, elegant, and context-aware answers.
- Enabled real multi-turn chat context.
- Preserved legacy data directory and database compatibility.

### Fixed
- Remaining old-brand strings in core product surfaces.
- Subtitle downloader user-agent branding.
- Settings panel copy consistency.

## [0.1.0] - 2026-04-28

### Added
- Initial library management.
- SQLite plus FTS5 full-text search.
- TMDB API v3 integration.
- Subtitle search.
- Poster wall browsing.
- Folder scanning and filename parsing.
- Dark and light themes.
- Watchlist, settings, and batch operations.

[Unreleased]: https://github.com/peixl/AI-Movie-Player/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/peixl/AI-Movie-Player/releases/tag/v0.2.1
[0.2.0]: https://github.com/peixl/AI-Movie-Player/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/peixl/AI-Movie-Player/releases/tag/v0.1.0
