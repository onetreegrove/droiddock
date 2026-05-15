<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { reconnectSuccessMessage } from '../domain/reconnectFeedback';
import { buildConnectEndpoint, splitConnectEndpoint, wirelessSourceLabel } from '../domain/wireless';
import { errorUserMessage } from '../lib/ipc/errors';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const host = ref('');
const port = ref('');
const errorMessage = ref('');
const portInput = ref<HTMLInputElement | null>(null);

const originalEndpoint = computed(() => ui.reconnectEndpoint);
const currentDevice = computed(() => store.devices.find((device) => device.serial === ui.reconnectDeviceSerial) ?? null);

const currentDeviceName = computed(() => {
  const device = currentDevice.value;
  return device?.alias || device?.display_name || device?.model || device?.serial || '当前无线设备';
});

const feedbackMessage = computed(() => {
  if (store.busy.wireless) return '正在重连，请稍候...';
  return errorMessage.value;
});

const feedbackTone = computed(() => {
  if (store.busy.wireless) return 'info';
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

// 输入时自动清除错误
watch([host, port], () => {
  if (errorMessage.value) errorMessage.value = '';
});

onMounted(() => {
  // 延迟聚焦确保弹窗动画完成或渲染完毕
  setTimeout(() => portInput.value?.focus(), 100);
});

function handlePaste(event: ClipboardEvent) {
  const text = event.clipboardData?.getData('text') || '';
  if (text.includes(':')) {
    event.preventDefault();
    const parsed = splitConnectEndpoint(text);
    host.value = parsed.host;
    port.value = parsed.port;
  }
}

async function reconnect() {
  if (store.busy.wireless || !originalEndpoint.value || !host.value || !port.value) return;

  errorMessage.value = '';
  
  try {
    const endpoint = buildConnectEndpoint(host.value, port.value);
    await store.adbConnect(endpoint, ui.reconnectSource, originalEndpoint.value);
    
    let successMsg = reconnectSuccessMessage(endpoint);
    if (ui.reconnectLaunchAfterConnect) {
      await store.startMirror(endpoint);
      successMsg = `已连接并启动投屏: ${endpoint}`;
    }

    // 成功：立即关闭弹窗并飘 Toast
    ui.closeModal();
    ui.pushToast(successMsg, 'success');
  } catch (error) {
    errorMessage.value = errorUserMessage(error);
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="!store.busy.wireless && ui.closeModal()">
    <section class="modal-card">
      <header class="modal-header">
        <div>
          <div class="modal-title">重连无线设备</div>
          <div class="modal-subtitle">修改当前设备的无线调试地址后重新连接</div>
        </div>
        <button class="modal-close" :disabled="store.busy.wireless" aria-label="关闭" @click="ui.closeModal()">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M2 2l8 8M10 2L2 10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </button>
      </header>
      <div class="modal-body compact-layout">
        <div class="modal-section-title">当前设备</div>
        <div class="modal-device-summary">
          <span>{{ currentDeviceName }}</span>
          <span class="chip chip-gray">{{ wirelessSourceLabel(currentDevice?.wireless_source || ui.reconnectSource) }}</span>
          <span class="mono">{{ originalEndpoint || '-' }}</span>
        </div>

        <div class="modal-section-group">
          <div class="modal-section-title">连接地址</div>
          <div class="modal-note">手机重启或无线调试重新开启后，连接端口可能变化，请以手机显示为准。</div>
        </div>

        <div class="form-grid form-grid-ip">
          <label>设备 IP
            <input 
              v-model="host" 
              class="field-input" 
              placeholder="192.168.1.100" 
              @paste="handlePaste"
              @keyup.enter="reconnect"
            />
          </label>
          <label>连接端口
            <input 
              ref="portInput"
              v-model="port" 
              class="field-input" 
              placeholder="39845" 
              inputmode="numeric"
              @keyup.enter="reconnect"
            />
          </label>
        </div>
        
        <div class="modal-feedback-slot" aria-live="polite">
          <div v-if="feedbackMessage" :class="['modal-feedback', `modal-feedback-${feedbackTone}`]">{{ feedbackMessage }}</div>
        </div>
      </div>
      <footer class="modal-footer">
        <button class="btn btn-ghost" :disabled="store.busy.wireless" @click="ui.closeModal()">取消</button>
        <button class="btn btn-primary" :disabled="!originalEndpoint || !host || !port || store.busy.wireless" @click="reconnect">
          {{ store.busy.wireless ? '重连中...' : ui.reconnectLaunchAfterConnect ? '重连并投屏' : '重连' }}
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.compact-layout {
  gap: 12px;
  padding-bottom: 16px;
}

.modal-section-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 8px;
}

.modal-feedback-slot {
  min-height: 40px;
  margin-top: 4px;
}

.form-grid {
  margin-top: 4px;
}
</style>
