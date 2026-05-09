use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    now_secs,
    scrcpy::{build_scrcpy_args, ScrcpyOptions},
};
use tauri::{AppHandle, Emitter};

const LOG_LIMIT: usize = 400;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionInfo {
    pub(crate) session_id: String,
    pub(crate) serial: String,
    pub(crate) alias: Option<String>,
    pub(crate) pid: u32,
    pub(crate) status: String,
    pub(crate) started_at: u64,
    pub(crate) connection: String,
    pub(crate) args: Vec<String>,
    pub(crate) last_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionLogLine {
    pub(crate) timestamp: u64,
    pub(crate) level: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionLogEvent {
    pub(crate) session_id: String,
    pub(crate) line: SessionLogLine,
}

#[derive(Debug)]
struct SessionEntry {
    info: SessionInfo,
    child: Child,
    logs: Arc<Mutex<VecDeque<SessionLogLine>>>,
}

#[derive(Debug, Default)]
pub(crate) struct SessionManager {
    sessions: Mutex<HashMap<String, SessionEntry>>,
}

fn append_session_log(logs: &Arc<Mutex<VecDeque<SessionLogLine>>>, line: SessionLogLine) {
    if let Ok(mut logs) = logs.lock() {
        logs.push_back(line);
        while logs.len() > LOG_LIMIT {
            logs.pop_front();
        }
    }
}

fn push_session_log(
    logs: &Arc<Mutex<VecDeque<SessionLogLine>>>,
    level: impl Into<String>,
    message: impl Into<String>,
) {
    append_session_log(
        logs,
        SessionLogLine {
            timestamp: now_secs(),
            level: level.into(),
            message: message.into(),
        },
    );
}

fn spawn_log_reader(
    app: AppHandle,
    session_id: String,
    logs: Arc<Mutex<VecDeque<SessionLogLine>>>,
    level: &'static str,
    reader: impl std::io::Read + Send + 'static,
) {
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for message in reader.lines().map_while(Result::ok) {
            let line = SessionLogLine {
                timestamp: now_secs(),
                level: level.to_string(),
                message,
            };
            append_session_log(&logs, line.clone());
            let _ = app.emit(
                "session-log",
                SessionLogEvent {
                    session_id: session_id.clone(),
                    line,
                },
            );
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

impl SessionManager {
    pub(crate) fn start(
        &self,
        app: &AppHandle,
        scrcpy: &str,
        serial: String,
        alias: Option<String>,
        options: ScrcpyOptions,
    ) -> Result<SessionInfo, String> {
        let connection = if serial.contains(':') {
            "wireless"
        } else {
            "usb"
        }
        .to_string();
        let args = build_scrcpy_args(&serial, &options);
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| "sessions lock poisoned".to_string())?;
        for entry in sessions.values_mut() {
            refresh_session_status(entry);
        }
        if has_running_session_for_serial(sessions.values().map(|entry| &entry.info), &serial) {
            return Err("该设备已有运行中的投屏会话，请先停止后再重连".to_string());
        }

        let mut child = Command::new(scrcpy)
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
            spawn_log_reader(
                app.clone(),
                session_id.clone(),
                logs.clone(),
                "info",
                stdout,
            );
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_log_reader(
                app.clone(),
                session_id.clone(),
                logs.clone(),
                "warn",
                stderr,
            );
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

    pub(crate) fn list(&self) -> Result<Vec<SessionInfo>, String> {
        let mut sessions = self
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

    pub(crate) fn logs(&self, session_id: String) -> Result<Vec<SessionLogLine>, String> {
        let sessions = self
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

    pub(crate) fn stop(&self, session_id: String) -> Result<SessionInfo, String> {
        let mut sessions = self
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

    pub(crate) fn stop_all(&self) -> Result<Vec<SessionInfo>, String> {
        let mut sessions = self
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
