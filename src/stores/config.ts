import { defineStore } from 'pinia';
import type { PresetId, ScrcpyOptions } from '../types/app';
import type { AppConfig } from '../lib/ipc/types';
import { defaultScrcpyOptions } from '../domain/scrcpyOptions';
import { invokeCommand } from '../lib/ipc/client';

export const useConfigStore = defineStore('config', {
  state: () => ({
    appConfig: null as AppConfig | null,
    globalDraftOptions: { ...defaultScrcpyOptions } as ScrcpyOptions,
    globalDraftPresetId: 'daily' as PresetId,
    deviceDraftOptions: {} as Record<string, ScrcpyOptions>,
  }),
  getters: {
    deviceOptionEntry: (state) => (serial: string) => state.appConfig?.device_scrcpy_options[serial] ?? null,
  },
  actions: {
    async fetchAppConfig() {
      this.appConfig = await invokeCommand<AppConfig>('get_app_config');
      this.globalDraftOptions = { ...(this.appConfig.default_scrcpy_options ?? defaultScrcpyOptions) };
      this.globalDraftPresetId = this.appConfig.default_preset_id ?? 'daily';
    },
    async saveDefaultOptions(options: ScrcpyOptions, presetId: PresetId) {
      this.appConfig = await invokeCommand<AppConfig>('save_default_scrcpy_options', { options, presetId });
      this.globalDraftOptions = { ...options };
      this.globalDraftPresetId = presetId;
    },
    async saveDeviceOptions(serial: string, options: ScrcpyOptions, presetId: PresetId | null) {
      this.appConfig = await invokeCommand<AppConfig>('save_device_scrcpy_options', { serial, options, presetId });
    },
    async clearDeviceOptions(serial: string) {
      this.appConfig = await invokeCommand<AppConfig>('clear_device_scrcpy_options', { serial });
      delete this.deviceDraftOptions[serial];
    },
    async setToolPaths(adbPath: string | null, scrcpyPath: string | null) {
      this.appConfig = await invokeCommand<AppConfig>('set_tool_paths', { adbPath, scrcpyPath });
    },
    async saveDeviceAlias(serial: string, alias: string | null) {
      this.appConfig = await invokeCommand<AppConfig>('save_device_alias', { serial, alias: alias ?? '' });
    },
  },
});
