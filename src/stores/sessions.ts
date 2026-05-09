import { defineStore } from 'pinia';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { ScrcpyOptions } from '../types/app';
import type { SessionInfo, SessionLogLine } from '../lib/ipc/types';
import { invokeCommand } from '../lib/ipc/client';

type SessionLogEvent = {
  session_id: string;
  line: SessionLogLine;
};

export const useSessionsStore = defineStore('sessions', {
  state: () => ({
    sessions: [] as SessionInfo[],
    sessionLogs: {} as Record<string, SessionLogLine[]>,
    sessionDraftOptions: {} as Record<string, ScrcpyOptions>,
  }),
  getters: {
    activeSession: (state) => (serial: string) =>
      state.sessions.find((session) => session.serial === serial && session.status === 'running') ?? null,
  },
  actions: {
    async refreshSessions() {
      this.sessions = await invokeCommand<SessionInfo[]>('list_sessions');
    },
    async startMirror(serial: string, options: ScrcpyOptions) {
      const info = await invokeCommand<SessionInfo>('start_scrcpy', { serial, options });
      await this.refreshSessions();
      return info;
    },
    async stopMirror(sessionId: string) {
      await invokeCommand('stop_scrcpy', { sessionId });
      await this.refreshSessions();
    },
    async stopAllSessions() {
      this.sessions = await invokeCommand<SessionInfo[]>('stop_all_sessions');
    },
    async fetchSessionLogs(sessionId: string) {
      this.sessionLogs[sessionId] = await invokeCommand<SessionLogLine[]>('get_session_logs', { sessionId });
    },
    async listenSessionLogs(): Promise<UnlistenFn> {
      return await listen<SessionLogEvent>('session-log', (event) => {
        const logs = this.sessionLogs[event.payload.session_id] ?? [];
        this.sessionLogs[event.payload.session_id] = [...logs, event.payload.line].slice(-400);
      });
    },
  },
});
