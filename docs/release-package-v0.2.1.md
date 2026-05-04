# Release Package v0.2.1

English | [简体中文](release-package-v0.2.1-cn.md)

This is the first formal GitHub Releases package for AI-Movie-Player.

## Release Title

`AI-Movie-Player v0.2.1`

## Release Body

```md
AI-Movie-Player is an AI-native local movie library companion, built to make metadata, subtitles, recommendations, system-player launch, and cinematic conversation feel natural, refined, and genuinely useful.

### Highlights
- Added guided AI viewing workflows: pre-watch briefing, post-watch recap, and double-feature pairing.
- Added local file launch from the movie detail page through the system default player.
- Fixed Chinese UI rendering by loading cross-platform CJK system font fallbacks.
- Upgraded AI Companion from prompt shortcuts into structured workflow cards inside a dedicated panel.
- Split top-level docs into English-first and Chinese companion files for README, contributing, changelog, and release guidance.
- Added GitHub issue templates and a cleaner launch kit for open-source collaboration.
- Added GitHub Packages publishing through `ghcr.io/peixl/ai-movie-player`.
- Upgraded the grouped Rust dependency set and fixed reqwest/sha2 compatibility changes.
- Improved bilingual wording across deeper in-app states, empty states, and status surfaces.

### Platform Packages
- Windows package: `AI-Movie-Player-v0.2.1-windows-x64.zip`
- macOS package: `AI-Movie-Player-v0.2.1-macOS-ARCH.tar.gz` containing `AI-Movie-Player.app`
- Linux package: `AI-Movie-Player-v0.2.1-linux-x86_64.tar.gz`
- GitHub Packages: `ghcr.io/peixl/ai-movie-player:v0.2.1`

### Upgrade Notes
- Existing users keep legacy app data and database compatibility automatically.
- AI features still require an OpenAI-compatible provider.
- This release is focused on workflow quality, product language, and release readiness rather than backend migration.

### Feedback
- Issues: https://github.com/peixl/AI-Movie-Player/issues
- Discussions: https://github.com/peixl/AI-Movie-Player/discussions

Made with care by ifq.ai.
```

## Screenshot Order

Use this order in the release page:

1. Poster wall with a realistic personal library.
2. AI Companion with a selected movie and multi-turn conversation.
3. Workflow Studio card view showing pre-watch briefing or post-watch recap.
4. AI Taste Engine recommendation screen.
5. Movie detail page with metadata, Open action, and AI entry.
6. Subtitle search flow.

## Upgrade Notes

Use this shorter upgrade block if you need a compact release note section:

```md
### Upgrade Notes
- Keeps legacy app data compatibility.
- Adds structured AI workflow cards.
- Fixes Chinese UI font rendering on all supported desktop platforms.
- Improves bilingual product copy and release surfaces.
- Publishes native Release assets and a GHCR package.
- Continues to support OpenAI-compatible providers including OpenAI, Ollama, and LM Studio.
```

## Announcement Copy

### Short Announcement

```text
AI-Movie-Player v0.2.1 is live.

This release deepens the product in three directions: structured AI viewing workflows, cleaner bilingual documentation, and a more polished GitHub launch surface.

Now included: pre-watch briefing, post-watch recap, double-feature pairing, local file launch through the system player, CJK font rendering fixes, Windows + macOS + Linux release packaging, and a GHCR package.

GitHub: https://github.com/peixl/AI-Movie-Player
by ifq.ai
```

### Longer Announcement

```text
AI-Movie-Player v0.2.1 is the first release that starts to feel like the product we actually want to build.

It is still an early local-library movie companion, but now the AI side is more deliberate: instead of only offering prompt shortcuts, it can generate structured viewing workflow cards for pre-watch framing, post-watch recap, and double-feature pairing. The detail page can also launch the stored local file through the system default player.

We also cleaned up the repository presentation with separate English and Chinese docs, issue templates, a launch kit, and clearer release packaging for Windows, macOS, and Linux.

The goal remains the same: a quietly premium AI-native local movie tool that feels composed, useful, and respectful to both users and developers.

Project: https://github.com/peixl/AI-Movie-Player
Made with care by ifq.ai
```