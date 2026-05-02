# Contributing to AI-Movie-Player

English | [简体中文](contributing-cn.md)

Contributions are welcome across product design, AI workflows, UI polish, documentation, and engineering quality. AI-Movie-Player is not trying to become a noisy feature pile. Good contributions make the app feel clearer, calmer, more useful, and more trustworthy.

## Getting Started

```bash
git clone https://github.com/peixl/AI-Movie-Player.git
cd AI-Movie-Player
cargo run
```

## Development Workflow

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

If your environment cannot reach crates.io, prefer local diagnostics or offline caches where possible and mention the limitation in your validation notes.

## Engineering Standards

- Avoid `unwrap()` in production-facing paths unless failure is truly impossible and documented.
- Prefer `Result`-based flows and coherent error surfaces over silent failure.
- Compile regex patterns once and reuse them.
- Keep async or blocking work off the UI thread.
- Keep database access parameterized and explicit.
- Avoid unnecessary rendering work in egui panels, especially on large libraries.
- Keep new APIs and UI flows minimal, legible, and easy to extend.

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
- If you changed AI prompts or UX copy, describe the intended user-facing effect in the PR.

## Pull Requests

1. Create a focused branch.
2. Keep the diff intentional and avoid unrelated cleanup.
3. Add or update tests when behavior changes.
4. Explain the user-facing impact clearly.
5. Reference any related issue, workflow idea, or design discussion.
6. Request review only after local checks are complete.

## Issues and Discussions

- Use the repository issue templates for bugs, feature requests, and AI workflow ideas.
- Use GitHub Discussions when the topic is exploratory, comparative, or not yet implementation-ready.

## Product Direction

AI-Movie-Player aims for quiet premium quality. When changing UI, copy, or AI behavior, prefer what feels more natural, restrained, and trustworthy.