# 新手任务包

[English](starter-issues.md) | 简体中文

这份任务包把路线图拆成一组小而具体的 issue，方便新贡献者在不需要私有媒体文件、API Key 或大量架构背景的情况下开始参与。

建议根据任务使用 `good first issue`、`help wanted`、`documentation`、`testing`、`ai`、`metadata`、`subtitles`、`ui`、`release` 等标签。

## 推荐种子 Issue

| Issue 标题 | 标签 | 范围 | 建议验证 |
| --- | --- | --- | --- |
| 给 README 和 release notes 添加真实截图 | `good first issue`, `documentation`, `release` | 用一个小型演示片库截图：海报墙、电影详情、AI Companion、AI Taste Engine 和字幕搜索。 | 确认 README 预览和 release 草稿中的图片路径正常渲染。 |
| 为 filename parser 补充常见发布命名测试 | `good first issue`, `testing`, `metadata` | 增加包含年份、分辨率、来源、编码和标点变体的聚焦测试。 | `cargo test filename_parser --locked` |
| 补充 TMDB Key 首次配置文档 | `good first issue`, `documentation` | 改进 README/wiki 中创建和保存 TMDB Key 的上手流程。 | Markdown 预览和链接检查。 |
| 改进字幕源排序说明 | `help wanted`, `subtitles`, `documentation` | 说明当前如何选择字幕，以及未来排序应考虑哪些信号。 | Markdown 预览。 |
| 增加 Ollama 和 LM Studio 诊断示例 | `good first issue`, `ai`, `documentation` | 添加可复制的 curl 检查命令，验证本地 OpenAI-compatible endpoint。 | 对本地提供方执行命令，或标记为 docs-only。 |
| 为 AI 流式解析补充边界测试 | `help wanted`, `ai`, `testing` | 覆盖空 chunk、`[DONE]`、provider error 和不完整 SSE 行。 | `cargo test ai --locked` |
| 补充大体量片库海报墙 smoke test 说明 | `help wanted`, `ui`, `documentation` | 说明测试数百部本地电影时应该观察哪些性能和交互表现。 | wiki 中有明确手动检查清单。 |
| 增加 Windows PowerShell 校验和验证示例 | `good first issue`, `release`, `documentation` | 在已有 Linux/macOS 命令之外补充 Windows 校验方式。 | 验证 PowerShell 命令语法。 |
| 做一次键盘导航和标签可访问性检查 | `help wanted`, `ui` | 检查主导航和常用控件的标签、焦点和可读性。 | 手动 smoke 说明，必要时附截图。 |
| 起草系统凭据存储设计说明 | `help wanted`, `security`, `architecture` | 在实现前比较 macOS Keychain、Windows Credential Manager 和 Linux Secret Service 的约束。 | 维护者审核设计说明。 |

## Issue 模板

```md
## Goal / 目标

描述这个小任务要改善的用户或维护者问题。

## Scope / 范围

- 可能改动的文件或工作流。
- 明确哪些内容不在本任务范围内。

## Acceptance Criteria / 验收标准

- 可观察的具体结果。
- 文档、测试或截图要求。

## Validation / 验证

- 命令或手动 smoke check。
```

## 选择规则

- 好的新手任务通常只改一两个文件。
- 不应要求真实 API Key、私有影片文件或大型本地片库。
- 应包含清晰的验证路径。
- 应改善可信度、上手清晰度、测试覆盖或可见用户流程。
- 更宽的设计问题更适合 Discussions 或 `help wanted` issue，而不是 `good first issue`。
