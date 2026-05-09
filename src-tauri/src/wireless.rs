use serde::Deserialize;
use std::time::Duration;

use crate::{
    command::{run_command_with_input_timeout, run_command_with_timeout, CommandResult},
    config::AppConfig,
};

const DEFAULT_TCPIP_PORT: u16 = 5555;

#[derive(Debug, Deserialize)]
pub(crate) struct PairRequest {
    pub(crate) host: String,
    pub(crate) pair_port: u16,
    pub(crate) pairing_code: String,
    pub(crate) connect_host: Option<String>,
    pub(crate) connect_port: Option<u16>,
}

pub(crate) fn remember_endpoint(config: &mut AppConfig, endpoint: String) {
    config.recent_endpoints.retain(|item| item != &endpoint);
    config.recent_endpoints.insert(0, endpoint);
    config.recent_endpoints.truncate(20);
}

pub(crate) fn adb_tcpip_with_adb(
    adb: &str,
    serial: String,
    port: Option<u16>,
) -> Result<CommandResult, String> {
    let port = port.unwrap_or(DEFAULT_TCPIP_PORT).to_string();
    let result = run_command_with_timeout(
        adb,
        &["-s", &serial, "tcpip", &port],
        Duration::from_secs(15),
    );
    result.ok.then_some(result.clone()).ok_or(result.message)
}

pub(crate) fn adb_connect_with_adb(
    adb: &str,
    config: &mut AppConfig,
    endpoint: String,
) -> Result<CommandResult, String> {
    let endpoint = endpoint.trim().to_string();
    if endpoint.is_empty() {
        return Err("请输入无线调试连接地址".to_string());
    }

    let result = run_command_with_timeout(adb, &["connect", &endpoint], Duration::from_secs(15));
    if result.ok {
        remember_endpoint(config, endpoint);
        Ok(result)
    } else {
        Err(result.message)
    }
}

pub(crate) fn adb_disconnect_with_adb(
    adb: &str,
    endpoint: Option<String>,
) -> Result<CommandResult, String> {
    let result = if let Some(endpoint) = endpoint.as_deref() {
        run_command_with_timeout(adb, &["disconnect", endpoint], Duration::from_secs(15))
    } else {
        run_command_with_timeout(adb, &["disconnect"], Duration::from_secs(15))
    };
    result.ok.then_some(result.clone()).ok_or(result.message)
}

pub(crate) fn adb_pair_with_adb(
    adb: &str,
    config: &mut AppConfig,
    request: PairRequest,
) -> Result<CommandResult, String> {
    let endpoint = format!("{}:{}", request.host.trim(), request.pair_port);
    let pair_result = run_command_with_input_timeout(
        adb,
        &["pair", &endpoint],
        &format!("{}\n", request.pairing_code.trim()),
        Duration::from_secs(30),
    );

    if !pair_result.ok {
        return Err(pair_result.message);
    }

    if let Some(connect_port) = request.connect_port {
        let connect_host = request
            .connect_host
            .as_deref()
            .map(str::trim)
            .filter(|host| !host.is_empty())
            .unwrap_or_else(|| request.host.trim());
        let connect_endpoint = format!("{connect_host}:{connect_port}");
        let connect_result = run_command_with_timeout(
            adb,
            &["connect", &connect_endpoint],
            Duration::from_secs(15),
        );
        if !connect_result.ok {
            return Err(connect_result.message);
        }

        remember_endpoint(config, connect_endpoint);

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_endpoint_deduplicates_and_caps_recent_list() {
        let mut config = AppConfig::default();
        for index in 0..25 {
            remember_endpoint(&mut config, format!("192.168.1.{index}:5555"));
        }
        remember_endpoint(&mut config, "192.168.1.10:5555".to_string());
        assert_eq!(config.recent_endpoints[0], "192.168.1.10:5555");
        assert_eq!(config.recent_endpoints.len(), 20);
    }
}
