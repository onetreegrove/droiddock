export type PageKey = 'devices' | 'sessions' | 'setup' | 'settings';

export type ModalKey = null | 'pair' | 'wireless' | 'reconnect' | 'logs';

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
  keepActive?: boolean;
  backgroundColor?: string;
  windowAspectRatioLock?: boolean;
};

export type ScrcpyCapabilities = {
  supportsKeepActive: boolean;
  supportsBackgroundColor: boolean;
  supportsWindowAspectRatioLock: boolean;
};

export type PresetId = 'daily' | 'lowBandwidth' | 'demo' | 'batterySaver' | 'viewOnly';

export type {
  AppConfig,
  Device,
  DeviceRecord,
  ManagedDevice,
  DeviceOptionEntry,
  PairRequest,
  SessionInfo,
  SessionLogLine,
  ToolInstallResult,
  ToolStatus,
  WirelessSource,
} from '../lib/ipc/types';
