# AI-Movie-Player

一个面向电影爱好者的 AI 原生播放器，不只是打开文件，更帮助你理解和经营自己的片库。

[English](README.md) | 简体中文

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/peixl/AI-Movie-Player/releases)
[![CI](https://github.com/peixl/AI-Movie-Player/actions/workflows/ci.yml/badge.svg)](https://github.com/peixl/AI-Movie-Player/actions/workflows/ci.yml)
[![AI](https://img.shields.io/badge/AI-OpenAI%20Compatible-6366f1.svg)](https://github.com/peixl/AI-Movie-Player#ai-features)

项目由 [ifq.ai](https://ifq.ai) 打造，开源仓库位于 [peixl/AI-Movie-Player](https://github.com/peixl/AI-Movie-Player)。

## 项目简介

AI-Movie-Player 是一个基于 Rust 与 egui 构建的桌面电影播放器与片库助手。它把本地片库管理、TMDB 元数据、字幕工作流、海报墙浏览和 OpenAI-compatible AI 能力整合在一个更安静、更自然的电影体验里。

这个项目想做的不是“给播放器加一个 AI 按钮”，而是把“选片、看片、看后理解、片库管理”这整个链路做得更完整、更优雅。

## 为什么是 AI-Movie-Player

- 针对单部电影的 AI 对话，并且支持真正的多轮上下文记忆。
- 基于你自己的片库生成更有依据的推荐，而不是空泛的猜你喜欢。
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

AI-Movie-Player 兼容：

- OpenAI
- Azure OpenAI
- Ollama
- LM Studio
- 任意 OpenAI-compatible 接口

### 观影工作流

AI-Movie-Player 现在开始把 AI 能力做成更自然的观影流程，而不是零散问答：

- 观影前 briefing：在播放前给出情绪、背景和值得留意的细节。
- 观影后复盘：在片尾之后帮助用户整理主题、结构和关键选择。
- 双片连看建议：推荐一部真正能放大第一部电影价值的搭配影片，而不是偷懒的同类型相似片。

## 核心模块

| 模块 | 说明 |
| --- | --- |
| 片库 | 扫描文件夹、识别影片文件、避免重复导入，管理个人收藏。 |
| 元数据 | 通过 TMDB 补全标题、海报、演员、评分与简介。 |
| AI | 提供电影对话、AI 解析、观影画像和推荐流程。 |
| 字幕 | 从多个来源搜索并下载适合本地媒体的字幕。 |
| 海报墙 | 用更直观的视觉方式浏览收藏。 |
| 片单 | 管理接下来准备看的影片。 |

## 截图规划

推荐在 GitHub 首页展示这样一组截图：

| 顺序 | 建议内容 | 目的 |
| --- | --- | --- |
| 1 | 海报墙与较完整的本地片库 | 建立“这不是玩具，而是认真工具”的第一印象。 |
| 2 | 选中影片后的 AI Companion 对话 | 证明产品是 AI 原生，而不是贴一个 AI 按钮。 |
| 3 | AI 推荐页 | 展示基于片库的智能理解能力。 |
| 4 | 电影详情页与 AI 入口 | 展示浏览、元数据和 AI 流程是连在一起的。 |
| 5 | 字幕搜索与下载流程 | 证明它也解决真实观影里的实用问题。 |

更完整的截图和发布说明见 [docs/github-launch-kit-cn.md](docs/github-launch-kit-cn.md)。

## 与普通播放器相比

| 能力 | 普通播放器 | AI-Movie-Player |
| --- | --- | --- |
| 打开本地文件 | 支持 | 支持 |
| TMDB 元数据补全 | 有时支持 | 内建 |
| 围绕单部电影的 AI 对话 | 少见 | 原生工作流 |
| 基于片库的推荐 | 少见 | 内建 |
| 观影工作流 | 基本没有 | 观影前、观影后、双片连看 |
| 字幕工作流 | 基础能力 | 更偏向搜索与下载闭环 |
| 产品气质 | 工具导向 | 电影导向，安静高级 |

## 快速开始

### 环境要求

- Rust 1.85+
- 一个来自 [themoviedb.org/settings/api](https://www.themoviedb.org/settings/api) 的 TMDB API Key
- 可选：任意 OpenAI-compatible API Key

### 克隆并运行

```bash
git clone https://github.com/peixl/AI-Movie-Player.git
cd AI-Movie-Player
cargo run --release
```

### Releases

可以在 [Releases](https://github.com/peixl/AI-Movie-Player/releases) 页面发布并下载 Windows 与 macOS 两个平台的软件包。

仓库现在已经补上了面向两个主要桌面平台的发布工作流：

- Windows 软件包：`.zip`
- macOS 软件包：包含 `.app` bundle 的 `.tar.gz`

首个正式版本的完整发布文案包见 [docs/release-package-v0.2.1-cn.md](docs/release-package-v0.2.1-cn.md)。

在依赖可用后，也可以本地执行打包脚本：

- Windows：`pwsh ./scripts/package-windows.ps1`
- macOS：`bash ./scripts/package-macos.sh`

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
│   ├── main.rs
│   ├── app.rs
│   ├── ai/
│   ├── api/
│   ├── core/
│   ├── db/
│   ├── thumbnail/
│   ├── ui/
│   ├── config/
│   └── util/
├── README.md
├── readme-cn.md
├── CONTRIBUTING.md
└── CHANGELOG.md
```

## 开发

```bash
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings
cargo build --release
```

如果当前环境无法访问 crates.io，建议优先使用本地诊断或离线缓存做验证。

## 文档

- 英文文档： [README.md](README.md)
- 中文文档： [readme-cn.md](readme-cn.md)
- 英文贡献指南： [CONTRIBUTING.md](CONTRIBUTING.md)
- 中文贡献指南： [contributing-cn.md](contributing-cn.md)
- 英文更新日志： [CHANGELOG.md](CHANGELOG.md)
- 中文更新日志： [changelog-cn.md](changelog-cn.md)
- GitHub 英文发布说明： [docs/github-launch-kit.md](docs/github-launch-kit.md)
- GitHub 中文发布说明： [docs/github-launch-kit-cn.md](docs/github-launch-kit-cn.md)
- 首个正式版本发布包： [docs/release-package-v0.2.1-cn.md](docs/release-package-v0.2.1-cn.md)

## 路线图

- 继续把 AI 观影工作流做得更像观影的一部分，而不是看完后才补一句聊天。
- 提升字幕质量排序和来源可靠性。
- 进一步优化大体量片库下的海报墙性能与观感。
- 完善跨平台打包与发布流程。
- 加强本地 AI 提供方与自托管接口的上手体验。

## FAQ

### 这是流媒体应用吗？

不是。AI-Movie-Player 主要围绕本地片库和个人媒体工作流构建。

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

由 [ifq.ai](https://ifq.ai) 用心打造。