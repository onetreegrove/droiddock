use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;

const LOG_LIMIT: usize = 400;
const DEFAULT_TCPIP_PORT: u16 = 5555;
const PLATFORM_TOOLS_URL: &str =
    "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip";
const SCRCPY_RELEASE_API: &str = "https://api.github.com/repos/Genymobile/scrcpy/releases/latest";

#[derive(Debug, Default)]
struct AppState {
    config: Mutex<AppConfig>,
    sessions: Mutex<HashMap<String, SessionEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct AppConfig {
    adb_path: Option<String>,
    scrcpy_path: Option<String>,
    device_aliases: HashMap<String, String>,
    recent_endpoints: Vec<String>,
    default_scrcpy_options: ScrcpyOptions,
    default_preset_id: String,
    device_scrcpy_options: HashMap<String, DeviceOptionEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            adb_path: None,
            scrcpy_path: None,
            device_aliases: HashMap::new(),
            recent_endpoints: Vec::new(),
            default_scrcpy_options: ScrcpyOptions::default(),
            default_preset_id: "daily".to_string(),
            device_scrcpy_options: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ToolStatus {
    adb_path: Option<String>,
    scrcpy_path: Option<String>,
    adb_version: Option<String>,
    scrcpy_version: Option<String>,
    adb_arch: Option<String>,
    scrcpy_arch: Option<String>,
    adb_ok: bool,
    scrcpy_ok: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CommandResult {
    ok: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ToolInstallResult {
    adb_path: String,
    scrcpy_path: String,
    logs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Device {
    serial: String,
    state: String,
    model: Option<String>,
    product: Option<String>,
    connection: String,
    alias: Option<String>,
    raw: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScrcpyOptions {
    max_size: Option<u32>,
    max_fps: Option<u32>,
    video_bit_rate: Option<String>,
    video_codec: Option<String>,
    no_audio: Option<bool>,
    no_control: Option<bool>,
    stay_awake: Option<bool>,
    turn_screen_off: Option<bool>,
    show_touches: Option<bool>,
    always_on_top: Option<bool>,
    fullscreen: Option<bool>,
}

impl Default for ScrcpyOptions {
    fn default() -> Self {
        Self {
            max_size: Some(1920),
            max_fps: Some(60),
            video_bit_rate: None,
            video_codec: Some("default".to_string()),
            no_audio: Some(true),
            no_control: None,
            stay_awake: Some(true),
            turn_screen_off: None,
            show_touches: None,
            always_on_top: None,
            fullscreen: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceOptionEntry {
    preset_id: Option<String>,
    options: ScrcpyOptions,
    updated_at: u64,
}

#[derive(Debug, Clone, Serialize)]
struct SessionInfo {
    session_id: String,
    serial: String,
    alias: Option<String>,
    pid: u32,
    status: String,
    started_at: u64,
    connection: String,
    args: Vec<String>,
    last_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionLogLine {
    timestamp: u64,
    level: String,
    message: String,
}

#[derive(Debug)]
struct SessionEntry {
    info: SessionInfo,
    child: Child,
    logs: Arc<Mutex<VecDeque<SessionLogLine>>>,
}

#[derive(Debug, Deserialize)]
struct PairRequest {
    host: String,
    pair_port: u16,
    pairing_code: String,
    connect_host: Option<String>,
    connect_port: Option<u16>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn config_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("DroidDock"))
}

fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("config.json"))
}

fn tools_dir() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("tools"))
}

fn load_config() -> AppConfig {
    let Ok(path) = config_path() else {
        return AppConfig::default();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return AppConfig::default();
    };

    serde_json::from_str(&content).unwrap_or_default()
}

fn save_config(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let content = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    fs::write(dir.join("config.json"), content).map_err(|error| error.to_string())
}

fn validate_executable(path: &str) -> bool {
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

fn resolve_tool(name: &str, config: &AppConfig) -> Option<String> {
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

fn run_command(path: &str, args: &[&str]) -> CommandResult {
    match Command::new(path).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let ok = output.status.success();
            let message = if ok {
                stdout.trim().to_string()
            } else {
                translate_error(&stdout, &stderr)
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

fn run_required(path: &str, args: &[&str]) -> Result<CommandResult, String> {
    let result = run_command(path, args);
    result.ok.then_some(result.clone()).ok_or(result.message)
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
    run_required("/usr/bin/curl", &["-fL", "--retry", "3", "-o", target, url])?;
    Ok(())
}

fn unzip_archive(archive: &Path, destination: &Path) -> Result<(), String> {
    let archive = archive
        .to_str()
        .ok_or_else(|| "archive path is not valid UTF-8".to_string())?;
    let destination = destination
        .to_str()
        .ok_or_else(|| "destination path is not valid UTF-8".to_string())?;
    run_required("/usr/bin/unzip", &["-q", "-o", archive, "-d", destination])?;
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
        run_required("/usr/bin/tar", &["-xzf", archive, "-C", destination])?;
        Ok(())
    } else {
        unzip_archive(archive, destination)
    }
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let path = path
        .to_str()
        .ok_or_else(|| "checksum path is not valid UTF-8".to_string())?;
    let result = run_required("/usr/bin/shasum", &["-a", "256", path])?;
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
                && (name.ends_with(".zip") || name.ends_with(".tar.gz") || name.ends_with(".tgz")))
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
    download_file(PLATFORM_TOOLS_URL, &archive)?;
    logs.push(format!("platform-tools sha256 {}", file_sha256(&archive)?));
    unzip_archive(&archive, &unzip_dir)?;

    let source = unzip_dir.join("platform-tools");
    let target = tools_dir.join("platform-tools");
    copy_dir_recursive(&source, &target)?;
    let adb = target.join("adb");
    fs::set_permissions(&adb, fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;
    let adb_path = adb.to_string_lossy().to_string();
    run_required(&adb_path, &["version"])?;
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
            SCRCPY_RELEASE_API,
        ],
    )?;
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
    run_required(&scrcpy_path, &["--version"])?;
    Ok(scrcpy_path)
}

fn translate_error(stdout: &str, stderr: &str) -> String {
    let text = format!("{stdout}\n{stderr}").to_lowercase();

    if text.contains("unauthorized") {
        "请解锁手机，并在手机弹窗中允许 USB 调试".to_string()
    } else if text.contains("offline") {
        "设备已离线，请重新插拔数据线或重新连接无线调试".to_string()
    } else if text.contains("more than one device") || text.contains("more than one emulator") {
        "当前有多台设备，请先选择要操作的手机".to_string()
    } else if text.contains("connection refused") {
        "无线调试端口不可用，请检查 IP、端口和手机无线调试是否开启".to_string()
    } else if text.contains("failed to authenticate") {
        "配对失败，请重新生成配对码后再试".to_string()
    } else if text.contains("device not found") {
        "设备不存在或已断开，请刷新设备列表".to_string()
    } else if text.contains("unknown command") && text.contains("pair") {
        "当前 adb 版本不支持无线配对，请升级 Android Platform Tools".to_string()
    } else {
        stderr
            .trim()
            .lines()
            .next()
            .or_else(|| stdout.trim().lines().next())
            .unwrap_or("命令执行失败")
            .to_string()
    }
}

fn first_line(text: Option<String>) -> Option<String> {
    text.and_then(|value| value.lines().next().map(ToOwned::to_owned))
}

fn second_line(text: Option<String>) -> Option<String> {
    text.and_then(|value| value.lines().nth(1).map(ToOwned::to_owned))
}

fn parse_devices(output: &str, config: &AppConfig) -> Vec<Device> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let raw = line.trim();
            if raw.is_empty() {
                return None;
            }

            let parts = raw.split_whitespace().collect::<Vec<_>>();
            let serial = parts.first()?.to_string();
            let state = parts.get(1).unwrap_or(&"unknown").to_string();
            let mut model = None;
            let mut product = None;

            for part in parts.iter().skip(2) {
                if let Some(value) = part.strip_prefix("model:") {
                    model = Some(value.replace('_', " "));
                }
                if let Some(value) = part.strip_prefix("product:") {
                    product = Some(value.to_string());
                }
            }

            let connection = if serial.contains(':') {
                "wireless"
            } else {
                "usb"
            }
            .to_string();

            Some(Device {
                alias: config.device_aliases.get(&serial).cloned(),
                serial,
                state,
                model,
                product,
                connection,
                raw: raw.to_string(),
            })
        })
        .collect()
}

fn with_adb<T>(
    state: &State<'_, AppState>,
    action: impl FnOnce(String, AppConfig) -> Result<T, String>,
) -> Result<T, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();
    let adb = resolve_tool("adb", &config).ok_or_else(|| "adb not found".to_string())?;
    action(adb, config)
}

fn build_scrcpy_args(serial: &str, options: &ScrcpyOptions) -> Vec<String> {
    let mut args = vec!["-s".to_string(), serial.to_string()];

    if let Some(max_size) = options.max_size {
        args.push(format!("--max-size={max_size}"));
    }
    if let Some(max_fps) = options.max_fps {
        args.push(format!("--max-fps={max_fps}"));
    }
    if let Some(video_bit_rate) = &options.video_bit_rate {
        if !video_bit_rate.trim().is_empty() {
            args.push(format!("--video-bit-rate={}", video_bit_rate.trim()));
        }
    }
    if let Some(video_codec) = &options.video_codec {
        if !video_codec.trim().is_empty() && video_codec != "default" {
            args.push(format!("--video-codec={}", video_codec.trim()));
        }
    }
    if options.no_audio.unwrap_or(false) {
        args.push("--no-audio".to_string());
    }
    if options.no_control.unwrap_or(false) {
        args.push("--no-control".to_string());
    }
    if options.stay_awake.unwrap_or(false) {
        args.push("--stay-awake".to_string());
    }
    if options.turn_screen_off.unwrap_or(false) {
        args.push("--turn-screen-off".to_string());
    }
    if options.show_touches.unwrap_or(false) {
        args.push("--show-touches".to_string());
    }
    if options.always_on_top.unwrap_or(false) {
        args.push("--always-on-top".to_string());
    }
    if options.fullscreen.unwrap_or(false) {
        args.push("--fullscreen".to_string());
    }

    args
}

fn push_session_log(
    logs: &Arc<Mutex<VecDeque<SessionLogLine>>>,
    level: impl Into<String>,
    message: impl Into<String>,
) {
    if let Ok(mut logs) = logs.lock() {
        logs.push_back(SessionLogLine {
            timestamp: now_secs(),
            level: level.into(),
            message: message.into(),
        });
        while logs.len() > LOG_LIMIT {
            logs.pop_front();
        }
    }
}

fn spawn_log_reader(
    logs: Arc<Mutex<VecDeque<SessionLogLine>>>,
    level: &'static str,
    reader: impl std::io::Read + Send + 'static,
) {
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            push_session_log(&logs, level, line);
        }
    });
}

fn refresh_session_status(entry: &mut SessionEntry) {
    if entry.info.status == "running" {
        match entry.child.try_wait() {
            Ok(Some(status)) => {
                entry.info.status = "stopped".to_string();
                entry.info.last_message = Some(format!("scrcpy exited with {status}"));
                push_session_log(
                    &entry.logs,
                    "info",
                    entry.info.last_message.clone().unwrap_or_default(),
                );
            }
            Ok(None) => {}
            Err(error) => {
                entry.info.status = "failed".to_string();
                entry.info.last_message = Some(error.to_string());
                push_session_log(&entry.logs, "error", error.to_string());
            }
        }
    }
}

fn has_running_session_for_serial<'a>(
    sessions: impl IntoIterator<Item = &'a SessionInfo>,
    serial: &str,
) -> bool {
    sessions
        .into_iter()
        .any(|session| session.serial == serial && session.status == "running")
}

#[tauri::command]
fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state
        .config
        .lock()
        .map(|config| config.clone())
        .map_err(|_| "config lock poisoned".to_string())
}

#[tauri::command]
fn set_tool_paths(
    state: State<'_, AppState>,
    adb_path: Option<String>,
    scrcpy_path: Option<String>,
) -> Result<AppConfig, String> {
    if let Some(path) = &adb_path {
        if !validate_executable(path) {
            return Err("adb 路径不存在或不可执行".to_string());
        }
    }
    if let Some(path) = &scrcpy_path {
        if !validate_executable(path) {
            return Err("scrcpy 路径不存在或不可执行".to_string());
        }
    }

    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    config.adb_path = adb_path;
    config.scrcpy_path = scrcpy_path;
    save_config(&config)?;

    Ok(config.clone())
}

#[tauri::command]
fn save_device_alias(
    state: State<'_, AppState>,
    serial: String,
    alias: String,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;

    if alias.trim().is_empty() {
        config.device_aliases.remove(&serial);
    } else {
        config
            .device_aliases
            .insert(serial, alias.trim().to_string());
    }
    save_config(&config)?;

    Ok(config.clone())
}

#[tauri::command]
fn save_default_scrcpy_options(
    state: State<'_, AppState>,
    options: ScrcpyOptions,
    preset_id: String,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;

    config.default_scrcpy_options = options;
    config.default_preset_id = preset_id;
    save_config(&config)?;

    Ok(config.clone())
}

#[tauri::command]
fn save_device_scrcpy_options(
    state: State<'_, AppState>,
    serial: String,
    options: ScrcpyOptions,
    preset_id: Option<String>,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;

    config.device_scrcpy_options.insert(
        serial,
        DeviceOptionEntry {
            preset_id,
            options,
            updated_at: now_secs(),
        },
    );
    save_config(&config)?;

    Ok(config.clone())
}

#[tauri::command]
fn clear_device_scrcpy_options(
    state: State<'_, AppState>,
    serial: String,
) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;

    config.device_scrcpy_options.remove(&serial);
    save_config(&config)?;

    Ok(config.clone())
}

#[tauri::command]
fn get_tool_status(state: State<'_, AppState>) -> Result<ToolStatus, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();

    let adb_path = resolve_tool("adb", &config);
    let scrcpy_path = resolve_tool("scrcpy", &config);
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

#[tauri::command]
fn install_tools(state: State<'_, AppState>) -> Result<ToolInstallResult, String> {
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

    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    config.adb_path = Some(adb_path.clone());
    config.scrcpy_path = Some(scrcpy_path.clone());
    save_config(&config)?;

    logs.push("工具安装完成".to_string());
    Ok(ToolInstallResult {
        adb_path,
        scrcpy_path,
        logs,
    })
}

#[tauri::command]
fn list_devices(state: State<'_, AppState>) -> Result<Vec<Device>, String> {
    with_adb(&state, |adb, config| {
        let result = run_command(&adb, &["devices", "-l"]);
        if !result.ok {
            return Err(result.message);
        }

        Ok(parse_devices(&result.stdout, &config))
    })
}

#[tauri::command]
fn adb_tcpip(
    state: State<'_, AppState>,
    serial: String,
    port: Option<u16>,
) -> Result<CommandResult, String> {
    with_adb(&state, |adb, _config| {
        let port = port.unwrap_or(DEFAULT_TCPIP_PORT).to_string();
        let result = run_command(&adb, &["-s", &serial, "tcpip", &port]);
        result.ok.then_some(result.clone()).ok_or(result.message)
    })
}

#[tauri::command]
fn adb_connect(state: State<'_, AppState>, endpoint: String) -> Result<CommandResult, String> {
    with_adb(&state, |adb, mut config| {
        let endpoint = endpoint.trim().to_string();
        if endpoint.is_empty() {
            return Err("请输入无线调试连接地址".to_string());
        }

        let result = run_command(&adb, &["connect", &endpoint]);
        if result.ok {
            config.recent_endpoints.retain(|item| item != &endpoint);
            config.recent_endpoints.insert(0, endpoint);
            config.recent_endpoints.truncate(20);
            save_config(&config)?;
            if let Ok(mut state_config) = state.config.lock() {
                *state_config = config;
            }
            Ok(result)
        } else {
            Err(result.message)
        }
    })
}

#[tauri::command]
fn adb_disconnect(
    state: State<'_, AppState>,
    endpoint: Option<String>,
) -> Result<CommandResult, String> {
    with_adb(&state, |adb, _config| {
        let result = if let Some(endpoint) = endpoint.as_deref() {
            run_command(&adb, &["disconnect", endpoint])
        } else {
            run_command(&adb, &["disconnect"])
        };
        result.ok.then_some(result.clone()).ok_or(result.message)
    })
}

#[tauri::command]
fn adb_pair(state: State<'_, AppState>, request: PairRequest) -> Result<CommandResult, String> {
    with_adb(&state, |adb, mut config| {
        let endpoint = format!("{}:{}", request.host.trim(), request.pair_port);
        let mut child = Command::new(&adb)
            .args(["pair", &endpoint])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(format!("{}\n", request.pairing_code.trim()).as_bytes())
                .map_err(|error| error.to_string())?;
        }

        let output = child
            .wait_with_output()
            .map_err(|error| error.to_string())?;
        let pair_result = CommandResult {
            ok: output.status.success(),
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            message: String::new(),
        };

        if !pair_result.ok {
            return Err(translate_error(&pair_result.stdout, &pair_result.stderr));
        }

        if let Some(connect_port) = request.connect_port {
            let connect_host = request
                .connect_host
                .as_deref()
                .map(str::trim)
                .filter(|host| !host.is_empty())
                .unwrap_or_else(|| request.host.trim());
            let connect_endpoint = format!("{connect_host}:{connect_port}");
            let connect_result = run_command(&adb, &["connect", &connect_endpoint]);
            if !connect_result.ok {
                return Err(connect_result.message);
            }

            config
                .recent_endpoints
                .retain(|item| item != &connect_endpoint);
            config.recent_endpoints.insert(0, connect_endpoint);
            config.recent_endpoints.truncate(20);
            save_config(&config)?;
            if let Ok(mut state_config) = state.config.lock() {
                *state_config = config;
            }

            Ok(CommandResult {
                ok: true,
                code: connect_result.code,
                stdout: format!("{}\n{}", pair_result.stdout, connect_result.stdout),
                stderr: format!("{}\n{}", pair_result.stderr, connect_result.stderr),
                message: "配对并连接成功".to_string(),
            })
        } else {
            Ok(CommandResult {
                message: "配对成功，请继续连接无线调试端口".to_string(),
                ..pair_result
            })
        }
    })
}

#[tauri::command]
fn start_scrcpy(
    state: State<'_, AppState>,
    serial: String,
    options: ScrcpyOptions,
) -> Result<SessionInfo, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();
    let scrcpy = resolve_tool("scrcpy", &config).ok_or_else(|| "scrcpy not found".to_string())?;
    let alias = config.device_aliases.get(&serial).cloned();
    let connection = if serial.contains(':') {
        "wireless"
    } else {
        "usb"
    }
    .to_string();
    let args = build_scrcpy_args(&serial, &options);
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "sessions lock poisoned".to_string())?;
    for entry in sessions.values_mut() {
        refresh_session_status(entry);
    }
    if has_running_session_for_serial(sessions.values().map(|entry| &entry.info), &serial) {
        return Err("该设备已有运行中的投屏会话，请先停止后再重连".to_string());
    }

    let mut child = Command::new(&scrcpy)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;

    let pid = child.id();
    let session_id = format!("{}-{pid}", serial.replace([':', '.'], "_"));
    let logs = Arc::new(Mutex::new(VecDeque::new()));

    push_session_log(&logs, "info", format!("$ {} {}", scrcpy, args.join(" ")));

    if let Some(stdout) = child.stdout.take() {
        spawn_log_reader(logs.clone(), "info", stdout);
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_reader(logs.clone(), "warn", stderr);
    }

    let info = SessionInfo {
        session_id: session_id.clone(),
        serial,
        alias,
        pid,
        status: "running".to_string(),
        started_at: now_secs(),
        connection,
        args,
        last_message: None,
    };

    sessions.insert(
        session_id,
        SessionEntry {
            info: info.clone(),
            child,
            logs,
        },
    );

    Ok(info)
}

#[tauri::command]
fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "sessions lock poisoned".to_string())?;

    let mut values = Vec::new();
    for entry in sessions.values_mut() {
        refresh_session_status(entry);
        values.push(entry.info.clone());
    }

    values.sort_by_key(|session| session.started_at);
    values.reverse();
    Ok(values)
}

#[tauri::command]
fn get_session_logs(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<SessionLogLine>, String> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|_| "sessions lock poisoned".to_string())?;
    let entry = sessions
        .get(&session_id)
        .ok_or_else(|| "session not found".to_string())?;
    let logs = entry
        .logs
        .lock()
        .map_err(|_| "session logs lock poisoned".to_string())?;

    Ok(logs.iter().cloned().collect())
}

#[tauri::command]
fn stop_scrcpy(state: State<'_, AppState>, session_id: String) -> Result<SessionInfo, String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "sessions lock poisoned".to_string())?;
    let entry = sessions
        .get_mut(&session_id)
        .ok_or_else(|| "session not found".to_string())?;

    refresh_session_status(entry);
    if entry.info.status == "running" {
        entry.child.kill().map_err(|error| error.to_string())?;
        let _ = entry.child.wait();
        entry.info.status = "stopped".to_string();
        entry.info.last_message = Some("用户已停止投屏".to_string());
        push_session_log(&entry.logs, "info", "用户已停止投屏");
    }

    Ok(entry.info.clone())
}

#[tauri::command]
fn stop_all_sessions(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "sessions lock poisoned".to_string())?;
    let mut stopped = Vec::new();

    for entry in sessions.values_mut() {
        refresh_session_status(entry);
        if entry.info.status == "running" {
            let _ = entry.child.kill();
            let _ = entry.child.wait();
            entry.info.status = "stopped".to_string();
            entry.info.last_message = Some("应用已停止投屏".to_string());
            push_session_log(&entry.logs, "info", "应用已停止投屏");
        }
        stopped.push(entry.info.clone());
    }

    Ok(stopped)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            config: Mutex::new(load_config()),
            sessions: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            set_tool_paths,
            save_device_alias,
            save_default_scrcpy_options,
            save_device_scrcpy_options,
            clear_device_scrcpy_options,
            get_tool_status,
            install_tools,
            list_devices,
            adb_tcpip,
            adb_connect,
            adb_disconnect,
            adb_pair,
            start_scrcpy,
            stop_scrcpy,
            stop_all_sessions,
            list_sessions,
            get_session_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running DroidDock");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn validate_executable_rejects_plain_files_without_execute_bits() {
        let path = std::env::temp_dir().join(format!("droiddock-test-{}", now_secs()));
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

    #[test]
    fn running_session_detector_blocks_duplicate_serials_only_when_active() {
        let running = SessionInfo {
            session_id: "one".to_string(),
            serial: "SERIAL-1".to_string(),
            alias: None,
            pid: 1,
            status: "running".to_string(),
            started_at: 1,
            connection: "usb".to_string(),
            args: vec![],
            last_message: None,
        };
        let stopped = SessionInfo {
            status: "stopped".to_string(),
            ..running.clone()
        };

        assert!(has_running_session_for_serial([&running], "SERIAL-1"));
        assert!(!has_running_session_for_serial([&stopped], "SERIAL-1"));
        assert!(!has_running_session_for_serial([&running], "SERIAL-2"));
    }
}
