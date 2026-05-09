# Device Management History and Reconnect Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep historical devices visible, show their connection method, support wireless reconnect with editable ports, and make USB discovery refresh safely on a timer.

**Architecture:** Persist `DeviceRecord` entries in Tauri config, merge live `adb devices -l` output with saved records in the backend, and return a unified managed-device list to the Vue store. Frontend components render online/offline state, route wireless reconnect through existing endpoint helpers, and guard periodic refresh from overlapping ADB calls.

**Tech Stack:** Tauri v2, Rust, Vue 3, TypeScript, Pinia, Vitest, Cargo test.

---

## Files and Responsibilities

- Modify `src-tauri/src/config.rs`: add persisted `device_records` and Rust structs for saved records.
- Modify `src-tauri/src/devices.rs`: create `ManagedDevice`, merge live ADB output with saved history, sort records, and return offline historical rows.
- Modify `src-tauri/src/wireless.rs`: preserve wireless source and upsert device records after successful connect.
- Modify `src-tauri/src/lib.rs`: update command wrappers and IPC return types for managed devices.
- Modify `src/lib/ipc/types.ts`: add `DeviceRecord`, `ManagedDevice`, and wireless source types.
- Modify `src/stores/devices.ts`: store managed devices, preserve selected offline records, and expose live counts.
- Modify `src/stores/app.ts`: pass wireless source through connect flows and expose managed device data.
- Modify `src/components/DeviceList.vue`: render historical/offline devices and reconnect actions.
- Modify `src/components/DeviceDetailPanel.vue`: disable start for offline records and show actionable status text.
- Modify `src/components/WirelessModal.vue`: accept an optional reconnect endpoint/source and keep port editable.
- Modify `src/App.vue`: replace overlapping interval with single-flight refresh.
- Modify `src/domain/wireless.ts`: add small helpers for source labels and endpoint updates.
- Modify `src/domain/wireless.test.ts`: cover endpoint edit and source labeling.

## Task 1: Add Persisted Device Record Types

**Files:**
- Modify: `src-tauri/src/config.rs`
- Modify: `src/lib/ipc/types.ts`

- [ ] **Step 1: Add Rust config types**

In `src-tauri/src/config.rs`, add the device record fields and types.

```rust
pub(crate) type DeviceRecords = HashMap<String, DeviceRecord>;

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
```

Add this field to `AppConfig`:

```rust
pub(crate) device_records: DeviceRecords,
```

Initialize it in `Default`:

```rust
device_records: HashMap::new(),
```

- [ ] **Step 2: Add frontend IPC types**

In `src/lib/ipc/types.ts`, add:

```ts
export type DeviceConnection = 'usb' | 'wireless';
export type DevicePresence = 'online' | 'offline';
export type WirelessSource = 'adb_pair' | 'usb_tcpip' | 'manual';

export type DeviceRecord = {
  serial: string;
  display_name: string | null;
  model: string | null;
  product: string | null;
  connection: DeviceConnection;
  wireless_source: WirelessSource | null;
  endpoint: string | null;
  last_seen_at: number;
  last_connected_at: number | null;
};

export type ManagedDevice = DeviceRecord & {
  state: string;
  presence: DevicePresence;
  alias: string | null;
  raw: string | null;
};
```

Keep the existing `Device` export temporarily if nearby code still imports it, but point device-store imports to `ManagedDevice` in later tasks.

- [ ] **Step 3: Run type and Rust checks for this slice**

Run:

```bash
npm run test -- src/domain/wireless.test.ts
cd src-tauri && cargo test config
```

Expected:

- Vitest still passes existing wireless tests.
- Cargo either passes or reports only tests unrelated to the new unused types. If Rust warns about unused items, continue; later tasks will use them.

## Task 2: Merge Live ADB Devices With Historical Records

**Files:**
- Modify: `src-tauri/src/devices.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add tests for merge behavior**

In `src-tauri/src/devices.rs`, add tests in the existing test module:

```rust
#[test]
fn merges_live_devices_into_records_and_returns_online_rows() {
    let mut config = AppConfig::default();
    let output = "List of devices attached\nR9YT301WXXX device product:test model:Pixel_8 transport_id:1\n192.168.1.2:5555 device product:test model:Mi_14 transport_id:2\n";

    let devices = parse_and_merge_devices(output, &mut config, 100);

    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].presence, "online");
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

    let devices = parse_and_merge_devices("List of devices attached\n", &mut config, 100);

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].serial, "192.168.1.2:5555");
    assert_eq!(devices[0].presence, "offline");
    assert_eq!(devices[0].connection, "wireless");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cd src-tauri && cargo test devices
```

Expected: FAIL because `parse_and_merge_devices`, `ManagedDevice`, and the config record fields are not fully implemented.

- [ ] **Step 3: Implement managed device merge**

In `src-tauri/src/devices.rs`, keep the existing parser, then add a managed output type:

```rust
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
```

Add helpers:

```rust
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
```

Add `parse_and_merge_devices` that updates `config.device_records` and returns live plus offline rows:

```rust
pub(crate) fn parse_and_merge_devices(
    output: &str,
    config: &mut AppConfig,
    now: u64,
) -> Vec<ManagedDevice> {
    let live_devices = parse_devices(output, config);
    let mut live_serials = std::collections::HashSet::new();

    for device in &live_devices {
        live_serials.insert(device.serial.clone());
        let connection = connection_from_serial(&device.serial);
        let endpoint = if matches!(connection, DeviceConnection::Wireless) {
            Some(device.serial.clone())
        } else {
            None
        };
        let existing_source = config
            .device_records
            .get(&device.serial)
            .and_then(|record| record.wireless_source.clone());

        config.device_records.insert(
            device.serial.clone(),
            DeviceRecord {
                serial: device.serial.clone(),
                display_name: device.model.clone().or_else(|| Some(device.serial.clone())),
                model: device.model.clone(),
                product: device.product.clone(),
                connection,
                wireless_source: existing_source.or_else(|| endpoint.as_ref().map(|_| WirelessSource::Manual)),
                endpoint,
                last_seen_at: now,
                last_connected_at: config
                    .device_records
                    .get(&device.serial)
                    .and_then(|record| record.last_connected_at),
            },
        );
    }

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

    managed
}
```

- [ ] **Step 4: Update `list_devices_with_adb`**

Change its signature and body:

```rust
pub(crate) fn list_devices_with_adb(
    adb: &str,
    config: &mut AppConfig,
    now: u64,
) -> Result<Vec<ManagedDevice>, String> {
    let result = run_command_with_timeout(adb, &["devices", "-l"], Duration::from_secs(5));
    if !result.ok {
        return Err(result.message);
    }

    Ok(parse_and_merge_devices(&result.stdout, config, now))
}
```

Update `src-tauri/src/lib.rs` command wrapper to pass mutable config and save it after merge:

```rust
fn list_devices(state: State<'_, AppState>) -> Result<Vec<devices::ManagedDevice>, String> {
    let mut config = load_config();
    let adb = resolve_adb_path(&config)?;
    let devices = devices::list_devices_with_adb(&adb, &mut config, now_secs())?;
    save_config_atomic(&config)?;
    Ok(devices)
}
```

- [ ] **Step 5: Run Rust tests**

Run:

```bash
cd src-tauri && cargo test devices
```

Expected: PASS.

## Task 3: Record Wireless Source on Connect

**Files:**
- Modify: `src-tauri/src/wireless.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/ipc/types.ts`
- Modify: `src/stores/app.ts`

- [ ] **Step 1: Add backend tests**

In `src-tauri/src/wireless.rs`, add:

```rust
#[test]
fn remember_wireless_record_updates_endpoint_and_source() {
    let mut config = AppConfig::default();
    remember_wireless_device(
        &mut config,
        "192.168.1.10:41235".to_string(),
        WirelessSource::AdbPair,
        200,
    );

    let record = config.device_records.get("192.168.1.10:41235").unwrap();
    assert_eq!(record.endpoint.as_deref(), Some("192.168.1.10:41235"));
    assert_eq!(record.wireless_source, Some(WirelessSource::AdbPair));
    assert_eq!(record.last_connected_at, Some(200));
}
```

- [ ] **Step 2: Implement record helper**

In `src-tauri/src/wireless.rs`, import config types and add:

```rust
pub(crate) fn remember_wireless_device(
    config: &mut AppConfig,
    endpoint: String,
    source: WirelessSource,
    now: u64,
) {
    let existing = config.device_records.get(&endpoint).cloned();
    config.device_records.insert(
        endpoint.clone(),
        DeviceRecord {
            serial: endpoint.clone(),
            display_name: existing
                .as_ref()
                .and_then(|record| record.display_name.clone())
                .or_else(|| Some(endpoint.clone())),
            model: existing.as_ref().and_then(|record| record.model.clone()),
            product: existing.as_ref().and_then(|record| record.product.clone()),
            connection: DeviceConnection::Wireless,
            wireless_source: Some(source),
            endpoint: Some(endpoint),
            last_seen_at: now,
            last_connected_at: Some(now),
        },
    );
}
```

- [ ] **Step 3: Thread source through connect commands**

In frontend type definitions, add:

```ts
export type AdbConnectRequest = {
  endpoint: string;
  source: WirelessSource;
};
```

Change `store.adbConnect` to accept a source default:

```ts
async function adbConnect(endpoint: string, source: WirelessSource = 'manual') {
  setBusy('wireless', true);
  try {
    await invokeCommand('adb_connect', { endpoint, source });
    log(`已连接无线设备: ${endpoint}`);
    await fetchAppConfig();
    await refreshDevices();
  } catch (error) {
    log(`无线连接失败: ${errorMessage(error)}`);
    throw error;
  } finally {
    setBusy('wireless', false);
  }
}
```

In `src-tauri/src/lib.rs`, change the command wrapper:

```rust
fn adb_connect(
    state: State<'_, AppState>,
    endpoint: String,
    source: Option<config::WirelessSource>,
) -> Result<CommandResult, String> {
    let mut config = load_config();
    let adb = resolve_adb_path(&config)?;
    let result = wireless::adb_connect_with_adb(
        &adb,
        &mut config,
        endpoint,
        source.unwrap_or(config::WirelessSource::Manual),
        now_secs(),
    )?;
    save_config_atomic(&config)?;
    Ok(result)
}
```

Change `adb_connect_with_adb` signature to accept `source` and `now`, and call `remember_wireless_device` after successful connect.

- [ ] **Step 4: Keep Pair source explicit**

In `adb_pair_with_adb`, after connect succeeds:

```rust
remember_endpoint(config, connect_endpoint.clone());
remember_wireless_device(config, connect_endpoint, WirelessSource::AdbPair, now);
```

Add `now: u64` to the function signature and pass it from `lib.rs`.

- [ ] **Step 5: Run tests**

Run:

```bash
cd src-tauri && cargo test wireless
npm run test -- src/domain/wireless.test.ts
```

Expected: PASS.

## Task 4: Update Frontend Device Store and Selection Rules

**Files:**
- Modify: `src/stores/devices.ts`
- Modify: `src/types/app.ts`

- [ ] **Step 1: Update imports and state**

Change the store to use `ManagedDevice`.

```ts
import type { ManagedDevice } from '../lib/ipc/types';

export const useDevicesStore = defineStore('devices', {
  state: () => ({
    devices: [] as ManagedDevice[],
  }),
  getters: {
    availableDeviceCount: (state) =>
      state.devices.filter((device) => device.presence === 'online' && device.state === 'device').length,
    selectedDevice: (state) => {
      const ui = useUiStore();
      return state.devices.find((device) => device.serial === ui.selectedSerial) ?? null;
    },
  },
});
```

- [ ] **Step 2: Add selection helper**

Add a helper in the actions block:

```ts
function preferredSerial(devices: ManagedDevice[]): string | null {
  return (
    devices.find((device) => device.presence === 'online' && device.state === 'device')?.serial ??
    devices[0]?.serial ??
    null
  );
}
```

Update `refreshDevices`:

```ts
async refreshDevices() {
  this.devices = await invokeCommand<ManagedDevice[]>('list_devices');
  const ui = useUiStore();
  if (!ui.selectedSerial && this.devices.length > 0) {
    ui.selectedSerial = preferredSerial(this.devices);
  }
  if (ui.selectedSerial && !this.devices.some((device) => device.serial === ui.selectedSerial)) {
    ui.selectedSerial = preferredSerial(this.devices);
  }
}
```

- [ ] **Step 3: Export the new type alias if needed**

In `src/types/app.ts`, export `ManagedDevice`, `DeviceRecord`, and `WirelessSource` from IPC types.

```ts
export type {
  AppConfig,
  Device,
  DeviceRecord,
  ManagedDevice,
  WirelessSource,
  DeviceOptionEntry,
  PairRequest,
  SessionInfo,
  SessionLogLine,
  ToolInstallResult,
  ToolStatus,
} from '../lib/ipc/types';
```

- [ ] **Step 4: Run frontend tests**

Run:

```bash
npm run test -- src
```

Expected: PASS or type errors only in components that still expect the old `Device` shape. Resolve those in Task 5.

## Task 5: Render Historical Devices and Reconnect Actions

**Files:**
- Modify: `src/components/DeviceList.vue`
- Modify: `src/components/DeviceDetailPanel.vue`
- Modify: `src/components/WirelessModal.vue`
- Modify: `src/stores/ui.ts`

- [ ] **Step 1: Add display helpers in `DeviceList.vue`**

Add helpers:

```ts
function presenceLabel(device: { presence: string; state: string }) {
  if (device.presence === 'offline') return '离线';
  return stateLabel(device.state);
}

function connectionLabel(device: { connection: string; wireless_source: string | null }) {
  if (device.connection === 'usb') return 'USB';
  if (device.wireless_source === 'adb_pair') return 'ADB Pair 无线';
  if (device.wireless_source === 'usb_tcpip') return 'USB 转无线';
  return '无线';
}

function connectionHint(device: { presence: string; connection: string }) {
  if (device.presence !== 'offline') return '';
  if (device.connection === 'usb') return '插入 USB 后会自动刷新';
  return '可重连，端口可修改';
}
```

Use `presenceLabel(device)` for the status chip and `connectionLabel(device)` for the connection chip.

- [ ] **Step 2: Add reconnect button for offline wireless**

In each device card, render:

```vue
<button
  v-if="device.presence === 'offline' && device.connection === 'wireless' && device.endpoint"
  class="btn btn-ghost compact-button"
  @click.stop="ui.openWirelessReconnect(device.endpoint, device.wireless_source || 'manual')"
>
  重连
</button>
```

This requires adding `openWirelessReconnect` in `src/stores/ui.ts`.

- [ ] **Step 3: Extend UI store modal state**

In `src/stores/ui.ts`, add:

```ts
wirelessReconnectEndpoint: null as string | null,
wirelessReconnectSource: 'manual' as WirelessSource,
```

Add actions:

```ts
openWirelessReconnect(endpoint: string, source: WirelessSource) {
  this.wirelessReconnectEndpoint = endpoint;
  this.wirelessReconnectSource = source;
  this.modal = 'wireless';
},
closeModal() {
  this.modal = null;
  this.wirelessReconnectEndpoint = null;
  this.wirelessReconnectSource = 'manual';
},
```

- [ ] **Step 4: Prefill reconnect modal**

In `WirelessModal.vue`, initialize reconnect fields from `ui.wirelessReconnectEndpoint` before falling back to recent endpoints:

```ts
watch(
  () => ui.wirelessReconnectEndpoint,
  (endpoint) => {
    if (!endpoint) return;
    selectRecentEndpoint(endpoint);
  },
  { immediate: true },
);
```

Change reconnect to pass source:

```ts
await store.adbConnect(
  buildConnectEndpoint(reconnectHost.value, reconnectPort.value),
  ui.wirelessReconnectSource,
);
```

Change USB-to-wireless submit to pass source:

```ts
await store.adbConnect(buildConnectEndpoint(host.value, port.value || '5555'), 'usb_tcpip');
```

- [ ] **Step 5: Guard detail panel start action**

In `DeviceDetailPanel.vue`, disable start if the selected device is offline:

```vue
<button
  class="btn btn-primary"
  :disabled="!device || device.presence === 'offline' || device.state !== 'device' || store.busy.start"
  @click="launch"
>
  启动投屏
</button>
```

Show a short offline hint near the device status:

```vue
<div v-if="device?.presence === 'offline'" class="device-warning">
  {{ device.connection === 'wireless' ? '设备当前不在线，可先重连无线调试。' : '设备当前不在线，插入 USB 后会自动刷新。' }}
</div>
```

- [ ] **Step 6: Run component type check through build**

Run:

```bash
npm run build
```

Expected: PASS. If CSS class names are missing, reuse existing button and warning classes from the current components.

## Task 6: Make Timer Refresh Single-Flight

**Files:**
- Modify: `src/App.vue`
- Modify: `src/stores/app.ts`

- [ ] **Step 1: Add guarded refresh action**

In `src/stores/app.ts`, add:

```ts
let refreshInFlight = false;

async function refreshRuntimeState() {
  if (refreshInFlight) return;
  refreshInFlight = true;
  try {
    await refreshDevices();
    await refreshSessions();
  } finally {
    refreshInFlight = false;
  }
}
```

Return `refreshRuntimeState` from the store.

- [ ] **Step 2: Replace interval body**

In `src/App.vue`, change mounted refresh calls:

```ts
await store.refreshRuntimeState();
unlistenLogs = await store.listenSessionLogs();

poller = window.setInterval(() => {
  void store.refreshRuntimeState();
}, 3000);
```

Keep the existing cleanup:

```ts
if (poller) window.clearInterval(poller);
```

- [ ] **Step 3: Run frontend tests**

Run:

```bash
npm run test -- src
npm run build
```

Expected: PASS.

## Task 7: Full Verification

**Files:**
- No new files.

- [ ] **Step 1: Run full frontend tests**

Run:

```bash
npm run test
```

Expected: PASS.

- [ ] **Step 2: Run Rust tests**

Run:

```bash
cd src-tauri && cargo test
```

Expected: PASS.

- [ ] **Step 3: Run production build**

Run:

```bash
npm run build
```

Expected: PASS and Vite emits the production build.

- [ ] **Step 4: Manual smoke test**

Run the app using the repo's normal dev command, then verify:

```text
1. With no connected device, historical device records remain visible.
2. Plug in a USB device and wait up to 3 seconds; it becomes online USB.
3. Unplug the USB device; it remains in the list as offline USB.
4. Connect or fake a wireless endpoint; it appears as wireless history.
5. Reconnect an offline wireless device and edit the port before submitting.
6. Confirm pairing code is never displayed or persisted in config.
```

## Self-Review

- Spec coverage: covered historical list, connection method labels, wireless reconnect with editable port, timer refresh, no pairing-code persistence, and verification.
- Placeholder scan: no placeholder markers or open-ended implementation instructions remain.
- Type consistency: plan uses `DeviceRecord`, `ManagedDevice`, `WirelessSource`, `device_records`, `wireless_source`, and `endpoint` consistently across Rust and TypeScript.
