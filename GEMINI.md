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
- **工具管理：** 自动检测或安装适配 Apple Silicon 的 `adb` 和 `scrcpy` 二进制文件。
- **会话管理：** 将 `scrcpy` 作为子进程启动，监控其生命周期，并将 `stdout`/`stderr` 日志实时流式传输至前端。
- **指令系统：** 提供丰富的 Tauri `invoke` 句柄，用于：
  - 设备列表查询 (`adb devices`)。
  - 无线连接管理 (`adb connect`, `adb pair`, `adb tcpip`)。
  - 会话控制（使用特定参数启动/停止 `scrcpy`）。
  - 配置持久化（存储于 `~/Library/Application Support/DroidDock/config.json`）。

### 前端 (Vue 3 - `src/`)
- **状态管理 (`src/stores/app.ts`)：** 使用 Pinia 跟踪工具状态、设备列表、活跃会话及应用配置。
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
  - `stores/`：Pinia 状态定义。
  - `domain/`：业务逻辑（如 `scrcpy` 参数合并）。
  - `types/`：与后端共享的 TypeScript 接口定义。
- `src-tauri/`：Rust 后端源码。
  - `src/lib.rs`：核心逻辑、Tauri 指令实现及工具管理。
- `docs/`：完善的项目文档（需求文档、UI 设计、调研报告等）。

## 开发规范

- **Tauri 指令：** 所有后端逻辑应通过 `src-tauri/src/lib.rs` 中的 Tauri 指令暴露，并在前端通过 `src/stores/app.ts` 的 `useAppStore` 调用。
- **错误处理：** 后端错误应使用 `lib.rs` 中的 `translate_error` 工具函数翻译为用户友好的中文消息。
- **样式：** 主要采用原生 CSS (`src/styles.css`)。
- **二进制文件管理：** 应用在 `~/Library/Application Support/DroidDock/tools/` 目录下管理其依赖工具。不应假设系统 PATH 中已存在 `adb` 或 `scrcpy`。
- **会话日志：** `scrcpy` 日志在后端进行缓冲（上限 400 行），并由前端按需获取。
