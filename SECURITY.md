# Security Policy

English | [简体中文](#安全政策)

## Supported Versions

| Version | Supported |
| --- | --- |
| 0.2.x | Yes |
| < 0.2 | No |

## Reporting a Vulnerability

If you discover a security vulnerability in AI Movie Player, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please email: **security@ifq.ai**

### What to include

- A description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if you have one)

### Response timeline

- **Acknowledgment**: within 48 hours
- **Initial assessment**: within 5 business days
- **Fix or mitigation**: within 30 days for confirmed vulnerabilities

### Scope

This policy covers:
- The AI Movie Player application code
- Build and release infrastructure
- Dependencies with known security implications

This policy does not cover:
- Third-party AI provider APIs (OpenAI, Ollama, etc.)
- TMDB API security
- User's local environment configuration

## Security Considerations

- API keys are stored locally in the SQLite database. The database file is not encrypted by default.
- System credential storage is planned for a future 0.2.x/0.3.x release, targeting macOS Keychain, Windows Credential Manager, and Linux Secret Service where available.
- Until that migration lands, avoid sharing your application data directory and remove saved keys from Settings before publishing logs, database files, screenshots, or support bundles.
- Network requests are made to TMDB and configured AI endpoints. No telemetry or analytics are collected.
- The application runs with user-level permissions and does not require elevated access.

---

# 安全政策

## 支持的版本

| 版本 | 支持状态 |
| --- | --- |
| 0.2.x | 支持 |
| < 0.2 | 不支持 |

## 报告漏洞

如果您发现 AI Movie Player 的安全漏洞，请负责任地报告。

**请不要在公开的 GitHub Issue 中报告安全漏洞。**

请发送邮件至：**security@ifq.ai**

### 请包含以下内容

- 漏洞描述
- 复现步骤
- 潜在影响
- 修复建议（如有）

### 响应时间

- **确认收到**：48 小时内
- **初步评估**：5 个工作日内
- **修复或缓解**：确认的漏洞在 30 天内

### 范围

本政策覆盖：
- AI Movie Player 应用代码
- 构建与发布基础设施
- 有安全影响的依赖项

本政策不覆盖：
- 第三方 AI 提供方 API（OpenAI、Ollama 等）
- TMDB API 安全
- 用户本地环境配置

## 安全注意事项

- API Key 当前存储在本地 SQLite 数据库中，数据库默认不加密。
- 后续 0.2.x/0.3.x 版本计划接入系统凭据存储，在可用平台上优先使用 macOS Keychain、Windows Credential Manager 与 Linux Secret Service。
- 在该迁移完成前，请避免分享应用数据目录；发布日志、数据库文件、截图或支持包之前，请先在 Settings 中移除保存的密钥。
- 应用会请求 TMDB 和用户配置的 AI endpoint。项目不收集 telemetry 或 analytics。
- 应用以普通用户权限运行，不需要提权访问。
