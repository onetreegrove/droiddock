<script setup lang="ts">
import StatusChip from './StatusChip.vue';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();

function stateLabel(state: string) {
  if (state === 'device') return '可用';
  if (state === 'unauthorized') return '待授权';
  if (state === 'offline') return '离线';
  return state;
}

function stateTone(state: string) {
  if (state === 'device') return 'green';
  if (state === 'unauthorized') return 'yellow';
  if (state === 'offline') return 'red';
  return 'gray';
}
</script>

<template>
  <section class="device-column">
    <div class="device-list">
      <button
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
            <div class="device-name">{{ device.alias || device.model || '未知设备' }}</div>
            <div class="device-model mono">{{ device.model || device.product || device.serial }}</div>
          </div>
          <div class="device-chips">
            <StatusChip :tone="stateTone(device.state)" :label="stateLabel(device.state)" dot />
            <StatusChip tone="gray" :label="device.connection === 'usb' ? 'USB' : '无线'" />
          </div>
        </div>
        <div v-if="device.state === 'unauthorized'" class="device-warning">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M6 1.5L10.5 10H1.5L6 1.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
            <path d="M6 5v2M6 8.5v.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
          请在手机上允许 USB 调试授权
        </div>
      </button>
      <div v-if="store.devices.length === 0" class="empty-panel">未检测到设备，请连接 USB 或使用首页的无线连接向导。</div>
    </div>
  </section>
</template>
