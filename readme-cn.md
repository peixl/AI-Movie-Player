# AI Movie Player

一个面向电影爱好者的 AI 原生本地片库助手，不只是打开文件，更帮助你理解和经营自己的收藏。

[English](README.md) | 简体中文

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/peixl/AI-Movie-Player/releases)
[![CI](https://github.com/peixl/AI-Movie-Player/actions/workflows/ci.yml/badge.svg)](https://github.com/peixl/AI-Movie-Player/actions/workflows/ci.yml)
[![AI](https://img.shields.io/badge/AI-OpenAI%20Compatible-6366f1.svg)](https://github.com/peixl/AI-Movie-Player#ai-features)

项目由 [ifq.ai](https://ifq.ai) 打造，开源仓库位于 [peixl/AI-Movie-Player](https://github.com/peixl/AI-Movie-Player)。

## 项目简介

AI Movie Player 是一个基于 Rust 与 egui 构建的早期桌面本地片库助手。它把本地片库管理、TMDB 元数据、字幕工作流、海报墙浏览、系统播放器调用和 OpenAI-compatible AI 能力整合在一个更安静、更自然的电影体验里。

这个项目想做的不是"给播放器加一个 AI 按钮"，而是把"选片、看片、看后理解、片库管理"这整个链路做得更完整、更优雅。

## 当前状态

AI Movie Player 目前是 beta 阶段的本地片库应用，而不是已经内置完整播放内核的媒体播放器。电影详情页可以调用系统默认播放器打开本地影片文件；应用内原生播放控制仍在路线图中。

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 语言 | Rust (edition 2024, MSRV 1.85) |
| GUI 框架 | [egui](https://github.com/emilk/egui) / eframe 0.31 |
| 异步运行时 | [Tokio](https://tokio.rs) |
| 数据库 | [SQLite](https://sqlite.org) via rusqlite (WAL 模式, FTS5) |
| HTTP 客户端 | [reqwest](https://github.com/seanmonstar/reqwest) (gzip, brotli, stream) |
| 图像处理 | [image](https://github.com/image-rs/image) crate |
| 错误处理 | [thiserror](https://github.com/dtolnay/thiserror) + [anyhow](https://github.com/dtolnay/anyhow) |
| CI/CD | GitHub Actions (多平台矩阵) |

## 架构

```mermaid
graph TB
    subgraph UI["UI 层 (egui)"]
        App[app.rs]
        Layout[layout.rs]
        PosterWall[poster_wall.rs]
        MovieDetail[movie_detail.rs]
        AiChat[ai_chat_panel.rs]
        AiRec[ai_recommend_panel.rs]
        Settings[settings_panel.rs]
    end

    subgraph Core["核心层"]
        MovieModel[core/movie.rs]
        SubParser[core/subtitle_parser.rs]
        FilenameParser[core/filename_parser.rs]
    end

    subgraph API["API 层"]
        AiClient[api/ai.rs]
        TmdbClient[api/tmdb.rs]
        SubClient[api/subtitle.rs]
    end

    subgraph DB["数据库层"]
        MoviesDB[db/movies.rs]
        SettingsDB[db/settings.rs]
        Schema[db/schema.rs]
    end

    subgraph External["外部服务"]
        OpenAI[OpenAI / Ollama / LM Studio]
        TMDB[TMDB API v3]
        SubSources[字幕来源]
    end

    App --> Layout
    App --> PosterWall
    App --> MovieDetail
    App --> AiChat
    App --> AiRec
    App --> Settings
    App --> MoviesDB
    App --> SettingsDB

    AiChat --> AiClient
    AiRec --> AiClient
    MovieDetail --> TmdbClient
    Settings --> TmdbClient

    AiClient --> OpenAI
    TmdbClient --> TMDB
    SubClient --> SubSources

    MoviesDB --> Schema
    SettingsDB --> Schema
    PosterWall --> MoviesDB
    MovieDetail --> MoviesDB
```

## 为什么是 AI Movie Player

- 针对单部电影的 AI 对话，并且支持真正的多轮上下文记忆。
- 基于你自己的片库生成更有依据的推荐，而不是空泛的猜你喜欢。
- 可以从电影详情页调用系统默认播放器打开本地影片文件。
- 从详情页一键进入 AI 解析、短评和观影建议。
- 使用 TMDB 自动补全标题、海报、导演、演员、评分和简介。
- 提供多来源字幕搜索与下载能力，服务本地观影场景。
- 支持海报墙浏览、批量导入、片单和设置等完整桌面工作流。
- ifq.ai 的标识保持克制，像作品署名，而不是宣传位。

## AI 功能

### AI Companion

选择一部电影后，AI 会结合片名、年份、导演、类型、剧情简介和演员阵容与你对话。现在聊天会保留上下文，所以追问和深入分析会自然很多，不再像一次性问答。

适合的问题包括：

- 深度解析
- 结局解读
- 相似电影推荐
- 幕后趣闻
- 观影陪伴提示
- 值不值得看

### AI Taste Engine

AI 会从你的片库中分析口味结构，告诉你现在最该看什么、为什么适合你、你的观影盲区在哪里，以及哪些你还没收藏的电影值得补进来。

### AI Review

在电影详情页可以直接进入 AI 解析，快速获取短评、优点、不足、适合人群和观影建议。

### 支持的 AI 提供方

AI Movie Player 兼容：

- OpenAI
- Azure OpenAI
- Ollama
- LM Studio
- 任意 OpenAI-compatible 接口

### 观影工作流

AI Movie Player 现在开始把 AI 能力做成更自然的观影流程，而不是零散问答：

- 观影前 briefing：在播放前给出情绪、背景和值得留意的细节。
- 观影后复盘：在片尾之后帮助用户整理主题、结构和关键选择。
- 双片连看建议：推荐一部真正能放大第一部电影价值的搭配影片，而不是偷懒的同类型相似片。

## AI 上下文样例

AI 功能会尽量使用应用已经掌握的影片和片库上下文，而不是表现成一个脱离场景的普通聊天框。

| 工作流 | 使用的上下文 | 适合的提问 | 理想输出形态 |
| --- | --- | --- | --- |
| AI Companion | 当前影片的片名、年份、导演、类型、简介、演员、评分和对话历史。 | `给我一份不剧透的观影前 briefing。` | 情绪基调、前提边界、值得留意的细节，以及元数据不足时的说明。 |
| AI Review | 电影详情字段和用户问题。 | `这部片适合深夜看吗，为什么？` | 简短判断、适合人群、优点、不足和观影建议。 |
| AI Taste Engine | 本地片库标题、类型、片单状态、评分和可见元数据。 | `从我自己的片库里推荐下一部。` | 带依据的排序推荐，而不是泛泛的类型猜测。 |
| 双片连看 | 当前影片，以及可用的片库和口味上下文。 | `帮我搭配一部能改变我理解角度的电影。` | 一个聚焦的搭配选择、清楚的呼应或反差，以及第二部带来的增量。 |

好的 AI 输出应该在信息不足时明确说明，避免编造幕后事实，并用已有证据解释推荐理由。

## 核心模块

| 模块 | 说明 |
| --- | --- |
| 片库 | 扫描文件夹、识别影片文件、避免重复导入，管理个人收藏。 |
| 播放调用 | 从电影详情页用系统默认播放器打开已记录的本地影片文件。 |
| 元数据 | 通过 TMDB 补全标题、海报、演员、评分与简介。 |
| AI | 提供电影对话、AI 解析、观影画像和推荐流程。 |
| 字幕 | 从多个来源搜索并下载适合本地媒体的字幕。 |
| 海报墙 | 用更直观的视觉方式浏览收藏。 |
| 片单 | 管理接下来准备看的影片。 |

## 与普通播放器相比

| 能力 | 普通播放器 | AI Movie Player |
| --- | --- | --- |
| 播放模式 | 内置控制 | 当前调用系统播放器；内置播放控制在规划中 |
| TMDB 元数据补全 | 有时支持 | 内建 |
| 围绕单部电影的 AI 对话 | 少见 | 原生工作流 |
| 基于片库的推荐 | 少见 | 内建 |
| 观影工作流 | 基本没有 | 观影前、观影后、双片连看 |
| 字幕工作流 | 基础能力 | 更偏向搜索与下载闭环 |
| 产品气质 | 工具导向 | 电影导向，安静高级 |

## 视觉预览

v0.2.1 已经发布原生系统安装包，但公开截图和短 GIF/视频 demo 仍然是当前最需要补齐的信任信号。第一组预览应优先展示海报墙、选中电影后的 AI Companion、AI Taste Engine、带 Open 动作的电影详情页，以及字幕搜索流程。

## 快速开始

### 预编译版本

项目已经通过发布工作流提供 Windows、macOS、Linux 安装包。可以从 [Releases](https://github.com/peixl/AI-Movie-Player/releases) 页面下载适合您平台的最新版本：

- **Windows**: `.zip` 压缩包
- **macOS**: 包含 `.app` bundle 的 `.tar.gz`
- **Linux**: `.tar.gz` 压缩包
- **校验和**: 每个压缩包旁边都有对应的 `.sha256` 文件

GitHub Packages 会随发布标签同步发布 OCI 包：`ghcr.io/peixl/ai-movie-player`。桌面用户仍建议优先下载 Releases 页面中的原生系统包。

如果 Releases 页面暂时还没有可下载资产，请先按下方命令从源码运行。

### 环境要求（从源码构建）

- Rust 1.85+
- 一个来自 [themoviedb.org/settings/api](https://www.themoviedb.org/settings/api) 的 TMDB API Key
- 可选：任意 OpenAI-compatible API Key

### 克隆并运行

```bash
git clone https://github.com/peixl/AI-Movie-Player.git
cd AI-Movie-Player
cargo run --release
```

## 快速演示路径

先用一个很小的测试片库验证流程，再扫描真实收藏：

1. 建一个文件夹，放入一两个命名干净的文件，例如 `Dune (2021).mkv` 或 `Inception (2010).mp4`。
2. 打开 Settings，填写 TMDB API Key，以便补全海报、演员、片长和简介。
3. 导入这个文件夹，然后进入海报墙，选择其中一部电影。
4. 打开电影详情页，确认元数据、海报、文件路径和 Open 动作都可用。
5. 如果要体验 AI，再配置一个 OpenAI-compatible 提供方。
6. 在 AI Companion 中请求一份不剧透的观影前 briefing。
7. 再试一次双片连看建议，观察应用如何把单部电影扩展成观影路径。

即使不配置 AI Key，本地片库、元数据、海报、片单、字幕和系统播放器调用流程仍然可以使用。

## AI 配置

1. 打开 Settings。
2. 填写 AI Endpoint、API Key 和 Model。
3. 保存设置。
4. 如果使用本地模型，可以直接选择 Ollama 或 LM Studio 预设。

常见接口示例：

```text
OpenAI    -> https://api.openai.com/v1
Ollama    -> http://localhost:11434/v1
LM Studio -> http://localhost:1234/v1
```

## 快捷键

| 快捷键 | 功能 |
| --- | --- |
| Ctrl+1 | 片库 |
| Ctrl+2 | 导入影片 |
| Ctrl+3 | 字幕搜索 |
| Ctrl+4 | 批量操作 |
| Ctrl+5 | 片单 |
| Ctrl+6 | 设置 |
| Ctrl+7 | AI 对话 |
| Ctrl+8 | AI 推荐 |
| Ctrl+F | 搜索片库 |
| Esc | 返回 |

## 项目结构

```text
ai-movie-player/
├── src/
│   ├── main.rs                  # 入口，Tokio 运行时初始化
│   ├── app.rs                   # 应用核心结构体 (eframe::App)
│   ├── ai/                      # AI 提示词构建与工作流逻辑
│   ├── api/
│   │   ├── ai.rs                # OpenAI 兼容流式客户端
│   │   └── tmdb.rs              # TMDB API v3 客户端
│   ├── core/
│   │   ├── filename_parser.rs   # 从文件名提取元数据
│   │   ├── file_organizer.rs    # 文件整理与重命名
│   │   ├── library_manager.rs   # 片库扫描与导入
│   │   ├── metadata_service.rs  # TMDB 元数据补全
│   │   └── subtitle_finder.rs   # 字幕搜索协调
│   ├── db/
│   │   ├── connection.rs        # 数据库启动与连接选项
│   │   ├── migrations.rs        # SQLite schema 迁移
│   │   ├── models.rs            # 持久化数据模型
│   │   ├── movies.rs            # Movie CRUD 操作
│   │   ├── settings.rs          # 设置键值存储
│   │   ├── subtitles.rs         # 字幕持久化辅助
│   │   ├── watchlist.rs         # 片单持久化辅助
│   │   └── tests.rs             # 数据库相关测试
│   ├── ui/
│   │   ├── layout.rs            # 侧边栏导航与视图路由
│   │   ├── theme.rs             # 颜色系统与主题辅助
│   │   ├── icons.rs             # 程序化手绘图标系统
│   │   ├── animation.rs         # hover、shimmer、toast 动效
│   │   ├── poster_wall.rs       # 海报墙视觉浏览
│   │   ├── movie_detail.rs      # 电影详情面板（含海报缓存）
│   │   ├── ai_chat_panel.rs     # AI 伴侣流式对话
│   │   ├── ai_recommend_panel.rs# AI 口味引擎推荐
│   │   ├── settings_panel.rs    # 设置与 AI 配置
│   │   ├── add_movie.rs         # 影片导入工作流
│   │   ├── subtitle_panel.rs    # 字幕搜索与下载
│   │   ├── batch_ops.rs         # 批量操作
│   │   └── watchlist_panel.rs   # 片单管理
│   ├── config/
│   │   └── settings.rs          # AppSettings 模型
│   ├── thumbnail/
│   │   └── cache.rs             # 海报与缩略图缓存辅助
│   └── util/
│       └── error.rs             # AppError 类型
├── .github/
│   ├── workflows/
│   │   ├── ci.yml               # CI: fmt, clippy, test, doc, package smoke
│   │   ├── dependency-review.yml# PR 依赖风险检查
│   │   ├── release.yml          # Release: 多平台打包 + 校验和
│   │   ├── package.yml          # GitHub Packages: GHCR OCI 包
│   │   ├── labeler.yml          # 按文件路径自动标记 PR
│   │   └── stale.yml            # 自动关闭过期 issue
│   ├── ISSUE_TEMPLATE/          # Bug 报告、功能请求模板
│   ├── PULL_REQUEST_TEMPLATE.md # PR 检查清单
│   └── CODEOWNERS               # 代码所有权定义
├── docs/                        # 发布工具包、发布说明
├── Cargo.toml                   # 依赖与元数据
├── rustfmt.toml                 # 格式化配置
├── clippy.toml                  # Clippy 检查配置
├── README.md                    # 英文文档
├── readme-cn.md                 # 中文文档
├── CONTRIBUTING.md              # 英文贡献指南
├── contributing-cn.md           # 中文贡献指南
├── CHANGELOG.md                 # 英文更新日志
├── changelog-cn.md              # 中文更新日志
├── SECURITY.md                  # 安全政策
└── LICENSE                      # MIT 许可证
```

## 开发

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
cargo build --release --locked
```

如果当前环境无法访问 crates.io，建议优先使用本地诊断或离线缓存做验证。

### Live AI 冒烟测试

仓库内置了一个默认忽略的 OpenAI-compatible live 测试。服务参数只应通过环境变量传入，不要把 endpoint、model、token 或本地媒体路径提交到源码：

```bash
AI_MOVIE_PLAYER_LIVE_ENDPOINT="https://example.com/v1" \
AI_MOVIE_PLAYER_LIVE_API_KEY="your-api-key" \
AI_MOVIE_PLAYER_LIVE_MODEL="your-model" \
AI_MOVIE_PLAYER_LIVE_VIDEO_PATH="/path/to/movie.mp4" \
cargo test --test ai_live -- --ignored --nocapture
```

测试会从本地文件生成影片上下文，但只发送文件名、文件大小和可检测到的媒体特征等隐私友好的元数据。

## 文档

- 英文文档： [README.md](README.md)
- 中文文档： [readme-cn.md](readme-cn.md)
- 英文贡献指南： [CONTRIBUTING.md](CONTRIBUTING.md)
- 中文贡献指南： [contributing-cn.md](contributing-cn.md)
- 英文更新日志： [CHANGELOG.md](CHANGELOG.md)
- 中文更新日志： [changelog-cn.md](changelog-cn.md)
- 安全政策： [SECURITY.md](SECURITY.md)
- GitHub 英文发布说明： [docs/github-launch-kit.md](docs/github-launch-kit.md)
- GitHub 中文发布说明： [docs/github-launch-kit-cn.md](docs/github-launch-kit-cn.md)
- 新手任务包： [docs/starter-issues-cn.md](docs/starter-issues-cn.md)

## 路线图

- 持续验证 v0.2.x GitHub Release，确保 Windows、macOS、Linux 包、SHA256 校验和与发布说明保持一致。
- 给仓库首页和 Release 页面补齐真实截图与短 GIF/视频 demo。
- 维护一组可认领的新手任务，用 `good first issue` 和 `help wanted` 标签连接路线图与真实贡献入口。
- 将 API Key 从明文 SQLite 存储迁移到可用平台的系统凭据存储。
- 探索应用内原生播放控制，同时保留系统播放器调用作为稳定后备路径。
- 继续把 AI 观影工作流做得更像观影的一部分，而不是看完后才补一句聊天。
- 提升字幕质量排序和来源可靠性。
- 进一步优化大体量片库下的海报墙性能与观感。
- 加强本地 AI 提供方与自托管接口的上手体验。

## FAQ

### 这是流媒体应用吗？

不是。AI Movie Player 主要围绕本地片库和个人媒体工作流构建。

### 现在内置视频播放器吗？

暂时没有。应用当前会从详情页调用系统默认播放器打开已记录的本地影片文件。等片库、元数据、字幕、发布和安全基础更稳之后，再推进应用内原生播放控制。

### 不填 AI API Key 也能用吗？

可以。片库、海报、元数据和字幕等流程仍然可以使用；AI 功能才需要 OpenAI-compatible 提供方。

### 选哪个 AI 提供方比较好？

取决于你的使用方式。OpenAI 适合最快开始，Ollama 和 LM Studio 更适合本地模型方案。

### 为什么 ifq.ai 的标识这么克制？

因为这个项目把 ifq.ai 视为作品署名，而不是打断体验的宣传位。产品应该先显得优雅，再让品牌被自然认出来。

## 参与贡献

开发流程、代码标准和 PR 要求见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

MIT，详见 [LICENSE](LICENSE)。

## 致谢

- [TMDB](https://www.themoviedb.org/) 提供电影元数据和海报 API。
- [egui](https://github.com/emilk/egui) 提供即时模式 GUI 框架。
- [OpenAI](https://openai.com) 定义了被生态系统广泛采用的 Chat Completions API 标准。
- [Ollama](https://ollama.com) 和 [LM Studio](https://lmstudio.ai) 社区推动本地 AI 工具发展。

由 [ifq.ai](https://ifq.ai) 用心打造。
