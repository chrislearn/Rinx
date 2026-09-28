# Rinx

[English](README.md) | 简体中文

Rinx 是原生 Matrix 客户端，包含聊天、联系人、发现、朋友圈和文章编辑器，支持独立运行及作为 OctoSense 原生模块运行。完整构建与兼容性说明见 [英文文档](README.md)。

## 内置应用

`system-apps.json` 定义随 Rinx 发布的原生和 OctoScript 应用。原生文章编辑器位于 `apps/article-editor/`；共享文档和 Makepad 控件库保留在 `crates/`。

独立版由 Rinx 管理 Octos 运行时和提供商配置；托管版使用 OctoSense 注入的服务、共享内核与配置。小程序通过受限的宿主接口访问 Matrix 和 Octos，不启动自己的内核。

应用目录构建验证、开发和发布流程见 [应用开发指南](apps/README.zh-CN.md)；架构决策见 [ADR 0008](docs/adr/0008-rinx-system-app-catalog.md)。
