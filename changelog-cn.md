# 更新日志

[English](CHANGELOG.md) | 简体中文

## [未发布]

### 新增
- 新增 AI 观影工作流，包括观影前 briefing、观影后复盘与双片连看建议。
- 新增 GitHub Issue 模板，覆盖缺陷、功能请求和 AI 工作流想法。
- 新增 GitHub 发布面文档，整理 repo description、topics、release 文案、social preview 方向与截图规划。
- 为贡献指南和更新日志补充独立中文文档。

### 变更
- 扩展 README 展示层，加入截图规划、功能对比、路线图、FAQ 和更强的 GitHub 首页结构。
- 顶层读者文档改为按语言拆分维护，不再长期混排在同一个文件中。

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
- 项目正式更名为 AI-Movie-Player，并统一为克制自然的 ifq.ai 署名风格。
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