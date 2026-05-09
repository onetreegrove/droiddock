use serde::Serialize;

use std::time::Duration;

use crate::{command::run_command_with_timeout, config::AppConfig};

#[derive(Debug, Serialize)]
pub(crate) struct Device {
    pub(crate) serial: String,
    pub(crate) state: String,
    pub(crate) model: Option<String>,
    pub(crate) product: Option<String>,
    pub(crate) connection: String,
    pub(crate) alias: Option<String>,
    pub(crate) raw: String,
}

pub(crate) fn parse_devices(output: &str, config: &AppConfig) -> Vec<Device> {
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

pub(crate) fn list_devices_with_adb(adb: &str, config: &AppConfig) -> Result<Vec<Device>, String> {
    let result = run_command_with_timeout(adb, &["devices", "-l"], Duration::from_secs(5));
    if !result.ok {
        return Err(result.message);
    }

    Ok(parse_devices(&result.stdout, config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn parses_usb_and_wireless_devices() {
        let config = AppConfig {
            device_aliases: HashMap::from([("R9YT301WXXX".to_string(), "测试手机".to_string())]),
            ..AppConfig::default()
        };
        let output = "List of devices attached\nR9YT301WXXX device product:test model:Pixel_8 transport_id:1\n192.168.1.2:5555 device product:test model:Mi_14 transport_id:2\n";
        let devices = parse_devices(output, &config);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].alias.as_deref(), Some("测试手机"));
        assert_eq!(devices[0].connection, "usb");
        assert_eq!(devices[1].connection, "wireless");
    }
}
