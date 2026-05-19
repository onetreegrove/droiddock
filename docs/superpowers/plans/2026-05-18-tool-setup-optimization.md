# Tool Setup Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把工具配置页从“路径展示页”升级为可诊断、可修复、可观察、符合非技术用户心智的 adb/scrcpy 工具管理流程。

**Architecture:** 后端新增结构化工具诊断模型，负责候选路径发现、身份验证、架构判断、安装结果和错误原因；前端只消费诊断结果，不再靠布尔值猜测失败原因。手动配置改为单工具更新和即时验证，自动安装改为可观察的任务面板，并为固定版本和 sha256 校验留出明确边界。

**Tech Stack:** Tauri v2, Rust, Vue 3, TypeScript, Pinia, Vitest, Cargo tests.

---

## File Structure

- Modify: `src-tauri/src/tools.rs`
  - 定义 `ToolKind`、`ToolSource`、`ToolHealth`、`ToolDiagnostic`、`ToolStatus`。
  - 拆分候选路径发现、工具身份验证、架构判断和安装逻辑。
  - 增加固定下载 manifest 校验入口。
- Modify: `src-tauri/src/lib.rs`
  - 保留 `get_tool_status`。
  - 新增 `set_tool_path(tool, path)`、`clear_tool_path(tool)`。
  - 调整 `install_tools`，避免长时间持有 config lock。
- Modify: `src-tauri/src/tool_manifest.rs`
  - 增加固定版本、下载 URL、期望 sha256。
- Modify: `src/lib/ipc/types.ts`
  - 同步后端诊断类型。
- Modify: `src/stores/tools.ts`
  - 管理工具状态、安装状态、安装日志和错误。
- Modify: `src/stores/app.ts`
  - 将 `setToolPath` 改为单工具路径更新。
  - 增加 `clearToolPath`、`refreshToolStatus` 语义封装。
- Modify: `src/components/SetupView.vue`
  - 重构工具卡片、安装面板、错误说明、重新检测、清除路径。
- Modify: `src/styles.css`
  - 补齐诊断卡片、安装日志、操作栏样式。
- Create: `src/domain/toolDiagnostics.ts`
  - 前端纯函数：状态标签、动作建议、来源标签、详情摘要。
- Create: `src/domain/toolDiagnostics.test.ts`
  - 覆盖工具诊断文案与状态映射。

---

## Task 1: Define Tool Diagnostic Model

**Files:**
- Modify: `src-tauri/src/tools.rs`
- Modify: `src/lib/ipc/types.ts`

- [ ] **Step 1: Add Rust diagnostic enums and structs**

In `src-tauri/src/tools.rs`, replace the old `ToolStatus` struct with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ToolKind {
    Adb,
    Scrcpy,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ToolSource {
    Configured,
    Bundled,
    AndroidSdk,
    LocalBin,
    Homebrew,
    SystemPath,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ToolHealth {
    Ready,
    Missing,
    NotExecutable,
    WrongTool,
    VersionFailed,
    IncompatibleArch,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolDiagnostic {
    pub(crate) kind: ToolKind,
    pub(crate) path: Option<String>,
    pub(crate) source: Option<ToolSource>,
    pub(crate) version: Option<String>,
    pub(crate) arch: Option<String>,
    pub(crate) health: ToolHealth,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ToolStatus {
    pub(crate) adb: ToolDiagnostic,
    pub(crate) scrcpy: ToolDiagnostic,
    pub(crate) adb_ok: bool,
    pub(crate) scrcpy_ok: bool,
}
```

- [ ] **Step 2: Update frontend IPC types**

In `src/lib/ipc/types.ts`, replace the old flat `ToolStatus` definition with:

```ts
export type ToolKind = 'adb' | 'scrcpy';

export type ToolSource = 'configured' | 'bundled' | 'android_sdk' | 'local_bin' | 'homebrew' | 'system_path';

export type ToolHealth =
  | 'ready'
  | 'missing'
  | 'not_executable'
  | 'wrong_tool'
  | 'version_failed'
  | 'incompatible_arch';

export type ToolDiagnostic = {
  kind: ToolKind;
  path: string | null;
  source: ToolSource | null;
  version: string | null;
  arch: string | null;
  health: ToolHealth;
  message: string;
};

export type ToolStatus = {
  adb: ToolDiagnostic;
  scrcpy: ToolDiagnostic;
  adb_ok: boolean;
  scrcpy_ok: boolean;
};
```

- [ ] **Step 3: Run typecheck through tests**

Run: `npm run test -- src/domain/scrcpyOptions.test.ts`

Expected: current unrelated tests still pass or TypeScript compile errors point to old `toolStatus.adb_path` style call sites.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tools.rs src/lib/ipc/types.ts
git commit -m "refactor: introduce structured tool diagnostics"
```

---

## Task 2: Build Backend Tool Validation

**Files:**
- Modify: `src-tauri/src/tools.rs`

- [ ] **Step 1: Add candidate source tracking**

Replace `tool_candidates(name, config) -> Vec<String>` with:

```rust
#[derive(Debug, Clone)]
struct ToolCandidate {
    path: String,
    source: ToolSource,
}

fn which_candidate(name: &str) -> Option<ToolCandidate> {
    let output = Command::new("/usr/bin/which").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if path.is_empty() {
        return None;
    }
    Some(ToolCandidate { path, source: ToolSource::SystemPath })
}

fn tool_candidates(kind: ToolKind, config: &AppConfig) -> Vec<ToolCandidate> {
    let mut candidates = Vec::new();
    match kind {
        ToolKind::Adb => {
            if let Some(path) = &config.adb_path {
                candidates.push(ToolCandidate { path: path.clone(), source: ToolSource::Configured });
            }
            if let Ok(home) = std::env::var("HOME") {
                candidates.push(ToolCandidate { path: format!("{home}/Library/Application Support/DroidDock/tools/platform-tools/adb"), source: ToolSource::Bundled });
                candidates.push(ToolCandidate { path: format!("{home}/Library/Android/sdk/platform-tools/adb"), source: ToolSource::AndroidSdk });
            }
            candidates.push(ToolCandidate { path: "/opt/homebrew/bin/adb".to_string(), source: ToolSource::Homebrew });
            candidates.push(ToolCandidate { path: "/usr/local/bin/adb".to_string(), source: ToolSource::Homebrew });
            if let Some(candidate) = which_candidate("adb") {
                candidates.push(candidate);
            }
        }
        ToolKind::Scrcpy => {
            if let Some(path) = &config.scrcpy_path {
                candidates.push(ToolCandidate { path: path.clone(), source: ToolSource::Configured });
            }
            if let Ok(home) = std::env::var("HOME") {
                candidates.push(ToolCandidate { path: format!("{home}/Library/Application Support/DroidDock/tools/scrcpy/scrcpy"), source: ToolSource::Bundled });
                candidates.push(ToolCandidate { path: format!("{home}/.local/bin/scrcpy"), source: ToolSource::LocalBin });
            }
            candidates.push(ToolCandidate { path: "/opt/homebrew/bin/scrcpy".to_string(), source: ToolSource::Homebrew });
            candidates.push(ToolCandidate { path: "/usr/local/bin/scrcpy".to_string(), source: ToolSource::Homebrew });
            if let Some(candidate) = which_candidate("scrcpy") {
                candidates.push(candidate);
            }
        }
    }
    candidates
}
```

Keep this priority order unless product direction changes: user-configured path, DroidDock-managed bundled tools, common SDK/local paths, Homebrew paths, then system `PATH`. `PATH` is a fallback only; DroidDock must not assume it exists.

- [ ] **Step 2: Add diagnostic validation function**

Add:

```rust
fn host_support_diagnostic(kind: ToolKind) -> Option<ToolDiagnostic> {
    if std::env::consts::ARCH == "aarch64" {
        return None;
    }
    Some(ToolDiagnostic {
        kind,
        path: None,
        source: None,
        version: None,
        arch: Some(std::env::consts::ARCH.to_string()),
        health: ToolHealth::IncompatibleArch,
        message: "当前版本仅支持 Apple Silicon Mac，暂不支持 Intel Mac".to_string(),
    })
}

fn diagnose_candidate(kind: ToolKind, candidate: ToolCandidate) -> ToolDiagnostic {
    let expected_name = match kind {
        ToolKind::Adb => "adb",
        ToolKind::Scrcpy => "scrcpy",
    };

    let path = Path::new(&candidate.path);
    if !path.exists() {
        return ToolDiagnostic {
            kind,
            path: Some(candidate.path),
            source: Some(candidate.source),
            version: None,
            arch: None,
            health: ToolHealth::Missing,
            message: format!("{expected_name} 路径不存在"),
        };
    }
    if !validate_executable(&candidate.path) {
        return ToolDiagnostic {
            kind,
            path: Some(candidate.path),
            source: Some(candidate.source),
            version: None,
            arch: executable_arch(path.to_string_lossy().as_ref()),
            health: ToolHealth::NotExecutable,
            message: format!("{expected_name} 文件不可执行"),
        };
    }

    let arch = executable_arch(&candidate.path);
    let arch_ok = if std::env::consts::ARCH == "aarch64" {
        arch.as_deref().map(is_apple_silicon_compatible_file_output).unwrap_or(false)
    } else {
        false
    };
    if !arch_ok {
        return ToolDiagnostic {
            kind,
            path: Some(candidate.path),
            source: Some(candidate.source),
            version: None,
            arch,
            health: ToolHealth::IncompatibleArch,
            message: "当前工具不适合 Apple Silicon，请选择 arm64 或 universal 版本".to_string(),
        };
    }

    let version_result = match kind {
        ToolKind::Adb => run_command(&candidate.path, &["version"]),
        ToolKind::Scrcpy => run_command(&candidate.path, &["--version"]),
    };
    if !version_result.ok {
        return ToolDiagnostic {
            kind,
            path: Some(candidate.path),
            source: Some(candidate.source),
            version: None,
            arch,
            health: ToolHealth::VersionFailed,
            message: format!("{expected_name} 无法运行版本检查"),
        };
    }

    let stdout = version_result.stdout;
    let version = match kind {
        ToolKind::Adb => second_line(Some(stdout.clone())),
        ToolKind::Scrcpy => first_line(Some(stdout.clone())),
    };
    let identity_ok = match kind {
        ToolKind::Adb => stdout.to_ascii_lowercase().contains("android debug bridge"),
        ToolKind::Scrcpy => stdout.to_ascii_lowercase().contains("scrcpy"),
    };
    if !identity_ok {
        return ToolDiagnostic {
            kind,
            path: Some(candidate.path),
            source: Some(candidate.source),
            version,
            arch,
            health: ToolHealth::WrongTool,
            message: format!("选择的文件不是可用的 {expected_name}"),
        };
    }

    ToolDiagnostic {
        kind,
        path: Some(candidate.path),
        source: Some(candidate.source),
        version,
        arch,
        health: ToolHealth::Ready,
        message: "工具可用".to_string(),
    }
}
```

- [ ] **Step 3: Update status selection**

Implement:

```rust
fn missing_diagnostic(kind: ToolKind) -> ToolDiagnostic {
    let name = match kind {
        ToolKind::Adb => "adb",
        ToolKind::Scrcpy => "scrcpy",
    };
    ToolDiagnostic {
        kind,
        path: None,
        source: None,
        version: None,
        arch: None,
        health: ToolHealth::Missing,
        message: format!("未找到 {name}，请自动安装或手动选择路径"),
    }
}

fn diagnose_tool(kind: ToolKind, config: &AppConfig) -> ToolDiagnostic {
    if let Some(diagnostic) = host_support_diagnostic(kind) {
        return diagnostic;
    }
    let mut first_failure: Option<ToolDiagnostic> = None;
    for candidate in tool_candidates(kind, config) {
        let diagnostic = diagnose_candidate(kind, candidate);
        if diagnostic.health == ToolHealth::Ready {
            return diagnostic;
        }
        if first_failure.is_none() {
            first_failure = Some(diagnostic);
        }
    }
    first_failure.unwrap_or_else(|| missing_diagnostic(kind))
}

pub(crate) fn get_tool_status_for_config(config: &AppConfig) -> Result<ToolStatus, String> {
    let adb = diagnose_tool(ToolKind::Adb, config);
    let scrcpy = diagnose_tool(ToolKind::Scrcpy, config);
    Ok(ToolStatus {
        adb_ok: adb.health == ToolHealth::Ready,
        scrcpy_ok: scrcpy.health == ToolHealth::Ready,
        adb,
        scrcpy,
    })
}
```

- [ ] **Step 4: Update `resolve_tool`**

Keep command callers simple:

```rust
pub(crate) fn resolve_tool(name: &str, config: &AppConfig) -> Option<String> {
    let kind = match name {
        "adb" => ToolKind::Adb,
        "scrcpy" => ToolKind::Scrcpy,
        _ => return None,
    };
    let diagnostic = diagnose_tool(kind, config);
    diagnostic.health.eq(&ToolHealth::Ready).then(|| diagnostic.path).flatten()
}
```

- [ ] **Step 5: Add Rust tests**

Add tests in `src-tauri/src/tools.rs`:

```rust
#[test]
fn missing_tool_diagnostic_has_actionable_message() {
    let diagnostic = missing_diagnostic(ToolKind::Adb);
    assert_eq!(diagnostic.health, ToolHealth::Missing);
    assert!(diagnostic.message.contains("自动安装"));
    assert!(diagnostic.message.contains("手动选择"));
}

#[test]
fn apple_silicon_arch_parser_rejects_x86_64() {
    assert!(!is_apple_silicon_compatible_file_output(
        "Mach-O 64-bit executable x86_64"
    ));
}

#[test]
fn unsupported_host_arch_has_product_level_message() {
    if std::env::consts::ARCH == "aarch64" {
        assert!(host_support_diagnostic(ToolKind::Adb).is_none());
    } else {
        let diagnostic = host_support_diagnostic(ToolKind::Adb).unwrap();
        assert_eq!(diagnostic.health, ToolHealth::IncompatibleArch);
        assert!(diagnostic.message.contains("不支持 Intel Mac"));
    }
}
```

- [ ] **Step 6: Verify**

Run: `cd src-tauri && cargo test tools::`

Expected: all `tools::` tests pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/tools.rs
git commit -m "feat: diagnose tool health with actionable reasons"
```

---

## Task 3: Add Single-Tool Path Commands

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/stores/tools.ts`
- Modify: `src/stores/app.ts`

- [ ] **Step 1: Add backend commands**

In `src-tauri/src/lib.rs`, add:

```rust
#[tauri::command]
fn set_tool_path(
    state: State<'_, AppState>,
    tool: String,
    path: String,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;

    match tool.as_str() {
        "adb" => config.adb_path = Some(path),
        "scrcpy" => config.scrcpy_path = Some(path),
        _ => return Err("未知工具类型".to_string()),
    }

    let status = get_tool_status_for_config(&config)?;
    let diagnostic = match tool.as_str() {
        "adb" => status.adb,
        "scrcpy" => status.scrcpy,
        _ => unreachable!(),
    };
    if diagnostic.health != crate::tools::ToolHealth::Ready {
        return Err(diagnostic.message);
    }

    save_config_atomic(&config)?;
    Ok(config.clone())
}

#[tauri::command]
fn clear_tool_path(
    state: State<'_, AppState>,
    tool: String,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;

    match tool.as_str() {
        "adb" => config.adb_path = None,
        "scrcpy" => config.scrcpy_path = None,
        _ => return Err("未知工具类型".to_string()),
    }

    save_config_atomic(&config)?;
    Ok(config.clone())
}
```

Register both commands in `invoke_handler`.

- [ ] **Step 2: Update tools store**

In `src/stores/tools.ts`, add:

```ts
import type { AppConfig } from '../lib/ipc/types';

async setToolPath(tool: 'adb' | 'scrcpy', path: string) {
  const appConfig = await invokeCommand<AppConfig>('set_tool_path', { tool, path });
  await this.fetchToolStatus();
  return appConfig;
},
async clearToolPath(tool: 'adb' | 'scrcpy') {
  const appConfig = await invokeCommand<AppConfig>('clear_tool_path', { tool });
  await this.fetchToolStatus();
  return appConfig;
},
```

- [ ] **Step 3: Update app store wrapper**

In `src/stores/app.ts`, replace `setToolPath` body with:

```ts
async function setToolPath(tool: 'adb' | 'scrcpy', path: string) {
  setBusy('tools', true);
  try {
    config.appConfig = await tools.setToolPath(tool, path);
    log(`已更新 ${tool} 路径`);
  } catch (error) {
    log(`更新工具路径失败: ${errorMessage(error)}`);
    throw error;
  } finally {
    setBusy('tools', false);
  }
}

async function clearToolPath(tool: 'adb' | 'scrcpy') {
  setBusy('tools', true);
  try {
    config.appConfig = await tools.clearToolPath(tool);
    log(`已清除 ${tool} 自定义路径`);
  } catch (error) {
    log(`清除工具路径失败: ${errorMessage(error)}`);
    throw error;
  } finally {
    setBusy('tools', false);
  }
}
```

Return `clearToolPath` from the store.

This keeps state synchronization to one config update plus one tool status refresh. Do not call `fetchAppConfig()` again after `set_tool_path` or `clear_tool_path`, because the backend command already returns the updated `AppConfig`.

- [ ] **Step 4: Verify**

Run: `npm run test`

Expected: frontend tests pass.

Run: `cd src-tauri && cargo test`

Expected: backend tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src/stores/tools.ts src/stores/app.ts
git commit -m "feat: update tool paths independently"
```

---

## Task 4: Harden Automatic Installation

**Files:**
- Modify: `src-tauri/src/tool_manifest.rs`
- Modify: `src-tauri/src/tools.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add manifest fields**

In `src-tauri/src/tool_manifest.rs`, change manifest to:

```rust
pub(crate) struct ToolDownload {
    pub(crate) version: &'static str,
    pub(crate) url: &'static str,
    pub(crate) sha256: &'static str,
}

pub(crate) struct ToolManifest {
    pub(crate) platform_tools: ToolDownload,
    pub(crate) scrcpy: ToolDownload,
    pub(crate) allowed_scrcpy_asset_suffixes: &'static [&'static str],
}

pub(crate) const DEFAULT_TOOL_MANIFEST: ToolManifest = ToolManifest {
    platform_tools: ToolDownload {
        version: "fixed-by-release-owner",
        url: "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip",
        sha256: "",
    },
    scrcpy: ToolDownload {
        version: "fixed-by-release-owner",
        url: "",
        sha256: "",
    },
    allowed_scrcpy_asset_suffixes: &[".zip", ".tar.gz", ".tgz"],
};
```

Note: Before release, replace empty sha256 and scrcpy URL with verified values. Release builds must reject empty hashes; debug builds may continue with an explicit warning log so developers can test the automatic installation flow locally.

- [ ] **Step 2: Add checksum verification**

In `src-tauri/src/tools.rs`, add:

```rust
fn verify_sha256(path: &Path, expected: &str) -> Result<String, String> {
    if expected.trim().is_empty() {
        #[cfg(debug_assertions)]
        {
            return file_sha256(path);
        }
        #[cfg(not(debug_assertions))]
        {
            return Err("工具下载清单缺少 sha256，已停止安装以保护安全".to_string());
        }
    }
    let actual = file_sha256(path)?;
    if actual != expected {
        return Err("下载文件校验失败，请重新安装或稍后再试".to_string());
    }
    Ok(actual)
}
```

When calling `verify_sha256` in debug builds with an empty hash, also push a visible install log entry before verification:

```rust
if cfg!(debug_assertions) && DEFAULT_TOOL_MANIFEST.platform_tools.sha256.trim().is_empty() {
    logs.push("开发模式：platform-tools 缺少 sha256，已跳过强校验".to_string());
}
```

Use the same pattern for scrcpy. Call `verify_sha256` after each download and before extraction.

- [ ] **Step 3: Avoid holding config lock during install**

In `src-tauri/src/lib.rs`, replace `install_tools` with:

```rust
#[tauri::command]
fn install_tools(state: State<'_, AppState>) -> Result<ToolInstallResult, String> {
    let result = install_tools_into_config()?;
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    config.adb_path = Some(result.adb_path.clone());
    config.scrcpy_path = Some(result.scrcpy_path.clone());
    save_config_atomic(&config)?;
    Ok(result)
}
```

Then change `install_tools_into_config` to no longer accept `&mut AppConfig` and only return paths/logs.

- [ ] **Step 4: Add tests**

In `src-tauri/src/tools.rs`, add:

```rust
#[test]
#[cfg(not(debug_assertions))]
fn verify_sha256_rejects_empty_manifest_hash_in_release() {
    let path = std::env::temp_dir().join(format!("droiddock-sha-test-{}", crate::now_secs()));
    fs::write(&path, "payload").unwrap();
    let error = verify_sha256(&path, "").unwrap_err();
    assert!(error.contains("sha256"));
    let _ = fs::remove_file(path);
}

#[test]
#[cfg(debug_assertions)]
fn verify_sha256_allows_empty_manifest_hash_in_debug() {
    let path = std::env::temp_dir().join(format!("droiddock-sha-test-{}", crate::now_secs()));
    fs::write(&path, "payload").unwrap();
    let actual = verify_sha256(&path, "").unwrap();
    assert!(!actual.is_empty());
    let _ = fs::remove_file(path);
}
```

- [ ] **Step 5: Verify**

Run: `cd src-tauri && cargo test tools::`

Expected: checksum and tool tests pass in debug mode.

Run: `cd src-tauri && cargo test --release tools::verify_sha256_rejects_empty_manifest_hash_in_release`

Expected: release-mode empty-hash test passes, proving production builds reject missing checksums.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tool_manifest.rs src-tauri/src/tools.rs src-tauri/src/lib.rs
git commit -m "feat: harden tool installation checks"
```

---

## Task 5: Add Frontend Diagnostic Helpers

**Files:**
- Create: `src/domain/toolDiagnostics.ts`
- Create: `src/domain/toolDiagnostics.test.ts`

- [ ] **Step 1: Write failing tests**

Create `src/domain/toolDiagnostics.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import type { ToolDiagnostic } from '../lib/ipc/types';
import { toolActionLabel, toolHealthTone, toolSourceLabel, toolSummary } from './toolDiagnostics';

function diagnostic(overrides: Partial<ToolDiagnostic>): ToolDiagnostic {
  return {
    kind: 'adb',
    path: null,
    source: null,
    version: null,
    arch: null,
    health: 'missing',
    message: '未找到 adb',
    ...overrides,
  };
}

describe('tool diagnostics UI helpers', () => {
  it('maps ready diagnostics to calm success copy', () => {
    const item = diagnostic({ health: 'ready', version: 'Android Debug Bridge version 1.0.41', source: 'bundled' });
    expect(toolHealthTone(item)).toBe('green');
    expect(toolSourceLabel(item.source)).toBe('DroidDock 管理');
    expect(toolSummary(item)).toContain('可用');
  });

  it('gives an install action for missing tools', () => {
    const item = diagnostic({ health: 'missing' });
    expect(toolHealthTone(item)).toBe('red');
    expect(toolActionLabel(item)).toBe('自动安装或手动选择');
  });

  it('gives a replacement action for incompatible tools', () => {
    const item = diagnostic({ health: 'incompatible_arch', arch: 'Mach-O 64-bit executable x86_64' });
    expect(toolActionLabel(item)).toBe('更换 Apple Silicon 版本');
  });

  it('gives an unsupported host action when the app runs outside Apple Silicon', () => {
    const item = diagnostic({ health: 'incompatible_arch', path: null, arch: 'x86_64', message: '当前版本仅支持 Apple Silicon Mac，暂不支持 Intel Mac' });
    expect(toolActionLabel(item)).toBe('当前 Mac 不受支持');
    expect(toolSummary(item)).toContain('不支持 Intel Mac');
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run: `npm run test -- src/domain/toolDiagnostics.test.ts`

Expected: FAIL because `toolDiagnostics.ts` does not exist.

- [ ] **Step 3: Implement helper**

Create `src/domain/toolDiagnostics.ts`:

```ts
import type { ToolDiagnostic, ToolHealth, ToolSource } from '../lib/ipc/types';

export function toolSourceLabel(source: ToolSource | null): string {
  if (source === 'configured') return '手动配置';
  if (source === 'bundled') return 'DroidDock 管理';
  if (source === 'android_sdk') return 'Android SDK';
  if (source === 'local_bin') return '本地用户目录';
  if (source === 'homebrew') return 'Homebrew';
  if (source === 'system_path') return '系统 PATH';
  return '未找到来源';
}

export function toolHealthTone(diagnostic: ToolDiagnostic): 'green' | 'red' | 'yellow' | 'gray' {
  if (diagnostic.health === 'ready') return 'green';
  if (diagnostic.health === 'missing') return 'red';
  if (diagnostic.health === 'incompatible_arch') return 'red';
  if (diagnostic.health === 'not_executable') return 'yellow';
  if (diagnostic.health === 'wrong_tool') return 'yellow';
  return 'yellow';
}

export function toolActionLabel(diagnostic: ToolDiagnostic): string {
  const unsupportedHost = diagnostic.health === 'incompatible_arch' && diagnostic.path === null;
  if (unsupportedHost) return '当前 Mac 不受支持';

  const actions: Record<ToolHealth, string> = {
    ready: '重新检测',
    missing: '自动安装或手动选择',
    not_executable: '选择可执行文件',
    wrong_tool: '重新选择正确工具',
    version_failed: '重新选择或重新安装',
    incompatible_arch: '更换 Apple Silicon 版本',
  };
  return actions[diagnostic.health];
}

export function toolSummary(diagnostic: ToolDiagnostic): string {
  if (diagnostic.health === 'ready') {
    return `${diagnostic.kind} 可用，来源：${toolSourceLabel(diagnostic.source)}`;
  }
  return diagnostic.message;
}
```

- [ ] **Step 4: Run tests**

Run: `npm run test -- src/domain/toolDiagnostics.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/domain/toolDiagnostics.ts src/domain/toolDiagnostics.test.ts
git commit -m "test: cover tool diagnostic display helpers"
```

---

## Task 6: Redesign SetupView Interaction

**Files:**
- Modify: `src/components/SetupView.vue`
- Modify: `src/styles.css`

- [ ] **Step 1: Replace flat path reads**

In `SetupView.vue`, replace old `store.toolStatus?.adb_path` and `store.toolStatus?.scrcpy_path` usage with a reusable card pattern over `store.toolStatus.adb` and `store.toolStatus.scrcpy`.

Add imports:

```ts
import { open } from '@tauri-apps/plugin-dialog';
import { ref } from 'vue';
import { errorUserMessage } from '../lib/ipc/errors';
import type { ToolDiagnostic } from '../lib/ipc/types';
import { toolActionLabel, toolHealthTone, toolSourceLabel, toolSummary } from '../domain/toolDiagnostics';
import { useUiStore } from '../stores/ui';
```

Preserve the existing `open` import if it is already present; it is required by `chooseToolPath`.

Add state:

```ts
const ui = useUiStore();
const setupError = ref('');

async function refreshTools() {
  setupError.value = '';
  await store.fetchToolStatus();
  ui.pushToast('工具状态已刷新', 'info');
}
```

- [ ] **Step 2: Update choose and clear flows**

Replace `chooseToolPath` and `installTools` with:

```ts
async function chooseToolPath(tool: 'adb' | 'scrcpy') {
  setupError.value = '';
  const selected = await open({
    title: `选择 ${tool} 可执行文件`,
    multiple: false,
    directory: false,
  });
  if (typeof selected !== 'string') return;

  try {
    await store.setToolPath(tool, selected);
    ui.pushToast(`${tool} 路径已更新`, 'success');
  } catch (error) {
    setupError.value = errorUserMessage(error);
    ui.pushToast(setupError.value, 'error');
  }
}

async function clearToolPath(tool: 'adb' | 'scrcpy') {
  setupError.value = '';
  try {
    await store.clearToolPath(tool);
    ui.pushToast(`已清除 ${tool} 手动路径`, 'success');
  } catch (error) {
    setupError.value = errorUserMessage(error);
  }
}

async function installTools() {
  setupError.value = '';
  try {
    await store.installTools();
    ui.pushToast('工具安装完成', 'success');
  } catch (error) {
    setupError.value = errorUserMessage(error);
    ui.pushToast(setupError.value, 'error');
  }
}
```

- [ ] **Step 3: Add diagnostic card markup**

Use this repeated card structure for adb and scrcpy:

```vue
<div class="tool-card">
  <div :class="['tool-icon', diagnostic.health === 'ready' ? 'ok' : 'missing']">...</div>
  <div class="tool-info">
    <div class="tool-name-row">
      <strong class="mono">{{ diagnostic.kind }}</strong>
      <span>{{ diagnostic.kind === 'adb' ? 'Android Debug Bridge' : 'Screen Copy' }}</span>
    </div>
    <div class="tool-path mono">{{ diagnostic.path || '未找到' }}</div>
    <div class="tool-tags">
      <StatusChip :tone="toolHealthTone(diagnostic)" :label="diagnostic.health === 'ready' ? '正常' : toolActionLabel(diagnostic)" />
      <StatusChip tone="gray" :label="toolSourceLabel(diagnostic.source)" />
      <StatusChip tone="gray" :label="diagnostic.version || '版本未知'" />
    </div>
    <p class="tool-message">{{ toolSummary(diagnostic) }}</p>
  </div>
  <div class="tool-actions">
    <button class="btn btn-ghost compact-button" @click="chooseToolPath(diagnostic.kind)">手动选择</button>
    <button v-if="diagnostic.source === 'configured'" class="btn btn-ghost compact-button" @click="clearToolPath(diagnostic.kind)">清除路径</button>
  </div>
</div>
```

- [ ] **Step 4: Replace alert with page-level feedback**

Add below install panel:

```vue
<div v-if="setupError" class="setup-error">{{ setupError }}</div>
<div v-if="store.busy.install" class="install-log">
  <div class="install-log-title">正在安装工具</div>
  <div class="install-log-line">下载、解压和验证可能需要几分钟，请保持网络连接。</div>
</div>
```

- [ ] **Step 5: Add styles**

In `src/styles.css`, add:

```css
.tool-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.tool-message {
  margin: 8px 0 0;
  color: var(--t2);
  font-size: 12px;
  line-height: 1.45;
}

.setup-error {
  border: 1px solid rgba(248, 113, 113, 0.3);
  background: rgba(248, 113, 113, 0.08);
  color: var(--red);
  border-radius: 8px;
  padding: 10px 12px;
  font-size: 12px;
  line-height: 1.5;
}

.install-log {
  border: 1px solid var(--border2);
  background: var(--bg3);
  border-radius: 8px;
  padding: 12px;
}

.install-log-title {
  color: var(--t1);
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 6px;
}

.install-log-line {
  color: var(--t3);
  font-size: 12px;
}
```

- [ ] **Step 6: Verify**

Run: `npm run test -- src/domain/toolDiagnostics.test.ts`

Expected: PASS.

Run: `npm run build`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/components/SetupView.vue src/styles.css
git commit -m "feat: improve tool setup diagnostics UI"
```

---

## Task 7: Final Verification

**Files:**
- No new files.

- [ ] **Step 1: Run frontend tests**

Run: `npm run test`

Expected: PASS.

- [ ] **Step 2: Run backend tests**

Run: `cd src-tauri && cargo test`

Expected: PASS.

- [ ] **Step 3: Run production build**

Run: `npm run build`

Expected: PASS.

- [ ] **Step 4: Manual UI check**

Run: `npm run tauri:dev`

Expected:
- 工具配置页可以显示 adb 和 scrcpy 的路径、来源、版本、状态和建议动作。
- 选择错误文件时，不保存路径，并显示中文错误。
- 清除自定义路径后，页面回到自动发现结果。
- 自动安装按钮进入安装中状态，失败时页面显示原因，不使用系统 alert。
- debug 模式下，如果下载 manifest 暂无 sha256，自动安装日志明确显示“开发模式”警告。
- 工具全部 ready 后，侧边栏工具状态和设备页启动状态正常。
- 非 Apple Silicon 环境如果被误运行，工具配置页显示“当前版本仅支持 Apple Silicon Mac，暂不支持 Intel Mac”，而不是提示用户更换 adb/scrcpy。

- [ ] **Step 5: Commit verification notes if needed**

If docs or screenshots are updated:

```bash
git add docs/superpowers/plans/2026-05-18-tool-setup-optimization.md
git commit -m "docs: plan tool setup optimization"
```

---

## Risk Notes

- `ToolHealth` 被 `lib.rs` 比较时需要从 `tools.rs` 可见，枚举必须是 `pub(crate)`。
- `set_tool_path` 先写入内存再诊断，但诊断失败不能保存配置；实现时不要调用 `save_config_atomic`。
- 固定下载 sha256 需要发布负责人确认。debug 模式允许空 hash 并输出警告；release 模式必须阻断空 hash。
- `system_path` 来源已经通过 `/usr/bin/which` 作为最后 fallback 纳入 `tool_candidates`；不要回退到旧的无来源路径。
- 旧前端仍可能读取 `toolStatus.adb_path`、`scrcpy_path`；迁移后用 `rg "adb_path|scrcpy_path" src` 检查。

## Self-Review

- Spec coverage: 覆盖 PRD 的工具检测、手动配置、自动安装、来源展示、完整性校验、不修改 PATH、不要求 Homebrew、不要求 sudo。
- Placeholder scan: 计划中的固定 sha256 明确标为发布前必须替换；debug 空值可测，release 空值会阻断安装，不是静默 TODO。
- Type consistency: 后端 `ToolKind` 序列化为 `adb/scrcpy`，前端 `ToolKind` 同名；`ToolDiagnostic.health` 与 `toolDiagnostics.ts` 分支一致。
