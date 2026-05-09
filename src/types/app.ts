export type PageKey = 'dashboard' | 'devices' | 'sessions' | 'setup' | 'settings';

export type ModalKey = null | 'pair' | 'wireless' | 'logs';

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

export type {
  AppConfig,
  Device,
  DeviceOptionEntry,
  PairRequest,
  SessionInfo,
  SessionLogLine,
  ToolInstallResult,
  ToolStatus,
} from '../lib/ipc/types';
