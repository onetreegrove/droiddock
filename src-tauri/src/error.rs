use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum AppErrorCode {
    CommandFailed,
    ToolNotFound,
    InvalidInput,
    DeviceUnauthorized,
    DeviceOffline,
    MultipleDevices,
    WirelessPortUnavailable,
    PairFailed,
    UnsupportedAdbPair,
    ConfigReadFailed,
    ConfigWriteFailed,
    SessionNotFound,
    LockPoisoned,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: AppErrorCode,
    pub user_message: String,
    pub technical_detail: Option<String>,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: AppErrorCode, user_message: impl Into<String>) -> Self {
        Self {
            code,
            user_message: user_message.into(),
            technical_detail: None,
            retryable: false,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.technical_detail = Some(detail.into());
        self
    }

    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::new(AppErrorCode::CommandFailed, error.to_string()).with_detail(error.to_string())
    }
}

pub fn translate_command_error(stdout: &str, stderr: &str) -> AppError {
    let detail = format!("{stdout}\n{stderr}").trim().to_string();
    let text = detail.to_lowercase();

    if text.contains("unauthorized") {
        AppError::new(
            AppErrorCode::DeviceUnauthorized,
            "请解锁手机，并在手机弹窗中允许 USB 调试",
        )
        .with_detail(detail)
        .retryable()
    } else if text.contains("offline") {
        AppError::new(
            AppErrorCode::DeviceOffline,
            "设备已离线，请重新插拔数据线或重新连接无线调试",
        )
        .with_detail(detail)
        .retryable()
    } else if text.contains("more than one device") || text.contains("more than one emulator") {
        AppError::new(
            AppErrorCode::MultipleDevices,
            "当前有多台设备，请先选择要操作的手机",
        )
        .with_detail(detail)
    } else if text.contains("connection refused") {
        AppError::new(
            AppErrorCode::WirelessPortUnavailable,
            "无线调试端口不可用，请检查 IP、端口和手机无线调试是否开启",
        )
        .with_detail(detail)
        .retryable()
    } else if text.contains("failed to connect")
        || text.contains("unable to connect")
        || text.contains("cannot connect")
    {
        AppError::new(
            AppErrorCode::WirelessPortUnavailable,
            "无线调试连接失败，请检查 IP、端口、手机无线调试状态和当前网络",
        )
        .with_detail(detail)
        .retryable()
    } else if text.contains("failed to authenticate") {
        AppError::new(AppErrorCode::PairFailed, "配对失败，请重新生成配对码后再试")
            .with_detail(detail)
            .retryable()
    } else if text.contains("unknown command") && text.contains("pair") {
        AppError::new(
            AppErrorCode::UnsupportedAdbPair,
            "当前 adb 版本不支持无线配对，请升级 Android Platform Tools",
        )
        .with_detail(detail)
    } else {
        let message = stderr
            .trim()
            .lines()
            .next()
            .or_else(|| stdout.trim().lines().next())
            .unwrap_or("命令执行失败")
            .to_string();
        AppError::new(AppErrorCode::CommandFailed, message).with_detail(detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_unauthorized_adb_error() {
        let error = translate_command_error("", "device unauthorized");
        assert_eq!(error.code, AppErrorCode::DeviceUnauthorized);
        assert_eq!(
            error.user_message,
            "请解锁手机，并在手机弹窗中允许 USB 调试"
        );
        assert!(error.retryable);
    }

    #[test]
    fn preserves_technical_detail_for_unknown_error() {
        let error = translate_command_error("", "some low level failure");
        assert_eq!(error.code, AppErrorCode::CommandFailed);
        assert_eq!(error.user_message, "some low level failure");
        assert_eq!(
            error.technical_detail.as_deref(),
            Some("some low level failure")
        );
    }

    #[test]
    fn translates_adb_connect_failure_with_success_exit_code_output() {
        let error = translate_command_error(
            "failed to connect to '192.168.1.10:39407': Operation timed out",
            "",
        );

        assert_eq!(error.code, AppErrorCode::WirelessPortUnavailable);
        assert_eq!(
            error.user_message,
            "无线调试连接失败，请检查 IP、端口、手机无线调试状态和当前网络"
        );
        assert!(error.retryable);
    }
}
