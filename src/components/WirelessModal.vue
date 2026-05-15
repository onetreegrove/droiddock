<script setup lang="ts">
import { computed, ref } from 'vue';
import { buildConnectEndpoint } from '../domain/wireless';
import { errorUserMessage } from '../lib/ipc/errors';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const selectedSerial = ref(store.devices.find((device) => device.connection === 'usb' && device.state === 'device')?.serial ?? '');
const host = ref('');
const port = ref('5555');
const errorMessage = ref('');
const usbDevices = computed(() => store.devices.filter((device) => device.connection === 'usb' && device.state === 'device'));

async function submit() {
  if (!selectedSerial.value || !host.value) return;
  errorMessage.value = '';
  try {
    await store.adbTcpip(selectedSerial.value, Number(port.value || 5555));
    await store.adbConnect(buildConnectEndpoint(host.value, port.value || '5555'), 'usb_tcpip');
    ui.closeModal();
  } catch (error) {
    errorMessage.value = errorUserMessage(error);
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="ui.closeModal()">
    <section class="modal-card">
      <header class="modal-header">
        <div><div class="modal-title">USB 转无线连接</div><div class="modal-subtitle">适用于 Android 10 及以下，或通过 USB 建立无线</div></div>
        <button class="modal-close" aria-label="关闭" @click="ui.closeModal()">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M2 2l8 8M10 2L2 10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </button>
      </header>
      <div class="modal-body">
        <div class="modal-section-title">选择 USB 连接设备</div>
        <div class="wireless-device-list">
          <label v-for="device in usbDevices" :key="device.serial" class="wireless-device-option">
            <input v-model="selectedSerial" type="radio" :value="device.serial" />
            <span><strong>{{ device.alias || device.model || '未知设备' }}</strong><span class="mono">{{ device.serial }}</span></span>
          </label>
          <div v-if="usbDevices.length === 0" class="modal-note">暂无可用 USB 设备</div>
        </div>
        <div class="modal-section-title">输入手机 IP 地址</div>
        <div class="modal-note">设置 → 关于手机 → 状态 → IP 地址</div>
        <div class="form-grid form-grid-ip">
          <label>手机 IP<input v-model="host" class="field-input" placeholder="192.168.1.100" /></label>
          <label>端口<input v-model="port" class="field-input" /></label>
        </div>
        <div v-if="errorMessage" class="modal-error">{{ errorMessage }}</div>
      </div>
      <footer class="modal-footer">
        <button class="btn btn-ghost" @click="ui.closeModal()">取消</button>
        <button class="btn btn-primary" :disabled="!selectedSerial || !host" @click="submit">连接</button>
      </footer>
    </section>
  </div>
</template>
