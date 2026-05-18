use serde::{Deserialize, Serialize};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use crate::{
    command::run_command,
    command::run_required,
    command::run_required_with_timeout,
    config::{config_dir, tools_dir, AppConfig},
    tool_manifest::DEFAULT_TOOL_MANIFEST,
};

const TOOL_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(180);
const SCRCPY_RELEASE_QUERY_TIMEOUT: Duration = Duration::from_secs(30);

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ToolDiagnostic {
    pub(crate) kind: ToolKind,
    pub(crate) path: Option<String>,
    pub(crate) source: Option<ToolSource>,
    pub(crate) version: Option<String>,
    pub(crate) arch: Option<String>,
    pub(crate) health: ToolHealth,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolStatus {
    pub(crate) adb: ToolDiagnostic,
    pub(crate) scrcpy: ToolDiagnostic,
    pub(crate) adb_ok: bool,
    pub(crate) scrcpy_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolCandidate {
    path: String,
    source: ToolSource,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolInstallResult {
    pub(crate) target: ToolInstallTarget,
    pub(crate) adb_path: Option<String>,
    pub(crate) scrcpy_path: Option<String>,
    pub(crate) logs: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ToolInstallTarget {
    Adb,
    Scrcpy,
    All,
}

impl ToolInstallTarget {
    pub(crate) fn kinds(self) -> Vec<ToolKind> {
        match self {
            ToolInstallTarget::Adb => vec![ToolKind::Adb],
            ToolInstallTarget::Scrcpy => vec![ToolKind::Scrcpy],
            ToolInstallTarget::All => vec![ToolKind::Adb, ToolKind::Scrcpy],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolInstallProgress {
    pub(crate) target: ToolInstallTarget,
    pub(crate) level: &'static str,
    pub(crate) message: String,
}

pub(crate) fn validate_executable(path: &str) -> bool {
    let path = Path::new(path);
    path.exists()
        && path.is_file()
        && path
            .metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

fn tool_name(kind: ToolKind) -> &'static str {
    match kind {
        ToolKind::Adb => "adb",
        ToolKind::Scrcpy => "scrcpy",
    }
}

fn tool_kind_from_name(name: &str) -> Option<ToolKind> {
    match name {
        "adb" => Some(ToolKind::Adb),
        "scrcpy" => Some(ToolKind::Scrcpy),
        _ => None,
    }
}

fn which_candidate(name: &str) -> Option<ToolCandidate> {
    let output = Command::new("/usr/bin/which").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!path.is_empty()).then_some(ToolCandidate {
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
                    path: format!(
                        "{home}/Library/Application Support/DroidDock/tools/platform-tools/adb"
                    ),
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
                    path: format!(
                        "{home}/Library/Application Support/DroidDock/tools/scrcpy/scrcpy"
                    ),
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
        }
    }

    if let Some(candidate) = which_candidate(tool_name(kind)) {
        candidates.push(candidate);
    }

    candidates
}

pub(crate) fn resolve_tool(name: &str, config: &AppConfig) -> Option<String> {
    let kind = tool_kind_from_name(name)?;
    let diagnostic = diagnose_tool(kind, config);
    (diagnostic.health == ToolHealth::Ready).then_some(diagnostic.path?)
}

fn is_apple_silicon_compatible_file_output(output: &str) -> bool {
    let output = output.to_ascii_lowercase();
    output.contains("arm64")
        || output.contains("universal binary")
        || output.contains("shell script")
        || output.contains("text executable")
}

fn executable_arch(path: &str) -> Option<String> {
    let result = run_command("/usr/bin/file", &[path]);
    result.ok.then_some(result.stdout.trim().to_string())
}

fn version_args(kind: ToolKind) -> &'static [&'static str] {
    match kind {
        ToolKind::Adb => &["version"],
        ToolKind::Scrcpy => &["--version"],
    }
}

fn parsed_version(kind: ToolKind, stdout: String) -> Option<String> {
    match kind {
        ToolKind::Adb => second_line(Some(stdout)),
        ToolKind::Scrcpy => first_line(Some(stdout)),
    }
}

fn command_identifies_tool(kind: ToolKind, stdout: &str) -> bool {
    let stdout = stdout.to_ascii_lowercase();
    match kind {
        ToolKind::Adb => stdout.contains("android debug bridge"),
        ToolKind::Scrcpy => stdout.contains("scrcpy"),
    }
}

fn diagnostic(
    kind: ToolKind,
    candidate: Option<&ToolCandidate>,
    version: Option<String>,
    arch: Option<String>,
    health: ToolHealth,
    message: impl Into<String>,
) -> ToolDiagnostic {
    ToolDiagnostic {
        kind,
        path: candidate.map(|candidate| candidate.path.clone()),
        source: candidate.map(|candidate| candidate.source.clone()),
        version,
        arch,
        health,
        message: message.into(),
    }
}

fn host_support_diagnostic_for_arch(kind: ToolKind, arch: &str) -> Option<ToolDiagnostic> {
    (arch != "aarch64").then(|| ToolDiagnostic {
        kind,
        path: None,
        source: None,
        version: None,
        arch: Some(arch.to_string()),
        health: ToolHealth::IncompatibleArch,
        message: "当前版本仅支持 Apple Silicon Mac，暂不支持 Intel Mac".to_string(),
    })
}

fn host_support_diagnostic(kind: ToolKind) -> Option<ToolDiagnostic> {
    host_support_diagnostic_for_arch(kind, std::env::consts::ARCH)
}

fn missing_diagnostic(kind: ToolKind) -> ToolDiagnostic {
    let name = tool_name(kind);
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

fn diagnose_candidate(kind: ToolKind, candidate: &ToolCandidate) -> ToolDiagnostic {
    let name = tool_name(kind);
    let path = Path::new(&candidate.path);
    if !path.exists() {
        return diagnostic(
            kind,
            Some(candidate),
            None,
            None,
            ToolHealth::Missing,
            format!("{name} 路径不存在"),
        );
    }

    let arch = executable_arch(&candidate.path);
    if !validate_executable(&candidate.path) {
        return diagnostic(
            kind,
            Some(candidate),
            None,
            arch,
            ToolHealth::NotExecutable,
            format!("{name} 文件不可执行"),
        );
    }

    if !arch
        .as_deref()
        .map(is_apple_silicon_compatible_file_output)
        .unwrap_or(false)
    {
        return diagnostic(
            kind,
            Some(candidate),
            None,
            arch,
            ToolHealth::IncompatibleArch,
            "当前工具不适合 Apple Silicon，请选择 arm64 或 universal 版本",
        );
    }

    let version_result = run_command(&candidate.path, version_args(kind));
    if !version_result.ok {
        return diagnostic(
            kind,
            Some(candidate),
            None,
            arch,
            ToolHealth::VersionFailed,
            format!("{name} 无法运行版本检查"),
        );
    }

    if !command_identifies_tool(kind, &version_result.stdout) {
        return diagnostic(
            kind,
            Some(candidate),
            parsed_version(kind, version_result.stdout),
            arch,
            ToolHealth::WrongTool,
            format!("选择的文件不是可用的 {name}"),
        );
    }

    diagnostic(
        kind,
        Some(candidate),
        parsed_version(kind, version_result.stdout),
        arch,
        ToolHealth::Ready,
        "工具可用",
    )
}

pub(crate) fn diagnose_configured_tool_path(kind: ToolKind, path: &str) -> ToolDiagnostic {
    diagnose_candidate(
        kind,
        &ToolCandidate {
            path: path.to_string(),
            source: ToolSource::Configured,
        },
    )
}

fn select_final_tool_diagnostic(
    kind: ToolKind,
    diagnostics: Vec<ToolDiagnostic>,
) -> ToolDiagnostic {
    if let Some(diagnostic) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.health == ToolHealth::Ready)
    {
        return diagnostic.clone();
    }

    if let Some(diagnostic) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.source == Some(ToolSource::Configured))
    {
        return diagnostic.clone();
    }

    if let Some(diagnostic) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.health != ToolHealth::Missing)
    {
        return diagnostic.clone();
    }

    missing_diagnostic(kind)
}

fn diagnose_tool(kind: ToolKind, config: &AppConfig) -> ToolDiagnostic {
    if let Some(diagnostic) = host_support_diagnostic(kind) {
        return diagnostic;
    }

    let mut diagnostics = Vec::new();
    for candidate in tool_candidates(kind, config) {
        let diagnostic = diagnose_candidate(kind, &candidate);
        if diagnostic.health == ToolHealth::Ready {
            return diagnostic;
        }
        diagnostics.push(diagnostic);
    }

    select_final_tool_diagnostic(kind, diagnostics)
}

fn download_file(url: &str, target: &Path) -> Result<(), String> {
    let target = target
        .to_str()
        .ok_or_else(|| "download target path is not valid UTF-8".to_string())?;
    run_required_with_timeout(
        "/usr/bin/curl",
        &[
            "-fL",
            "--connect-timeout",
            "15",
            "--max-time",
            "180",
            "--retry",
            "3",
            "-o",
            target,
            url,
        ],
        TOOL_DOWNLOAD_TIMEOUT,
    )
        .map_err(|error| error.user_message)?;
    Ok(())
}

fn unzip_archive(archive: &Path, destination: &Path) -> Result<(), String> {
    let archive = archive
        .to_str()
        .ok_or_else(|| "archive path is not valid UTF-8".to_string())?;
    let destination = destination
        .to_str()
        .ok_or_else(|| "destination path is not valid UTF-8".to_string())?;
    run_required("/usr/bin/unzip", &["-q", "-o", archive, "-d", destination])
        .map_err(|error| error.user_message)?;
    Ok(())
}

fn extract_archive(archive: &Path, destination: &Path) -> Result<(), String> {
    let archive_name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if archive_name.ends_with(".tar.gz") || archive_name.ends_with(".tgz") {
        fs::create_dir_all(destination).map_err(|error| error.to_string())?;
        let archive = archive
            .to_str()
            .ok_or_else(|| "archive path is not valid UTF-8".to_string())?;
        let destination = destination
            .to_str()
            .ok_or_else(|| "destination path is not valid UTF-8".to_string())?;
        run_required("/usr/bin/tar", &["-xzf", archive, "-C", destination])
            .map_err(|error| error.user_message)?;
        Ok(())
    } else {
        unzip_archive(archive, destination)
    }
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let path = path
        .to_str()
        .ok_or_else(|| "checksum path is not valid UTF-8".to_string())?;
    let result = run_required("/usr/bin/shasum", &["-a", "256", path])
        .map_err(|error| error.user_message)?;
    Ok(result
        .stdout
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string())
}

fn verify_sha256(path: &Path, expected: &str) -> Result<String, String> {
    let expected = expected.trim();
    if expected.is_empty() {
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
    if actual.eq_ignore_ascii_case(expected) {
        Ok(actual)
    } else {
        Err("下载文件校验失败，请重新安装或稍后再试".to_string())
    }
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        fs::remove_dir_all(target).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(target).map_err(|error| error.to_string())?;

    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|error| error.to_string())?;
            if source_path
                .metadata()
                .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
            {
                fs::set_permissions(&target_path, fs::Permissions::from_mode(0o755))
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    Ok(())
}

fn find_file_named(root: &Path, file_name: &str) -> Option<PathBuf> {
    for entry in fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file_named(&path, file_name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|name| name.to_str()) == Some(file_name) {
            return Some(path);
        }
    }
    None
}

fn find_scrcpy_macos_aarch64_asset(payload: &str) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(payload).map_err(|error| error.to_string())?;
    let assets = value
        .get("assets")
        .and_then(|assets| assets.as_array())
        .ok_or_else(|| "scrcpy release response did not include assets".to_string())?;

    assets
        .iter()
        .find_map(|asset| {
            let name = asset.get("name")?.as_str()?.to_ascii_lowercase();
            let url = asset.get("browser_download_url")?.as_str()?;
            (name.contains("macos")
                && name.contains("aarch64")
                && DEFAULT_TOOL_MANIFEST
                    .allowed_scrcpy_asset_suffixes
                    .iter()
                    .any(|suffix| name.ends_with(suffix)))
            .then_some(url.to_string())
        })
        .ok_or_else(|| "未找到 scrcpy macOS Apple Silicon 下载包".to_string())
}

fn push_install_log(
    target: ToolInstallTarget,
    logs: &mut Vec<String>,
    emit_progress: &mut impl FnMut(ToolInstallProgress),
    message: impl Into<String>,
) {
    let message = message.into();
    logs.push(message.clone());
    emit_progress(ToolInstallProgress {
        target,
        level: "info",
        message,
    });
}

fn install_platform_tools(
    target: ToolInstallTarget,
    logs: &mut Vec<String>,
    emit_progress: &mut impl FnMut(ToolInstallProgress),
    temp_dir: &Path,
    tools_dir: &Path,
) -> Result<String, String> {
    let archive = temp_dir.join("platform-tools.zip");
    let unzip_dir = temp_dir.join("platform-tools-unzip");
    push_install_log(
        target,
        logs,
        emit_progress,
        "下载 Android SDK Platform Tools",
    );
    download_file(DEFAULT_TOOL_MANIFEST.platform_tools.url, &archive)?;
    if cfg!(debug_assertions)
        && DEFAULT_TOOL_MANIFEST
            .platform_tools
            .sha256
            .trim()
            .is_empty()
    {
        push_install_log(
            target,
            logs,
            emit_progress,
            "开发模式：platform-tools 缺少 sha256，已跳过强校验",
        );
    }
    let sha256 = verify_sha256(&archive, DEFAULT_TOOL_MANIFEST.platform_tools.sha256)?;
    push_install_log(
        target,
        logs,
        emit_progress,
        format!("platform-tools sha256 {sha256}"),
    );
    push_install_log(target, logs, emit_progress, "解压 Android SDK Platform Tools");
    unzip_archive(&archive, &unzip_dir)?;

    let source = unzip_dir.join("platform-tools");
    let target_dir = tools_dir.join("platform-tools");
    copy_dir_recursive(&source, &target_dir)?;
    let adb = target_dir.join("adb");
    fs::set_permissions(&adb, fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;
    let adb_path = adb.to_string_lossy().to_string();
    run_required(&adb_path, &["version"]).map_err(|error| error.user_message)?;
    push_install_log(target, logs, emit_progress, "adb 安装并验证完成");
    Ok(adb_path)
}

fn install_scrcpy(
    target: ToolInstallTarget,
    logs: &mut Vec<String>,
    emit_progress: &mut impl FnMut(ToolInstallProgress),
    temp_dir: &Path,
    tools_dir: &Path,
) -> Result<String, String> {
    push_install_log(
        target,
        logs,
        emit_progress,
        "查询 scrcpy 最新 macOS Apple Silicon 版本",
    );
    let api_result = run_required_with_timeout(
        "/usr/bin/curl",
        &[
            "-fsSL",
            "--connect-timeout",
            "15",
            "--max-time",
            "30",
            "-H",
            "Accept: application/vnd.github+json",
            DEFAULT_TOOL_MANIFEST.scrcpy_release_api,
        ],
        SCRCPY_RELEASE_QUERY_TIMEOUT,
    )
    .map_err(|error| error.user_message)?;
    let url = find_scrcpy_macos_aarch64_asset(&api_result.stdout)?;
    let archive_name = url.rsplit('/').next().unwrap_or("scrcpy-download");
    let archive = temp_dir.join(archive_name);
    let unzip_dir = temp_dir.join("scrcpy-unzip");
    push_install_log(
        target,
        logs,
        emit_progress,
        "下载 scrcpy macOS Apple Silicon 包",
    );
    download_file(&url, &archive)?;
    let sha256 = file_sha256(&archive)?;
    push_install_log(target, logs, emit_progress, format!("scrcpy sha256 {sha256}"));
    push_install_log(
        target,
        logs,
        emit_progress,
        "scrcpy 当前使用 GitHub latest 下载源；固定版本 sha256 校验将在 manifest 固定下载 URL 后启用"
            .to_string(),
    );
    push_install_log(target, logs, emit_progress, "解压 scrcpy");
    extract_archive(&archive, &unzip_dir)?;

    let scrcpy_bin = find_file_named(&unzip_dir, "scrcpy")
        .ok_or_else(|| "scrcpy 下载包中未找到 scrcpy 可执行文件".to_string())?;
    let source = scrcpy_bin
        .parent()
        .ok_or_else(|| "scrcpy 可执行文件路径异常".to_string())?;
    let target_dir = tools_dir.join("scrcpy");
    copy_dir_recursive(source, &target_dir)?;
    let scrcpy = target_dir.join("scrcpy");
    fs::set_permissions(&scrcpy, fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;
    let scrcpy_path = scrcpy.to_string_lossy().to_string();
    run_required(&scrcpy_path, &["--version"]).map_err(|error| error.user_message)?;
    push_install_log(target, logs, emit_progress, "scrcpy 安装并验证完成");
    Ok(scrcpy_path)
}

fn first_line(text: Option<String>) -> Option<String> {
    text.and_then(|value| value.lines().next().map(ToOwned::to_owned))
}

fn second_line(text: Option<String>) -> Option<String> {
    text.and_then(|value| value.lines().nth(1).map(ToOwned::to_owned))
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

pub(crate) fn install_tools_into_config(
    target: ToolInstallTarget,
    mut emit_progress: impl FnMut(ToolInstallProgress),
) -> Result<ToolInstallResult, String> {
    let dir = tools_dir()?;
    let temp_dir = config_dir()?.join(format!("install-tmp-{target:?}").to_ascii_lowercase());
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&temp_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;

    let mut logs = Vec::new();
    push_install_log(target, &mut logs, &mut emit_progress, "准备工具安装目录");
    let mut adb_path = None;
    let mut scrcpy_path = None;
    for kind in target.kinds() {
        match kind {
            ToolKind::Adb => {
                adb_path = Some(install_platform_tools(
                    target,
                    &mut logs,
                    &mut emit_progress,
                    &temp_dir,
                    &dir,
                )?);
            }
            ToolKind::Scrcpy => {
                scrcpy_path = Some(install_scrcpy(
                    target,
                    &mut logs,
                    &mut emit_progress,
                    &temp_dir,
                    &dir,
                )?);
            }
        }
    }
    let _ = fs::remove_dir_all(&temp_dir);

    push_install_log(target, &mut logs, &mut emit_progress, "工具安装完成");
    Ok(ToolInstallResult {
        target,
        adb_path,
        scrcpy_path,
        logs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_target_expands_to_requested_tools() {
        assert_eq!(ToolInstallTarget::Adb.kinds(), vec![ToolKind::Adb]);
        assert_eq!(ToolInstallTarget::Scrcpy.kinds(), vec![ToolKind::Scrcpy]);
        assert_eq!(
            ToolInstallTarget::All.kinds(),
            vec![ToolKind::Adb, ToolKind::Scrcpy]
        );
    }

    #[test]
    fn validate_executable_rejects_plain_files_without_execute_bits() {
        let path = std::env::temp_dir().join(format!("droiddock-test-{}", crate::now_secs()));
        fs::write(&path, "not executable").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        assert!(!validate_executable(path.to_str().unwrap()));

        let _ = fs::remove_file(path);
    }

    #[test]
    #[cfg(debug_assertions)]
    fn verify_sha256_allows_empty_manifest_hash_in_debug() {
        let path =
            std::env::temp_dir().join(format!("droiddock-test-empty-hash-{}", crate::now_secs()));
        fs::write(&path, "payload").unwrap();

        let hash = verify_sha256(&path, "").unwrap();

        assert_eq!(
            hash,
            "239f59ed55e737c77147cf55ad0c1b030b6d7ee748a7426952f9b852d5a935e5"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn verify_sha256_rejects_wrong_hash() {
        let path =
            std::env::temp_dir().join(format!("droiddock-test-wrong-hash-{}", crate::now_secs()));
        fs::write(&path, "payload").unwrap();

        let error = verify_sha256(&path, "000000").unwrap_err();

        assert!(error.contains("校验失败"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn scrcpy_release_parser_prefers_macos_aarch64_zip_asset() {
        let payload = r#"{
          "assets": [
            { "name": "scrcpy-win64.zip", "browser_download_url": "https://example.test/win.zip" },
            { "name": "scrcpy-macos-aarch64-v3.3.3.zip", "browser_download_url": "https://example.test/scrcpy-macos-aarch64-v3.3.3.zip" }
          ]
        }"#;

        assert_eq!(
            find_scrcpy_macos_aarch64_asset(payload).unwrap(),
            "https://example.test/scrcpy-macos-aarch64-v3.3.3.zip"
        );
    }

    #[test]
    fn scrcpy_release_parser_accepts_current_macos_aarch64_tarball_asset() {
        let payload = r#"{
          "assets": [
            { "name": "scrcpy-macos-aarch64-v3.3.4.tar.gz", "browser_download_url": "https://example.test/scrcpy-macos-aarch64-v3.3.4.tar.gz" }
          ]
        }"#;

        assert_eq!(
            find_scrcpy_macos_aarch64_asset(payload).unwrap(),
            "https://example.test/scrcpy-macos-aarch64-v3.3.4.tar.gz"
        );
    }

    #[test]
    fn apple_silicon_arch_parser_accepts_arm64_universal_and_scripts() {
        assert!(is_apple_silicon_compatible_file_output(
            "Mach-O 64-bit executable arm64"
        ));
        assert!(is_apple_silicon_compatible_file_output(
            "Mach-O universal binary with 2 architectures"
        ));
        assert!(is_apple_silicon_compatible_file_output(
            "POSIX shell script text executable"
        ));
        assert!(!is_apple_silicon_compatible_file_output(
            "Mach-O 64-bit executable x86_64"
        ));
    }

    #[test]
    fn missing_tool_diagnostic_has_actionable_message() {
        let diagnostic = missing_diagnostic(ToolKind::Adb);

        assert_eq!(diagnostic.health, ToolHealth::Missing);
        assert_eq!(diagnostic.message, "未找到 adb，请自动安装或手动选择路径");
    }

    #[test]
    fn configured_path_diagnostic_reports_exact_selected_path() {
        let diagnostic = diagnose_configured_tool_path(ToolKind::Adb, "/tmp/droiddock-missing-adb");

        assert_eq!(
            diagnostic.path.as_deref(),
            Some("/tmp/droiddock-missing-adb")
        );
        assert_eq!(diagnostic.source, Some(ToolSource::Configured));
        assert_eq!(diagnostic.health, ToolHealth::Missing);
        assert_eq!(diagnostic.message, "adb 路径不存在");
    }

    #[test]
    fn final_diagnostic_collapses_auto_discovered_missing_failures() {
        let diagnostic = select_final_tool_diagnostic(
            ToolKind::Adb,
            vec![
                diagnostic(
                    ToolKind::Adb,
                    Some(&ToolCandidate {
                        path: "/tmp/droiddock-bundled-missing-adb".to_string(),
                        source: ToolSource::Bundled,
                    }),
                    None,
                    None,
                    ToolHealth::Missing,
                    "adb 路径不存在",
                ),
                diagnostic(
                    ToolKind::Adb,
                    Some(&ToolCandidate {
                        path: "/opt/homebrew/bin/adb".to_string(),
                        source: ToolSource::Homebrew,
                    }),
                    None,
                    None,
                    ToolHealth::Missing,
                    "adb 路径不存在",
                ),
            ],
        );

        assert_eq!(diagnostic.health, ToolHealth::Missing);
        assert_eq!(diagnostic.path, None);
        assert_eq!(diagnostic.source, None);
        assert_eq!(diagnostic.message, "未找到 adb，请自动安装或手动选择路径");
    }

    #[test]
    fn final_diagnostic_preserves_configured_missing_failure() {
        let diagnostic = select_final_tool_diagnostic(
            ToolKind::Adb,
            vec![
                diagnostic(
                    ToolKind::Adb,
                    Some(&ToolCandidate {
                        path: "/tmp/user-selected-adb".to_string(),
                        source: ToolSource::Configured,
                    }),
                    None,
                    None,
                    ToolHealth::Missing,
                    "adb 路径不存在",
                ),
                diagnostic(
                    ToolKind::Adb,
                    Some(&ToolCandidate {
                        path: "/opt/homebrew/bin/adb".to_string(),
                        source: ToolSource::Homebrew,
                    }),
                    None,
                    None,
                    ToolHealth::Missing,
                    "adb 路径不存在",
                ),
            ],
        );

        assert_eq!(diagnostic.path.as_deref(), Some("/tmp/user-selected-adb"));
        assert_eq!(diagnostic.source, Some(ToolSource::Configured));
        assert_eq!(diagnostic.health, ToolHealth::Missing);
        assert_eq!(diagnostic.message, "adb 路径不存在");
    }

    #[test]
    fn apple_silicon_arch_parser_rejects_x86_64() {
        assert!(!is_apple_silicon_compatible_file_output(
            "Mach-O 64-bit executable x86_64"
        ));
    }

    #[test]
    fn unsupported_host_arch_has_product_level_message() {
        let diagnostic = host_support_diagnostic_for_arch(ToolKind::Scrcpy, "x86_64").unwrap();

        assert_eq!(diagnostic.kind, ToolKind::Scrcpy);
        assert_eq!(diagnostic.health, ToolHealth::IncompatibleArch);
        assert_eq!(diagnostic.arch.as_deref(), Some("x86_64"));
        assert_eq!(
            diagnostic.message,
            "当前版本仅支持 Apple Silicon Mac，暂不支持 Intel Mac"
        );
    }
}
