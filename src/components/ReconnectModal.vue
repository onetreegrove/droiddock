<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue';
import { canAutoCloseReconnectModal, reconnectSuccessCloseDelayMs, reconnectSuccessMessage } from '../domain/reconnectFeedback';
import { buildConnectEndpoint, splitConnectEndpoint, wirelessSourceLabel } from '../domain/wireless';
import { errorUserMessage } from '../lib/ipc/errors';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const host = ref('');
const port = ref('');
const errorMessage = ref('');
const successMessage = ref('');
const isClosingAfterSuccess = ref(false);
const originalEndpoint = computed(() => ui.reconnectEndpoint);
const currentDevice = computed(() => store.devices.find((device) => device.serial === ui.reconnectDeviceSerial) ?? null);
let successCloseTimer: ReturnType<typeof window.setTimeout> | null = null;
const currentDeviceName = computed(() => {
  const device = currentDevice.value;
  return device?.alias || device?.display_name || device?.model || device?.serial || '当前无线设备';
});
const feedbackMessage = computed(() => {
  if (store.busy.wireless) return '正在重连，请稍候...';
  return successMessage.value || errorMessage.value;
});
const feedbackTone = computed(() => {
  if (store.busy.wireless) return 'info';
  if (successMessage.value) return 'success';
  if (errorMessage.value) return 'error';
  return 'empty';
});

watch(
  originalEndpoint,
  (endpoint) => {
    if (!endpoint) return;
    const parsed = splitConnectEndpoint(endpoint);
    host.value = parsed.host;
    port.value = parsed.port;
  },
  { immediate: true },
);

function clearSuccessCloseTimer() {
  if (!successCloseTimer) return;
  window.clearTimeout(successCloseTimer);
  successCloseTimer = null;
}

onUnmounted(() => {
  clearSuccessCloseTimer();
});

async function reconnect() {
  if (!originalEndpoint.value) {
    errorMessage.value = '缺少当前设备连接地址，请从无线设备卡片重新打开重连。';
    return;
  }

  clearSuccessCloseTimer();
  errorMessage.value = '';
  successMessage.value = '';
  isClosingAfterSuccess.value = false;
  try {
    const endpoint = buildConnectEndpoint(host.value, port.value);
    await store.adbConnect(endpoint, ui.reconnectSource, originalEndpoint.value);
    if (ui.reconnectLaunchAfterConnect) {
      await store.startMirror(endpoint);
    }
    successMessage.value = reconnectSuccessMessage(endpoint);
    isClosingAfterSuccess.value = true;
    successCloseTimer = window.setTimeout(() => {
      successCloseTimer = null;
      if (canAutoCloseReconnectModal(ui.modal)) {
        ui.closeModal();
      }
    }, reconnectSuccessCloseDelayMs);
  } catch (error) {
    clearSuccessCloseTimer();
    isClosingAfterSuccess.value = false;
    errorMessage.value = errorUserMessage(error);
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="!store.busy.wireless && !isClosingAfterSuccess && ui.closeModal()">
    <section class="modal-card">
      <header class="modal-header">
        <div>
          <div class="modal-title">重连无线设备</div>
          <div class="modal-subtitle">修改当前设备的无线调试地址后重新连接</div>
        </div>
        <button class="modal-close" :disabled="store.busy.wireless || isClosingAfterSuccess" aria-label="关闭" @click="ui.closeModal()">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M2 2l8 8M10 2L2 10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </button>
      </header>
      <div class="modal-body">
        <div class="modal-section-title">当前设备</div>
        <div class="modal-device-summary">
          <span>{{ currentDeviceName }}</span>
          <span class="chip chip-gray">{{ wirelessSourceLabel(currentDevice?.wireless_source || ui.reconnectSource) }}</span>
          <span class="mono">{{ originalEndpoint || '-' }}</span>
        </div>

        <div class="modal-section-title">连接地址</div>
        <div class="modal-note">手机重启或无线调试重新开启后，连接端口可能变化，请以手机“无线调试”页面显示的 IP 和端口为准。</div>
        <div class="form-grid form-grid-ip">
          <label>设备 IP<input v-model="host" class="field-input" placeholder="192.168.1.100" /></label>
          <label>连接端口<input v-model="port" class="field-input" placeholder="39845" /></label>
        </div>
        <div class="modal-feedback-slot" aria-live="polite">
          <div v-if="feedbackMessage" :class="['modal-feedback', `modal-feedback-${feedbackTone}`]">{{ feedbackMessage }}</div>
        </div>
      </div>
      <footer class="modal-footer">
        <button class="btn btn-ghost" :disabled="store.busy.wireless || isClosingAfterSuccess" @click="ui.closeModal()">取消</button>
        <button class="btn btn-primary" :disabled="!originalEndpoint || !host || !port || store.busy.wireless || isClosingAfterSuccess" @click="reconnect">
          {{ store.busy.wireless ? '重连中...' : isClosingAfterSuccess ? '已连接' : ui.reconnectLaunchAfterConnect ? '重连并投屏' : '重连' }}
        </button>
      </footer>
    </section>
  </div>
</template>
