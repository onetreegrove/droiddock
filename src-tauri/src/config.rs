use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{now_secs, scrcpy::ScrcpyOptions};

pub(crate) const CURRENT_CONFIG_SCHEMA_VERSION: u32 = 2;
pub(crate) type DeviceRecords = HashMap<String, DeviceRecord>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppConfig {
    pub(crate) schema_version: u32,
    pub(crate) adb_path: Option<String>,
    pub(crate) scrcpy_path: Option<String>,
    pub(crate) device_aliases: HashMap<String, String>,
    pub(crate) recent_endpoints: Vec<String>,
    pub(crate) device_records: DeviceRecords,
    pub(crate) default_scrcpy_options: ScrcpyOptions,
    pub(crate) default_preset_id: String,
    pub(crate) device_scrcpy_options: HashMap<String, DeviceOptionEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_CONFIG_SCHEMA_VERSION,
            adb_path: None,
            scrcpy_path: None,
            device_aliases: HashMap::new(),
            recent_endpoints: Vec::new(),
            device_records: HashMap::new(),
            default_scrcpy_options: ScrcpyOptions::default(),
            default_preset_id: "daily".to_string(),
            device_scrcpy_options: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DeviceConnection {
    Usb,
    Wireless,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WirelessSource {
    AdbPair,
    UsbTcpip,
    Manual,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DeviceRecord {
    pub(crate) serial: String,
    pub(crate) display_name: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) product: Option<String>,
    pub(crate) connection: DeviceConnection,
    pub(crate) wireless_source: Option<WirelessSource>,
    pub(crate) endpoint: Option<String>,
    pub(crate) last_seen_at: u64,
    pub(crate) last_connected_at: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeviceOptionEntry {
    pub(crate) preset_id: Option<String>,
    pub(crate) options: ScrcpyOptions,
    pub(crate) updated_at: u64,
}

pub(crate) fn config_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("DroidDock"))
}

pub(crate) fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("config.json"))
}

pub(crate) fn tools_dir() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("tools"))
}

pub(crate) fn load_config() -> AppConfig {
    let Ok(path) = config_path() else {
        return AppConfig::default();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return AppConfig::default();
    };

    match serde_json::from_str(&content) {
        Ok(config) => migrate_config(config),
        Err(_) => {
            let backup = path.with_extension(format!("json.bak-{}", now_secs()));
            let _ = fs::copy(&path, backup);
            AppConfig::default()
        }
    }
}

pub(crate) fn migrate_config(mut config: AppConfig) -> AppConfig {
    if config.schema_version < CURRENT_CONFIG_SCHEMA_VERSION {
        config.schema_version = CURRENT_CONFIG_SCHEMA_VERSION;
    }
    config
}

pub(crate) fn save_config_atomic(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let target = dir.join("config.json");
    let temp = dir.join("config.json.tmp");
    let content = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    fs::write(&temp, content).map_err(|error| error.to_string())?;
    fs::rename(&temp, &target).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trips_default_values() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let restored: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.default_preset_id, "daily");
        assert_eq!(restored.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
        assert_eq!(restored.default_scrcpy_options.max_size, Some(1920));
        assert!(restored.device_records.is_empty());
    }

    #[test]
    fn config_defaults_when_new_fields_are_missing() {
        let restored: AppConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(restored.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
        assert_eq!(restored.default_preset_id, "daily");
        assert!(restored.device_aliases.is_empty());
        assert!(restored.device_records.is_empty());
    }

    #[test]
    fn fresh_config_uses_scrcpy_4_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.schema_version, 2);
        assert_eq!(config.default_scrcpy_options.keep_active, Some(true));
        assert_eq!(config.default_scrcpy_options.stay_awake, Some(false));
        assert_eq!(
            config.default_scrcpy_options.window_aspect_ratio_lock,
            Some(true)
        );
    }

    #[test]
    fn migrated_legacy_config_preserves_existing_scrcpy_defaults() {
        let restored: AppConfig = serde_json::from_str(
            r#"{
                "schema_version": 1,
                "default_scrcpy_options": {
                    "maxSize": 1920,
                    "maxFps": 60,
                    "videoCodec": "default",
                    "noAudio": true,
                    "stayAwake": true
                },
                "default_preset_id": "daily"
            }"#,
        )
        .unwrap();

        let migrated = migrate_config(restored);
        assert_eq!(migrated.schema_version, 2);
        assert_eq!(migrated.default_scrcpy_options.stay_awake, Some(true));
        assert_eq!(migrated.default_scrcpy_options.keep_active, None);
        assert_eq!(migrated.default_scrcpy_options.window_aspect_ratio_lock, None);
    }
}
