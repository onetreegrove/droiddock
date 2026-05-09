---
name: droiddock-preview-gap-review
description: 当需要对照 DroidDock 产品需求、UI 预览文档或计划流程检查当前 Vue 实现时使用。
---

# DroidDock 预览稿缺口检查

## 必读上下文

判断缺口前先读取相关内容：

- `AGENTS.md`
- `docs/product-requirements.md`
- `docs/droiddock_ui_preview.html`
- `docs/ui-design.md`
- `src/App.vue`
- `src/components/`
- `src/stores/app.ts`
- `src/domain/`

## 检查方法

1. 从 PRD 和预览文档中提取期望用户流程。
2. 将每个流程映射到当前路由、组件、store 动作和后端调用。
3. 对缺口分类：
   - 行为缺失。
   - 行为已存在，但 UI 入口或提示缺失。
   - UI 已存在，但状态或动作串联不完整。
   - 文案或错误处理不匹配。
   - 验证缺口。
4. 按产品影响排序，而不是按代码风格排序。
5. 如果用户要求补齐缺口，优先实现最小但完整的一段，并完成验证。

## DroidDock 重点检查流程

- 首次启动工具检测与安装引导。
- USB 设备发现，以及 unauthorized/offline 状态处理。
- 使用 `adb tcpip` 和 `adb connect` 的 USB 转无线流程。
- Android 11+ 无线配对流程，必须区分配对端口和连接端口。
- 已配对设备重连，且支持编辑连接端口。
- 全局、设备、会话临时 scrcpy 参数。
- 多设备会话列表、停止/重连动作和日志查看。
- 手动配置 adb/scrcpy 路径。

## 输出格式

先按严重程度列出问题。每个问题包含：

- 期望行为来源。
- 当前实现证据。
- 用户影响。
- 建议的最小修复或验证方式。

如果没有发现问题，明确说明，并列出剩余测试风险。
