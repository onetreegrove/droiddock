# DroidDock 架构说明

DroidDock 采用 Tauri 2 + Vue 3 + Pinia + Rust。当前架构目标是把设备连接、工具管理、投屏会话和 UI 状态拆成明确边界，避免前后端都依赖单个大文件承载全部业务。

## 后端模块

后端源码位于 `src-tauri/src/`，Tauri 命令仍由 `lib.rs` 统一注册，对内委托到领域模块。

- `lib.rs`：Tauri 命令入口、应用状态、命令注册和少量编排逻辑。
- `config.rs`：应用配置模型、默认值、原子写入、配置 schema version、损坏配置备份。
- `command.rs`：外部命令执行封装，统一收集 stdout/stderr，并支持超时和 stdin 输入。
- `devices.rs`：`adb devices -l` 解析、设备状态建模和设备列表刷新。
- `wireless.rs`：USB 转无线、无线连接、断开、配对后直连。
- `sessions.rs`：`scrcpy` 子进程生命周期、会话状态、日志缓冲和 `session-log` 事件推送。
- `scrcpy.rs`：投屏参数模型和命令行参数生成。
- `tools.rs`：`adb`、`scrcpy` 检测、下载、安装、校验。
- `tool_manifest.rs`：工具下载来源和资产选择规则。
- `error.rs`：结构化错误模型和外部命令错误翻译。

后端新增的通用约束：

- 设备刷新、无线连接和无线配对等 adb 操作必须设置超时，避免 UI 长时间无反馈。
- 需要写入磁盘的配置必须走 `save_config_atomic`。
- 配置文件读取失败时保留 `.bak-<timestamp>` 备份，并回退默认配置。
- `scrcpy` 会话日志由后端缓冲 400 行，同时通过 Tauri 事件推送给前端。

## 前端模块

前端源码位于 `src/`，状态管理从单一 `app` store 拆分为资源 store 和 UI store。

- `src/lib/ipc/client.ts`：Tauri `invoke` 的统一封装。
- `src/lib/ipc/errors.ts`：后端错误归一化，向 UI 暴露 `userMessage`、`technicalDetail` 和 `retryable`。
- `src/lib/ipc/types.ts`：与后端 IPC 直接对应的 DTO 类型。
- `src/stores/config.ts`：配置、全局参数、设备参数。
- `src/stores/tools.ts`：工具检测和安装。
- `src/stores/devices.ts`：设备列表、设备选择派生状态。
- `src/stores/sessions.ts`：投屏会话、会话日志、会话参数草稿、日志事件监听。
- `src/stores/ui.ts`：页面、弹窗、选中设备、选中日志会话。
- `src/stores/app.ts`：兼容门面，保留既有组件调用入口，并把真实状态委托给资源 store。

`src/types/app.ts` 保留前端领域类型，例如 `ScrcpyOptions`、预设、UI 页面枚举；后端 DTO 类型应优先放在 `src/lib/ipc/types.ts`。

## IPC 边界

前端组件不直接调用 `@tauri-apps/api/core`。新增 IPC 调用应按以下路径接入：

1. 在 Rust 模块中实现领域函数。
2. 在 `src-tauri/src/lib.rs` 暴露 Tauri command，并注册到 `generate_handler!`。
3. 在 `src/lib/ipc/types.ts` 补齐请求或响应 DTO。
4. 在对应资源 store 中通过 `invokeCommand` 调用。
5. 组件消费 store 状态或 action。

## 投屏参数

投屏参数由三层合并：

1. 全局默认参数：`AppConfig.default_scrcpy_options`
2. 设备级参数：`AppConfig.device_scrcpy_options[serial]`
3. 单次会话草稿：`sessionsStore.sessionDraftOptions[serial]`

前端合并逻辑位于 `src/domain/scrcpyOptions.ts`。后端参数生成逻辑位于 `src-tauri/src/scrcpy.rs`，`preview_scrcpy_args` 命令用于校验前后端参数理解一致。

## 验证命令

常规改动至少运行：

```bash
npm run test
npm run build
cd src-tauri && cargo test
```

涉及 Tauri 打包或原生资源时，再运行：

```bash
npm run tauri:build:app
```
