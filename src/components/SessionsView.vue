<script setup lang="ts">
import { computed } from 'vue';
import AppHeader from './AppHeader.vue';
import SessionCard from './SessionCard.vue';
import SessionLogModal from './SessionLogModal.vue';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const runningCount = computed(() => store.sessions.filter((session) => session.status === 'running').length);
const selectedLogSession = computed(() => store.sessions.find((session) => session.session_id === ui.selectedLogSessionId) ?? null);

function closeLogModal() {
  ui.selectedLogSessionId = null;
}
</script>

<template>
  <section class="page">
    <AppHeader title="投屏总览" :subtitle="`${store.sessions.length} 个会话 · ${runningCount} 个运行中`">
      <template #actions>
        <button class="btn btn-danger" :disabled="runningCount === 0" @click="store.stopAllSessions">停止全部</button>
        <button class="btn btn-ghost" @click="ui.openPage('devices')">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M3 2L10 6L3 10V2Z" fill="currentColor" />
          </svg>
          新建投屏
        </button>
      </template>
    </AppHeader>
    <div class="session-list">
      <div v-for="session in store.sessions" :key="session.session_id" class="session-stack">
        <SessionCard :session="session" show-device-action />
      </div>
      <div v-if="store.sessions.length === 0" class="empty-panel">暂无投屏会话，请到设备页选择设备并启动投屏。</div>
    </div>
    <SessionLogModal
      v-if="selectedLogSession"
      :session="selectedLogSession"
      :lines="store.sessionLogs[selectedLogSession.session_id] ?? []"
      @close="closeLogModal"
    />
  </section>
</template>
