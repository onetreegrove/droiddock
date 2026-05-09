use serde::Serialize;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    command::run_command,
    command::run_required,
    config::{config_dir, tools_dir, AppConfig},
    tool_manifest::DEFAULT_TOOL_MANIFEST,
};

#[derive(Debug, Serialize)]
pub(crate) struct ToolStatus {
    pub(crate) adb_path: Option<String>,
    pub(crate) scrcpy_path: Option<String>,
    pub(crate) adb_version: Option<String>,
    pub(crate) scrcpy_version: Option<String>,
    pub(crate) adb_arch: Option<String>,
    pub(crate) scrcpy_arch: Option<String>,
    pub(crate) adb_ok: bool,
    pub(crate) scrcpy_ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolInstallResult {
    pub(crate) adb_path: String,
    pub(crate) scrcpy_path: String,
    pub(crate) logs: Vec<String>,
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

fn tool_candidates(name: &str, config: &AppConfig) -> Vec<String> {
    let mut candidates = Vec::new();

    match name {
        "adb" => {
            if let Some(path) = &config.adb_path {
                candidates.push(path.clone());
            }
            if let Ok(home) = std::env::var("HOME") {
                candidates.push(format!("{home}/Library/Android/sdk/platform-tools/adb"));
                candidates.push(format!(
                    "{home}/Library/Application Support/DroidDock/tools/platform-tools/adb"
                ));
            }
            candidates.push("/opt/homebrew/bin/adb".to_string());
            candidates.push("/usr/local/bin/adb".to_string());
        }
        "scrcpy" => {
            if let Some(path) = &config.scrcpy_path {
                candidates.push(path.clone());
            }
            if let Ok(home) = std::env::var("HOME") {
                candidates.push(format!("{home}/.local/bin/scrcpy"));
                candidates.push(format!(
                    "{home}/Library/Application Support/DroidDock/tools/scrcpy/scrcpy"
                ));
            }
            candidates.push("/opt/homebrew/bin/scrcpy".to_string());
            candidates.push("/usr/local/bin/scrcpy".to_string());
        }
        _ => {}
    }

    candidates
}

pub(crate) fn resolve_tool(name: &str, config: &AppConfig) -> Option<String> {
    for candidate in tool_candidates(name, config) {
        if validate_executable(&candidate) {
            return Some(candidate);
        }
    }

    let output = Command::new("/usr/bin/which").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?.trim().to_string();
    validate_executable(&path).then_some(path)
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

fn download_file(url: &str, target: &Path) -> Result<(), String> {
    let target = target
        .to_str()
        .ok_or_else(|| "download target path is not valid UTF-8".to_string())?;
    run_required("/usr/bin/curl", &["-fL", "--retry", "3", "-o", target, url])
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

fn install_platform_tools(
    logs: &mut Vec<String>,
    temp_dir: &Path,
    tools_dir: &Path,
) -> Result<String, String> {
    let archive = temp_dir.join("platform-tools.zip");
    let unzip_dir = temp_dir.join("platform-tools-unzip");
    logs.push("下载 Android SDK Platform Tools".to_string());
    download_file(DEFAULT_TOOL_MANIFEST.platform_tools_url, &archive)?;
    logs.push(format!("platform-tools sha256 {}", file_sha256(&archive)?));
    unzip_archive(&archive, &unzip_dir)?;

    let source = unzip_dir.join("platform-tools");
    let target = tools_dir.join("platform-tools");
    copy_dir_recursive(&source, &target)?;
    let adb = target.join("adb");
    fs::set_permissions(&adb, fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;
    let adb_path = adb.to_string_lossy().to_string();
    run_required(&adb_path, &["version"]).map_err(|error| error.user_message)?;
    Ok(adb_path)
}

fn install_scrcpy(
    logs: &mut Vec<String>,
    temp_dir: &Path,
    tools_dir: &Path,
) -> Result<String, String> {
    logs.push("查询 scrcpy 最新 macOS Apple Silicon 版本".to_string());
    let api_result = run_required(
        "/usr/bin/curl",
        &[
            "-fsSL",
            "-H",
            "Accept: application/vnd.github+json",
            DEFAULT_TOOL_MANIFEST.scrcpy_release_api,
        ],
    )
    .map_err(|error| error.user_message)?;
    let url = find_scrcpy_macos_aarch64_asset(&api_result.stdout)?;
    let archive_name = url.rsplit('/').next().unwrap_or("scrcpy-download");
    let archive = temp_dir.join(archive_name);
    let unzip_dir = temp_dir.join("scrcpy-unzip");
    logs.push("下载 scrcpy macOS Apple Silicon 包".to_string());
    download_file(&url, &archive)?;
    logs.push(format!("scrcpy sha256 {}", file_sha256(&archive)?));
    extract_archive(&archive, &unzip_dir)?;

    let scrcpy_bin = find_file_named(&unzip_dir, "scrcpy")
        .ok_or_else(|| "scrcpy 下载包中未找到 scrcpy 可执行文件".to_string())?;
    let source = scrcpy_bin
        .parent()
        .ok_or_else(|| "scrcpy 可执行文件路径异常".to_string())?;
    let target = tools_dir.join("scrcpy");
    copy_dir_recursive(source, &target)?;
    let scrcpy = target.join("scrcpy");
    fs::set_permissions(&scrcpy, fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;
    let scrcpy_path = scrcpy.to_string_lossy().to_string();
    run_required(&scrcpy_path, &["--version"]).map_err(|error| error.user_message)?;
    Ok(scrcpy_path)
}

fn first_line(text: Option<String>) -> Option<String> {
    text.and_then(|value| value.lines().next().map(ToOwned::to_owned))
}

fn second_line(text: Option<String>) -> Option<String> {
    text.and_then(|value| value.lines().nth(1).map(ToOwned::to_owned))
}

pub(crate) fn get_tool_status_for_config(config: &AppConfig) -> Result<ToolStatus, String> {
    let adb_path = resolve_tool("adb", config);
    let scrcpy_path = resolve_tool("scrcpy", config);
    let adb_version = adb_path
        .as_deref()
        .map(|path| run_command(path, &["version"]))
        .and_then(|result| result.ok.then_some(result.stdout))
        .and_then(|text| second_line(Some(text)));
    let adb_arch = adb_path.as_deref().and_then(executable_arch);
    let scrcpy_version = scrcpy_path
        .as_deref()
        .map(|path| run_command(path, &["--version"]))
        .and_then(|result| result.ok.then_some(result.stdout))
        .and_then(|text| first_line(Some(text)));
    let scrcpy_arch = scrcpy_path.as_deref().and_then(executable_arch);
    let adb_arch_ok = adb_arch
        .as_deref()
        .map(is_apple_silicon_compatible_file_output)
        .unwrap_or(false);
    let scrcpy_arch_ok = scrcpy_arch
        .as_deref()
        .map(is_apple_silicon_compatible_file_output)
        .unwrap_or(false);

    Ok(ToolStatus {
        adb_ok: adb_version.is_some() && adb_arch_ok,
        scrcpy_ok: scrcpy_version.is_some() && scrcpy_arch_ok,
        adb_path,
        scrcpy_path,
        adb_version,
        scrcpy_version,
        adb_arch,
        scrcpy_arch,
    })
}

pub(crate) fn install_tools_into_config(
    config: &mut AppConfig,
) -> Result<ToolInstallResult, String> {
    let dir = tools_dir()?;
    let temp_dir = config_dir()?.join("install-tmp");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&temp_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;

    let mut logs = Vec::new();
    let adb_path = install_platform_tools(&mut logs, &temp_dir, &dir)?;
    let scrcpy_path = install_scrcpy(&mut logs, &temp_dir, &dir)?;
    let _ = fs::remove_dir_all(&temp_dir);

    config.adb_path = Some(adb_path.clone());
    config.scrcpy_path = Some(scrcpy_path.clone());

    logs.push("工具安装完成".to_string());
    Ok(ToolInstallResult {
        adb_path,
        scrcpy_path,
        logs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_executable_rejects_plain_files_without_execute_bits() {
        let path = std::env::temp_dir().join(format!("droiddock-test-{}", crate::now_secs()));
        fs::write(&path, "not executable").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        assert!(!validate_executable(path.to_str().unwrap()));

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
}
