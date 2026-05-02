# Security Policy

English | [简体中文](#安全政策)

## Supported Versions

| Version | Supported |
| --- | --- |
| 0.2.x | Yes |
| < 0.2 | No |

## Reporting a Vulnerability

If you discover a security vulnerability in AI-Movie-Player, please report it responsibly.

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
- The AI-Movie-Player application code
- Build and release infrastructure
- Dependencies with known security implications

This policy does not cover:
- Third-party AI provider APIs (OpenAI, Ollama, etc.)
- TMDB API security
- User's local environment configuration

## Security Considerations

- API keys are stored locally in the SQLite database. The database file is not encrypted by default.
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

如果您发现 AI-Movie-Player 的安全漏洞，请负责任地报告。

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
