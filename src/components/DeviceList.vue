<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ManagedDevice } from '../types/app';
import ConfirmModal from './ConfirmModal.vue';
import StatusChip from './StatusChip.vue';
import { wirelessSourceLabel } from '../domain/wireless';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const pendingForgetDevice = ref<ManagedDevice | null>(null);
const pendingForgetDeviceName = computed(() => {
  const device = pendingForgetDevice.value;
  return device?.alias || device?.display_name || device?.model || device?.serial || '';
});

function stateLabel(state: string) {
  if (state === 'device') return '可用';
  if (state === 'unauthorized') return '待授权';
  if (state === 'offline') return '离线';
  return state;
}

function presenceLabel(device: { presence: string; state: string }) {
  if (device.presence === 'offline') return '离线';
  return stateLabel(device.state);
}

function stateTone(state: string) {
  if (state === 'device') return 'green';
  if (state === 'unauthorized') return 'yellow';
  if (state === 'offline') return 'red';
  return 'gray';
}

function presenceTone(device: { presence: string; state: string }): 'green' | 'yellow' | 'red' | 'gray' {
  if (device.presence === 'offline') return 'gray';
  return stateTone(device.state);
}

function connectionLabel(device: { connection: string; wireless_source: 'adb_pair' | 'usb_tcpip' | 'manual' | null }) {
  if (device.connection === 'usb') return 'USB';
  return wirelessSourceLabel(device.wireless_source);
}

function connectionHint(device: { presence: string; connection: string }) {
  if (device.presence !== 'offline') return '';
  if (device.connection === 'usb') return '插入 USB 后会自动刷新';
  return '可重连，端口可修改';
}

function requestForgetDevice(device: ManagedDevice) {
  pendingForgetDevice.value = device;
}

async function confirmForgetDevice() {
  const device = pendingForgetDevice.value;
  if (!device) return;
  await store.forgetDevice(device.serial);
  pendingForgetDevice.value = null;
}
</script>

<template>
  <section class="device-column">
    <div class="device-list">
      <article
        v-for="device in store.devices"
        :key="device.serial"
        :class="['device-card', { selected: ui.selectedSerial === device.serial }]"
        @click="ui.selectedSerial = device.serial"
      >
        <div class="device-card-main">
          <div class="device-icon">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <rect x="3.5" y="1.5" width="9" height="13" rx="1.5" stroke="currentColor" stroke-width="1.2" />
              <circle cx="8" cy="12" r=".8" fill="currentColor" />
            </svg>
          </div>
          <div class="device-info">
            <div class="device-name">{{ device.alias || device.display_name || device.model || '未知设备' }}</div>
            <div class="device-model mono">{{ device.model || device.product || device.serial }}</div>
          </div>
          <div class="device-chips">
            <StatusChip :tone="presenceTone(device)" :label="presenceLabel(device)" dot />
            <StatusChip tone="gray" :label="connectionLabel(device)" />
          </div>
        </div>
        <div v-if="connectionHint(device)" class="device-warning">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M6 1.5L10.5 10H1.5L6 1.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
            <path d="M6 5v2M6 8.5v.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
          {{ connectionHint(device) }}
          <button
            v-if="device.connection === 'wireless' && device.endpoint"
            class="btn btn-ghost compact-button"
            @click.stop="ui.openReconnectModal(device.serial, device.endpoint, device.wireless_source)"
          >
            重连
          </button>
          <button
            v-if="device.presence === 'offline'"
            class="btn btn-danger compact-button"
            :disabled="store.busy.devices"
            @click.stop="requestForgetDevice(device)"
          >
            删除
          </button>
        </div>
        <div v-if="device.state === 'unauthorized'" class="device-warning">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M6 1.5L10.5 10H1.5L6 1.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
            <path d="M6 5v2M6 8.5v.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
          请在手机上允许 USB 调试授权
        </div>
      </article>
      <div v-if="store.devices.length === 0" class="empty-panel">未检测到设备，请连接 USB，或使用设备页右上角的无线连接入口。</div>
    </div>
    <ConfirmModal
      v-if="pendingForgetDevice"
      title="删除历史设备"
      :message="`删除历史设备“${pendingForgetDeviceName}”？删除后不会影响手机，重新连接后会再次出现在列表中。`"
      confirm-text="删除"
      danger
      @confirm="confirmForgetDevice"
      @cancel="pendingForgetDevice = null"
    />
  </section>
</template>
