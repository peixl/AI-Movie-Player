# Contributing to AI-Movie-Player

English | [简体中文](contributing-cn.md)

Contributions are welcome across product design, AI workflows, UI polish, documentation, and engineering quality. AI-Movie-Player is not trying to become a noisy feature pile. Good contributions make the app feel clearer, calmer, more useful, and more trustworthy.

## Getting Started

### Prerequisites

- Rust 1.85+ (install via [rustup](https://rustup.rs))
- A TMDB API key from [themoviedb.org/settings/api](https://www.themoviedb.org/settings/api) (for metadata features)
- Optional: an OpenAI-compatible API key (for AI features)

### Setup

```bash
git clone https://github.com/peixl/AI-Movie-Player.git
cd AI-Movie-Player
cargo build --locked
```

### Running

```bash
cargo run --release
```

## Development Workflow

Before opening a PR, run the full check suite:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
cargo build --release --locked
```

If your environment cannot reach crates.io, prefer local diagnostics or offline caches where possible and mention the limitation in your validation notes.

For packaging changes, run the same scripts used by CI and release automation:

```bash
./scripts/package-macos.sh
./scripts/package-linux.sh
pwsh ./scripts/package-windows.ps1
```

## Architecture Overview

The codebase follows a layered architecture:

```text
UI Layer            -> src/app.rs, src/ui/
Workflow Layer      -> src/ai/, src/core/
Integration Layer   -> src/api/
Persistence Layer   -> src/db/, src/config/, src/thumbnail/
```

Key design decisions:
- **Immediate-mode UI**: egui re-renders every frame. Avoid expensive work in UI code.
- **Async with Tokio**: Network calls (AI, TMDB) run on the Tokio runtime, not the UI thread.
- **Streaming AI**: Server-Sent Events (SSE) parsing for OpenAI-compatible streaming responses.
- **Poster cache**: Poster and thumbnail assets are cached under src/thumbnail.
- **SQLite WAL mode**: Concurrent reads, single writer, FTS5 for full-text search.
- **Single packaging path**: CI and release jobs both call the scripts under scripts/ instead of duplicating packaging logic in YAML.

## Engineering Standards

- Avoid `unwrap()` in production-facing paths unless failure is truly impossible and documented.
- Prefer `Result`-based flows and coherent error surfaces over silent failure.
- Compile regex patterns once and reuse them.
- Keep async or blocking work off the UI thread.
- Keep database access parameterized and explicit.
- Avoid unnecessary rendering work in egui panels, especially on large libraries.
- Reuse existing packaging and validation scripts instead of reimplementing build logic inside workflows.
- Keep new APIs and UI flows minimal, legible, and easy to extend.

## Commit Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

```text
<type>(<scope>): <description>

[optional body]
[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding or correcting tests
- `chore`: Maintenance tasks

Examples:
```text
feat(ai): add taste-aware recommendation engine
fix(db): handle FTS5 migration for existing databases
docs(readme): add architecture diagram
perf(ui): cache poster textures across frames
```

## Branch Naming

Use descriptive branch names:

```text
feature/ai-double-feature-pairing
fix/subtitle-encoding-detection
docs/contributing-guide-update
refactor/extract-theme-helpers
```

## AI Contribution Standards

- Prompts should sound calm, elegant, specific, and useful.
- The AI should not invent facts when details are uncertain.
- Recommendations should be evidence-based rather than generic genre guessing.
- ifq.ai attribution should remain subtle, contextual, and non-promotional.
- New AI flows should feel like natural viewing workflows, not gimmicks.

## Documentation and Language

- Top-level reader docs are maintained as separate language files instead of mixed bilingual blocks.
- Keep `README.md`, `CONTRIBUTING.md`, and `CHANGELOG.md` as English-primary entry documents.
- Put Chinese counterparts in dedicated companion files such as `readme-cn.md`, `contributing-cn.md`, and `changelog-cn.md`.
- User-facing UI copy inside the app should remain bilingual where it improves usability.

## Testing Expectations

- Put unit tests close to the code they verify.
- Use isolated temp directories for database or filesystem tests.
- Run at least one focused validation step before opening a pull request.
- If you touch release engineering, validate the relevant package script locally or in CI artifacts.
- If you changed AI prompts or UX copy, describe the intended user-facing effect in the PR.

## Automation Expectations

- CI should stay reproducible: prefer locked Cargo commands and deterministic packaging.
- Keep GitHub Actions changes small and observable; one workflow should have one clear responsibility.
- Dependabot and dependency review are part of the maintenance baseline, not optional extras.

## Pull Requests

1. Create a focused branch from `main`.
2. Keep the diff intentional and avoid unrelated cleanup.
3. Add or update tests when behavior changes.
4. Explain the user-facing impact clearly.
5. Reference any related issue, workflow idea, or design discussion.
6. Request review only after local checks are complete.

Use the [PR template](.github/PULL_REQUEST_TEMPLATE.md) when opening a pull request.

## Issues and Discussions

- Use the repository issue templates for bugs, feature requests, and AI workflow ideas.
- Use GitHub Discussions when the topic is exploratory, comparative, or not yet implementation-ready.

## Good First Issues

Look for issues labeled [`good first issue`](https://github.com/peixl/AI-Movie-Player/labels/good%20first%20issue) to get started. These are scoped to be approachable for new contributors.

## Product Direction

AI-Movie-Player aims for quiet premium quality. When changing UI, copy, or AI behavior, prefer what feels more natural, restrained, and trustworthy.
