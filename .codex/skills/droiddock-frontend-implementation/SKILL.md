---
name: droiddock-frontend-implementation
description: 当需要实现或修改 DroidDock 的 Vue、Pinia、TypeScript 领域逻辑、设备流程、scrcpy 参数、会话、设置或应用 UI 时使用。
---

# DroidDock 前端实现

## 必读上下文

先读取相关文件：

- `AGENTS.md`
- `docs/product-requirements.md`
- `src/App.vue`
- `src/components/`
- `src/domain/`
- `src/stores/app.ts`
- `src/types/app.ts`
- `src/styles.css`

## 实现规则

- 命令编排放在 `src/stores/app.ts` 或领域 helper 中，不要把 Tauri invoke 细节散落在展示组件里。
- 可复用业务逻辑放在 `src/domain/`，并用聚焦的 Vitest 测试覆盖。
- 保持参数层级：全局默认值、设备覆盖值、会话临时参数。
- 区分 pair 和 connect 概念。配对端口和连接端口是不同字段。
- 已配对设备重连必须允许直接编辑连接端口，不强制重新配对。
- UI 文案应使用面向非技术用户的实用中文。
- 添加新抽象前，优先复用现有组件和 CSS 模式。

## 验证

使用能证明改动的最小检查：

```bash
npm run test
npm run build
```

如果改动偏 UI，条件允许时运行应用或预览，并检查受影响页面。

## 常见错误

- 把配对成功当成连接成功。
- 持久化一次性的配对码。
- 假设系统 `PATH` 中一定存在 `adb` 或 `scrcpy`。
- 直接展示原始命令错误，却不给用户可执行的中文解释。
- 用当前设备默认值渲染会话卡片，而不是使用已记录的会话参数。
