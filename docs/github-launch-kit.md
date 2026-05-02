# GitHub Launch Kit

English | [简体中文](github-launch-kit-cn.md)

This document is a maintainer-facing kit for polishing the public GitHub presentation of AI-Movie-Player.

## Recommended Repo Description

Use one of these for the GitHub About field:

- AI-native movie player for local libraries, cinematic chat, library-aware recommendations, metadata, subtitles, and poster-wall browsing.
- AI-native desktop movie player with TMDB metadata, subtitle workflow, poster wall, and OpenAI-compatible film intelligence.

## Suggested Topics

Recommended repository topics:

- movie-player
- media-player
- ai
- rust
- egui
- desktop-app
- tmdb
- subtitles
- movie-library
- recommendation-engine
- openai-compatible
- ollama
- lm-studio
- cinematic-ui
- local-first

## Social Preview Direction

Target a social preview image that feels premium, cinematic, and calm.

Recommended composition:

- Use a warm, restrained dark canvas instead of a generic neon AI look.
- Show one clean hero screenshot: poster wall or AI companion with a selected film.
- Use one short line only, such as: `AI-native movie player for people who care about cinema.`
- Keep the ifq.ai signature small and quiet in one corner.
- Avoid busy grids, large logos, or feature overload.

Suggested size:

- 1280 x 640 px

Suggested headline options:

- `AI-native movie player for people who care about cinema.`
- `Local library viewing, with calmer and smarter film intelligence.`
- `A quieter AI movie player for people who actually watch films.`

Suggested supporting line options:

- `Poster wall, subtitles, metadata, recommendations, and structured viewing workflows.`
- `Built for local libraries, cinematic taste, and OpenAI-compatible film guidance.`

## Screenshot Shot List

Capture these first:

1. Poster wall with a tasteful, real-looking library.
2. AI Companion discussing a single film with multi-turn context visible.
3. AI Taste Engine recommendation screen.
4. Movie detail page with metadata, cast, and AI entry point.
5. Subtitle search flow.

Screenshot principles:

- Prefer realistic libraries over empty-state demos.
- Do not overload every screenshot with overlays or labels.
- Use the same theme and visual tone across all captures.
- Crop tightly enough to feel intentional but not claustrophobic.

Recommended file naming:

- `01-poster-wall-library.png`
- `02-ai-companion-selected-film.png`
- `03-workflow-studio-pre-watch-briefing.png`
- `04-ai-taste-engine.png`
- `05-movie-detail-metadata.png`
- `06-subtitle-search-flow.png`

## Suggested Discussions Categories

Recommended categories for GitHub Discussions:

- `Announcements`: release posts, roadmap shifts, launch notes.
- `Ideas`: product suggestions, workflow improvements, feature direction.
- `Workflow Showcase`: examples of good prompts, saved workflow cards, viewing setups.
- `Setup Help`: provider configuration, TMDB setup, local-library troubleshooting.
- `Cinema Talk`: film recommendations, pairing ideas, interpretation threads.

Recommended ordering:

1. Announcements
2. Ideas
3. Workflow Showcase
4. Setup Help
5. Cinema Talk

## Release Title Format

Recommended stable release format:

- `AI-Movie-Player v0.2.1`

Recommended preview format:

- `AI-Movie-Player v0.3.0-beta.1`

## Release Notes Template

```md
## AI-Movie-Player v0.2.1

AI-Movie-Player is an AI-native movie player for local libraries, built to make metadata, subtitles, recommendations, and cinematic conversation feel natural and refined.

### Highlights
- Added guided AI viewing workflows: pre-watch briefing, post-watch recap, and double-feature pairing.
- Split top-level docs into English-first and Chinese companion files.
- Improved GitHub collaboration surfaces with issue templates and launch documentation.

### Upgrade Notes
- Existing users keep legacy data compatibility automatically.
- AI features still require an OpenAI-compatible provider.

### Feedback
- Issues: https://github.com/peixl/AI-Movie-Player/issues
- Discussions: https://github.com/peixl/AI-Movie-Player/discussions

Made with care by ifq.ai.
```

## Homepage Story Order

For the README and repo landing experience, keep the story in this order:

1. What it is.
2. Why it is different.
3. AI features that feel practical.
4. Screenshot sequence.
5. Comparison with ordinary players.
6. Roadmap and FAQ.

## Quick Setup Checklist

### Repository Settings

- [ ] Set repository description: `AI-native movie player for local libraries with cinematic chat, library-aware recommendations, metadata, subtitles, and poster-wall browsing.`
- [ ] Set repository website: `https://ifq.ai`
- [ ] Add topics (see Suggested Topics above)
- [ ] Enable Discussions with categories (see Suggested Discussions Categories above)
- [ ] Upload social preview image (1280x640px)
- [ ] Set CODEOWNERS file (already in `.github/CODEOWNERS`)

### GitHub Features

- [ ] Enable issue templates (already in `.github/ISSUE_TEMPLATE/`)
- [ ] Enable PR template (already in `.github/PULL_REQUEST_TEMPLATE.md`)
- [ ] Configure branch protection for `main` (require PR reviews)
- [ ] Enable vulnerability alerts in Security tab
- [ ] Enable Dependabot for dependency updates

### Social Preview Image

Create a 1280x640px image with:
- Dark, cinematic background
- One hero screenshot (poster wall or AI companion)
- Single headline: "AI-native movie player for people who care about cinema."
- Small ifq.ai signature in corner
- No busy grids or feature overload

## Tone Rules

- Quiet premium beats loud futurism.
- Cinema-first beats feature-dump marketing.
- ifq.ai should read like authorship, not ad inventory.
- Show taste and judgment, not just capability.