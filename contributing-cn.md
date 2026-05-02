# 参与 AI-Movie-Player 开发

[English](CONTRIBUTING.md) | 简体中文

欢迎贡献代码、产品设计、AI 工作流、界面打磨、文档改进和工程质量优化。AI-Movie-Player 不是要堆一堆响亮但空洞的功能，而是要把电影播放器这件事做得更完整、更安静、更值得长期使用。

## 开始之前

```bash
git clone https://github.com/peixl/AI-Movie-Player.git
cd AI-Movie-Player
cargo run
```

## 开发流程

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

如果当前环境无法访问 crates.io，优先使用本地诊断或离线缓存，并在验证说明中写清限制。

## 工程标准

- 除非失败在逻辑上真正不可能且有明确说明，否则不要在生产路径直接使用 `unwrap()`。
- 优先使用清晰一致的 `Result` 错误返回，而不是静默失败。
- 正则模式要复用静态初始化，避免重复编译。
- 异步或阻塞任务不要压在 UI 线程上。
- 数据库访问继续保持参数化与显式控制。
- 在 egui 面板里避免大规模片库场景下的无意义重绘。
- 新 API 与新界面流程要保持克制、清楚、易扩展。

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

1. 建立聚焦分支。
2. 保持改动有明确意图，不要顺手清理无关内容。
3. 行为变化时同步补充或更新测试。
4. 清楚说明用户层面的影响。
5. 关联相关 issue、工作流想法或讨论。
6. 本地检查完成后再发起评审。

## Issues 与 Discussions

- 缺陷、功能需求、AI 工作流想法请优先使用仓库的 Issue 模板。
- 如果话题仍处于探索阶段、对比阶段或还不适合实施，请优先发到 GitHub Discussions。

## 产品方向

AI-Movie-Player 追求的是“安静的高级感”。当你修改界面、文案或 AI 行为时，优先选择更自然、更克制、更可信的方案。