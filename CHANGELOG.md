# Changelog

English | [简体中文](changelog-cn.md)

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Rebranded the project to AI-Movie-Player with subtle ifq.ai attribution.
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
