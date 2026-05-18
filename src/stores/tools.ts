import { defineStore } from 'pinia';
import type { AppConfig, ToolInstallResult, ToolKind, ToolStatus } from '../lib/ipc/types';
import { invokeCommand } from '../lib/ipc/client';

export const useToolsStore = defineStore('tools', {
  state: () => ({
    toolStatus: null as ToolStatus | null,
  }),
  getters: {
    isToolsReady: (state) => Boolean(state.toolStatus?.adb_ok && state.toolStatus?.scrcpy_ok),
  },
  actions: {
    async fetchToolStatus() {
      this.toolStatus = await invokeCommand<ToolStatus>('get_tool_status');
    },
    async installTools() {
      return await invokeCommand<ToolInstallResult>('install_tools');
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
