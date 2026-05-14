<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import SessionLogModal from './SessionLogModal.vue';
import StatusChip from './StatusChip.vue';
import { canRestoreConnection, type LaunchAvailability } from '../domain/deviceDetail';
import { optionSummaryTagsFromArgs } from '../domain/scrcpyOptions';
import type { ManagedDevice, SessionInfo } from '../types/app';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const props = defineProps<{
  device: ManagedDevice;
  session: SessionInfo | null;
  launchState: LaunchAvailability;
}>();

const emit = defineEmits<{
  launch: [];
}>();

const store = useAppStore();
const ui = useUiStore();
const logModalOpen = ref(false);

const running = computed(() => props.session?.status === 'running');
const failed = computed(() => props.session?.status === 'failed');
const unauthorized = computed(() => props.device.state === 'unauthorized');
const tags = computed(() => (props.session ? optionSummaryTagsFromArgs(props.session.args) : []));
const restoreAvailable = computed(() => canRestoreConnection(props.device, store.isToolsReady));
const disconnected = computed(() => props.device.presence === 'offline' || props.device.state === 'offline');
const primaryAction = computed<'restore' | 'launch' | 'stop' | null>(() => {
  if (running.value) return 'stop';
  if (disconnected.value) return restoreAvailable.value ? 'restore' : null;
  if (props.device.presence === 'online' && props.device.state === 'device' && props.launchState.canLaunch) return 'launch';
  return null;
});
const primaryDisabled = computed(() => {
  if (primaryAction.value === 'stop') return store.busy.stop;
  if (primaryAction.value === 'launch') return store.busy.start || !props.launchState.canLaunch;
  return primaryAction.value === null;
});
const title = computed(() => {
  if (!running.value && disconnected.value) return '会话';
  if (!props.session && props.device.presence === 'online') return '会话';
  if (!props.session) return '会话';
  if (running.value) return '当前会话';
  if (failed.value) return '最近投屏失败';
  return '会话';
});
const statusLabel = computed(() => {
  if (!running.value && disconnected.value) return '未连接';
  if (unauthorized.value) return '待授权';
  if (!props.session && props.device.presence === 'online') return '已连接';
  if (!props.session) return '未投屏';
  if (running.value) return '投屏中';
  if (failed.value) return '失败';
  return '已连接';
});
const statusTone = computed(() => {
  if (running.value) return 'green';
  if (disconnected.value) return restoreAvailable.value ? 'gray' : 'red';
  if (unauthorized.value || failed.value) return 'red';
  if (!props.launchState.canLaunch) return 'gray';
  return 'gray';
});
const panelMessage = computed(() => {
  if (running.value) return 'scrcpy 投屏窗口正在运行，可在这里停止投屏或查看日志。';
  if (disconnected.value) {
    if (restoreAvailable.value) return '设备当前不在线，可先恢复无线连接。恢复后再按需要启动投屏。';
    return props.device.connection === 'wireless'
      ? '缺少可用的无线连接地址，请重新配对或通过 USB 转无线。'
      : '请重新插入 USB 数据线，设备列表会自动刷新。';
  }
  if (unauthorized.value) return '请在手机上允许 USB 调试授权，授权后再启动投屏。';
  if (!store.isToolsReady) return '请先完成 adb 和 scrcpy 工具配置。';
  if (failed.value) return '上次投屏失败。确认设备正常后，可使用当前参数重新投屏。';
  if (props.session) return '投屏已停止，可使用当前参数重新投屏。';
  return '设备已连接，确认下方参数后可直接启动投屏。';
});
const primaryLabel = computed(() => {
  if (primaryAction.value === 'stop') return '停止投屏';
  if (primaryAction.value === 'restore') return '恢复连接';
  if (primaryAction.value === 'launch') {
    if (props.session?.status === 'failed' || props.session?.status === 'stopped') return '重新投屏';
    return '启动投屏';
  }
  return props.launchState.buttonText;
});
const primaryTitle = computed(() => {
  if (primaryAction.value === 'restore') return '打开重连窗口，只恢复设备连接';
  if (primaryAction.value === 'launch') return props.launchState.hint || '使用当前参数启动投屏';
  if (primaryAction.value === 'stop') return '停止当前投屏会话';
  return props.launchState.hint || '当前设备不可用，请先恢复连接或授权';
});
const actionCardTitle = computed(() => {
  if (primaryAction.value === 'stop') return '正在投屏';
  if (primaryAction.value === 'restore') return '先恢复设备连接';
  if (primaryAction.value === 'launch') {
    if (props.session?.status === 'failed') return '可以重新投屏';
    if (props.session?.status === 'stopped') return '可以再次投屏';
    return '可以开始投屏';
  }
  return '暂时不能投屏';
});

watch(
  () => props.session,
  async (session) => {
    logModalOpen.value = false;
    if (session?.status === 'failed') {
      await store.fetchSessionLogs(session.session_id);
    }
  },
  { immediate: true },
);

async function openLogModal() {
  if (!props.session) return;
  await store.fetchSessionLogs(props.session.session_id);
  ui.selectedLogSessionId = props.session.session_id;
  logModalOpen.value = true;
}

function closeLogModal() {
  if (props.session && ui.selectedLogSessionId === props.session.session_id) {
    ui.selectedLogSessionId = null;
  }
  logModalOpen.value = false;
}

async function runPrimaryAction() {
  if (primaryAction.value === 'stop' && props.session) {
    await store.stopMirror(props.session.session_id);
    return;
  }
  if (primaryAction.value === 'restore' && props.device.endpoint) {
    ui.openReconnectModal(props.device.serial, props.device.endpoint, props.device.wireless_source, false);
    return;
  }
  if (primaryAction.value === 'launch') {
    emit('launch');
  }
}
</script>

<template>
  <section class="detail-section device-session-panel">
    <div class="section-head">
      <div>
        <div class="section-title">{{ title }}</div>
      </div>
      <StatusChip :tone="statusTone" :label="statusLabel" dot />
    </div>

    <div v-if="session" class="device-session-body">
      <div class="session-action-card">
        <div>
          <div class="session-action-title">{{ actionCardTitle }}</div>
          <div class="session-action-copy">{{ panelMessage }}</div>
        </div>
        <div class="session-card-actions">
          <button
            :class="['btn', primaryAction === 'stop' ? 'btn-danger' : primaryAction === 'launch' || primaryAction === 'restore' ? 'btn-primary' : 'btn-ghost', 'compact-button']"
            :disabled="primaryDisabled"
            :title="primaryTitle"
            @click="runPrimaryAction"
          >
            <svg v-if="primaryAction === 'launch'" width="12" height="12" viewBox="0 0 13 13" fill="none" aria-hidden="true">
              <path d="M3 2L11 6.5L3 11V2Z" fill="currentColor" />
            </svg>
            {{ primaryLabel }}
          </button>
          <button class="btn btn-ghost compact-button" @click="openLogModal">
            查看日志
          </button>
        </div>
      </div>
      <div class="session-detail-meta">
        <div class="session-detail-item">
          <span>Serial</span>
          <strong class="mono">{{ session.serial }}</strong>
        </div>
        <div class="session-detail-item compact">
          <span>PID</span>
          <strong class="mono">{{ session.pid }}</strong>
        </div>
        <div class="session-detail-item compact">
          <span>连接</span>
          <strong>{{ session.connection === 'usb' ? 'USB' : '无线' }}</strong>
        </div>
      </div>
      <div v-if="session.last_message" :class="['session-message', { error: failed }]">
        {{ session.last_message }}
      </div>
      <div class="tag-row">
        <span v-for="tag in tags" :key="tag" class="param-tag mono">{{ tag }}</span>
      </div>
    </div>

    <div v-else class="session-action-card">
      <div>
        <div class="session-action-title">{{ actionCardTitle }}</div>
        <div class="session-action-copy">{{ panelMessage }}</div>
      </div>
      <div class="session-card-actions">
        <button
          :class="['btn', primaryAction === 'launch' || primaryAction === 'restore' ? 'btn-primary' : 'btn-ghost', 'compact-button']"
          :disabled="primaryDisabled"
          :title="primaryTitle"
          @click="runPrimaryAction"
        >
          <svg v-if="primaryAction === 'launch'" width="12" height="12" viewBox="0 0 13 13" fill="none" aria-hidden="true">
            <path d="M3 2L11 6.5L3 11V2Z" fill="currentColor" />
          </svg>
          {{ primaryLabel }}
        </button>
      </div>
    </div>

    <SessionLogModal
      v-if="session && logModalOpen"
      :session="session"
      :lines="store.sessionLogs[session.session_id] ?? []"
      @close="closeLogModal"
    />
  </section>
</template>
