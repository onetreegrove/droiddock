<script setup lang="ts">
import { computed } from 'vue';
import { optionSummaryTagsFromArgs } from '../domain/scrcpyOptions';
import type { SessionInfo } from '../types/app';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const props = defineProps<{ session: SessionInfo }>();
const store = useAppStore();
const ui = useUiStore();
const tags = computed(() => optionSummaryTagsFromArgs(props.session.args));
const running = computed(() => props.session.status === 'running');
</script>

<template>
  <article :class="['session-card', session.status]">
    <div :class="['session-strip', session.status]"></div>
    <div class="session-body">
      <div class="session-main">
        <div class="session-title-row">
          <span class="session-title">{{ session.alias || session.serial }}</span>
          <span :class="['chip', running ? 'chip-green' : session.status === 'failed' ? 'chip-red' : 'chip-gray']">
            {{ running ? '运行中' : session.status === 'failed' ? '失败' : '已停止' }}
          </span>
          <span class="chip chip-gray">{{ session.connection === 'usb' ? 'USB' : '无线' }}</span>
        </div>
        <div class="session-meta mono">{{ session.serial }} · PID {{ session.pid }}</div>
        <div v-if="session.last_message" class="session-message">{{ session.last_message }}</div>
        <div class="tag-row"><span v-for="tag in tags" :key="tag" class="param-tag mono">{{ tag }}</span></div>
      </div>
      <div class="session-actions">
        <button v-if="running" class="btn btn-danger compact-button" @click="store.stopMirror(session.session_id)">
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <rect x="2" y="2" width="6" height="6" rx=".8" fill="currentColor" />
          </svg>
          停止
        </button>
        <button v-else class="btn btn-ghost compact-button" @click="store.startMirror(session.serial)">
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
            <path d="M1.5 5A3.5 3.5 0 1 1 5 8.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
            <path d="M1.5 3v2h2" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          重连
        </button>
        <button class="btn btn-ghost icon-button" :aria-label="ui.selectedLogSessionId === session.session_id ? '收起日志' : '日志'" @click="store.openSessionLogs(session.session_id)">
          <svg width="13" height="13" viewBox="0 0 13 13" fill="none" aria-hidden="true">
            <rect x="1.5" y="1.5" width="10" height="10" rx="1.5" stroke="currentColor" stroke-width="1.1" />
            <path d="M4 5h5M4 7h4" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    </div>
    <div v-if="running" class="runbar"><div class="runfill"></div></div>
  </article>
</template>
