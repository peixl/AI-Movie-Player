# 参与 AI-Movie-Player 开发

[English](CONTRIBUTING.md) | 简体中文

欢迎贡献代码、产品设计、AI 工作流、界面打磨、文档改进和工程质量优化。AI-Movie-Player 不是要堆一堆响亮但空洞的功能，而是要把本地电影片库和观影助手这件事做得更完整、更安静、更值得长期使用。

## 开始之前

### 环境要求

- Rust 1.85+（通过 [rustup](https://rustup.rs) 安装）
- 一个来自 [themoviedb.org/settings/api](https://www.themoviedb.org/settings/api) 的 TMDB API Key（元数据功能需要）
- 可选：一个 OpenAI-compatible API Key（AI 功能需要）

### 设置

```bash
git clone https://github.com/peixl/AI-Movie-Player.git
cd AI-Movie-Player
cargo build
```

### 运行

```bash
cargo run --release
```

## 开发流程

提交 PR 前，请运行完整的检查套件：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

如果当前环境无法访问 crates.io，优先使用本地诊断或离线缓存，并在验证说明中写清限制。

## 架构概览

代码库采用分层架构：

```text
UI 层 (egui)        -> src/ui/, src/app.rs
    |
核心层              -> src/core/ (模型, 解析器)
    |
API 层              -> src/api/ (AI, TMDB, 字幕)
    |
数据库层            -> src/db/ (SQLite, FTS5)
```

关键设计决策：
- **即时模式 UI**：egui 每帧重绘。避免在 UI 代码中做昂贵操作。
- **Tokio 异步**：网络请求（AI、TMDB）在 Tokio 运行时上执行，不阻塞 UI 线程。
- **流式 AI**：解析 Server-Sent Events (SSE) 实现 OpenAI 兼容的流式响应。
- **LRU 纹理缓存**：海报图片使用有界 HashMap + VecDeque 缓存。
- **SQLite WAL 模式**：并发读、单写、FTS5 全文检索。

## 工程标准

- 除非失败在逻辑上真正不可能且有明确说明，否则不要在生产路径直接使用 `unwrap()`。
- 优先使用清晰一致的 `Result` 错误返回，而不是静默失败。
- 正则模式要复用静态初始化，避免重复编译。
- 异步或阻塞任务不要压在 UI 线程上。
- 数据库访问继续保持参数化与显式控制。
- 在 egui 面板里避免大规模片库场景下的无意义重绘。
- 新 API 与新界面流程要保持克制、清楚、易扩展。

## 提交规范

本项目遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```text
<type>(<scope>): <description>

[optional body]
[optional footer(s)]
```

类型：
- `feat`：新功能
- `fix`：Bug 修复
- `docs`：仅文档变更
- `style`：格式化、缺少分号等
- `refactor`：既不修复 bug 也不添加功能的代码变更
- `perf`：性能改进
- `test`：添加或修正测试
- `chore`：维护任务

示例：
```text
feat(ai): add taste-aware recommendation engine
fix(db): handle FTS5 migration for existing databases
docs(readme): add architecture diagram
perf(ui): cache poster textures across frames
```

## 分支命名

使用描述性的分支名称：

```text
feature/ai-double-feature-pairing
fix/subtitle-encoding-detection
docs/contributing-guide-update
refactor/extract-theme-helpers
```

## AI 贡献标准

- 提示词要自然、克制、具体、真正有帮助。
- 当信息不确定时，AI 不能编造事实。
- 推荐逻辑要尽量基于证据，而不是泛泛的类型猜测。
- ifq.ai 的署名应该保持自然克制，不做硬广式曝光。
- 新 AI 能力应优先构造成自然的观影工作流，而不是噱头功能。

## 文档与语言规则

- 顶层面向读者的文档按语言拆分维护，不再使用长篇混排双语。
- `README.md`、`CONTRIBUTING.md`、`CHANGELOG.md` 作为英文主入口。
- 中文内容放在独立配套文件中，例如 `readme-cn.md`、`contributing-cn.md`、`changelog-cn.md`。
- 应用内面向用户的界面文案在合适场景下继续保持中英双语。

## 测试要求

- 单元测试尽量靠近对应实现。
- 数据库或文件系统测试请使用隔离的临时目录。
- 提交 PR 前至少完成一次聚焦验证。
- 如果改动了 AI 提示词或体验文案，要在 PR 中说明预期的用户影响。

## Pull Request 要求

1. 从 `main` 建立聚焦分支。
2. 保持改动有明确意图，不要顺手清理无关内容。
3. 行为变化时同步补充或更新测试。
4. 清楚说明用户层面的影响。
5. 关联相关 issue、工作流想法或讨论。
6. 本地检查完成后再发起评审。

提交 PR 时请使用 [PR 模板](.github/PULL_REQUEST_TEMPLATE.md)。

## Issues 与 Discussions

- 缺陷、功能需求、AI 工作流想法请优先使用仓库的 Issue 模板。
- 如果话题仍处于探索阶段、对比阶段或还不适合实施，请优先发到 GitHub Discussions。

## 好的首次贡献

查找标记为 [`good first issue`](https://github.com/peixl/AI-Movie-Player/labels/good%20first%20issue) 的 issue 开始贡献。这些问题的范围适合新贡献者。

## 产品方向

AI-Movie-Player 追求的是"安静的高级感"。当你修改界面、文案或 AI 行为时，优先选择更自然、更克制、更可信的方案。
