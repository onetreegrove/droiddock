import { defineStore } from 'pinia';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppConfig,
  ToolInstallProgress,
  ToolInstallResult,
  ToolInstallStatus,
  ToolInstallTarget,
  ToolKind,
  ToolStatus,
} from '../lib/ipc/types';
import { invokeCommand } from '../lib/ipc/client';
import { errorUserMessage } from '../lib/ipc/errors';

let installProgressUnlisten: UnlistenFn | null = null;

export const useToolsStore = defineStore('tools', {
  state: () => ({
    toolStatus: null as ToolStatus | null,
    installLogs: [] as string[],
    installTarget: 'all' as ToolInstallTarget,
    installStatus: 'idle' as ToolInstallStatus,
    installError: null as string | null,
  }),
  getters: {
    isToolsReady: (state) => Boolean(state.toolStatus?.adb_ok && state.toolStatus?.scrcpy_ok),
  },
  actions: {
    async fetchToolStatus() {
      this.toolStatus = await invokeCommand<ToolStatus>('get_tool_status');
    },
    resetInstallLogs(target: ToolInstallTarget) {
      this.installTarget = target;
      this.installLogs = [];
      this.installStatus = 'running';
      this.installError = null;
    },
    appendInstallLog(message: string) {
      if (this.installLogs[this.installLogs.length - 1] === message) return;
      this.installLogs = [...this.installLogs, message].slice(-300);
    },
    clearInstallLogs() {
      this.installLogs = [];
      this.installStatus = 'idle';
      this.installError = null;
    },
    markInstallFailed(message: string) {
      this.installStatus = 'failed';
      this.installError = message;
      this.appendInstallLog(message);
    },
    async installTools(target: ToolInstallTarget) {
      this.resetInstallLogs(target);
      try {
        const result = await invokeCommand<ToolInstallResult>('install_tools', { target });
        this.installLogs = result.logs;
        this.installStatus = 'success';
        this.installError = null;
        return result;
      } catch (error) {
        const message = errorUserMessage(error);
        this.markInstallFailed(message);
        throw error;
      }
    },
    async listenToolInstallProgress(): Promise<UnlistenFn> {
      if (installProgressUnlisten) {
        return () => {};
      }

      installProgressUnlisten = await listen<ToolInstallProgress>('tool-install-progress', (event) => {
        this.installTarget = event.payload.target;
        this.appendInstallLog(event.payload.message);
        if (event.payload.level === 'error') {
          this.markInstallFailed(event.payload.message);
        }
      });
      return () => {
        installProgressUnlisten?.();
        installProgressUnlisten = null;
      };
    },
    async setToolPath(tool: ToolKind, path: string) {
      const appConfig = await invokeCommand<AppConfig>('set_tool_path', { tool, path });
      await this.fetchToolStatus();
      return appConfig;
    },
    async clearToolPath(tool: ToolKind) {
      const appConfig = await invokeCommand<AppConfig>('clear_tool_path', { tool });
      await this.fetchToolStatus();
      return appConfig;
    },
  },
});
