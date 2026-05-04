# GitHub 发布面说明

[English](github-launch-kit.md) | 简体中文

这是一份给维护者使用的 GitHub 发布面整理文档，用来统一 AI Movie Player 在仓库首页、Releases、About 区域和截图素材上的对外表达。

## 推荐的 Repo Description

GitHub About 区域可以优先使用下面两版英文描述之一：

- AI-native local movie library companion with system-player launch, cinematic chat, recommendations, metadata, subtitles, and poster-wall browsing.
- AI-native local movie library companion with TMDB metadata, subtitle workflow, poster wall, and OpenAI-compatible film intelligence.

## 推荐 Topics

建议设置这些 topics：

- movie-library
- media-player
- ai
- rust
- egui
- desktop-app
- tmdb
- subtitles
- recommendation-engine
- openai-compatible
- ollama
- lm-studio
- cinematic-ui
- local-first

## Social Preview 方向

社交分享图应该走克制、高级、电影化的路线，而不是常见的霓虹 AI 风格。

建议构图：

- 使用温暖、克制的深色底，而不是廉价炫光背景。
- 只放一张干净的主截图，例如海报墙或带选中电影的 AI Companion。
- 文案保持一行，例如：`AI-native local movie library companion for people who care about cinema.`
- ifq.ai 署名小而安静地放在角落。
- 不要堆太多功能点、Logo 或视觉噪音。

建议尺寸：

- 1280 x 640 px

建议主文案可选：

- `AI-native local movie library companion for people who care about cinema.`
- `Local library viewing, with calmer and smarter film intelligence.`
- `A quieter AI movie library companion for people who actually watch films.`

建议副文案可选：

- `Poster wall, subtitles, metadata, recommendations, and structured viewing workflows.`
- `Built for local libraries, cinematic taste, and OpenAI-compatible film guidance.`

## 截图清单

v0.2.1 已经有正式 release 资产，截图现在是最重要的剩余信任缺口。优先准备这 6 张：

1. 一个观感好的海报墙片库截图。
2. 展示多轮上下文的 AI Companion 对话截图。
3. 展示观影前 briefing 或观影后复盘的结构化工作流卡片。
4. AI 推荐页截图。
5. 电影详情页截图，带元数据、Open 动作和 AI 入口。
6. 字幕搜索流程截图。

截图原则：

- 优先使用真实感强的片库，而不是空状态演示。
- 不要给每一张截图都叠满说明文字。
- 所有截图保持统一主题和视觉语气。
- 裁切要克制，既显得有设计感，也不要过度局促。

建议文件命名：

- `01-poster-wall-library.png`
- `02-ai-companion-selected-film.png`
- `03-workflow-studio-pre-watch-briefing.png`
- `04-ai-taste-engine.png`
- `05-movie-detail-metadata.png`
- `06-subtitle-search-flow.png`

## Discussions 分类建议

建议在 GitHub Discussions 中设置这些分类：

- `Announcements`：发布公告、路线变化、版本说明。
- `Ideas`：产品建议、工作流改进、功能方向。
- `Workflow Showcase`：优秀 prompt、保存下来的工作流卡片、真实观影用法示例。
- `Setup Help`：AI 提供方配置、TMDB 设置、本地片库故障排查。
- `Cinema Talk`：电影推荐、双片连看、剧情解读和讨论。

推荐排序：

1. Announcements
2. Ideas
3. Workflow Showcase
4. Setup Help
5. Cinema Talk

## Release 标题格式

稳定版建议格式：

- `AI Movie Player v0.2.1`

预览版建议格式：

- `AI Movie Player v0.3.0-beta.1`

## Release 文案模板

```md
## AI Movie Player v0.2.1

AI Movie Player 是一个面向本地片库的 AI 原生电影助手，目标是让元数据、字幕、推荐、系统播放器调用和电影对话都变得自然、优雅、可信。

### 本次亮点
- 新增 AI 观影工作流：观影前 briefing、观影后复盘、双片连看建议。
- 电影详情页新增本地文件打开动作，通过系统默认播放器播放。
- 顶层文档改为英文主文档 + 中文独立配套文档。
- 改进 GitHub 协作与发布面，加入 Issue 模板和发布说明文档。

### 升级说明
- 旧用户会自动保留历史数据兼容。
- AI 功能仍需配置 OpenAI-compatible 提供方。

### 反馈入口
- Issues: https://github.com/peixl/AI-Movie-Player/issues
- Discussions: https://github.com/peixl/AI-Movie-Player/discussions

Made with care by ifq.ai.
```

## 新手 Issue 包

仓库外部贡献者还少时，更要保留一组可见、可认领的新手任务，避免项目看起来像只允许内部维护。种子清单见 [starter-issues-cn.md](starter-issues-cn.md)。

建议第一批创建：

- 给 README 和 release notes 添加真实截图。
- 为 filename parser 补充常见发布命名测试。
- 补充 TMDB Key 首次配置文档。
- 增加 Ollama 和 LM Studio 诊断示例。
- 增加 Windows PowerShell 校验和验证示例。

好的新手 issue 应该包含明确范围、验收标准，以及一个验证命令或手动 smoke check。

## GitHub 配置清单

- [ ] 设置 repository description：`AI-native local movie library companion with system-player launch, cinematic chat, recommendations, metadata, subtitles, and poster-wall browsing.`
- [ ] 设置 repository website：`https://ifq.ai`
- [ ] 添加推荐 topics
- [ ] 启用 Discussions 并配置分类
- [ ] 上传 1280x640px social preview image
- [ ] 创建 5-10 个来自 `docs/starter-issues-cn.md` 的新手 issue
- [ ] 配置 main 分支保护
- [ ] 启用 Security tab 的 vulnerability alerts
- [ ] 确认 Dependabot 已启用

## GitHub 首页叙事顺序

建议 README 和仓库首页按这个顺序讲故事：

1. 它是什么。
2. 它为什么和普通播放器不同。
3. 它有哪些真正有用的 AI 能力。
4. 它应该展示哪些截图。
5. 它与普通播放器相比多做了什么。
6. 它接下来要往哪里走。

## 文案气质规则

- 安静高级，优先于喧闹未来感。
- 电影导向，优先于营销式功能堆叠。
- ifq.ai 更像作者署名，而不是广告位。
- 要展现判断力和品味，而不是只展示“我也有 AI”。