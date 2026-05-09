# DroidDock 项目上下文

DroidDock 是一个专为 macOS（针对 Apple Silicon 优化）设计的桌面应用，旨在提供 Android 设备的屏幕投屏与控制功能。本项目采用 Tauri 框架开发，前端使用 Vue 3 + TypeScript，后端使用 Rust。

## 核心指令

- **语言要求：AI 在本项目的后续所有对话和响应中，必须严格使用中文。**

## 项目概览

- **用途：** 一个用户友好的控制台，用于管理 Android 连接（USB 和无线），并启动高性能的 `scrcpy` 投屏。
- **核心技术栈：**
  - **前端：** Vue 3, Pinia (状态管理), TypeScript, Vite。
  - **后端：** Tauri (v2), Rust。
  - **外部工具：** `adb` (Android Debug Bridge) 和 `scrcpy`。
- **目标平台：** macOS (aarch64/Apple Silicon)。

## 架构设计

### 后端 (Rust - `src-tauri/`)
- **命令入口：** `src-tauri/src/lib.rs` 只负责 Tauri command 注册、应用状态和少量编排。
- **模块边界：** 设备、无线连接、会话、工具、配置、命令执行、错误翻译、`scrcpy` 参数分别拆分在独立模块中。
- **工具管理：** 自动检测或安装适配 Apple Silicon 的 `adb` 和 `scrcpy` 二进制文件，下载来源集中在 `tool_manifest.rs`。
- **会话管理：** 将 `scrcpy` 作为子进程启动，监控其生命周期，并将 `stdout`/`stderr` 日志通过 `session-log` 事件实时推送至前端。
- **配置管理：** 配置持久化在 `~/Library/Application Support/DroidDock/config.json`，读取失败会备份损坏文件并回退默认配置。
- **外部命令：** adb 等外部命令应通过 `command.rs` 执行，并根据场景设置超时。

### 前端 (Vue 3 - `src/`)
- **IPC 层：** `src/lib/ipc/client.ts` 统一封装 Tauri `invoke`，`src/lib/ipc/errors.ts` 统一归一化后端错误。
- **类型边界：** `src/lib/ipc/types.ts` 存放后端 DTO，`src/types/app.ts` 存放前端领域类型。
- **状态管理：** Pinia 按资源拆分为 `config`、`tools`、`devices`、`sessions`、`ui` store；`app` store 保留为兼容门面。
- **UI 组件：** 模块化组件，用于设备列表显示、会话监控、日志查看及连接配置弹窗。
- **领域逻辑：** 处理全局、设备特定及会话特定 `scrcpy` 参数的合并逻辑。

## 构建与运行

### 前置条件
- **Node.js：** 24+
- **npm：** 11+
- **Rust：** Stable 版本
- **Xcode Command Line Tools**

### 开发常用命令
- **安装依赖：** `npm install`
- **启动开发模式 (Tauri + Vite)：** `npm run tauri:dev`
- **纯前端开发：** `npm run dev`
- **运行前端测试：** `npm run test` (使用 Vitest)
- **运行后端测试：** `cd src-tauri && cargo test`

### 生产构建
- **完整构建 (DMG)：** `npm run tauri:build`
- **仅构建 App 包：** `npm run tauri:build:app`

## 核心目录结构

- `src/`：Vue 前端源码。
  - `components/`：UI 组件库。
  - `stores/`：Pinia 状态定义，按配置、工具、设备、会话和 UI 状态拆分。
  - `lib/ipc/`：Tauri IPC 调用封装、错误归一化和 DTO 类型。
  - `domain/`：业务逻辑（如 `scrcpy` 参数合并）。
  - `types/`：前端领域类型定义。
- `src-tauri/`：Rust 后端源码。
  - `src/lib.rs`：Tauri 指令入口和模块编排。
  - `src/config.rs`：配置模型、原子写入和损坏配置恢复。
  - `src/command.rs`：外部命令执行、超时和输出收集。
  - `src/devices.rs`：adb 设备列表解析。
  - `src/wireless.rs`：无线调试连接、断开和配对。
  - `src/sessions.rs`：scrcpy 会话和日志事件。
  - `src/tools.rs`：工具检测、安装和校验。
- `docs/`：完善的项目文档（需求文档、UI 设计、调研报告等）。
  - `architecture.md`：当前架构边界和开发约束。

## 开发规范

- **Tauri 指令：** 所有后端逻辑应通过 `src-tauri/src/lib.rs` 中的 Tauri 指令暴露，业务实现放入对应领域模块。
- **IPC 调用：** 前端新增 Tauri 调用优先放在对应资源 store 中，并通过 `invokeCommand` 进入后端。
- **错误处理：** 后端命令错误应优先使用 `error.rs` 中的结构化错误翻译，前端通过 `normalizeIpcError` 展示用户友好消息。
- **样式：** 主要采用原生 CSS (`src/styles.css`)。
- **二进制文件管理：** 应用在 `~/Library/Application Support/DroidDock/tools/` 目录下管理其依赖工具。不应假设系统 PATH 中已存在 `adb` 或 `scrcpy`。
- **会话日志：** `scrcpy` 日志在后端进行缓冲（上限 400 行），并通过 `session-log` 事件推送；前端仍可按需拉取历史日志。
