<script setup lang="ts">
import { computed } from 'vue';
import { optionSummaryTags } from '../domain/scrcpyOptions';
import type { SessionInfo } from '../types/app';
import { useAppStore } from '../stores/app';

const props = defineProps<{ session: SessionInfo }>();
const store = useAppStore();
const tags = computed(() => optionSummaryTags(store.effectiveOptions(props.session.serial)));
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
        <button v-if="running" class="btn btn-danger" @click="store.stopMirror(session.session_id)">停止</button>
        <button v-else class="btn btn-ghost" @click="store.startMirror(session.serial)">重连</button>
        <button class="btn btn-ghost" @click="store.fetchSessionLogs(session.session_id)">日志</button>
      </div>
    </div>
  </article>
</template>
