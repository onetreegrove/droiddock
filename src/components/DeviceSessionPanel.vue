<script setup lang="ts">
import { computed, watch } from 'vue';
import LogPanel from './LogPanel.vue';
import StatusChip from './StatusChip.vue';
import { optionSummaryTagsFromArgs } from '../domain/scrcpyOptions';
import type { ManagedDevice, SessionInfo } from '../types/app';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const props = defineProps<{
  device: ManagedDevice;
  session: SessionInfo | null;
}>();

const store = useAppStore();
const ui = useUiStore();

const running = computed(() => props.session?.status === 'running');
const failed = computed(() => props.session?.status === 'failed');
const unauthorized = computed(() => props.device.state === 'unauthorized');
const tags = computed(() => (props.session ? optionSummaryTagsFromArgs(props.session.args) : []));
const title = computed(() => {
  if (!props.session) return '当前未投屏';
  if (running.value) return '当前会话';
  if (failed.value) return '最近投屏失败';
  return '最近投屏已停止';
});
const statusLabel = computed(() => {
  if (!props.session) return '未投屏';
  if (running.value) return '投屏中';
  if (failed.value) return '失败';
  return '已停止';
});
const statusTone = computed(() => {
  if (running.value) return 'green';
  if (failed.value) return 'red';
  return 'gray';
});
const emptyMessage = computed(() => {
  if (unauthorized.value) return '请在手机上允许 USB 调试授权，授权后再启动投屏。';
  return '这台设备当前没有投屏会话。调整参数后可直接启动投屏。';
});

watch(
  () => props.session,
  async (session) => {
    if (session?.status === 'failed' && ui.selectedLogSessionId !== session.session_id) {
      await store.openSessionLogs(session.session_id);
    }
  },
  { immediate: true },
);

async function relaunch() {
  if (props.device.connection === 'wireless' && props.device.presence === 'offline' && props.device.endpoint) {
    ui.openReconnectModal(props.device.serial, props.device.endpoint, props.device.wireless_source, true);
    return;
  }
  await store.startMirror(props.device.serial);
}
</script>

<template>
  <section class="detail-section device-session-panel">
    <div class="section-head">
      <div>
        <div class="section-title">{{ title }}</div>
        <StatusChip :tone="statusTone" :label="statusLabel" dot />
      </div>
      <div v-if="session" class="inline-actions">
        <button
          v-if="running"
          class="btn btn-danger compact-button"
          :disabled="store.busy.stop"
          @click="store.stopMirror(session.session_id)"
        >
          停止投屏
        </button>
        <button
          v-else
          class="btn btn-ghost compact-button"
          :disabled="store.busy.start || (device.presence === 'offline' && device.connection === 'usb')"
          @click="relaunch"
        >
          {{ device.connection === 'wireless' && device.presence === 'offline' ? '重连投屏' : '重新投屏' }}
        </button>
        <button class="btn btn-ghost compact-button" @click="store.openSessionLogs(session.session_id)">
          {{ ui.selectedLogSessionId === session.session_id ? '收起日志' : '查看日志' }}
        </button>
      </div>
    </div>

    <div v-if="session" class="device-session-body">
      <div class="metadata-grid compact">
        <span>Serial</span><span class="mono">{{ session.serial }}</span>
        <span>PID</span><span class="mono">{{ session.pid }}</span>
        <span>连接</span><span>{{ session.connection === 'usb' ? 'USB' : '无线' }}</span>
      </div>
      <div v-if="session.last_message" :class="['session-message', { error: failed }]">
        {{ session.last_message }}
      </div>
      <div class="tag-row">
        <span v-for="tag in tags" :key="tag" class="param-tag mono">{{ tag }}</span>
      </div>
      <LogPanel
        v-if="ui.selectedLogSessionId === session.session_id"
        :lines="store.sessionLogs[session.session_id] ?? []"
      />
    </div>

    <div v-else class="empty-panel compact-empty">
      {{ emptyMessage }}
    </div>
  </section>
</template>
