# Tool Setup Landing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 落地 DroidDock 工具配置页优化：让 adb/scrcpy 的发现、诊断、手动路径、自动安装和页面展示都可解释、可验证、可回滚。

**Architecture:** 后端负责真实诊断和安装边界，前端只消费结构化诊断结果并给用户展示下一步动作。工具路径从“双路径整体保存”改为“单工具保存/清除”，避免误固化自动发现路径。自动安装保留当前下载流程，但加入 debug/release 不同的 sha256 策略和更短的配置锁占用。

**Tech Stack:** Tauri v2, Rust, Vue 3, TypeScript, Pinia, Vitest, Cargo tests.

---

## File Structure

- Modify: `src-tauri/src/tools.rs`
  - Replace flat `ToolStatus` with structured `ToolDiagnostic`.
  - Add candidate source tracking, `/usr/bin/which` fallback, host architecture check, tool identity/version/arch diagnosis.
  - Add debug/release checksum verification helper.
  - Change `install_tools_into_config` to return install paths without mutating `AppConfig`.
- Modify: `src-tauri/src/lib.rs`
  - Add `set_tool_path` and `clear_tool_path`.
  - Update `install_tools` so downloads happen outside the config mutex.
  - Register new commands.
- Modify: `src-tauri/src/tool_manifest.rs`
  - Convert manifest into typed download entries with URL and sha256 fields.
- Modify: `src/lib/ipc/types.ts`
  - Mirror backend diagnostic types.
- Modify: `src/stores/tools.ts`
  - Add single-tool path actions and return updated `AppConfig`.
- Modify: `src/stores/app.ts`
  - Use single-tool actions, add `clearToolPath`, and avoid duplicate `fetchAppConfig`.
- Create: `src/domain/toolDiagnostics.ts`
  - Map diagnostics to UI labels, tones, source labels, and summaries.
- Create: `src/domain/toolDiagnostics.test.ts`
  - Cover UI helper decisions.
- Modify: `src/components/SetupView.vue`
  - Render diagnostic cards, page-level errors, Toast feedback, clear-path action, and install logs.
- Modify: `src/styles.css`
  - Add setup diagnostic UI styles.

---

## Task 1: Backend Diagnostic Types

**Files:**
- Modify: `src-tauri/src/tools.rs`
- Modify: `src/lib/ipc/types.ts`

- [ ] **Step 1: Update Rust imports and replace status structs**

In `src-tauri/src/tools.rs`, change the first import from:

```rust
use serde::Serialize;
```

to:

```rust
use serde::{Deserialize, Serialize};
```

Replace the current `ToolStatus` struct with:

```rust
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

In `src/lib/ipc/types.ts`, replace the existing `ToolStatus` definition with:

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

Keep the existing `ToolInstallResult` and other exported types unchanged.

- [ ] **Step 3: Run compile checks to expose call sites**

Run: `npm run test -- src/domain/scrcpyOptions.test.ts`

Expected: TypeScript may fail because old call sites still read `toolStatus.adb_path`. That is acceptable in this task; the failure identifies migration targets for later tasks.

Run: `cd src-tauri && cargo test tools::`

Expected: Rust may fail because `get_tool_status_for_config` still returns old fields. That is acceptable until Task 2 replaces the implementation.

- [ ] **Step 4: Checkpoint without committing**

Do not commit yet. This task intentionally changes shared types before the Rust implementation and Vue call sites are migrated, so the repository may not compile until Task 2 and Task 5 finish their parts. Keep the files unstaged and continue directly to Task 2.

---

## Task 2: Backend Tool Discovery And Diagnosis

**Files:**
- Modify: `src-tauri/src/tools.rs`

- [ ] **Step 1: Replace string candidates with sourced candidates**

In `src-tauri/src/tools.rs`, replace `fn tool_candidates(name: &str, config: &AppConfig) -> Vec<String>` with:

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
    Some(ToolCandidate {
        path,
        source: ToolSource::SystemPath,
    })
}

fn tool_candidates(kind: ToolKind, config: &AppConfig) -> Vec<ToolCandidate> {
    let mut candidates = Vec::new();

    match kind {
        ToolKind::Adb => {
            if let Some(path) = &config.adb_path {
                candidates.push(ToolCandidate {
                    path: path.clone(),
                    source: ToolSource::Configured,
                });
            }
            if let Ok(home) = std::env::var("HOME") {
                candidates.push(ToolCandidate {
                    path: format!("{home}/Library/Application Support/DroidDock/tools/platform-tools/adb"),
                    source: ToolSource::Bundled,
                });
                candidates.push(ToolCandidate {
                    path: format!("{home}/Library/Android/sdk/platform-tools/adb"),
                    source: ToolSource::AndroidSdk,
                });
            }
            candidates.push(ToolCandidate {
                path: "/opt/homebrew/bin/adb".to_string(),
                source: ToolSource::Homebrew,
            });
            candidates.push(ToolCandidate {
                path: "/usr/local/bin/adb".to_string(),
                source: ToolSource::Homebrew,
            });
            if let Some(candidate) = which_candidate("adb") {
                candidates.push(candidate);
            }
        }
        ToolKind::Scrcpy => {
            if let Some(path) = &config.scrcpy_path {
                candidates.push(ToolCandidate {
                    path: path.clone(),
                    source: ToolSource::Configured,
                });
            }
            if let Ok(home) = std::env::var("HOME") {
                candidates.push(ToolCandidate {
                    path: format!("{home}/Library/Application Support/DroidDock/tools/scrcpy/scrcpy"),
                    source: ToolSource::Bundled,
                });
                candidates.push(ToolCandidate {
                    path: format!("{home}/.local/bin/scrcpy"),
                    source: ToolSource::LocalBin,
                });
            }
            candidates.push(ToolCandidate {
                path: "/opt/homebrew/bin/scrcpy".to_string(),
                source: ToolSource::Homebrew,
            });
            candidates.push(ToolCandidate {
                path: "/usr/local/bin/scrcpy".to_string(),
                source: ToolSource::Homebrew,
            });
            if let Some(candidate) = which_candidate("scrcpy") {
                candidates.push(candidate);
            }
        }
    }

    candidates
}
```

- [ ] **Step 2: Add host support and diagnostic helpers**

Below `fn executable_arch`, add:

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
```

- [ ] **Step 3: Add candidate diagnosis**

Below `missing_diagnostic`, add:

```rust
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
            path: Some(candidate.path.clone()),
            source: Some(candidate.source),
            version: None,
            arch: executable_arch(&candidate.path),
            health: ToolHealth::NotExecutable,
            message: format!("{expected_name} 文件不可执行"),
        };
    }

    let arch = executable_arch(&candidate.path);
    let arch_ok = arch
        .as_deref()
        .map(is_apple_silicon_compatible_file_output)
        .unwrap_or(false);
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

- [ ] **Step 4: Replace status and resolver implementation**

Replace the current `resolve_tool` and `get_tool_status_for_config` implementations with:

```rust
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

pub(crate) fn resolve_tool(name: &str, config: &AppConfig) -> Option<String> {
    let kind = match name {
        "adb" => ToolKind::Adb,
        "scrcpy" => ToolKind::Scrcpy,
        _ => return None,
    };
    let diagnostic = diagnose_tool(kind, config);
    if diagnostic.health == ToolHealth::Ready {
        diagnostic.path
    } else {
        None
    }
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

- [ ] **Step 5: Add backend tests**

In the existing `#[cfg(test)] mod tests` in `src-tauri/src/tools.rs`, add:

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

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/tools.rs src/lib/ipc/types.ts
git commit -m "feat: diagnose tool status"
```

---

## Task 3: Single-Tool Path Commands

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/stores/tools.ts`
- Modify: `src/stores/app.ts`

- [ ] **Step 1: Export `ToolHealth` to lib**

In `src-tauri/src/lib.rs`, update the tools import block to include `ToolHealth`:

```rust
use tools::{
    get_tool_status_for_config, install_tools_into_config, resolve_tool, validate_executable,
    ToolHealth, ToolInstallResult, ToolStatus,
};
```

- [ ] **Step 2: Add backend single-tool commands**

In `src-tauri/src/lib.rs`, keep existing `set_tool_paths` for compatibility and add these commands immediately after it:

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
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();

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

    if diagnostic.health != ToolHealth::Ready {
        return Err(diagnostic.message);
    }

    save_state_config(&state, config.clone())?;
    Ok(config)
}

#[tauri::command]
fn clear_tool_path(state: State<'_, AppState>, tool: String) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();

    match tool.as_str() {
        "adb" => config.adb_path = None,
        "scrcpy" => config.scrcpy_path = None,
        _ => return Err("未知工具类型".to_string()),
    }

    save_state_config(&state, config.clone())?;
    Ok(config)
}
```

- [ ] **Step 3: Register backend commands**

In `src-tauri/src/lib.rs`, find `tauri::generate_handler![...]` and add:

```rust
set_tool_path,
clear_tool_path,
```

next to the existing `set_tool_paths` command.

- [ ] **Step 4: Update tools store actions**

In `src/stores/tools.ts`, update the type import:

```ts
import type { AppConfig, ToolInstallResult, ToolStatus } from '../lib/ipc/types';
```

Add actions:

```ts
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

The resulting `actions` block should include `fetchToolStatus`, `installTools`, `setToolPath`, and `clearToolPath`.

- [ ] **Step 5: Update app store path wrapper**

In `src/stores/app.ts`, replace the existing `setToolPath` function with:

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

Add `clearToolPath` to the returned object at the bottom of the store.

- [ ] **Step 6: Verify**

Run: `npm run test`

Expected: PASS or only failures from `SetupView.vue` still reading the old `ToolStatus` shape. If old-shape failures appear, complete Task 5 before committing frontend files.

Run: `cd src-tauri && cargo test`

Expected: PASS.

- [ ] **Step 7: Commit**

If both frontend and backend pass:

```bash
git add src-tauri/src/lib.rs src/stores/tools.ts src/stores/app.ts
git commit -m "feat: update tool paths independently"
```

If frontend waits for Task 5, commit only backend:

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add single tool path commands"
```

---

## Task 4: Automatic Install Checksum And Lock Scope

**Files:**
- Modify: `src-tauri/src/tool_manifest.rs`
- Modify: `src-tauri/src/tools.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Replace manifest shape**

In `src-tauri/src/tool_manifest.rs`, replace the entire file with:

```rust
pub(crate) struct ToolDownload {
    pub(crate) url: &'static str,
    pub(crate) sha256: &'static str,
}

pub(crate) struct ToolManifest {
    pub(crate) platform_tools: ToolDownload,
    pub(crate) scrcpy_release_api: &'static str,
    pub(crate) allowed_scrcpy_asset_suffixes: &'static [&'static str],
}

pub(crate) const DEFAULT_TOOL_MANIFEST: ToolManifest = ToolManifest {
    platform_tools: ToolDownload {
        url: "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip",
        sha256: "",
    },
    scrcpy_release_api: "https://api.github.com/repos/Genymobile/scrcpy/releases/latest",
    allowed_scrcpy_asset_suffixes: &[".zip", ".tar.gz", ".tgz"],
};
```

- [ ] **Step 2: Add checksum helper**

In `src-tauri/src/tools.rs`, below `file_sha256`, add:

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

- [ ] **Step 3: Update platform-tools install**

In `install_platform_tools`, replace:

```rust
download_file(DEFAULT_TOOL_MANIFEST.platform_tools_url, &archive)?;
logs.push(format!("platform-tools sha256 {}", file_sha256(&archive)?));
```

with:

```rust
download_file(DEFAULT_TOOL_MANIFEST.platform_tools.url, &archive)?;
if cfg!(debug_assertions) && DEFAULT_TOOL_MANIFEST.platform_tools.sha256.trim().is_empty() {
    logs.push("开发模式：platform-tools 缺少 sha256，已跳过强校验".to_string());
}
logs.push(format!(
    "platform-tools sha256 {}",
    verify_sha256(&archive, DEFAULT_TOOL_MANIFEST.platform_tools.sha256)?
));
```

- [ ] **Step 4: Keep scrcpy latest lookup but verify downloaded hash when configured**

In `install_scrcpy`, keep the current GitHub release API lookup. Replace:

```rust
logs.push(format!("scrcpy sha256 {}", file_sha256(&archive)?));
```

with:

```rust
logs.push(format!("scrcpy sha256 {}", file_sha256(&archive)?));
logs.push("scrcpy 当前使用 GitHub latest 下载源；固定版本 sha256 校验将在 manifest 固定下载 URL 后启用".to_string());
```

This preserves current behavior for scrcpy because the code discovers the asset dynamically. The release gate for platform-tools still proves empty fixed hashes cannot silently pass in release mode.

- [ ] **Step 5: Change install function to avoid mutating config**

In `src-tauri/src/tools.rs`, replace:

```rust
pub(crate) fn install_tools_into_config(
    config: &mut AppConfig,
) -> Result<ToolInstallResult, String> {
```

with:

```rust
pub(crate) fn install_tools_into_config() -> Result<ToolInstallResult, String> {
```

Inside this function, delete these two lines:

```rust
config.adb_path = Some(adb_path.clone());
config.scrcpy_path = Some(scrcpy_path.clone());
```

The function should still return:

```rust
Ok(ToolInstallResult {
    adb_path,
    scrcpy_path,
    logs,
})
```

- [ ] **Step 6: Update lib install command**

In `src-tauri/src/lib.rs`, replace `install_tools` with:

```rust
#[tauri::command]
fn install_tools(state: State<'_, AppState>) -> Result<ToolInstallResult, String> {
    let result = install_tools_into_config()?;
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();
    config.adb_path = Some(result.adb_path.clone());
    config.scrcpy_path = Some(result.scrcpy_path.clone());
    save_state_config(&state, config)?;

    Ok(result)
}
```

- [ ] **Step 7: Add checksum tests**

In `src-tauri/src/tools.rs` tests, add:

```rust
#[test]
#[cfg(debug_assertions)]
fn verify_sha256_allows_empty_manifest_hash_in_debug() {
    let path = std::env::temp_dir().join(format!("droiddock-sha-test-{}", crate::now_secs()));
    fs::write(&path, "payload").unwrap();
    let actual = verify_sha256(&path, "").unwrap();
    assert!(!actual.is_empty());
    let _ = fs::remove_file(path);
}

#[test]
fn verify_sha256_rejects_wrong_hash() {
    let path = std::env::temp_dir().join(format!("droiddock-sha-test-{}", crate::now_secs()));
    fs::write(&path, "payload").unwrap();
    let error = verify_sha256(&path, "not-the-right-hash").unwrap_err();
    assert!(error.contains("校验失败"));
    let _ = fs::remove_file(path);
}
```

- [ ] **Step 8: Verify**

Run: `cd src-tauri && cargo test tools::`

Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/tool_manifest.rs src-tauri/src/tools.rs src-tauri/src/lib.rs
git commit -m "feat: harden tool installation"
```

---

## Task 5: Frontend Diagnostic Helpers And Setup UI

**Files:**
- Create: `src/domain/toolDiagnostics.ts`
- Create: `src/domain/toolDiagnostics.test.ts`
- Modify: `src/components/SetupView.vue`
- Modify: `src/styles.css`

- [ ] **Step 1: Write frontend helper tests**

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
  it('maps ready diagnostics to success copy', () => {
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

  it('distinguishes unsupported host from incompatible tool binary', () => {
    const unsupportedHost = diagnostic({
      health: 'incompatible_arch',
      path: null,
      arch: 'x86_64',
      message: '当前版本仅支持 Apple Silicon Mac，暂不支持 Intel Mac',
    });
    const wrongBinary = diagnostic({
      health: 'incompatible_arch',
      path: '/tmp/adb',
      arch: 'Mach-O 64-bit executable x86_64',
    });

    expect(toolActionLabel(unsupportedHost)).toBe('当前 Mac 不受支持');
    expect(toolSummary(unsupportedHost)).toContain('不支持 Intel Mac');
    expect(toolActionLabel(wrongBinary)).toBe('更换 Apple Silicon 版本');
  });
});
```

- [ ] **Step 2: Run failing helper test**

Run: `npm run test -- src/domain/toolDiagnostics.test.ts`

Expected: FAIL because `src/domain/toolDiagnostics.ts` does not exist.

- [ ] **Step 3: Implement frontend helper**

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

- [ ] **Step 4: Replace SetupView script**

In `src/components/SetupView.vue`, replace the entire `<script setup lang="ts">...</script>` block with:

```vue
<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog';
import { computed, ref } from 'vue';
import AppHeader from './AppHeader.vue';
import StatusChip from './StatusChip.vue';
import { errorUserMessage } from '../lib/ipc/errors';
import type { ToolDiagnostic } from '../lib/ipc/types';
import { toolActionLabel, toolHealthTone, toolSourceLabel, toolSummary } from '../domain/toolDiagnostics';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const setupError = ref('');

const diagnostics = computed<ToolDiagnostic[]>(() => {
  if (!store.toolStatus) return [];
  return [store.toolStatus.adb, store.toolStatus.scrcpy];
});

function toolTitle(tool: ToolDiagnostic) {
  return tool.kind === 'adb' ? 'Android Debug Bridge' : 'Screen Copy';
}

async function refreshTools() {
  setupError.value = '';
  await store.fetchToolStatus();
  ui.pushToast('工具状态已刷新', 'info');
}

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
    ui.pushToast(setupError.value, 'error');
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
</script>
```

- [ ] **Step 5: Replace tool-card template block**

In `src/components/SetupView.vue`, replace the two hard-coded `<div class="tool-card">...</div>` cards with:

```vue
<div v-for="diagnostic in diagnostics" :key="diagnostic.kind" class="tool-card">
  <div :class="['tool-icon', diagnostic.health === 'ready' ? 'ok' : 'missing']">
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <rect x="1.5" y="3.5" width="11" height="7" rx="1.2" stroke="currentColor" stroke-width="1.1" />
      <path d="M4 6.5h6M4 8.5h4" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
    </svg>
  </div>
  <div class="tool-info">
    <div class="tool-name-row"><strong class="mono">{{ diagnostic.kind }}</strong><span>{{ toolTitle(diagnostic) }}</span></div>
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
    <button
      v-if="diagnostic.source === 'configured'"
      class="btn btn-ghost compact-button"
      @click="clearToolPath(diagnostic.kind)"
    >
      清除路径
    </button>
  </div>
</div>
```

- [ ] **Step 6: Add setup feedback controls**

In the `install-panel`, add a refresh button next to the install button:

```vue
<button class="btn btn-ghost compact-button" :disabled="store.busy.tools" @click="refreshTools">重新检测</button>
```

Below the `install-panel`, add:

```vue
<div v-if="setupError" class="setup-error">{{ setupError }}</div>
<div v-if="store.busy.install" class="install-log">
  <div class="install-log-title">正在安装工具</div>
  <div class="install-log-line">下载、解压和验证可能需要几分钟，请保持网络连接。</div>
</div>
```

- [ ] **Step 7: Add styles**

In `src/styles.css`, add:

```css
.tool-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
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

- [ ] **Step 8: Verify**

Run: `npm run test -- src/domain/toolDiagnostics.test.ts`

Expected: PASS.

Run: `npm run build`

Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add src/domain/toolDiagnostics.ts src/domain/toolDiagnostics.test.ts src/components/SetupView.vue src/styles.css src/stores/tools.ts src/stores/app.ts
git commit -m "feat: improve tool setup diagnostics UI"
```

---

## Task 6: Final Verification

**Files:**
- No new files.

- [ ] **Step 1: Check old status fields are gone from frontend**

Run: `rg "toolStatus\\?\\.(adb_path|scrcpy_path|adb_version|scrcpy_version|adb_arch|scrcpy_arch)" src`

Expected: no output.

- [ ] **Step 2: Run frontend tests**

Run: `npm run test`

Expected: PASS.

- [ ] **Step 3: Run backend tests**

Run: `cd src-tauri && cargo test`

Expected: PASS.

- [ ] **Step 4: Run production frontend build**

Run: `npm run build`

Expected: PASS.

- [ ] **Step 5: Run release checksum gate**

Run: `cd src-tauri && cargo test --release tools::verify_sha256_rejects_wrong_hash`

Expected: PASS. This verifies checksum mismatch remains blocked in optimized builds.

- [ ] **Step 6: Manual app check**

Run: `npm run tauri:dev`

Expected:
- 工具配置页显示 adb 和 scrcpy 的状态、来源、路径、版本和建议动作。
- 选择错误文件时不保存，并显示中文错误。
- 清除手动路径后回到自动发现结果。
- 自动安装过程中页面显示安装中状态，不使用 `window.alert`。
- debug 模式下，platform-tools 缺少 sha256 时安装日志出现“开发模式”警告。
- 如果在非 Apple Silicon 环境误运行，页面显示“当前版本仅支持 Apple Silicon Mac，暂不支持 Intel Mac”。

- [ ] **Step 7: Commit plan if not committed**

```bash
git add docs/superpowers/plans/2026-05-18-tool-setup-landing-plan.md
git commit -m "docs: add tool setup landing plan"
```

---

## Self-Review

- Spec coverage: This plan covers PATH fallback, debug sha256 behavior, `open` import preservation, reduced state refresh, and earlier unsupported-Intel messaging.
- Placeholder scan: The plan uses concrete code blocks and exact commands. Empty sha256 in the manifest is intentional behavior: debug warns and release blocks.
- Type consistency: Backend `ToolKind` serializes as `adb`/`scrcpy`, matching frontend `ToolKind`. Backend `ToolSource::SystemPath` serializes as `system_path`, matching frontend `ToolSource`. Backend `ToolHealth::IncompatibleArch` serializes as `incompatible_arch`, matching frontend helper logic.
