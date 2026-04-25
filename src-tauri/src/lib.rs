use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;

const LOG_LIMIT: usize = 400;
const DEFAULT_TCPIP_PORT: u16 = 5555;

#[derive(Debug, Default)]
struct AppState {
    config: Mutex<AppConfig>,
    sessions: Mutex<HashMap<String, SessionEntry>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AppConfig {
    adb_path: Option<String>,
    scrcpy_path: Option<String>,
    device_aliases: HashMap<String, String>,
    recent_endpoints: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ToolStatus {
    adb_path: Option<String>,
    scrcpy_path: Option<String>,
    adb_version: Option<String>,
    scrcpy_version: Option<String>,
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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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
    path.exists() && path.is_file()
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
    let scrcpy_version = scrcpy_path
        .as_deref()
        .map(|path| run_command(path, &["--version"]))
        .and_then(|result| result.ok.then_some(result.stdout))
        .and_then(|text| first_line(Some(text)));

    Ok(ToolStatus {
        adb_ok: adb_path.is_some(),
        scrcpy_ok: scrcpy_path.is_some(),
        adb_path,
        scrcpy_path,
        adb_version,
        scrcpy_version,
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
            let connect_endpoint = format!("{}:{connect_port}", request.host.trim());
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

    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "sessions lock poisoned".to_string())?;
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
        .manage(AppState {
            config: Mutex::new(load_config()),
            sessions: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            set_tool_paths,
            save_device_alias,
            get_tool_status,
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
