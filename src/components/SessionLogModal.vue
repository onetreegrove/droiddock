<script setup lang="ts">
import LogPanel from './LogPanel.vue';
import type { SessionInfo, SessionLogLine } from '../types/app';

defineProps<{
  session: SessionInfo;
  lines: SessionLogLine[];
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <section class="modal-card log-modal">
      <header class="modal-header">
        <div>
          <div class="modal-title">投屏日志</div>
          <div class="modal-subtitle">{{ session.alias || session.serial }} · {{ session.session_id }}</div>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" @click="emit('close')">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M2 2l8 8M10 2L2 10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </button>
      </header>
      <div class="modal-body log-modal-body">
        <LogPanel :lines="lines" />
      </div>
    </section>
  </div>
</template>
