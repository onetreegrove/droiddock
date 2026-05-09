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
    get_tool_status_for_config, install_tools_into_config, resolve_tool, validate_executable,
    ToolInstallResult, ToolStatus,
};
use wireless::PairRequest;

#[derive(Debug, Default)]
struct AppState {
    config: Mutex<AppConfig>,
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
    let mut config = state
        .config
        .lock()
        .map_err(|_| "config lock poisoned".to_string())?;
    let result = install_tools_into_config(&mut config)?;
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
            sessions: SessionManager::default(),
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
