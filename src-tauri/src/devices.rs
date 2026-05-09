use serde::Serialize;

use std::time::Duration;

use crate::{
    command::run_command_with_timeout,
    config::{AppConfig, DeviceConnection, DeviceRecord, WirelessSource},
};

const MAX_DEVICE_RECORDS: usize = 100;

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ManagedDevice {
    pub(crate) serial: String,
    pub(crate) state: String,
    pub(crate) presence: String,
    pub(crate) model: Option<String>,
    pub(crate) product: Option<String>,
    pub(crate) connection: String,
    pub(crate) wireless_source: Option<String>,
    pub(crate) endpoint: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) alias: Option<String>,
    pub(crate) raw: Option<String>,
    pub(crate) last_seen_at: u64,
    pub(crate) last_connected_at: Option<u64>,
}

pub(crate) struct DeviceListResult {
    pub(crate) devices: Vec<ManagedDevice>,
    pub(crate) changed: bool,
}

fn connection_from_serial(serial: &str) -> DeviceConnection {
    if serial.contains(':') {
        DeviceConnection::Wireless
    } else {
        DeviceConnection::Usb
    }
}

fn connection_label(connection: &DeviceConnection) -> String {
    match connection {
        DeviceConnection::Usb => "usb".to_string(),
        DeviceConnection::Wireless => "wireless".to_string(),
    }
}

fn source_label(source: &Option<WirelessSource>) -> Option<String> {
    source.as_ref().map(|source| match source {
        WirelessSource::AdbPair => "adb_pair".to_string(),
        WirelessSource::UsbTcpip => "usb_tcpip".to_string(),
        WirelessSource::Manual => "manual".to_string(),
    })
}

fn device_record_needs_update(current: &DeviceRecord, next: &DeviceRecord) -> bool {
    current.display_name != next.display_name
        || current.model != next.model
        || current.product != next.product
        || current.connection != next.connection
        || current.wireless_source != next.wireless_source
        || current.endpoint != next.endpoint
        || current.last_connected_at != next.last_connected_at
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

fn trim_device_records(config: &mut AppConfig, live_serials: &std::collections::HashSet<String>) {
    if config.device_records.len() <= MAX_DEVICE_RECORDS {
        return;
    }

    let mut removable = config
        .device_records
        .values()
        .filter(|record| !live_serials.contains(&record.serial))
        .map(|record| (record.serial.clone(), record.last_seen_at))
        .collect::<Vec<_>>();
    removable.sort_by_key(|(_, last_seen_at)| *last_seen_at);

    for (serial, _) in removable {
        if config.device_records.len() <= MAX_DEVICE_RECORDS {
            break;
        }
        config.device_records.remove(&serial);
    }
}

pub(crate) fn parse_and_merge_devices(
    output: &str,
    config: &mut AppConfig,
    now: u64,
) -> DeviceListResult {
    let live_devices = parse_devices(output, config);
    let mut live_serials = std::collections::HashSet::new();
    let before_records = config.device_records.clone();

    for device in &live_devices {
        live_serials.insert(device.serial.clone());
        let connection = connection_from_serial(&device.serial);
        let endpoint = if matches!(connection, DeviceConnection::Wireless) {
            Some(device.serial.clone())
        } else {
            None
        };
        let existing = config.device_records.get(&device.serial).cloned();
        let display_name = device
            .model
            .clone()
            .or_else(|| {
                existing
                    .as_ref()
                    .and_then(|record| record.display_name.clone())
            })
            .or_else(|| Some(device.serial.clone()));
        let wireless_source = existing
            .as_ref()
            .and_then(|record| record.wireless_source.clone())
            .or_else(|| endpoint.as_ref().map(|_| WirelessSource::Manual));

        let next = DeviceRecord {
            serial: device.serial.clone(),
            display_name,
            model: device
                .model
                .clone()
                .or_else(|| existing.as_ref().and_then(|record| record.model.clone())),
            product: device
                .product
                .clone()
                .or_else(|| existing.as_ref().and_then(|record| record.product.clone())),
            connection,
            wireless_source,
            endpoint,
            last_seen_at: existing
                .as_ref()
                .map(|record| record.last_seen_at)
                .unwrap_or(now),
            last_connected_at: existing
                .as_ref()
                .and_then(|record| record.last_connected_at),
        };

        if existing
            .as_ref()
            .map(|current| device_record_needs_update(current, &next))
            .unwrap_or(true)
        {
            config.device_records.insert(
                device.serial.clone(),
                DeviceRecord {
                    last_seen_at: now,
                    ..next
                },
            );
        }
    }

    trim_device_records(config, &live_serials);
    let changed = before_records != config.device_records;

    let mut managed = Vec::new();
    for device in live_devices {
        if let Some(record) = config.device_records.get(&device.serial) {
            managed.push(ManagedDevice {
                serial: device.serial.clone(),
                state: device.state,
                presence: "online".to_string(),
                model: record.model.clone(),
                product: record.product.clone(),
                connection: connection_label(&record.connection),
                wireless_source: source_label(&record.wireless_source),
                endpoint: record.endpoint.clone(),
                display_name: record.display_name.clone(),
                alias: config.device_aliases.get(&device.serial).cloned(),
                raw: Some(device.raw),
                last_seen_at: record.last_seen_at,
                last_connected_at: record.last_connected_at,
            });
        }
    }

    for record in config.device_records.values() {
        if live_serials.contains(&record.serial) {
            continue;
        }
        managed.push(ManagedDevice {
            serial: record.serial.clone(),
            state: "offline".to_string(),
            presence: "offline".to_string(),
            model: record.model.clone(),
            product: record.product.clone(),
            connection: connection_label(&record.connection),
            wireless_source: source_label(&record.wireless_source),
            endpoint: record.endpoint.clone(),
            display_name: record.display_name.clone(),
            alias: config.device_aliases.get(&record.serial).cloned(),
            raw: None,
            last_seen_at: record.last_seen_at,
            last_connected_at: record.last_connected_at,
        });
    }

    managed.sort_by(|a, b| {
        let a_online = a.presence == "online";
        let b_online = b.presence == "online";
        b_online
            .cmp(&a_online)
            .then_with(|| b.last_seen_at.cmp(&a.last_seen_at))
            .then_with(|| a.serial.cmp(&b.serial))
    });

    DeviceListResult {
        devices: managed,
        changed,
    }
}

pub(crate) fn list_devices_with_adb(
    adb: &str,
    config: &mut AppConfig,
    now: u64,
) -> Result<DeviceListResult, String> {
    let result = run_command_with_timeout(adb, &["devices", "-l"], Duration::from_secs(5));
    if !result.ok {
        return Err(result.message);
    }

    Ok(parse_and_merge_devices(&result.stdout, config, now))
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

    #[test]
    fn merges_live_devices_into_records_and_returns_online_rows() {
        let mut config = AppConfig::default();
        let output = "List of devices attached\nR9YT301WXXX device product:test model:Pixel_8 transport_id:1\n192.168.1.2:5555 device product:test model:Mi_14 transport_id:2\n";

        let result = parse_and_merge_devices(output, &mut config, 100);

        assert!(result.changed);
        assert_eq!(result.devices.len(), 2);
        assert_eq!(result.devices[0].presence, "online");
        assert!(config.device_records.contains_key("R9YT301WXXX"));
        assert!(config.device_records.contains_key("192.168.1.2:5555"));
    }

    #[test]
    fn returns_missing_saved_devices_as_offline_rows() {
        let mut config = AppConfig::default();
        config.device_records.insert(
            "192.168.1.2:5555".to_string(),
            DeviceRecord {
                serial: "192.168.1.2:5555".to_string(),
                display_name: Some("Mi 14".to_string()),
                model: Some("Mi 14".to_string()),
                product: Some("test".to_string()),
                connection: DeviceConnection::Wireless,
                wireless_source: Some(WirelessSource::AdbPair),
                endpoint: Some("192.168.1.2:5555".to_string()),
                last_seen_at: 50,
                last_connected_at: Some(50),
            },
        );

        let result = parse_and_merge_devices("List of devices attached\n", &mut config, 100);

        assert!(!result.changed);
        assert_eq!(result.devices.len(), 1);
        assert_eq!(result.devices[0].serial, "192.168.1.2:5555");
        assert_eq!(result.devices[0].presence, "offline");
        assert_eq!(result.devices[0].connection, "wireless");
    }

    #[test]
    fn repeated_identical_scan_does_not_report_record_changes() {
        let mut config = AppConfig::default();
        let output =
            "List of devices attached\nR9YT301WXXX device product:test model:Pixel_8 transport_id:1\n";

        let first = parse_and_merge_devices(output, &mut config, 100);
        let second = parse_and_merge_devices(output, &mut config, 200);

        assert!(first.changed);
        assert!(!second.changed);
        assert_eq!(
            config
                .device_records
                .get("R9YT301WXXX")
                .map(|record| record.last_seen_at),
            Some(100)
        );
    }
}
