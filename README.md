# AI-Movie-Player

An AI-native movie player for people who care about cinema, not just files.

English | [简体中文](readme-cn.md)

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/peixl/AI-Movie-Player/releases)
[![CI](https://github.com/peixl/AI-Movie-Player/actions/workflows/ci.yml/badge.svg)](https://github.com/peixl/AI-Movie-Player/actions/workflows/ci.yml)
[![AI](https://img.shields.io/badge/AI-OpenAI%20Compatible-6366f1.svg)](https://github.com/peixl/AI-Movie-Player#ai-features)

Built by [ifq.ai](https://ifq.ai) and open sourced at [peixl/AI-Movie-Player](https://github.com/peixl/AI-Movie-Player).

## Overview

AI-Movie-Player is a desktop movie player and library companion built with Rust and egui. It combines local library management, TMDB metadata, subtitles, poster-wall browsing, and OpenAI-compatible AI features in one quiet, cinema-first experience.

This project is designed to feel more like a thoughtful film tool than a generic media utility. The AI is there to help you choose, understand, and revisit films naturally, not to dominate the product.

## Why AI-Movie-Player

- AI companion chat with real multi-turn memory for a selected film.
- Taste-aware recommendations generated from your own library.
- AI quick insight and review flows directly from movie details.
- TMDB-powered metadata enrichment for titles, cast, directors, ratings, and posters.
- Subtitle discovery and download workflow for local collections.
- Poster-wall browsing, batch import, watchlist, and settings in a native desktop app.
- Subtle ifq.ai authorship instead of heavy-handed branding.

## AI Features

### AI Companion

Select a movie and talk to the AI with actual movie context: title, year, director, genres, synopsis, and cast. The chat now keeps conversation history, so follow-up questions feel coherent instead of stateless.

Good prompts include:

- deep analysis
- ending interpretation
- similar films
- production trivia
- watch companion brief
- honest should-I-watch-it verdicts

### AI Taste Engine

The recommendation flow looks at your own library and suggests what to watch next, why it fits your taste, where your blind spots are, and which outside films you are likely to love.

### AI Review

From the movie detail view, you can jump into AI insight for a compact review, audience fit, strengths, weaknesses, and viewing guidance.

### OpenAI-Compatible Providers

AI-Movie-Player works with:

- OpenAI
- Azure OpenAI
- Ollama
- LM Studio
- any OpenAI-compatible endpoint

### Guided Viewing Workflows

AI-Movie-Player now exposes a more deliberate viewing loop around a selected film:

- Pre-Watch Briefing to frame mood, context, and what details matter before playback.
- Post-Watch Recap to help the viewer process meaning, structure, and memorable choices.
- Double Feature Pairing to recommend a second film that deepens the first instead of merely resembling it.

## Core Product Areas

| Area | What it does |
| --- | --- |
| Library | Scan folders, detect movie files, avoid duplicate imports, and organize a personal collection. |
| Metadata | Enrich local media with TMDB titles, posters, cast, ratings, and synopsis. |
| AI | Provide movie chat, quick insight, taste profiling, and recommendation workflows. |
| Subtitles | Search and download subtitles from multiple sources for local playback. |
| Poster Wall | Browse visually with cached posters and a cleaner discovery flow. |
| Watchlist | Keep track of what you want to watch next. |

## Screenshot Plan

Recommended homepage screenshot sequence:

| Slot | What to capture | Why it matters |
| --- | --- | --- |
| 1 | Poster wall with a rich local library | Establishes the product as a serious desktop movie tool. |
| 2 | AI Companion with a selected film | Shows the product is AI-native, not AI-badged. |
| 3 | AI Taste Engine recommendations | Demonstrates library-aware intelligence. |
| 4 | Movie detail page with metadata and AI entry | Connects browsing, metadata, and AI in one flow. |
| 5 | Subtitle search and download workflow | Proves the app solves practical viewing problems too. |

Detailed capture guidance lives in [docs/github-launch-kit.md](docs/github-launch-kit.md).

## Compared with a Typical Player

| Capability | Typical Player | AI-Movie-Player |
| --- | --- | --- |
| Open a file | Yes | Yes |
| TMDB metadata enrichment | Sometimes | Built in |
| AI conversation about a selected film | Rare | Native workflow |
| Library-aware recommendations | Rare | Built in |
| Guided viewing workflows | No | Pre-watch, post-watch, and double-feature flows |
| Subtitle workflow | Basic | Search and download oriented |
| Product tone | Utility-first | Cinema-first, quietly premium |

## Getting Started

### Requirements

- Rust 1.85+
- A TMDB API key from [themoviedb.org/settings/api](https://www.themoviedb.org/settings/api)
- Optional: any OpenAI-compatible API key

### Clone and Run

```bash
git clone https://github.com/peixl/AI-Movie-Player.git
cd AI-Movie-Player
cargo run --release
```

### Releases

Prebuilt Windows and macOS packages can be published on the [Releases](https://github.com/peixl/AI-Movie-Player/releases) page.

The repository now includes a dedicated release workflow for the two primary desktop targets:

- Windows package: `.zip`
- macOS package: `.tar.gz` containing an `.app` bundle

The first formal release copy pack lives in [docs/release-package-v0.2.1.md](docs/release-package-v0.2.1.md).

For local packaging after dependencies are available:

- Windows: `pwsh ./scripts/package-windows.ps1`
- macOS: `bash ./scripts/package-macos.sh`

## AI Setup

1. Open Settings.
2. Enter your AI endpoint, API key, and model.
3. Save the configuration.
4. Use a local preset if you prefer Ollama or LM Studio.

Common endpoint examples:

```text
OpenAI    -> https://api.openai.com/v1
Ollama    -> http://localhost:11434/v1
LM Studio -> http://localhost:1234/v1
```

## Keyboard Shortcuts

| Shortcut | Action |
| --- | --- |
| Ctrl+1 | Library |
| Ctrl+2 | Import Movies |
| Ctrl+3 | Subtitle Search |
| Ctrl+4 | Batch Operations |
| Ctrl+5 | Watchlist |
| Ctrl+6 | Settings |
| Ctrl+7 | AI Companion |
| Ctrl+8 | AI Taste Engine |
| Ctrl+F | Search library |
| Esc | Back |

## Project Structure

```text
ai-movie-player/
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── ai/
│   ├── api/
│   ├── core/
│   ├── db/
│   ├── thumbnail/
│   ├── ui/
│   ├── config/
│   └── util/
├── README.md
├── readme-cn.md
├── CONTRIBUTING.md
└── CHANGELOG.md
```

## Development

```bash
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
cargo build --release
```

If your environment cannot reach crates.io, use local diagnostics or offline caches for validation when possible.

## Documentation

- English: README.md
- Chinese: [readme-cn.md](readme-cn.md)
- Contribution guide: [CONTRIBUTING.md](CONTRIBUTING.md)
- 中文贡献指南: [contributing-cn.md](contributing-cn.md)
- Release history: [CHANGELOG.md](CHANGELOG.md)
- 中文更新日志: [changelog-cn.md](changelog-cn.md)
- GitHub launch kit: [docs/github-launch-kit.md](docs/github-launch-kit.md)
- GitHub 中文发布说明: [docs/github-launch-kit-cn.md](docs/github-launch-kit-cn.md)
- Release package draft: [docs/release-package-v0.2.1.md](docs/release-package-v0.2.1.md)

## Roadmap

- Deepen the AI viewing workflows so they feel like part of watching, not just chat after the fact.
- Improve subtitle quality ranking and source reliability.
- Expand poster-wall scale, speed, and polish for larger personal libraries.
- Strengthen release engineering and cross-platform packaging.
- Add better onboarding for local AI providers and self-hosted endpoints.

## FAQ

### Is this a streaming app?

No. AI-Movie-Player is designed around local movie libraries and personal media workflows.

### Do I need an AI API key to use the app?

No. Core library, poster, metadata, and subtitle workflows can still exist without AI. AI features require an OpenAI-compatible provider.

### Which AI provider is best?

That depends on your workflow. OpenAI is the easiest cloud starting point; Ollama and LM Studio are strong local options.

### Why does the product mention ifq.ai so quietly?

Because the project treats ifq.ai as authorship, not as intrusive promotion. The product should feel composed first and branded second.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for workflow, standards, and pull request expectations.

## License

MIT. See [LICENSE](LICENSE).

Made with care by [ifq.ai](https://ifq.ai).