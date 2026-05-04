# 首个正式版本发布包 v0.2.1

[English](release-package-v0.2.1.md) | 简体中文

这是 AI-Movie-Player 首个正式 GitHub Releases 发布包文案。

## 发布标题

`AI-Movie-Player v0.2.1`

## 发布正文

```md
AI-Movie-Player 是一个面向本地片库的 AI 原生电影助手，目标是让元数据、字幕、推荐、系统播放器调用和电影对话都变得自然、优雅、真正有用。

### 本次亮点
- 新增 AI 观影工作流：观影前 briefing、观影后复盘、双片连看建议。
- 电影详情页新增本地文件打开动作，通过系统默认播放器播放。
- AI Companion 从简单 prompt 快捷入口升级为带结构化卡片的专用工作流面板。
- README、贡献指南、更新日志和发布说明已拆成英文主文档 + 中文配套文档。
- 新增 GitHub Issue 模板和更完整的开源发布面说明。
- 继续清理应用内部更深层的双语状态提示、空状态和系统文案。

### 平台软件包
- Windows 软件包：`AI-Movie-Player-v0.2.1-windows-x64.zip`
- macOS 软件包：`AI-Movie-Player-v0.2.1-macOS-ARCH.tar.gz`，内含 `AI-Movie-Player.app`
- Linux 软件包：`AI-Movie-Player-v0.2.1-linux-x64.tar.gz`

### 升级说明
- 旧用户会自动保留历史数据与数据库兼容能力。
- AI 功能仍需配置 OpenAI-compatible 提供方。
- 这个版本重点在于观影工作流、产品语言和发布准备度，而不是底层架构迁移。

### 反馈入口
- Issues: https://github.com/peixl/AI-Movie-Player/issues
- Discussions: https://github.com/peixl/AI-Movie-Player/discussions

Made with care by ifq.ai.
```

## 截图顺序

建议在 release 页面按这个顺序放图：

1. 带真实片库的海报墙。
2. 选中影片后的 AI Companion 多轮对话。
3. 展示观影前 briefing 或观影后复盘的 Workflow Studio 结构化卡片。
4. AI 推荐页。
5. 带元数据、Open 动作和 AI 入口的电影详情页。
6. 字幕搜索流程。

## 简版升级说明

如果需要更短的升级说明，可以直接使用下面这段：

```md
### 升级说明
- 保留旧数据兼容。
- 新增结构化 AI 工作流卡片与系统播放器调用入口。
- 改进更深层的中英双语产品文案与发布面。
- 持续支持 OpenAI、Ollama、LM Studio 等 OpenAI-compatible 提供方。
```

## 公告文案

### 短公告

```text
AI-Movie-Player v0.2.1 已发布。

这次更新把产品往三个方向推进了一步：结构化 AI 观影工作流、更成熟的中英双语文档，以及更完整的 GitHub 发布面。

现在已经加入：观影前 briefing、观影后复盘、双片连看建议、本地影片系统播放器调用入口，以及 Windows + macOS + Linux 三平台打包发布。

GitHub: https://github.com/peixl/AI-Movie-Player
by ifq.ai
```

### 长公告

```text
AI-Movie-Player v0.2.1 是第一个开始接近我们真正想做的产品状态的版本。

它仍然是一个本地片库电影助手，但 AI 部分已经更进一步：不再只有 prompt 快捷入口，而是能生成结构化的观影工作流卡片，用于观影前导览、观影后复盘和双片连看建议；详情页也可以直接调用系统默认播放器打开本地影片。

这次我们也进一步整理了仓库首页和开源协作面，包括英文主文档、中文配套文档、Issue 模板、发布说明，以及更清晰的 Windows、macOS 与 Linux 三平台软件包交付路径。

目标依然没有变：把它做成一个安静高级、AI 原生、既让用户喜欢也让开发者尊重的本地电影工具。

项目地址：https://github.com/peixl/AI-Movie-Player
Made with care by ifq.ai
```