import type { PresetId, ScrcpyOptions } from '../../types/app';

export type ToolKind = 'adb' | 'scrcpy';
export type ToolInstallTarget = ToolKind | 'all';
export type ToolInstallStatus = 'idle' | 'running' | 'success' | 'failed';

export type ToolSource =
  | 'configured'
  | 'bundled'
  | 'android_sdk'
  | 'local_bin'
  | 'homebrew'
  | 'system_path';

export type ToolHealth =
  | 'ready'
  | 'missing'
  | 'not_executable'
  | 'wrong_tool'
  | 'version_failed'
  | 'incompatible_arch';

export type ToolDiagnostic = {
  kind: ToolKind;
  path: string | null;
  source: ToolSource | null;
  version: string | null;
  arch: string | null;
  health: ToolHealth;
  message: string;
};

export type ToolStatus = {
  adb: ToolDiagnostic;
  scrcpy: ToolDiagnostic;
  adb_ok: boolean;
  scrcpy_ok: boolean;
};

export type ToolInstallResult = {
  target: ToolInstallTarget;
  adb_path: string | null;
  scrcpy_path: string | null;
  logs: string[];
};

export type ToolInstallProgress = {
  target: ToolInstallTarget;
  level: 'info' | 'success' | 'error';
  message: string;
};

export type DeviceConnection = 'usb' | 'wireless';
export type DevicePresence = 'online' | 'offline';
export type WirelessSource = 'adb_pair' | 'usb_tcpip' | 'manual';

export type Device = {
  serial: string;
  state: string;
  model: string | null;
  product: string | null;
  connection: DeviceConnection;
  alias: string | null;
  raw: string;
};

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

export type SessionLogLine = {
  timestamp: number;
  level: string;
  message: string;
};

export type SessionInfo = {
  session_id: string;
  serial: string;
  alias: string | null;
  pid: number;
  status: 'idle' | 'starting' | 'running' | 'stopped' | 'failed';
  started_at: number;
  connection: string;
  args: string[];
  last_message: string | null;
};

export type DeviceOptionEntry = {
  presetId: PresetId | null;
  options: ScrcpyOptions;
  updatedAt: number;
};

export type AppConfig = {
  schema_version: number;
  adb_path: string | null;
  scrcpy_path: string | null;
  device_aliases: Record<string, string>;
  recent_endpoints: string[];
  device_records: Record<string, DeviceRecord>;
  default_scrcpy_options: ScrcpyOptions;
  default_preset_id: PresetId;
  device_scrcpy_options: Record<string, DeviceOptionEntry>;
};

export type PairRequest = {
  host: string;
  pair_port: number;
  pairing_code: string;
  connect_host: string | null;
  connect_port: number | null;
};
