# Starter Issue Pack

English | [简体中文](starter-issues-cn.md)

This pack turns the roadmap into small, concrete issues that new contributors can pick up without needing private media files, API keys, or broad architectural context.

Use labels such as `good first issue`, `help wanted`, `documentation`, `testing`, `ai`, `metadata`, `subtitles`, `ui`, and `release` as appropriate.

## Recommended Seed Issues

| Issue title | Labels | Scope | Suggested validation |
| --- | --- | --- | --- |
| Add real screenshots to README and release notes | `good first issue`, `documentation`, `release` | Capture poster wall, movie detail, AI Companion, AI Taste Engine, and subtitle search with a small demo library. | Confirm image paths render in README preview and release draft. |
| Add filename parser test cases for common release naming patterns | `good first issue`, `testing`, `metadata` | Add focused tests for titles with year, resolution, source, codec, and punctuation variants. | `cargo test filename_parser --locked` |
| Document TMDB key onboarding with first-run screenshots | `good first issue`, `documentation` | Improve README/wiki setup flow for creating and saving a TMDB key. | Markdown preview plus link check. |
| Improve subtitle source ranking notes | `help wanted`, `subtitles`, `documentation` | Document how users should choose subtitles today and what data future ranking should consider. | Markdown preview. |
| Add provider diagnostics examples for Ollama and LM Studio | `good first issue`, `ai`, `documentation` | Add copy-paste curl checks for local OpenAI-compatible endpoints. | Run the documented commands against a local provider, or mark as docs-only. |
| Add tests for AI stream parsing edge cases | `help wanted`, `ai`, `testing` | Cover empty chunks, `[DONE]`, provider errors, and partial SSE lines. | `cargo test ai --locked` |
| Improve large-library poster wall smoke notes | `help wanted`, `ui`, `documentation` | Document what to observe when testing hundreds of local movies. | Manual checklist in wiki. |
| Add release checksum verification examples for Windows PowerShell | `good first issue`, `release`, `documentation` | Complement existing Linux/macOS checksum commands with a Windows example. | Verify the PowerShell command syntax. |
| Add accessibility pass for keyboard navigation labels | `help wanted`, `ui` | Review main navigation and common controls for readable labels and focus behavior. | Manual smoke notes plus screenshots if possible. |
| Draft system credential storage design notes | `help wanted`, `security`, `architecture` | Compare macOS Keychain, Windows Credential Manager, and Linux Secret Service constraints before implementation. | Design note reviewed by maintainers. |

## Issue Template

```md
## Goal

Describe the small user or maintainer improvement.

## Scope

- File or workflow likely to change.
- What should stay out of scope.

## Acceptance Criteria

- Concrete observable result.
- Documentation, test, or screenshot expectation.

## Validation

- Command or manual smoke check.
```

## Selection Rules

- A good first issue should touch one or two files.
- It should not require real API keys, private movie files, or a large local library.
- It should include a clear validation path.
- It should improve trust, setup clarity, test coverage, or a visible user workflow.
- Broader design questions belong in Discussions or `help wanted` issues, not `good first issue`.
