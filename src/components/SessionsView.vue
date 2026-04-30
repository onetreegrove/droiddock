<script setup lang="ts">
import { computed } from 'vue';
import AppHeader from './AppHeader.vue';
import LogPanel from './LogPanel.vue';
import SessionCard from './SessionCard.vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const runningCount = computed(() => store.sessions.filter((session) => session.status === 'running').length);
</script>

<template>
  <section class="page">
    <AppHeader title="投屏会话" :subtitle="`${store.sessions.length} 个会话 · ${runningCount} 个运行中`">
      <template #actions>
        <button class="btn btn-danger" :disabled="runningCount === 0" @click="store.stopAllSessions">停止全部</button>
        <button class="btn btn-ghost" @click="store.currentPage = 'devices'">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M3 2L10 6L3 10V2Z" fill="currentColor" />
          </svg>
          新建投屏
        </button>
      </template>
    </AppHeader>
    <div class="session-list">
      <div v-for="session in store.sessions" :key="session.session_id" class="session-stack">
        <SessionCard :session="session" />
        <LogPanel
          v-if="store.selectedLogSessionId === session.session_id"
          :lines="store.sessionLogs[session.session_id] ?? []"
        />
      </div>
      <div v-if="store.sessions.length === 0" class="empty-panel">暂无投屏会话，请到设备页启动投屏。</div>
    </div>
  </section>
</template>
