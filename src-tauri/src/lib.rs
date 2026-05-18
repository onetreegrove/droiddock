mod command;
mod config;
mod devices;
mod error;
mod scrcpy;
mod sessions;
mod tool_manifest;
mod tools;
mod wireless;

use command::CommandResult;
use config::{load_config, save_config_atomic, AppConfig, DeviceOptionEntry, WirelessSource};
use scrcpy::{build_scrcpy_args, ScrcpyOptions};
use sessions::{SessionInfo, SessionLogLine, SessionManager};
use std::{
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, State};
use tools::{
    diagnose_configured_tool_path, get_tool_status_for_config, install_tools_into_config,
    resolve_tool, ToolHealth, ToolInstallResult, ToolKind, ToolStatus,
};
use wireless::PairRequest;

#[derive(Debug, Default)]
struct AppState {
    config: Mutex<AppConfig>,
    install: Mutex<()>,
    sessions: SessionManager,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
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

fn save_state_config(state: &State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    save_config_atomic(&config)?;
    let mut state_config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    *state_config = config;
    Ok(())
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
        let diagnostic = diagnose_configured_tool_path(ToolKind::Adb, path);
        if diagnostic.health != ToolHealth::Ready {
            return Err(diagnostic.message);
        }
    }
    if let Some(path) = &scrcpy_path {
        let diagnostic = diagnose_configured_tool_path(ToolKind::Scrcpy, path);
        if diagnostic.health != ToolHealth::Ready {
            return Err(diagnostic.message);
        }
    }

    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    config.adb_path = adb_path;
    config.scrcpy_path = scrcpy_path;
    save_config_atomic(&config)?;

    Ok(config.clone())
}

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

    let kind = match tool.as_str() {
        "adb" => ToolKind::Adb,
        "scrcpy" => ToolKind::Scrcpy,
        _ => return Err("未知工具类型".to_string()),
    };

    let diagnostic = diagnose_configured_tool_path(kind, &path);

    if diagnostic.health != ToolHealth::Ready {
        return Err(diagnostic.message);
    }

    match kind {
        ToolKind::Adb => config.adb_path = Some(path),
        ToolKind::Scrcpy => config.scrcpy_path = Some(path),
    }

    save_config_atomic(&config)?;
    Ok(config.clone())
}

#[tauri::command]
fn clear_tool_path(state: State<'_, AppState>, tool: String) -> Result<AppConfig, String> {
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
    save_config_atomic(&config)?;

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
    save_config_atomic(&config)?;

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
    save_config_atomic(&config)?;

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
    save_config_atomic(&config)?;

    Ok(config.clone())
}

#[tauri::command]
fn forget_device(state: State<'_, AppState>, serial: String) -> Result<AppConfig, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;

    devices::forget_device_record(&mut config, &serial);
    save_config_atomic(&config)?;

    Ok(config.clone())
}

#[tauri::command]
fn get_tool_status(state: State<'_, AppState>) -> Result<ToolStatus, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();

    get_tool_status_for_config(&config)
}

#[tauri::command]
fn install_tools(state: State<'_, AppState>) -> Result<ToolInstallResult, String> {
    let _install_guard = state
        .install
        .lock()
        .map_err(|_| "工具安装状态异常，请重启 DroidDock 后重试".to_string())?;
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

#[tauri::command]
fn list_devices(state: State<'_, AppState>) -> Result<Vec<devices::ManagedDevice>, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();
    let adb = resolve_tool("adb", &config).ok_or_else(|| "adb not found".to_string())?;
    let result = devices::list_devices_with_adb(&adb, &mut config, now_secs())?;
    if result.changed {
        save_state_config(&state, config)?;
    }
    Ok(result.devices)
}

#[tauri::command]
fn adb_tcpip(
    state: State<'_, AppState>,
    serial: String,
    port: Option<u16>,
) -> Result<CommandResult, String> {
    with_adb(&state, |adb, _config| {
        wireless::adb_tcpip_with_adb(&adb, serial, port)
    })
}

#[tauri::command]
fn adb_connect(
    state: State<'_, AppState>,
    endpoint: String,
    source: Option<WirelessSource>,
    previous_endpoint: Option<String>,
) -> Result<CommandResult, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();
    let adb = resolve_tool("adb", &config).ok_or_else(|| "adb not found".to_string())?;
    let result = wireless::adb_connect_with_adb(
        &adb,
        &mut config,
        endpoint,
        source.unwrap_or(WirelessSource::Manual),
        previous_endpoint,
        now_secs(),
    )?;
    save_state_config(&state, config)?;
    Ok(result)
}

#[tauri::command]
fn adb_disconnect(
    state: State<'_, AppState>,
    endpoint: Option<String>,
) -> Result<CommandResult, String> {
    with_adb(&state, |adb, _config| {
        wireless::adb_disconnect_with_adb(&adb, endpoint)
    })
}

#[tauri::command]
fn adb_pair(state: State<'_, AppState>, request: PairRequest) -> Result<CommandResult, String> {
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?
        .clone();
    let adb = resolve_tool("adb", &config).ok_or_else(|| "adb not found".to_string())?;
    let result = wireless::adb_pair_with_adb(&adb, &mut config, request, now_secs())?;
    save_state_config(&state, config)?;
    Ok(result)
}

#[tauri::command]
fn preview_scrcpy_args(serial: String, options: ScrcpyOptions) -> Vec<String> {
    build_scrcpy_args(&serial, &options)
}

#[tauri::command]
fn start_scrcpy(
    app: AppHandle,
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
    state.sessions.start(&app, &scrcpy, serial, alias, options)
}

#[tauri::command]
fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    state.sessions.list()
}

#[tauri::command]
fn get_session_logs(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<SessionLogLine>, String> {
    state.sessions.logs(session_id)
}

#[tauri::command]
fn stop_scrcpy(state: State<'_, AppState>, session_id: String) -> Result<SessionInfo, String> {
    state.sessions.stop(session_id)
}

#[tauri::command]
fn stop_all_sessions(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    state.sessions.stop_all()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            config: Mutex::new(load_config()),
            install: Mutex::new(()),
            sessions: SessionManager::default(),
        })
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            set_tool_paths,
            set_tool_path,
            clear_tool_path,
            save_device_alias,
            save_default_scrcpy_options,
            save_device_scrcpy_options,
            clear_device_scrcpy_options,
            forget_device,
            get_tool_status,
            install_tools,
            list_devices,
            adb_tcpip,
            adb_connect,
            adb_disconnect,
            adb_pair,
            preview_scrcpy_args,
            start_scrcpy,
            stop_scrcpy,
            stop_all_sessions,
            list_sessions,
            get_session_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running DroidDock");
}
