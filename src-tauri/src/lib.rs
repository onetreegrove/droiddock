use serde::Serialize;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize)]
struct ToolStatus {
    adb_path: Option<String>,
    scrcpy_path: Option<String>,
    adb_version: Option<String>,
    scrcpy_version: Option<String>,
}

#[derive(Debug, Serialize)]
struct Device {
    serial: String,
    state: String,
    model: Option<String>,
    product: Option<String>,
    connection: String,
    raw: String,
}

#[derive(Debug, serde::Deserialize)]
struct ScrcpyOptions {
    max_size: Option<u32>,
    max_fps: Option<u32>,
    no_audio: Option<bool>,
    stay_awake: Option<bool>,
}

fn resolve_tool(name: &str) -> Option<String> {
    let home = std::env::var("HOME").ok();
    let mut candidates = Vec::new();

    if name == "adb" {
        if let Some(home) = &home {
            candidates.push(format!("{home}/Library/Android/sdk/platform-tools/adb"));
        }
        candidates.push("/opt/homebrew/bin/adb".to_string());
        candidates.push("/usr/local/bin/adb".to_string());
    } else {
        if let Some(home) = &home {
            candidates.push(format!("{home}/.local/bin/scrcpy"));
        }
        candidates.push("/opt/homebrew/bin/scrcpy".to_string());
        candidates.push("/usr/local/bin/scrcpy".to_string());
    }

    candidates
        .into_iter()
        .find(|candidate| std::path::Path::new(candidate).exists())
}

fn command_output(path: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(path).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout).ok()
}

fn parse_devices(output: &str) -> Vec<Device> {
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

#[tauri::command]
fn get_tool_status() -> ToolStatus {
    let adb_path = resolve_tool("adb");
    let scrcpy_path = resolve_tool("scrcpy");
    let adb_version = adb_path
        .as_deref()
        .and_then(|path| command_output(path, &["version"]))
        .and_then(|text| text.lines().nth(1).map(ToOwned::to_owned));
    let scrcpy_version = scrcpy_path
        .as_deref()
        .and_then(|path| command_output(path, &["--version"]))
        .and_then(|text| text.lines().next().map(ToOwned::to_owned));

    ToolStatus {
        adb_path,
        scrcpy_path,
        adb_version,
        scrcpy_version,
    }
}

#[tauri::command]
fn list_devices() -> Result<Vec<Device>, String> {
    let adb = resolve_tool("adb").ok_or_else(|| "adb not found".to_string())?;
    let output = command_output(&adb, &["devices", "-l"])
        .ok_or_else(|| "failed to run adb devices -l".to_string())?;

    Ok(parse_devices(&output))
}

#[tauri::command]
fn start_scrcpy(serial: String, options: ScrcpyOptions) -> Result<(), String> {
    let scrcpy = resolve_tool("scrcpy").ok_or_else(|| "scrcpy not found".to_string())?;
    let mut args = vec!["-s".to_string(), serial];

    if let Some(max_size) = options.max_size {
        args.push(format!("--max-size={max_size}"));
    }
    if let Some(max_fps) = options.max_fps {
        args.push(format!("--max-fps={max_fps}"));
    }
    if options.no_audio.unwrap_or(false) {
        args.push("--no-audio".to_string());
    }
    if options.stay_awake.unwrap_or(false) {
        args.push("--stay-awake".to_string());
    }

    Command::new(scrcpy)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_tool_status,
            list_devices,
            start_scrcpy
        ])
        .run(tauri::generate_context!())
        .expect("error while running DroidDock");
}

