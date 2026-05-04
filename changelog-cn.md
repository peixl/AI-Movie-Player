# 更新日志

[English](CHANGELOG.md) | 简体中文

本文件记录项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增
- README 新增快速演示路径，串起导入、元数据、电影详情、AI briefing 和双片连看流程。
- README 新增 AI 上下文样例，说明各个 AI 工作流应使用哪些影片和片库上下文。
- 新增新手任务包，用于维护 `good first issue` 和 `help wanted` 路线图任务。

### 变更
- 更新发布面文档，把截图、演示视频和可见新手 issue 明确为 v0.2.1 之后最重要的社区准备度缺口。
- 调整 README release 文案，匹配 v0.2.1 原生系统包和校验和已经可下载的现状。

## [0.2.1] - 2026-05-04

### 新增
- 电影详情页新增本地文件打开动作，可调用操作系统默认播放器。
- 新增 GitHub Packages 发布工作流，发布 GHCR 包 `ghcr.io/peixl/ai-movie-player`。
- README 新增当前状态说明，澄清播放模式与 beta 阶段成熟度。
- 新增 AI 观影工作流，包括观影前 briefing、观影后复盘与双片连看建议。
- 新增 GitHub Issue 模板，覆盖缺陷、功能请求和 AI 工作流想法。
- 新增 GitHub 发布面文档，整理 repo description、topics、release 文案、social preview 方向与截图规划。
- 为贡献指南和更新日志补充独立中文文档。
- 新增安全政策 (SECURITY.md)。
- 新增 PR 模板（含检查清单）。
- 新增按文件路径自动标记 PR 的工作流。
- 新增过期 issue/PR 自动管理的工作流。
- 新增 CODEOWNERS 文件定义代码所有权。
- 新增 Linux 发布打包与 SHA256 校验和。
- CI 中新增文档构建验证。
- 新增 `rustfmt.toml` 和 `clippy.toml` 统一代码风格。
- README 新增架构图（mermaid）。
- README 新增技术栈章节。
- README 新增致谢章节。

### 变更
- 将后续 Dependabot 更新改为分组提交，减少 PR 噪音并保持维护信号更清爽。
- 对外文案调整为本地电影片库助手与系统播放器调用，应用内原生播放控制保留为路线图项目。
- 澄清发布二进制包依赖 GitHub Release 资产；资产发布前仍以源码运行作为当前后备路径。
- 安全说明补充系统凭据存储路线图和更清晰的密钥处理建议。
- 扩展 README 展示层，加入截图规划、功能对比、路线图、FAQ 和更强的 GitHub 首页结构。
- 顶层读者文档改为按语言拆分维护，不再长期混排在同一个文件中。
- 项目结构树新增文件描述。
- CONTRIBUTING.md 增强：新增架构概览、提交规范、分支命名。
- CI 工作流新增并发组、文档构建和依赖排序。
- Release 工作流为所有产物生成 SHA256 校验和。
- Cargo.toml 补充完整元数据：homepage、repository、license、keywords、categories、authors。
- 升级分组 Cargo 依赖，包括 eframe/egui、reqwest、rusqlite、rfd、zip、scraper、toml、sha2 等。

### 修复
- 当 egui 默认字体缺少 CJK 覆盖时，中文界面标签会显示为方块字。
- 修复 reqwest 0.13 query feature 与 sha2 0.11 digest 格式化兼容问题。

## [0.2.0] - 2026-05-02

### 新增
- AI Companion：围绕单部电影进行流式对话与深入分析。
- AI 推荐：支持片库推荐、片外发现与观影画像。
- AI 影评入口：从详情页快速进入 AI 解析。
- OpenAI-compatible API：兼容 OpenAI、Azure、Ollama、LM Studio 等提供方。
- 程序化图标系统。
- hover、shadow、toast、pulse、shimmer 等动效能力。
- 核心导航快捷键。
- Toast 轻量提示。
- 确认弹窗组件。
- 数据库与文件名解析测试覆盖。
- 跨平台 GitHub Actions CI。

### 变更
- 项目正式更名为 AI Movie Player，并统一为克制自然的 ifq.ai 署名风格。
- 核心 UI 文案在适合的场景下改为中英双语。
- AI 提示词升级为更自然、更优雅、更有上下文意识的表达。
- AI 对话正式支持真实多轮上下文。
- 保留旧数据目录与旧数据库文件名兼容能力。

### 修复
- 核心界面中的旧品牌残留。
- 字幕下载请求中的旧 User-Agent 标识。
- 设置面板文案一致性问题。

## [0.1.0] - 2026-04-28

### 新增
- 初始片库管理。
- SQLite 与 FTS5 全文检索。
- TMDB API v3 接入。
- 字幕搜索。
- 海报墙浏览。
- 文件夹扫描与文件名解析。
- 深浅色主题。
- 片单、设置与批量操作。

[未发布]: https://github.com/peixl/AI-Movie-Player/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/peixl/AI-Movie-Player/releases/tag/v0.2.1
[0.2.0]: https://github.com/peixl/AI-Movie-Player/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/peixl/AI-Movie-Player/releases/tag/v0.1.0
