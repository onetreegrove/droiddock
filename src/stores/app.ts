import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import {
  type AppConfig,
  type Device,
  type ModalKey,
  type PageKey,
  type PairRequest,
  type PresetId,
  type ScrcpyOptions,
  type SessionInfo,
  type SessionLogLine,
  type ToolStatus,
} from '../types/app';
import { defaultScrcpyOptions, mergeScrcpyOptions } from '../domain/scrcpyOptions';

type BusyKey =
  | 'config'
  | 'tools'
  | 'devices'
  | 'sessions'
  | 'start'
  | 'stop'
  | 'pair'
  | 'wireless'
  | 'settings';

export const useAppStore = defineStore('app', {
  state: () => ({
    toolStatus: null as ToolStatus | null,
    appConfig: null as AppConfig | null,
    devices: [] as Device[],
    sessions: [] as SessionInfo[],
    currentPage: 'devices' as PageKey,
    selectedSerial: null as string | null,
    modal: null as ModalKey,
    loading: false,
    busy: {} as Record<BusyKey, boolean>,
    logs: [] as string[],
    sessionLogs: {} as Record<string, SessionLogLine[]>,
    globalDraftOptions: { ...defaultScrcpyOptions } as ScrcpyOptions,
    globalDraftPresetId: 'daily' as PresetId,
    deviceDraftOptions: {} as Record<string, ScrcpyOptions>,
    sessionDraftOptions: {} as Record<string, ScrcpyOptions>,
  }),

  getters: {
    selectedDevice: (state) => state.devices.find((device) => device.serial === state.selectedSerial) ?? null,
    isToolsReady: (state) => Boolean(state.toolStatus?.adb_ok && state.toolStatus?.scrcpy_ok),
    availableDeviceCount: (state) => state.devices.filter((device) => device.state === 'device').length,
    activeSession: (state) => (serial: string) =>
      state.sessions.find((session) => session.serial === serial && session.status === 'running') ?? null,
    deviceOptionEntry: (state) => (serial: string) => state.appConfig?.device_scrcpy_options[serial] ?? null,
    effectiveOptions: (state) => (serial: string) => {
      const globalOptions = state.appConfig?.default_scrcpy_options ?? defaultScrcpyOptions;
      const deviceOptions = state.appConfig?.device_scrcpy_options[serial]?.options;
      const sessionOptions = state.sessionDraftOptions[serial];
      return mergeScrcpyOptions(globalOptions, deviceOptions, sessionOptions);
    },
  },

  actions: {
    log(message: string) {
      const time = new Date().toLocaleTimeString();
      this.logs.unshift(`[${time}] ${message}`);
      this.logs = this.logs.slice(0, 200);
    },

    setBusy(key: BusyKey, value: boolean) {
      this.busy[key] = value;
      this.loading = Object.values(this.busy).some(Boolean);
    },

    async fetchAppConfig() {
      this.setBusy('config', true);
      try {
        this.appConfig = await invoke<AppConfig>('get_app_config');
        this.globalDraftOptions = { ...(this.appConfig.default_scrcpy_options ?? defaultScrcpyOptions) };
        this.globalDraftPresetId = this.appConfig.default_preset_id ?? 'daily';
      } catch (error) {
        this.log(`读取配置失败: ${String(error)}`);
      } finally {
        this.setBusy('config', false);
      }
    },

    async saveDefaultOptions(options: ScrcpyOptions, presetId: PresetId) {
      this.setBusy('settings', true);
      try {
        this.appConfig = await invoke<AppConfig>('save_default_scrcpy_options', { options, presetId });
        this.globalDraftOptions = { ...options };
        this.globalDraftPresetId = presetId;
        this.log('已保存全局默认参数');
      } catch (error) {
        this.log(`保存全局默认参数失败: ${String(error)}`);
        throw error;
      } finally {
        this.setBusy('settings', false);
      }
    },

    async saveDeviceOptions(serial: string, options: ScrcpyOptions, presetId: PresetId | null) {
      this.setBusy('settings', true);
      try {
        this.appConfig = await invoke<AppConfig>('save_device_scrcpy_options', { serial, options, presetId });
        delete this.sessionDraftOptions[serial];
        this.log(`已保存设备参数: ${serial}`);
      } catch (error) {
        this.log(`保存设备参数失败: ${String(error)}`);
        throw error;
      } finally {
        this.setBusy('settings', false);
      }
    },

    async clearDeviceOptions(serial: string) {
      this.setBusy('settings', true);
      try {
        this.appConfig = await invoke<AppConfig>('clear_device_scrcpy_options', { serial });
        delete this.deviceDraftOptions[serial];
        delete this.sessionDraftOptions[serial];
        this.log(`已恢复全局默认: ${serial}`);
      } catch (error) {
        this.log(`恢复全局默认失败: ${String(error)}`);
        throw error;
      } finally {
        this.setBusy('settings', false);
      }
    },

    async fetchToolStatus() {
      this.setBusy('tools', true);
      try {
        this.toolStatus = await invoke<ToolStatus>('get_tool_status');
      } catch (error) {
        this.log(`工具检查失败: ${String(error)}`);
      } finally {
        this.setBusy('tools', false);
      }
    },

    async refreshDevices() {
      this.setBusy('devices', true);
      try {
        this.devices = await invoke<Device[]>('list_devices');
        if (!this.selectedSerial && this.devices.length > 0) {
          this.selectedSerial = this.devices[0].serial;
        }
        if (this.selectedSerial && !this.devices.some((device) => device.serial === this.selectedSerial)) {
          this.selectedSerial = this.devices[0]?.serial ?? null;
        }
      } catch (error) {
        this.log(`刷新设备失败: ${String(error)}`);
      } finally {
        this.setBusy('devices', false);
      }
    },

    async refreshSessions() {
      this.setBusy('sessions', true);
      try {
        this.sessions = await invoke<SessionInfo[]>('list_sessions');
      } catch (error) {
        this.log(`刷新会话失败: ${String(error)}`);
      } finally {
        this.setBusy('sessions', false);
      }
    },

    async startMirror(serial: string, options?: ScrcpyOptions) {
      this.setBusy('start', true);
      try {
        const finalOptions = options ?? this.effectiveOptions(serial);
        const info = await invoke<SessionInfo>('start_scrcpy', { serial, options: finalOptions });
        await this.refreshSessions();
        this.currentPage = 'sessions';
        this.log(`已启动投屏: ${serial}`);
        return info;
      } catch (error) {
        this.log(`启动失败: ${String(error)}`);
        throw error;
      } finally {
        this.setBusy('start', false);
      }
    },

    async stopMirror(sessionId: string) {
      this.setBusy('stop', true);
      try {
        await invoke('stop_scrcpy', { sessionId });
        await this.refreshSessions();
        this.log(`已停止投屏: ${sessionId}`);
      } catch (error) {
        this.log(`停止失败: ${String(error)}`);
      } finally {
        this.setBusy('stop', false);
      }
    },

    async stopAllSessions() {
      this.setBusy('stop', true);
      try {
        this.sessions = await invoke<SessionInfo[]>('stop_all_sessions');
        this.log('已停止全部投屏会话');
      } catch (error) {
        this.log(`停止全部失败: ${String(error)}`);
      } finally {
        this.setBusy('stop', false);
      }
    },

    async adbTcpip(serial: string, port?: number) {
      this.setBusy('wireless', true);
      try {
        await invoke('adb_tcpip', { serial, port: port ?? null });
        this.log(`设备 ${serial} 已切换到无线模式`);
      } catch (error) {
        this.log(`切换无线失败: ${String(error)}`);
        throw error;
      } finally {
        this.setBusy('wireless', false);
      }
    },

    async adbConnect(endpoint: string) {
      this.setBusy('wireless', true);
      try {
        await invoke('adb_connect', { endpoint });
        this.log(`已连接无线设备: ${endpoint}`);
        await this.fetchAppConfig();
        await this.refreshDevices();
      } catch (error) {
        this.log(`无线连接失败: ${String(error)}`);
        throw error;
      } finally {
        this.setBusy('wireless', false);
      }
    },

    async adbPair(request: PairRequest) {
      this.setBusy('pair', true);
      try {
        await invoke('adb_pair', { request });
        this.log(`配对并连接成功: ${request.host}`);
        await this.fetchAppConfig();
        await this.refreshDevices();
      } catch (error) {
        this.log(`配对失败: ${String(error)}`);
        throw error;
      } finally {
        this.setBusy('pair', false);
      }
    },

    async fetchSessionLogs(sessionId: string) {
      try {
        const logs = await invoke<SessionLogLine[]>('get_session_logs', { sessionId });
        this.sessionLogs[sessionId] = logs;
      } catch (error) {
        this.log(`读取会话日志失败: ${String(error)}`);
      }
    },
  },
});
