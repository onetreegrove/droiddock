# Device Management History and Reconnect Design

## Context

DroidDock currently treats `adb devices -l` as the authoritative device list. The frontend stores only the latest live devices in `src/stores/devices.ts`, while the Rust device parser returns only records that ADB reports in the current command output. Wireless reconnect is supported through `recent_endpoints`, and the wireless modal already lets users edit the connect port before running `adb connect`.

The requested change is to make device management more durable:

1. Keep historical devices in the list and show their connection method.
2. Support reconnecting devices. Wireless reconnect must allow editing the port for both ADB Pair and USB-to-wireless devices.
3. Refresh USB discovery on a timer.

This design keeps the feature inside the current Tauri v2, Vue 3, TypeScript, Pinia, and Rust command architecture.

## Goals

- Show both live and historical devices in the device list.
- Preserve the last known connection method for historical devices.
- Keep USB devices easy to rediscover by timer refresh.
- Give wireless historical devices a direct reconnect action with editable connection port.
- Preserve the existing separation between ADB Pair pairing endpoint and connect endpoint.
- Avoid storing ADB Pair pairing codes.
- Avoid requiring shell `PATH`, Homebrew, sudo, or terminal usage.

## Non-Goals

- Do not implement QR pairing or mDNS scanning in this change.
- Do not embed scrcpy video in DroidDock.
- Do not add logcat or file management.
- Do not migrate historical device identity to Android hardware IDs that are not currently available through `adb devices -l`.
- Do not remove the existing manual wireless connection modal.

## Design Alternatives

### Option A: Keep Using Only `recent_endpoints`

This is the smallest change. The UI would display recent wireless endpoints as pseudo devices, while USB devices would still only appear when connected.

Trade-offs:

- Low backend cost.
- Works only for wireless history.
- Cannot represent USB historical devices.
- Cannot preserve model, alias, source, and last-seen state cleanly.

This option does not satisfy the full request.

### Option B: Add a Unified Persistent `DeviceRecord`

Persist a map of device records in `AppConfig`, merge live ADB devices into it on refresh, and return a unified list to the frontend. Each item has a live/offline state, connection method, wireless source, endpoint, model, alias, and timestamps.

Trade-offs:

- Meets all requested requirements.
- Keeps reconnect logic attached to a device card instead of a generic endpoint list.
- Adds a small config schema extension.
- Requires focused frontend and backend tests.

This is the recommended approach.

### Option C: Backend Runtime Cache Only

Keep history in memory for the current app session but do not persist it.

Trade-offs:

- Avoids config changes.
- History disappears after restart.
- Does not match user expectation for "historical devices".

This option is not recommended.

## Recommended Approach

Implement Option B with a `DeviceRecord` persisted in config and a frontend-facing `ManagedDevice` list derived from live ADB output plus saved history.

The device list should render one row per known device identity:

- A live USB device appears as `online`, connection `usb`.
- A live wireless device appears as `online`, connection `wireless`.
- A previously seen USB device that is not currently in `adb devices -l` appears as `offline`, connection `usb`, with guidance to plug it in.
- A previously connected wireless device that is not currently in `adb devices -l` appears as `offline`, connection `wireless`, with a reconnect action.

Wireless reconnect continues to execute `adb connect <host>:<port>`. The UI lets the user edit only the connect port, not the Pair pairing port.

## Data Model

Add the following frontend and backend-compatible shape.

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

Persist records in `AppConfig`:

```ts
device_records: Record<string, DeviceRecord>;
```

Keying rules:

- USB devices use the ADB serial as the key.
- Wireless devices use the endpoint serial, such as `192.168.1.105:39845`.
- If the user reconnects a wireless device with a different port, the new endpoint becomes the current key. The old endpoint can remain as a historical record unless it exceeds the retention cap.

Retention:

- Keep at most 100 `device_records`.
- When trimming, remove the oldest offline records by `last_seen_at`.
- Do not trim live devices from the returned list.

## Backend Behavior

`parse_devices` keeps parsing live ADB rows as it does now, but the device management layer should merge those rows into config records.

On `list_devices`:

1. Load config.
2. Run `adb devices -l`.
3. Parse live devices.
4. For each live device, update or create a `DeviceRecord`.
5. Save config only if records changed.
6. Return all live records plus persisted offline records as `ManagedDevice[]`.

On `adb_connect` success:

1. Remember the endpoint in `recent_endpoints` for backwards compatibility.
2. Upsert a wireless `DeviceRecord` with `wireless_source` from the request.
3. Set `last_connected_at` and `last_seen_at`.
4. Refresh device list in the frontend after the command returns.

On `adb_pair` success followed by connect:

- Use `wireless_source = 'adb_pair'`.
- Persist only the connect endpoint.
- Never persist `pairing_code`.
- Never persist the Pair pairing endpoint as a reconnect target unless it is also the connect endpoint.

On USB-to-wireless success:

- After `adb_tcpip`, `adb_connect` should persist `wireless_source = 'usb_tcpip'`.
- The original USB serial remains a USB history record.
- The wireless endpoint appears as a separate historical wireless record.

## Frontend Behavior

The device store should expose `managedDevices` instead of only live `devices`. During implementation, this can replace `devices` if the returned IPC type changes to `ManagedDevice[]`.

Device list presentation:

- Show status chip: `在线`, `离线`, `待授权`, `离线状态`, or the current ADB state.
- Show connection chip: `USB`, `ADB Pair 无线`, `USB 转无线`, or `无线`.
- For offline USB devices, show "插入 USB 后会自动刷新".
- For offline wireless devices, show "可重连，端口可修改".
- Keep current alias behavior.

Device actions:

- Live `device` state: allow starting scrcpy.
- Live `unauthorized`: guide the user to approve USB debugging.
- Live `offline`: guide reconnect or replug.
- Offline USB historical device: disable start, allow manual refresh.
- Offline wireless historical device: show reconnect action.

Wireless reconnect interaction:

- Selecting reconnect opens the existing wireless modal or a focused reconnect modal.
- The host is prefilled from the endpoint.
- The connect port is editable.
- The action calls `adbConnect(endpoint, source)` or an equivalent typed wrapper.

## Timer Refresh

The app already refreshes devices every 3 seconds. The new behavior should make the timer single-flight:

- If a refresh is already running, skip the next timer tick.
- Manual refresh can reuse the same guarded method.
- Keep session refresh in the same loop unless it creates UI stalls.

This prevents slow ADB commands from overlapping and causing stale writes or selection flicker.

## Selection Rules

After refresh:

- Preserve selected serial if it still exists in the managed list.
- If the selected live device becomes offline, keep it selected.
- If no selected device exists, prefer the first live `device` state item.
- If no live device exists, prefer the most recently seen historical item.

This avoids bouncing the detail panel when a wireless device temporarily drops.

## Error Handling

Existing backend error translation should continue to turn ADB failures into user-readable Chinese messages.

Additional UI guidance:

- `Connection refused`: "无线调试端口不可用，请检查手机无线调试页面中的当前连接端口。"
- `device not found`: "设备当前不在线，请刷新列表，或重新插拔 USB / 重连无线。"
- `failed to authenticate`: "配对状态已失效，请重新生成配对码。"

Reconnect failures should not delete historical records. They should preserve the device card and show the last attempted endpoint.

## Testing

Rust tests:

- Parsing live USB and wireless devices still works.
- Merging live devices creates records.
- Missing live devices are returned as offline historical devices.
- `adb_pair` connect success records `wireless_source = 'adb_pair'`.
- `adb_connect` records editable endpoints and deduplicates recent endpoints.

TypeScript tests:

- Endpoint split/build helpers continue to validate ports.
- Managed device sorting prefers live available devices, then recent historical devices.
- Reconnect form pre-fills host/port from endpoint and rebuilds endpoint after port edit.
- Selection rules preserve offline selected devices.

Manual validation:

- Start app with no devices: historical list remains visible if config has records.
- Plug in USB device: row becomes online USB after timer refresh.
- Unplug USB device: row remains as offline USB.
- Connect wireless endpoint: row appears as online wireless.
- Disconnect wireless endpoint: row remains offline and can be reconnected with a changed port.

## Acceptance Criteria

- Historical USB and wireless devices remain visible after disconnect and app restart.
- Each historical row shows its last known connection method.
- ADB Pair wireless devices can reconnect without re-entering pairing code.
- USB-to-wireless devices can reconnect by editing the connect port.
- Timer refresh discovers newly plugged USB devices without manual refresh.
- Timer refresh does not run overlapping `adb devices -l` calls.
- Pairing code is not persisted.
- Existing `npm run test` and `cd src-tauri && cargo test` pass after implementation.

