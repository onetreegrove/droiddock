import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import type { PresetId, ScrcpyOptions, WirelessSource } from '../types/app';
import type { PairRequest } from '../lib/ipc/types';
import { defaultScrcpyOptions, mergeScrcpyOptions } from '../domain/scrcpyOptions';
import type { AppErrorPayload } from '../lib/ipc/errors';
import { invokeCommand } from '../lib/ipc/client';
import { useConfigStore } from './config';
import { useDevicesStore } from './devices';
import { useSessionsStore } from './sessions';
import { useToolsStore } from './tools';
import { useUiStore } from './ui';

type BusyKey =
  | 'config'
  | 'tools'
  | 'install'
  | 'devices'
  | 'sessions'
  | 'start'
  | 'stop'
  | 'pair'
  | 'wireless'
  | 'settings';

function errorMessage(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'userMessage' in error) {
    return String((error as AppErrorPayload).userMessage);
  }
  return String(error);
}

export const useAppStore = defineStore('app', () => {
  const config = useConfigStore();
  const devicesStore = useDevicesStore();
  const sessionsStore = useSessionsStore();
  const tools = useToolsStore();
  const ui = useUiStore();

  const busy = ref({} as Record<BusyKey, boolean>);
  const logs = ref<string[]>([]);
  const loading = computed(() => Object.values(busy.value).some(Boolean));
  let refreshInFlight = false;

  const toolStatus = computed(() => tools.toolStatus);
  const appConfig = computed(() => config.appConfig);
  const devices = computed(() => devicesStore.devices);
  const sessions = computed(() => sessionsStore.sessions);
  const sessionLogs = computed(() => sessionsStore.sessionLogs);
  const globalDraftOptions = computed({
    get: () => config.globalDraftOptions,
    set: (value: ScrcpyOptions) => {
      config.globalDraftOptions = value;
    },
  });
  const globalDraftPresetId = computed({
    get: () => config.globalDraftPresetId,
    set: (value: PresetId) => {
      config.globalDraftPresetId = value;
    },
  });
  const deviceDraftOptions = computed(() => config.deviceDraftOptions);
  const sessionDraftOptions = computed(() => sessionsStore.sessionDraftOptions);

  const selectedDevice = computed(() => devicesStore.selectedDevice);
  const isToolsReady = computed(() => tools.isToolsReady);
  const availableDeviceCount = computed(() => devicesStore.availableDeviceCount);
  const activeSession = (serial: string) => sessionsStore.activeSession(serial);
  const deviceOptionEntry = (serial: string) => config.deviceOptionEntry(serial);
  const effectiveOptions = (serial: string) => {
    const globalOptions = config.appConfig?.default_scrcpy_options ?? defaultScrcpyOptions;
    const deviceOptions = config.appConfig?.device_scrcpy_options[serial]?.options;
    const sessionOptions = sessionsStore.sessionDraftOptions[serial];
    return mergeScrcpyOptions(globalOptions, deviceOptions, sessionOptions);
  };

  function log(message: string) {
    const time = new Date().toLocaleTimeString();
    logs.value.unshift(`[${time}] ${message}`);
    logs.value = logs.value.slice(0, 200);
  }

  function setBusy(key: BusyKey, value: boolean) {
    busy.value[key] = value;
  }

  async function fetchAppConfig() {
    setBusy('config', true);
    try {
      await config.fetchAppConfig();
    } catch (error) {
      log(`读取配置失败: ${errorMessage(error)}`);
    } finally {
      setBusy('config', false);
    }
  }

  async function saveDefaultOptions(options: ScrcpyOptions, presetId: PresetId) {
    setBusy('settings', true);
    try {
      await config.saveDefaultOptions(options, presetId);
      log('已保存全局默认参数');
    } catch (error) {
      log(`保存全局默认参数失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('settings', false);
    }
  }

  async function saveDeviceOptions(serial: string, options: ScrcpyOptions, presetId: PresetId | null) {
    setBusy('settings', true);
    try {
      await config.saveDeviceOptions(serial, options, presetId);
      delete sessionsStore.sessionDraftOptions[serial];
      log(`已保存设备参数: ${serial}`);
    } catch (error) {
      log(`保存设备参数失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('settings', false);
    }
  }

  async function clearDeviceOptions(serial: string) {
    setBusy('settings', true);
    try {
      await config.clearDeviceOptions(serial);
      delete sessionsStore.sessionDraftOptions[serial];
      log(`已恢复全局默认: ${serial}`);
    } catch (error) {
      log(`恢复全局默认失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('settings', false);
    }
  }

  async function fetchToolStatus() {
    setBusy('tools', true);
    try {
      await tools.fetchToolStatus();
    } catch (error) {
      log(`工具检查失败: ${errorMessage(error)}`);
    } finally {
      setBusy('tools', false);
    }
  }

  async function setToolPath(tool: 'adb' | 'scrcpy', path: string) {
    setBusy('tools', true);
    try {
      const adbPath = tool === 'adb' ? path : (config.appConfig?.adb_path ?? tools.toolStatus?.adb_path ?? null);
      const scrcpyPath = tool === 'scrcpy' ? path : (config.appConfig?.scrcpy_path ?? tools.toolStatus?.scrcpy_path ?? null);
      await config.setToolPaths(adbPath, scrcpyPath);
      await fetchToolStatus();
      log(`已更新 ${tool} 路径`);
    } catch (error) {
      log(`更新工具路径失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('tools', false);
    }
  }

  async function installTools() {
    setBusy('install', true);
    try {
      const result = await tools.installTools();
      log(result.logs.join(' / '));
      await fetchAppConfig();
      await fetchToolStatus();
      return result;
    } catch (error) {
      log(`自动安装失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('install', false);
    }
  }

  async function refreshDevices() {
    setBusy('devices', true);
    try {
      await devicesStore.refreshDevices();
    } catch (error) {
      log(`刷新设备失败: ${errorMessage(error)}`);
    } finally {
      setBusy('devices', false);
    }
  }

  async function refreshSessions() {
    setBusy('sessions', true);
    try {
      await sessionsStore.refreshSessions();
    } catch (error) {
      log(`刷新会话失败: ${errorMessage(error)}`);
    } finally {
      setBusy('sessions', false);
    }
  }

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

  async function startMirror(serial: string, options?: ScrcpyOptions) {
    setBusy('start', true);
    try {
      const finalOptions = options ?? effectiveOptions(serial);
      const info = await sessionsStore.startMirror(serial, finalOptions);
      ui.openPage('sessions');
      log(`已启动投屏: ${serial}`);
      return info;
    } catch (error) {
      log(`启动失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('start', false);
    }
  }

  async function stopMirror(sessionId: string) {
    setBusy('stop', true);
    try {
      await sessionsStore.stopMirror(sessionId);
      log(`已停止投屏: ${sessionId}`);
    } catch (error) {
      log(`停止失败: ${errorMessage(error)}`);
    } finally {
      setBusy('stop', false);
    }
  }

  async function stopAllSessions() {
    setBusy('stop', true);
    try {
      await sessionsStore.stopAllSessions();
      log('已停止全部投屏会话');
    } catch (error) {
      log(`停止全部失败: ${errorMessage(error)}`);
    } finally {
      setBusy('stop', false);
    }
  }

  async function saveDeviceAlias(serial: string, alias: string | null) {
    try {
      await config.saveDeviceAlias(serial, alias);
      await refreshDevices();
      log(`已更新设备别名: ${serial} -> ${alias || '恢复默认'}`);
    } catch (error) {
      log(`保存别名失败: ${errorMessage(error)}`);
      throw error;
    }
  }

  async function adbTcpip(serial: string, port?: number) {
    setBusy('wireless', true);
    try {
      await invokeCommand('adb_tcpip', { serial, port: port ?? null });
      log(`设备 ${serial} 已切换到无线模式`);
    } catch (error) {
      log(`切换无线失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('wireless', false);
    }
  }

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

  async function adbPair(request: PairRequest) {
    setBusy('pair', true);
    try {
      await invokeCommand('adb_pair', { request });
      log(`配对并连接成功: ${request.host}`);
      await fetchAppConfig();
      await refreshDevices();
    } catch (error) {
      log(`配对失败: ${errorMessage(error)}`);
      throw error;
    } finally {
      setBusy('pair', false);
    }
  }

  async function fetchSessionLogs(sessionId: string) {
    try {
      await sessionsStore.fetchSessionLogs(sessionId);
    } catch (error) {
      log(`读取会话日志失败: ${errorMessage(error)}`);
    }
  }

  async function openSessionLogs(sessionId: string) {
    ui.toggleLogSession(sessionId);
    if (ui.selectedLogSessionId) {
      await fetchSessionLogs(sessionId);
    }
  }

  async function listenSessionLogs() {
    return await sessionsStore.listenSessionLogs();
  }

  return {
    toolStatus,
    appConfig,
    devices,
    sessions,
    loading,
    busy,
    logs,
    sessionLogs,
    globalDraftOptions,
    globalDraftPresetId,
    deviceDraftOptions,
    sessionDraftOptions,
    selectedDevice,
    isToolsReady,
    availableDeviceCount,
    activeSession,
    deviceOptionEntry,
    effectiveOptions,
    log,
    setBusy,
    fetchAppConfig,
    saveDefaultOptions,
    saveDeviceOptions,
    clearDeviceOptions,
    fetchToolStatus,
    setToolPath,
    installTools,
    refreshDevices,
    refreshSessions,
    refreshRuntimeState,
    startMirror,
    stopMirror,
    stopAllSessions,
    saveDeviceAlias,
    adbTcpip,
    adbConnect,
    adbPair,
    fetchSessionLogs,
    openSessionLogs,
    listenSessionLogs,
  };
});
