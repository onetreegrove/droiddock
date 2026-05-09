import { defineStore } from 'pinia';
import type { ToolInstallResult, ToolStatus } from '../lib/ipc/types';
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
  },
});
