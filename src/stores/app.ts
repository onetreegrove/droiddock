import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

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
  videoCodec?: string;
  noAudio?: boolean;
  noControl?: boolean;
  stayAwake?: boolean;
  turnScreenOff?: boolean;
  showTouches?: boolean;
  alwaysOnTop?: boolean;
  fullscreen?: boolean;
};

export type PairRequest = {
  host: string;
  pair_port: number;
  pairing_code: string;
  connect_port: number | null;
};

export const useAppStore = defineStore('app', {
  state: () => ({
    toolStatus: null as ToolStatus | null,
    devices: [] as Device[],
    sessions: [] as SessionInfo[],
    selectedSerial: 'discover' as string, // 'discover', 'settings', or device serial
    loading: false,
    logs: [] as string[], // Global app logs
    sessionLogs: {} as Record<string, SessionLogLine[]>,
  }),

  getters: {
    selectedDevice: (state) => state.devices.find(d => d.serial === state.selectedSerial),
    isToolsReady: (state) => !!(state.toolStatus?.adb_ok && state.toolStatus?.scrcpy_ok),
    activeSession: (state) => (serial: string) => state.sessions.find(s => s.serial === serial && s.status === 'running'),
  },

  actions: {
    log(message: string) {
      const time = new Date().toLocaleTimeString();
      this.logs.unshift(`[${time}] ${message}`);
    },

    async fetchToolStatus() {
      this.loading = true;
      try {
        this.toolStatus = await invoke<ToolStatus>('get_tool_status');
      } catch (error) {
        this.log(`工具检查失败: ${String(error)}`);
      } finally {
        this.loading = false;
      }
    },

    async refreshDevices() {
      this.loading = true;
      try {
        this.devices = await invoke<Device[]>('list_devices');
      } catch (error) {
        this.log(`刷新设备失败: ${String(error)}`);
      } finally {
        this.loading = false;
      }
    },

    async refreshSessions() {
      try {
        this.sessions = await invoke<SessionInfo[]>('list_sessions');
      } catch (error) {
        this.log(`刷新会话失败: ${String(error)}`);
      }
    },

    async startMirror(serial: string, options: ScrcpyOptions) {
      try {
        const info = await invoke<SessionInfo>('start_scrcpy', { serial, options });
        await this.refreshSessions();
        this.log(`已启动投屏: ${serial}`);
        return info;
      } catch (error) {
        this.log(`启动失败: ${String(error)}`);
        throw error;
      }
    },

    async stopMirror(sessionId: string) {
      try {
        await invoke('stop_scrcpy', { sessionId });
        await this.refreshSessions();
        this.log(`已停止投屏: ${sessionId}`);
      } catch (error) {
        this.log(`停止失败: ${String(error)}`);
      }
    },

    async adbTcpip(serial: string) {
      try {
        await invoke('adb_tcpip', { serial });
        this.log(`设备 ${serial} 已切换到无线模式`);
      } catch (error) {
        this.log(`切换无线失败: ${String(error)}`);
      }
    },

    async adbConnect(endpoint: string) {
      try {
        await invoke('adb_connect', { endpoint });
        this.log(`已连接无线设备: ${endpoint}`);
        await this.refreshDevices();
      } catch (error) {
        this.log(`无线连接失败: ${String(error)}`);
      }
    },

    async adbPair(request: PairRequest) {
      try {
        await invoke('adb_pair', { request });
        this.log(`配对并连接成功: ${request.host}`);
        await this.refreshDevices();
      } catch (error) {
        this.log(`配对失败: ${String(error)}`);
        throw error;
      }
    },

    async fetchSessionLogs(sessionId: string) {
      try {
        const logs = await invoke<SessionLogLine[]>('get_session_logs', { sessionId });
        this.sessionLogs[sessionId] = logs;
      } catch (error) {
        // Silent error for periodic log fetching
      }
    }
  }
});
