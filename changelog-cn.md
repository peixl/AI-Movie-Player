# 更新日志

[English](CHANGELOG.md) | 简体中文

本文件记录项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增
- 共享 HTTP 韧性层（`src/api/http.rs`）：按 API/图片/下载分档超时、连接池、gzip/brotli、重定向上限、统一 User-Agent、带抖动与 `Retry-After` 支持的指数退避重试，以及可重试状态码/网络错误分类。
- 模糊匹配器（`src/core/matcher.rs`）：综合归一化标题相似度（token Dice、包含度、Damerau–Levenshtein）、年份差距与热度决胜分，并设置明确置信阈值，拒绝"第一条结果即正确"的误匹配。
- 诚实兜底入库：无法匹配 TMDB 的文件也会以最小记录入库，不再被静默丢弃。
- 去重升级：除路径去重外，新增文件头尾内容哈希去重；批量导入有界并发（6 个 worker）并实时回报导入/跳过/未匹配/失败进度。
- 扫描过滤样片、预告片、花絮、featurette、访谈、删减片段与隐藏目录，限制扫描深度，并补充更多视频扩展名。
- 接入 OpenSubtitles 官方新 API（`api.opensubtitles.com/api/v1`）：设置中配置 API Key，支持 tmdb/imdb/关键词搜索、BCP-47 语言映射、按语言偏好/评分/下载量/HD 排序（AI 翻译与听障字幕降权）以及按 file_id 下载。
- 字幕压缩包安全处理：防 zip-slip 路径穿越、优选条目（srt > ass > vtt …）、UTF-8/UTF-16 BOM 与 GBK 解码，并以影片同名规则安装到影片旁，便于 IINA/VLC/MPC-HC 自动挂载。
- 跨平台分离式播放（`src/core/playback.rs`）："默认播放器播放"与"在文件夹中显示"完全脱离 UI 线程，Linux 下 `xdg-open` 缺失时回退 `gio open`。
- 详情页新增"在文件夹中显示"与"重新定位文件"；文件丢失时红色警告并禁用播放。
- 导入向导真正可用：实时进度条、当前文件、失败计数与完成摘要；文件夹选择每个请求只弹一次，取消后不再死循环弹窗。
- 字幕面板真正闭环：结果展示语言/来源/评分/下载量/HD/AI/HI 徽章，行内下载按钮防重入，未配置 Key 时引导跳转设置。
- 海报全异步：海报墙解码移入后台线程并配 LRU 纹理缓存与骨架占位；详情海报走 egui 图片加载器；缩略图临时文件 + 重命名原子落盘。
- 自适应重绘（有任务时快速刷新、空闲时降频）、Ctrl+F/Cmd+F 聚焦搜索框、全新 256px 过程式应用图标、SQLite `busy_timeout` 与 WAL 友好 PRAGMA。
- README 新增快速演示路径，串起导入、元数据、电影详情、AI briefing 和双片连看流程。
- README 新增 AI 上下文样例，说明各个 AI 工作流应使用哪些影片和片库上下文。
- 新增新手任务包，用于维护 `good first issue` 和 `help wanted` 路线图任务。

### 变更
- TMDB 客户端为 API 与图片 CDN 使用独立 HTTP 客户端和限速器；主语言与 zh-CN 详情并发获取；海报下载校验 content-type 与空体。
- AI 流式请求改为仅连接超时 + 每个数据块的空闲超时，长回答不再被固定总超时误杀；非流式对话支持退避重试。
- SSE 解码改用增量 UTF-8 解码器，替代逐块 `from_utf8_lossy`，消除多字节 token 被分块切开时产生的永久乱码。
- TMDB 客户端改为 `Arc<TmdbClient>` 共享，移除把所有请求串行化的全局互斥锁。
- 保存设置后就地重建网络客户端；字幕下载通过独立数据库连接写入记录。
- 更新发布面文档，把截图、演示视频和可见新手 issue 明确为 v0.2.1 之后最重要的社区准备度缺口。
- 调整 README release 文案，匹配 v0.2.1 原生系统包和校验和已经可下载的现状。

### 修复
- 导入向导永远停在搜索中：后台任务里的 TMDB 搜索结果与字幕搜索结果曾被直接丢弃，向导无法完成。
- 缺失或过短的上映日期会导致 TMDB 解析 panic（`&date[..4]`），日期解析改为安全边界检查。
- 取消文件夹对话框后可能每帧重复弹窗。
- 海报在 UI 线程同步解码导致大片库卡顿，现已移至后台线程。
- 移除切换视图时的 toast 噪音与永久 500ms 空转重绘。

### 安全
- 修复字幕压缩包解压的 zip-slip 路径穿越漏洞，拒绝恶意构造的成员路径。
- 字幕下载需用户自行配置 OpenSubtitles API Key，不再内置第三方 Key；展示前对服务商错误正文做密钥脱敏与截断。

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
