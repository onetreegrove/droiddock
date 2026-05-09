# DroidDock Architecture Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 DroidDock 从 MVP 闭环结构优化为职责清晰、协议一致、可测试、可扩展的 Tauri/Vue/Rust 桌面应用架构。

**Architecture:** 后端按 `config / tools / devices / wireless / sessions / errors / ipc` 拆分模块，Tauri command 只做薄适配。前端拆分 Pinia store，把资源状态、UI 状态和 IPC 调用解耦；同时建立前后端 scrcpy 参数一致性测试和结构化错误模型。

**Tech Stack:** Tauri 2, Rust, Vue 3, TypeScript, Pinia, Vitest, Cargo test.

---

## Scope And Principles

- 本计划只优化设计和维护性，不扩大产品功能范围。
- 每个阶段都必须保持 `npm run test`、`npm run build`、`cd src-tauri && cargo test` 可通过。
- 优先做“等价拆分”和测试护栏，再改行为。
- 每个任务完成后单独提交，便于回滚。
- 现有未跟踪文档不属于执行范围，除非用户明确要求纳入提交。

## Target File Structure

### Rust Backend

- Create: `src-tauri/src/error.rs`
  - 定义结构化错误 `AppError`、`AppErrorCode`、`AppResult<T>`，提供 `From<std::io::Error>` 和 `translate_command_error`。
- Create: `src-tauri/src/config.rs`
  - 管理 `AppConfig`、`DeviceOptionEntry`、配置目录、读取、原子写入、备份恢复。
- Create: `src-tauri/src/command.rs`
  - 封装外部命令执行：`CommandOutput`、`run_command`、`run_required`。
- Create: `src-tauri/src/tools.rs`
  - 管理 `adb/scrcpy` 候选路径、架构检测、安装、下载、解压、校验。
- Create: `src-tauri/src/devices.rs`
  - 管理设备模型、`adb devices -l` 解析、设备列表查询。
- Create: `src-tauri/src/wireless.rs`
  - 管理 `adb tcpip/connect/disconnect/pair`，recent endpoints 写入。
- Create: `src-tauri/src/scrcpy.rs`
  - 管理 `ScrcpyOptions`、参数构造、参数校验。
- Create: `src-tauri/src/sessions.rs`
  - 管理 `SessionManager`、会话启动/停止/状态刷新/日志缓冲。
- Modify: `src-tauri/src/lib.rs`
  - 保留 Tauri `run()` 和 command 薄封装，委托到以上模块。

### Frontend

- Create: `src/lib/ipc/client.ts`
  - 统一包装 `invoke`，将后端结构化错误规范化。
- Create: `src/lib/ipc/errors.ts`
  - 定义 `AppErrorPayload`、`normalizeIpcError`。
- Split: `src/stores/app.ts`
  - Create: `src/stores/ui.ts`
  - Create: `src/stores/config.ts`
  - Create: `src/stores/tools.ts`
  - Create: `src/stores/devices.ts`
  - Create: `src/stores/sessions.ts`
  - Keep: `src/stores/app.ts` 作为组合 facade，过渡期兼容旧组件。
- Modify: `src/types/app.ts`
  - 只保留前端展示层类型，后端协议类型逐步迁移到 `src/lib/ipc/types.ts`。
- Create: `src/lib/ipc/types.ts`
  - 集中 Tauri command 请求/响应类型。

### Tests

- Add Rust tests beside modules with `#[cfg(test)]`.
- Add frontend tests:
  - `src/lib/ipc/errors.test.ts`
  - `src/domain/scrcpyOptions.test.ts`
  - `src/stores/*.test.ts` where behavior is meaningful and not UI-only.

---

## Phase 0: Baseline And Guardrails

### Task 0.1: Record Current Baseline

**Files:**
- Modify: none

- [ ] **Step 1: Capture current status**

Run:

```bash
git status --short
npm run test
npm run build
cd src-tauri && cargo test
```

Expected:

```text
npm run test: PASS
npm run build: PASS
cargo test: PASS
```

If there are pre-existing failures, record the exact failures before starting refactor.

- [ ] **Step 2: Commit baseline marker if needed**

If only documentation or unrelated untracked files exist, do not stage them. If baseline command or config changes are required, commit them separately.

---

## Phase 1: Backend Error And Command Foundation

### Task 1.1: Add Structured Error Model

**Files:**
- Create: `src-tauri/src/error.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/error.rs`

- [ ] **Step 1: Create failing tests for error translation**

Add this test module in `src-tauri/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_unauthorized_adb_error() {
        let error = translate_command_error("", "device unauthorized");
        assert_eq!(error.code, AppErrorCode::DeviceUnauthorized);
        assert_eq!(error.user_message, "请解锁手机，并在手机弹窗中允许 USB 调试");
        assert!(error.retryable);
    }

    #[test]
    fn preserves_technical_detail_for_unknown_error() {
        let error = translate_command_error("", "some low level failure");
        assert_eq!(error.code, AppErrorCode::CommandFailed);
        assert_eq!(error.user_message, "some low level failure");
        assert_eq!(error.technical_detail.as_deref(), Some("some low level failure"));
    }
}
```

- [ ] **Step 2: Run failing Rust tests**

Run:

```bash
cd src-tauri && cargo test error::
```

Expected: FAIL because `error.rs` and its types are not wired yet.

- [ ] **Step 3: Implement `error.rs`**

Create `src-tauri/src/error.rs`:

```rust
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorCode {
    CommandFailed,
    ToolNotFound,
    InvalidInput,
    DeviceUnauthorized,
    DeviceOffline,
    MultipleDevices,
    WirelessPortUnavailable,
    PairFailed,
    UnsupportedAdbPair,
    ConfigReadFailed,
    ConfigWriteFailed,
    SessionNotFound,
    LockPoisoned,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: AppErrorCode,
    pub user_message: String,
    pub technical_detail: Option<String>,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: AppErrorCode, user_message: impl Into<String>) -> Self {
        Self {
            code,
            user_message: user_message.into(),
            technical_detail: None,
            retryable: false,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.technical_detail = Some(detail.into());
        self
    }

    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::new(AppErrorCode::CommandFailed, error.to_string()).with_detail(error.to_string())
    }
}

pub fn lock_poisoned(name: &str) -> AppError {
    AppError::new(AppErrorCode::LockPoisoned, format!("{name} lock poisoned"))
}

pub fn translate_command_error(stdout: &str, stderr: &str) -> AppError {
    let detail = format!("{stdout}\n{stderr}").trim().to_string();
    let text = detail.to_lowercase();

    if text.contains("unauthorized") {
        AppError::new(AppErrorCode::DeviceUnauthorized, "请解锁手机，并在手机弹窗中允许 USB 调试")
            .with_detail(detail)
            .retryable()
    } else if text.contains("offline") {
        AppError::new(AppErrorCode::DeviceOffline, "设备已离线，请重新插拔数据线或重新连接无线调试")
            .with_detail(detail)
            .retryable()
    } else if text.contains("more than one device") || text.contains("more than one emulator") {
        AppError::new(AppErrorCode::MultipleDevices, "当前有多台设备，请先选择要操作的手机")
            .with_detail(detail)
    } else if text.contains("connection refused") {
        AppError::new(AppErrorCode::WirelessPortUnavailable, "无线调试端口不可用，请检查 IP、端口和手机无线调试是否开启")
            .with_detail(detail)
            .retryable()
    } else if text.contains("failed to authenticate") {
        AppError::new(AppErrorCode::PairFailed, "配对失败，请重新生成配对码后再试")
            .with_detail(detail)
            .retryable()
    } else if text.contains("unknown command") && text.contains("pair") {
        AppError::new(AppErrorCode::UnsupportedAdbPair, "当前 adb 版本不支持无线配对，请升级 Android Platform Tools")
            .with_detail(detail)
    } else {
        let message = stderr
            .trim()
            .lines()
            .next()
            .or_else(|| stdout.trim().lines().next())
            .unwrap_or("命令执行失败")
            .to_string();
        AppError::new(AppErrorCode::CommandFailed, message).with_detail(detail)
    }
}
```

- [ ] **Step 4: Wire module without changing command return types**

In `src-tauri/src/lib.rs`, add:

```rust
mod error;
```

Keep existing `translate_error` for now to avoid wide behavior change.

- [ ] **Step 5: Verify**

Run:

```bash
cd src-tauri && cargo test error::
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/lib.rs
git commit -m "refactor: add structured backend error model"
```

### Task 1.2: Extract Command Runner

**Files:**
- Create: `src-tauri/src/command.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/command.rs`

- [ ] **Step 1: Create command runner module**

Move `CommandResult`, `run_command`, and `run_required` from `lib.rs` into `command.rs`. Keep serialized field names unchanged so frontend behavior is stable.

Use this public API:

```rust
use serde::Serialize;
use std::process::Command;

use crate::error::{translate_command_error, AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct CommandResult {
    pub ok: bool,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
}

pub fn run_command(path: &str, args: &[&str]) -> CommandResult {
    match Command::new(path).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let ok = output.status.success();
            let message = if ok {
                stdout.trim().to_string()
            } else {
                translate_command_error(&stdout, &stderr).user_message
            };

            CommandResult {
                ok,
                code: output.status.code(),
                stdout,
                stderr,
                message,
            }
        }
        Err(error) => CommandResult {
            ok: false,
            code: None,
            stdout: String::new(),
            stderr: error.to_string(),
            message: error.to_string(),
        },
    }
}

pub fn run_required(path: &str, args: &[&str]) -> AppResult<CommandResult> {
    let result = run_command(path, args);
    if result.ok {
        Ok(result)
    } else {
        Err(AppError::new(crate::error::AppErrorCode::CommandFailed, result.message)
            .with_detail(format!("{}\n{}", result.stdout, result.stderr)))
    }
}
```

- [ ] **Step 2: Update imports in `lib.rs`**

Add:

```rust
mod command;
use command::{run_command, run_required, CommandResult};
```

Remove the old local `CommandResult`, `run_command`, `run_required`.

- [ ] **Step 3: Fix `?` conversions**

Where a function still returns `Result<T, String>`, convert `AppError` with:

```rust
.map_err(|error| error.user_message)?;
```

Use this only during transition. Later phases will return structured errors.

- [ ] **Step 4: Verify**

Run:

```bash
cd src-tauri && cargo test
npm run build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/command.rs src-tauri/src/lib.rs
git commit -m "refactor: extract backend command runner"
```

---

## Phase 2: Backend Domain Modules

### Task 2.1: Extract Config Store With Atomic Writes

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/config.rs`

- [ ] **Step 1: Add config tests**

Add tests that verify:

```rust
#[test]
fn config_round_trips_default_values() {
    let config = AppConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let restored: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.default_preset_id, "daily");
    assert_eq!(restored.default_scrcpy_options.max_size, Some(1920));
}

#[test]
fn config_defaults_when_new_fields_are_missing() {
    let restored: AppConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(restored.default_preset_id, "daily");
    assert!(restored.device_aliases.is_empty());
}
```

- [ ] **Step 2: Move config structs and path helpers**

Move these from `lib.rs`:

```rust
AppConfig
DeviceOptionEntry
config_dir
config_path
tools_dir
load_config
save_config
```

Rename `save_config` to `save_config_atomic`.

- [ ] **Step 3: Implement atomic write**

In `save_config_atomic`, write to `config.json.tmp`, then `fs::rename` to `config.json`:

```rust
pub fn save_config_atomic(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let target = dir.join("config.json");
    let temp = dir.join("config.json.tmp");
    let content = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    fs::write(&temp, content).map_err(|error| error.to_string())?;
    fs::rename(&temp, &target).map_err(|error| error.to_string())
}
```

- [ ] **Step 4: Replace call sites**

Replace all `save_config(&config)` with `save_config_atomic(&config)`.

- [ ] **Step 5: Verify**

Run:

```bash
cd src-tauri && cargo test config::
cd src-tauri && cargo test
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/src/lib.rs
git commit -m "refactor: extract config store"
```

### Task 2.2: Extract Scrcpy Options

**Files:**
- Create: `src-tauri/src/scrcpy.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/scrcpy.rs`

- [ ] **Step 1: Add Rust tests matching frontend behavior**

Add:

```rust
#[test]
fn builds_scrcpy_args_in_stable_order() {
    let options = ScrcpyOptions {
        max_size: Some(1920),
        max_fps: Some(60),
        video_bit_rate: Some("4M".to_string()),
        video_codec: Some("h265".to_string()),
        no_audio: Some(true),
        no_control: None,
        stay_awake: Some(true),
        turn_screen_off: None,
        show_touches: None,
        always_on_top: Some(true),
        fullscreen: None,
    };

    assert_eq!(
        build_scrcpy_args("R9YT301WXXX", &options),
        vec![
            "-s",
            "R9YT301WXXX",
            "--max-size=1920",
            "--max-fps=60",
            "--video-bit-rate=4M",
            "--video-codec=h265",
            "--no-audio",
            "--stay-awake",
            "--always-on-top",
        ]
    );
}
```

- [ ] **Step 2: Move `ScrcpyOptions` and `build_scrcpy_args`**

Move from `lib.rs` to `scrcpy.rs` and export:

```rust
pub struct ScrcpyOptions { ... }
pub fn build_scrcpy_args(serial: &str, options: &ScrcpyOptions) -> Vec<String> { ... }
```

- [ ] **Step 3: Verify**

Run:

```bash
cd src-tauri && cargo test scrcpy::
npm run test -- src/domain/scrcpyOptions.test.ts
```

Expected: both PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/scrcpy.rs src-tauri/src/lib.rs
git commit -m "refactor: extract scrcpy option model"
```

### Task 2.3: Extract Devices Module

**Files:**
- Create: `src-tauri/src/devices.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/devices.rs`

- [ ] **Step 1: Add parser tests**

Add:

```rust
#[test]
fn parses_usb_and_wireless_devices() {
    let config = AppConfig {
        device_aliases: HashMap::from([("R9YT301WXXX".to_string(), "测试手机".to_string())]),
        ..AppConfig::default()
    };
    let output = "List of devices attached\nR9YT301WXXX device product:test model:Pixel_8 transport_id:1\n192.168.1.2:5555 device product:test model:Mi_14 transport_id:2\n";
    let devices = parse_devices(output, &config);
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].alias.as_deref(), Some("测试手机"));
    assert_eq!(devices[0].connection, "usb");
    assert_eq!(devices[1].connection, "wireless");
}
```

- [ ] **Step 2: Move device model and parser**

Move `Device` and `parse_devices` into `devices.rs`.

- [ ] **Step 3: Add list function**

Expose:

```rust
pub fn list_devices_with_adb(adb: &str, config: &AppConfig) -> Result<Vec<Device>, String>
```

Internally call `run_command(adb, &["devices", "-l"])`.

- [ ] **Step 4: Update Tauri command**

Keep `#[tauri::command] fn list_devices(...)` in `lib.rs`, but delegate to `devices::list_devices_with_adb`.

- [ ] **Step 5: Verify and commit**

```bash
cd src-tauri && cargo test devices::
cd src-tauri && cargo test
git add src-tauri/src/devices.rs src-tauri/src/lib.rs
git commit -m "refactor: extract device discovery"
```

### Task 2.4: Extract Tools Module

**Files:**
- Create: `src-tauri/src/tools.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/tools.rs`

- [ ] **Step 1: Move tool logic**

Move these into `tools.rs`:

```rust
ToolStatus
ToolInstallResult
validate_executable
tool_candidates
resolve_tool
is_apple_silicon_compatible_file_output
executable_arch
download_file
unzip_archive
extract_archive
file_sha256
copy_dir_recursive
find_file_named
find_scrcpy_macos_aarch64_asset
install_platform_tools
install_scrcpy
```

- [ ] **Step 2: Keep behavior-preserving public functions**

Expose:

```rust
pub fn get_tool_status_for_config(config: &AppConfig) -> Result<ToolStatus, String>
pub fn install_tools_into_config(config: &mut AppConfig) -> Result<ToolInstallResult, String>
pub fn resolve_tool(name: &str, config: &AppConfig) -> Option<String>
```

- [ ] **Step 3: Preserve existing tests**

Move existing tests for executable validation, release asset parsing, and arch parsing into `tools.rs`.

- [ ] **Step 4: Verify and commit**

```bash
cd src-tauri && cargo test tools::
cd src-tauri && cargo test
git add src-tauri/src/tools.rs src-tauri/src/lib.rs
git commit -m "refactor: extract tool management"
```

### Task 2.5: Extract Wireless Module

**Files:**
- Create: `src-tauri/src/wireless.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/wireless.rs`

- [ ] **Step 1: Move pair request and endpoint helpers**

Move `PairRequest` and add:

```rust
pub fn remember_endpoint(config: &mut AppConfig, endpoint: String) {
    config.recent_endpoints.retain(|item| item != &endpoint);
    config.recent_endpoints.insert(0, endpoint);
    config.recent_endpoints.truncate(20);
}
```

- [ ] **Step 2: Add tests**

```rust
#[test]
fn remember_endpoint_deduplicates_and_caps_recent_list() {
    let mut config = AppConfig::default();
    for index in 0..25 {
        remember_endpoint(&mut config, format!("192.168.1.{index}:5555"));
    }
    remember_endpoint(&mut config, "192.168.1.10:5555".to_string());
    assert_eq!(config.recent_endpoints[0], "192.168.1.10:5555");
    assert_eq!(config.recent_endpoints.len(), 20);
}
```

- [ ] **Step 3: Delegate commands**

Move body logic from `adb_tcpip`, `adb_connect`, `adb_disconnect`, `adb_pair` into module functions. Keep command wrappers in `lib.rs`.

- [ ] **Step 4: Verify and commit**

```bash
cd src-tauri && cargo test wireless::
cd src-tauri && cargo test
npm run test -- src/domain/wireless.test.ts
git add src-tauri/src/wireless.rs src-tauri/src/lib.rs
git commit -m "refactor: extract wireless adb flows"
```

### Task 2.6: Extract Session Manager

**Files:**
- Create: `src-tauri/src/sessions.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/sessions.rs`

- [ ] **Step 1: Move session models and helpers**

Move:

```rust
SessionInfo
SessionLogLine
SessionEntry
push_session_log
spawn_log_reader
refresh_session_status
has_running_session_for_serial
```

- [ ] **Step 2: Introduce `SessionManager`**

Implement:

```rust
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: Mutex<HashMap<String, SessionEntry>>,
}
```

Expose:

```rust
impl SessionManager {
    pub fn start(&self, scrcpy: &str, serial: String, alias: Option<String>, options: ScrcpyOptions) -> Result<SessionInfo, String>;
    pub fn list(&self) -> Result<Vec<SessionInfo>, String>;
    pub fn logs(&self, session_id: String) -> Result<Vec<SessionLogLine>, String>;
    pub fn stop(&self, session_id: String) -> Result<SessionInfo, String>;
    pub fn stop_all(&self) -> Result<Vec<SessionInfo>, String>;
}
```

- [ ] **Step 3: Simplify `AppState`**

Change `AppState` to:

```rust
struct AppState {
    config: Mutex<AppConfig>,
    sessions: SessionManager,
}
```

- [ ] **Step 4: Verify and commit**

```bash
cd src-tauri && cargo test sessions::
cd src-tauri && cargo test
npm run build
git add src-tauri/src/sessions.rs src-tauri/src/lib.rs
git commit -m "refactor: extract session manager"
```

---

## Phase 3: Frontend IPC And Store Boundaries

### Task 3.1: Add IPC Client And Error Normalization

**Files:**
- Create: `src/lib/ipc/errors.ts`
- Create: `src/lib/ipc/client.ts`
- Create: `src/lib/ipc/errors.test.ts`
- Modify: none initially

- [ ] **Step 1: Add error tests**

Create `src/lib/ipc/errors.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { normalizeIpcError } from './errors';

describe('normalizeIpcError', () => {
  it('normalizes a backend structured error', () => {
    expect(
      normalizeIpcError({
        code: 'device_unauthorized',
        user_message: '请解锁手机',
        technical_detail: 'unauthorized',
        retryable: true,
      }),
    ).toEqual({
      code: 'device_unauthorized',
      userMessage: '请解锁手机',
      technicalDetail: 'unauthorized',
      retryable: true,
    });
  });

  it('normalizes string errors from legacy commands', () => {
    expect(normalizeIpcError('adb not found')).toEqual({
      code: 'unknown',
      userMessage: 'adb not found',
      technicalDetail: 'adb not found',
      retryable: false,
    });
  });
});
```

- [ ] **Step 2: Implement `errors.ts`**

```ts
export type AppErrorPayload = {
  code: string;
  userMessage: string;
  technicalDetail: string | null;
  retryable: boolean;
};

type BackendErrorPayload = {
  code?: string;
  user_message?: string;
  technical_detail?: string | null;
  retryable?: boolean;
};

export function normalizeIpcError(error: unknown): AppErrorPayload {
  if (typeof error === 'object' && error !== null) {
    const payload = error as BackendErrorPayload;
    if (typeof payload.user_message === 'string') {
      return {
        code: payload.code ?? 'unknown',
        userMessage: payload.user_message,
        technicalDetail: payload.technical_detail ?? null,
        retryable: Boolean(payload.retryable),
      };
    }
  }

  const message = String(error);
  return {
    code: 'unknown',
    userMessage: message,
    technicalDetail: message,
    retryable: false,
  };
}
```

- [ ] **Step 3: Implement `client.ts`**

```ts
import { invoke } from '@tauri-apps/api/core';
import { normalizeIpcError } from './errors';

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeIpcError(error);
  }
}
```

- [ ] **Step 4: Verify and commit**

```bash
npm run test -- src/lib/ipc/errors.test.ts
npm run build
git add src/lib/ipc/errors.ts src/lib/ipc/client.ts src/lib/ipc/errors.test.ts
git commit -m "refactor: add frontend ipc client"
```

### Task 3.2: Split UI Store

**Files:**
- Create: `src/stores/ui.ts`
- Modify: `src/stores/app.ts`
- Modify components that use `currentPage`, `modal`, `selectedSerial`, `selectedLogSessionId`

- [ ] **Step 1: Create `ui.ts`**

```ts
import { defineStore } from 'pinia';
import type { ModalKey, PageKey } from '../types/app';

export const useUiStore = defineStore('ui', {
  state: () => ({
    currentPage: 'dashboard' as PageKey,
    selectedSerial: null as string | null,
    modal: null as ModalKey,
    selectedLogSessionId: null as string | null,
  }),
  actions: {
    openPage(page: PageKey) {
      this.currentPage = page;
    },
    openModal(modal: Exclude<ModalKey, null>) {
      this.modal = modal;
    },
    closeModal() {
      this.modal = null;
    },
    toggleLogSession(sessionId: string) {
      this.selectedLogSessionId = this.selectedLogSessionId === sessionId ? null : sessionId;
    },
  },
});
```

- [ ] **Step 2: Keep compatibility facade**

In `src/stores/app.ts`, keep existing state during this task if needed. Do not break all components at once.

- [ ] **Step 3: Migrate components gradually**

Update:

```text
src/App.vue
src/components/AppSidebar.vue
src/components/DashboardView.vue
src/components/SessionsView.vue
src/components/PairModal.vue
src/components/WirelessModal.vue
```

Use `const ui = useUiStore()` for page/modal state.

- [ ] **Step 4: Verify**

```bash
npm run test
npm run build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/stores/ui.ts src/stores/app.ts src/App.vue src/components
git commit -m "refactor: split ui navigation state"
```

### Task 3.3: Split Resource Stores

**Files:**
- Create: `src/stores/config.ts`
- Create: `src/stores/tools.ts`
- Create: `src/stores/devices.ts`
- Create: `src/stores/sessions.ts`
- Modify: `src/stores/app.ts`
- Modify components using migrated state

- [ ] **Step 1: Extract config store**

Move:

```text
appConfig
globalDraftOptions
globalDraftPresetId
saveDefaultOptions
saveDeviceOptions
clearDeviceOptions
saveDeviceAlias
```

Use `invokeCommand` instead of raw `invoke`.

- [ ] **Step 2: Extract tools store**

Move:

```text
toolStatus
fetchToolStatus
setToolPath
installTools
isToolsReady getter
```

- [ ] **Step 3: Extract devices store**

Move:

```text
devices
refreshDevices
availableDeviceCount getter
selectedDevice getter should read selectedSerial from ui store
```

- [ ] **Step 4: Extract sessions store**

Move:

```text
sessions
sessionLogs
refreshSessions
startMirror
stopMirror
stopAllSessions
fetchSessionLogs
```

- [ ] **Step 5: Keep `app.ts` as temporary facade**

Export a facade only if migration would otherwise be too broad. The facade should compose stores and contain no invoke calls.

- [ ] **Step 6: Verify and commit**

```bash
npm run test
npm run build
git add src/stores src/components src/App.vue
git commit -m "refactor: split frontend resource stores"
```

---

## Phase 4: Protocol Consistency

### Task 4.1: Add Backend Command Preview Endpoint

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/scrcpy.rs`
- Modify: `src/lib/ipc/types.ts`
- Modify: `src/domain/scrcpyOptions.ts`
- Test: `src/domain/scrcpyOptions.test.ts`

- [ ] **Step 1: Add Tauri command**

In `lib.rs`:

```rust
#[tauri::command]
fn preview_scrcpy_args(serial: String, options: scrcpy::ScrcpyOptions) -> Vec<String> {
    scrcpy::build_scrcpy_args(&serial, &options)
}
```

Register it in `generate_handler`.

- [ ] **Step 2: Update frontend preview path**

Keep current pure TS `buildScrcpyArgs` for fast UI display, but add an integration test or command-level check later. Do not block UI rendering on IPC.

- [ ] **Step 3: Add consistency fixture**

Create a shared fixture in `src/domain/scrcpyOptions.test.ts` matching Rust test:

```ts
const consistencyFixture = {
  serial: 'R9YT301WXXX',
  options: {
    maxSize: 1920,
    maxFps: 60,
    videoBitRate: '4M',
    videoCodec: 'h265' as const,
    noAudio: true,
    stayAwake: true,
    alwaysOnTop: true,
  },
  args: [
    '-s',
    'R9YT301WXXX',
    '--max-size=1920',
    '--max-fps=60',
    '--video-bit-rate=4M',
    '--video-codec=h265',
    '--no-audio',
    '--stay-awake',
    '--always-on-top',
  ],
};
```

- [ ] **Step 4: Verify**

```bash
npm run test -- src/domain/scrcpyOptions.test.ts
cd src-tauri && cargo test scrcpy::
npm run build
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/scrcpy.rs src/domain/scrcpyOptions.test.ts
git commit -m "test: lock scrcpy args across frontend and backend"
```

### Task 4.2: Reduce Duplicate Type Drift

**Files:**
- Create: `src/lib/ipc/types.ts`
- Modify: `src/types/app.ts`
- Modify imports in stores/components

- [ ] **Step 1: Move protocol types**

Move these to `src/lib/ipc/types.ts`:

```ts
export type ToolStatus = { ... };
export type ToolInstallResult = { ... };
export type Device = { ... };
export type SessionLogLine = { ... };
export type SessionInfo = { ... };
export type PairRequest = { ... };
export type AppConfig = { ... };
```

Keep UI-only types in `src/types/app.ts`:

```ts
export type PageKey = 'dashboard' | 'devices' | 'sessions' | 'setup' | 'settings';
export type ModalKey = null | 'pair' | 'wireless' | 'logs';
export type PresetId = 'daily' | 'lowBandwidth' | 'demo' | 'batterySaver' | 'viewOnly';
export type ScrcpyOptions = { ... };
```

- [ ] **Step 2: Update imports**

Stores should import backend DTOs from `src/lib/ipc/types.ts`; components should prefer store-derived types unless they directly construct IPC request payloads.

- [ ] **Step 3: Verify and commit**

```bash
npm run build
npm run test
git add src/lib/ipc/types.ts src/types/app.ts src
git commit -m "refactor: separate ipc dto types"
```

---

## Phase 5: Runtime Model Improvements

### Task 5.1: Replace Polling Logs With Event Push

**Files:**
- Modify: `src-tauri/src/sessions.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/App.vue`
- Modify: `src/stores/sessions.ts`

- [ ] **Step 1: Define event payload**

In Rust:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SessionLogEvent {
    pub session_id: String,
    pub line: SessionLogLine,
}
```

- [ ] **Step 2: Emit events from log readers**

Pass `tauri::AppHandle` into session start, and emit:

```rust
let _ = app.emit("session-log", SessionLogEvent {
    session_id: session_id.clone(),
    line,
});
```

- [ ] **Step 3: Frontend listen**

In sessions store:

```ts
import { listen } from '@tauri-apps/api/event';

export async function listenSessionLogs() {
  return listen<{ session_id: string; line: SessionLogLine }>('session-log', (event) => {
    const logs = this.sessionLogs[event.payload.session_id] ?? [];
    this.sessionLogs[event.payload.session_id] = [...logs, event.payload.line].slice(-400);
  });
}
```

- [ ] **Step 4: Keep polling only for device/session status**

In `App.vue`, remove per-session log fetching from the 3-second poller.

- [ ] **Step 5: Verify**

```bash
npm run build
cd src-tauri && cargo test
```

Manual check:

```bash
npm run tauri:dev
```

Expected: session logs still appear while scrcpy runs.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/sessions.rs src-tauri/src/lib.rs src/App.vue src/stores/sessions.ts
git commit -m "refactor: stream session logs via events"
```

### Task 5.2: Add Command Timeouts For ADB Operations

**Files:**
- Modify: `src-tauri/src/command.rs`
- Modify: call sites in `devices.rs`, `wireless.rs`, `tools.rs`
- Test: `src-tauri/src/command.rs`

- [ ] **Step 1: Add timeout API**

Implement:

```rust
pub fn run_command_with_timeout(path: &str, args: &[&str], timeout: std::time::Duration) -> CommandResult
```

Use a worker thread and `recv_timeout` for MVP simplicity.

- [ ] **Step 2: Apply conservative timeouts**

Use:

```text
adb devices -l: 5 seconds
adb connect/disconnect/tcpip: 15 seconds
adb pair: 60 seconds
tool version checks: 5 seconds
download/install: no timeout beyond curl retry for now
```

- [ ] **Step 3: Verify**

```bash
cd src-tauri && cargo test
npm run build
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/command.rs src-tauri/src/devices.rs src-tauri/src/wireless.rs src-tauri/src/tools.rs
git commit -m "refactor: add adb command timeouts"
```

---

## Phase 6: Configuration Hardening

### Task 6.1: Add Config Schema Version And Corruption Backup

**Files:**
- Modify: `src-tauri/src/config.rs`
- Test: `src-tauri/src/config.rs`

- [ ] **Step 1: Add schema field**

Add:

```rust
pub const CURRENT_CONFIG_SCHEMA_VERSION: u32 = 1;

#[serde(default)]
pub struct AppConfig {
    pub schema_version: u32,
    ...
}
```

Default:

```rust
schema_version: CURRENT_CONFIG_SCHEMA_VERSION
```

- [ ] **Step 2: On parse failure, backup damaged file**

In `load_config`, when parsing fails:

```rust
let backup = path.with_extension(format!("json.bak-{}", now_secs()));
let _ = fs::copy(&path, backup);
return AppConfig::default();
```

- [ ] **Step 3: Add tests**

Test missing schema defaults to current version.

- [ ] **Step 4: Verify and commit**

```bash
cd src-tauri && cargo test config::
cd src-tauri && cargo test
git add src-tauri/src/config.rs
git commit -m "refactor: harden config persistence"
```

### Task 6.2: Separate Verified Tool Versions From Latest Downloads

**Files:**
- Modify: `src-tauri/src/tools.rs`
- Create: `src-tauri/src/tool_manifest.rs`
- Test: `src-tauri/src/tools.rs`

- [ ] **Step 1: Create manifest model**

```rust
pub struct ToolManifest {
    pub platform_tools_url: &'static str,
    pub scrcpy_release_api: &'static str,
    pub allowed_scrcpy_asset_suffixes: &'static [&'static str],
}

pub const DEFAULT_TOOL_MANIFEST: ToolManifest = ToolManifest {
    platform_tools_url: "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip",
    scrcpy_release_api: "https://api.github.com/repos/Genymobile/scrcpy/releases/latest",
    allowed_scrcpy_asset_suffixes: &[".zip", ".tar.gz", ".tgz"],
};
```

- [ ] **Step 2: Route installer through manifest**

Replace direct constants in `tools.rs` with manifest fields.

- [ ] **Step 3: Keep checksum logging, do not falsely claim verification**

Rename UI/log text from “校验” if present to “记录 sha256”. Only call it verified after adding a trusted checksum list.

- [ ] **Step 4: Verify and commit**

```bash
cd src-tauri && cargo test tools::
npm run build
git add src-tauri/src/tool_manifest.rs src-tauri/src/tools.rs
git commit -m "refactor: isolate tool download manifest"
```

---

## Phase 7: Final Cleanup

### Task 7.1: Remove Legacy Facade And Dead State

**Files:**
- Modify: `src/stores/app.ts`
- Modify imports across `src/`

- [ ] **Step 1: Search legacy store usage**

Run:

```bash
rg -n "useAppStore|currentPage|modal|selectedSerial|sessionDraftOptions" src
```

- [ ] **Step 2: Remove state that has moved**

`src/stores/app.ts` should either be deleted or reduced to an intentionally documented compatibility layer with no mutable source of truth.

- [ ] **Step 3: Verify**

```bash
npm run test
npm run build
```

- [ ] **Step 4: Commit**

```bash
git add src
git commit -m "refactor: remove legacy app store responsibilities"
```

### Task 7.2: Documentation Update

**Files:**
- Modify: `GEMINI.md`
- Modify: `README.md`
- Create: `docs/architecture.md`

- [ ] **Step 1: Write architecture doc**

Create `docs/architecture.md` with sections:

```markdown
# DroidDock Architecture

## Backend Modules

- `config`: App configuration schema, load/save, atomic persistence.
- `tools`: adb/scrcpy discovery, validation, install.
- `devices`: adb device parsing and list queries.
- `wireless`: tcpip/connect/disconnect/pair flows.
- `scrcpy`: scrcpy options and argument construction.
- `sessions`: scrcpy process lifecycle and log streaming.
- `error`: structured app errors.
- `command`: external command execution.

## Frontend Stores

- `ui`: navigation, modal, selected device/session UI state.
- `config`: persisted app configuration and scrcpy defaults.
- `tools`: adb/scrcpy status and installation.
- `devices`: device discovery state.
- `sessions`: session lifecycle and logs.

## Verification

- Frontend: `npm run test`, `npm run build`
- Backend: `cd src-tauri && cargo test`
- App bundle: `npm run tauri:build:app`
```

- [ ] **Step 2: Update project context**

Update `GEMINI.md` architecture section to match actual module boundaries.

- [ ] **Step 3: Verify docs links**

Run:

```bash
rg -n "src-tauri/src/lib.rs|stores/app.ts|architecture.md" README.md GEMINI.md docs
```

Expected: docs no longer describe `lib.rs` and `app.ts` as the main place for all logic.

- [ ] **Step 4: Commit**

```bash
git add README.md GEMINI.md docs/architecture.md
git commit -m "docs: document optimized architecture"
```

---

## Recommended Execution Order

1. Phase 0: establish baseline.
2. Phase 1: introduce error/command foundation with minimal behavior change.
3. Phase 2: split Rust backend modules.
4. Phase 3: split frontend IPC and stores.
5. Phase 4: lock protocol consistency.
6. Phase 5: improve runtime events and timeouts.
7. Phase 6: harden persistence and tool install boundaries.
8. Phase 7: remove compatibility leftovers and update docs.

## Acceptance Criteria

- `src-tauri/src/lib.rs` only contains module declarations, Tauri command wrappers, `run()`, and minimal glue.
- No single frontend store owns UI navigation, tools, devices, sessions, config, and logs at the same time.
- Frontend and backend scrcpy arg generation are covered by matching tests.
- Config writes are atomic and corrupted config files are backed up instead of silently overwritten.
- ADB command paths have bounded timeouts where hanging is likely.
- Session logs can update through events without polling every running session.
- Documentation matches the implemented architecture.

## Verification Matrix

Run after each phase:

```bash
npm run test
npm run build
cd src-tauri && cargo test
```

Run before final handoff:

```bash
npm run tauri:build:app
```

If DMG packaging fails because of local `hdiutil` environment issues, use `.app` build as the packaging verification path and record the exact DMG failure separately.
