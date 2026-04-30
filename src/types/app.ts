export type PageKey = 'devices' | 'sessions' | 'setup' | 'settings';

export type ModalKey = null | 'pair' | 'wireless' | 'logs';

export type ToolStatus = {
  adb_path: string | null;
  scrcpy_path: string | null;
  adb_version: string | null;
  scrcpy_version: string | null;
  adb_ok: boolean;
  scrcpy_ok: boolean;
};

export type Device = {
  serial: string;
  state: string;
  model: string | null;
  product: string | null;
  connection: 'usb' | 'wireless';
  alias: string | null;
  raw: string;
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

export type ScrcpyOptions = {
  maxSize?: number;
  maxFps?: number;
  videoBitRate?: string;
  videoCodec?: 'default' | 'h264' | 'h265';
  noAudio?: boolean;
  noControl?: boolean;
  stayAwake?: boolean;
  turnScreenOff?: boolean;
  showTouches?: boolean;
  alwaysOnTop?: boolean;
  fullscreen?: boolean;
};

export type PresetId = 'daily' | 'lowBandwidth' | 'demo' | 'batterySaver' | 'viewOnly';

export type DeviceOptionEntry = {
  presetId: PresetId | null;
  options: ScrcpyOptions;
  updatedAt: number;
};

export type AppConfig = {
  adb_path: string | null;
  scrcpy_path: string | null;
  device_aliases: Record<string, string>;
  recent_endpoints: string[];
  default_scrcpy_options: ScrcpyOptions;
  default_preset_id: PresetId;
  device_scrcpy_options: Record<string, DeviceOptionEntry>;
};

export type PairRequest = {
  host: string;
  pair_port: number;
  pairing_code: string;
  connect_port: number | null;
};
