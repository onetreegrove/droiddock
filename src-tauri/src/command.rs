use serde::Serialize;
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::error::{translate_command_error, AppError, AppErrorCode, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct CommandResult {
    pub ok: bool,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
}

pub fn run_command(path: &str, args: &[&str]) -> CommandResult {
    run_command_inner(path, args, None, None)
}

pub fn run_command_with_timeout(path: &str, args: &[&str], timeout: Duration) -> CommandResult {
    run_command_inner(path, args, None, Some(timeout))
}

pub fn run_command_with_input_timeout(
    path: &str,
    args: &[&str],
    input: &str,
    timeout: Duration,
) -> CommandResult {
    run_command_inner(path, args, Some(input), Some(timeout))
}

fn run_command_inner(
    path: &str,
    args: &[&str],
    input: Option<&str>,
    timeout: Option<Duration>,
) -> CommandResult {
    let mut child = match Command::new(path)
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return CommandResult {
                ok: false,
                code: None,
                stdout: String::new(),
                stderr: error.to_string(),
                message: error.to_string(),
            }
        }
    };

    if let Some(input) = input {
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(error) = stdin.write_all(input.as_bytes()) {
                let _ = child.kill();
                let _ = child.wait();
                return CommandResult {
                    ok: false,
                    code: None,
                    stdout: String::new(),
                    stderr: error.to_string(),
                    message: error.to_string(),
                };
            }
        }
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_reader = stdout.map(|mut stdout| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stdout.read_to_end(&mut bytes);
            bytes
        })
    });
    let stderr_reader = stderr.map(|mut stderr| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stderr.read_to_end(&mut bytes);
            bytes
        })
    });

    let started_at = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {
                if let Some(timeout) = timeout {
                    if started_at.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        break Err("命令执行超时".to_string());
                    }
                }
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => break Err(error.to_string()),
        }
    };

    let stdout = stdout_reader
        .and_then(|reader| reader.join().ok())
        .map(|bytes| String::from_utf8_lossy(bytes.as_slice()).to_string())
        .unwrap_or_default();
    let stderr = stderr_reader
        .and_then(|reader| reader.join().ok())
        .map(|bytes| String::from_utf8_lossy(bytes.as_slice()).to_string())
        .unwrap_or_default();

    match status {
        Ok(status) => {
            let ok = status.success();
            let message = if ok {
                stdout.trim().to_string()
            } else {
                translate_command_error(&stdout, &stderr).user_message
            };

            CommandResult {
                ok,
                code: status.code(),
                stdout,
                stderr,
                message,
            }
        }
        Err(message) => CommandResult {
            ok: false,
            code: None,
            stdout,
            stderr,
            message,
        },
    }
}

pub fn run_required(path: &str, args: &[&str]) -> AppResult<CommandResult> {
    let result = run_command_inner(path, args, None, None);
    required_result(result)
}

pub fn run_required_with_timeout(
    path: &str,
    args: &[&str],
    timeout: Duration,
) -> AppResult<CommandResult> {
    let result = run_command_inner(path, args, None, Some(timeout));
    required_result(result)
}

fn required_result(result: CommandResult) -> AppResult<CommandResult> {
    if result.ok {
        Ok(result)
    } else {
        Err(
            AppError::new(AppErrorCode::CommandFailed, result.message.clone())
                .with_detail(format!("{}\n{}", result.stdout, result.stderr)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_timeout_returns_failure() {
        let result =
            run_command_with_timeout("/bin/sh", &["-c", "sleep 1"], Duration::from_millis(20));
        assert!(!result.ok);
        assert_eq!(result.message, "命令执行超时");
    }

    #[test]
    fn required_command_timeout_returns_app_error() {
        let error =
            run_required_with_timeout("/bin/sh", &["-c", "sleep 1"], Duration::from_millis(20))
                .unwrap_err();

        assert_eq!(error.user_message, "命令执行超时");
    }

    #[test]
    fn command_input_is_written_to_stdin() {
        let result = run_command_with_input_timeout(
            "/bin/sh",
            &["-c", "read value; printf \"%s\" \"$value\""],
            "pair-code\n",
            Duration::from_secs(1),
        );
        assert!(result.ok);
        assert_eq!(result.stdout, "pair-code");
    }
}
