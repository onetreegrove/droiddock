<script setup lang="ts">
import StatusChip from './StatusChip.vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();

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
        :class="['device-card', { selected: store.selectedSerial === device.serial }]"
        @click="store.selectedSerial = device.serial"
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
        <div class="device-serial mono">{{ device.serial }}</div>
        <div v-if="device.state === 'unauthorized'" class="device-warning">
          <svg width="11" height="11" viewBox="0 0 11 11" fill="none" aria-hidden="true">
            <path d="M5.5 1L10 9.5H1L5.5 1Z" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round" />
            <path d="M5.5 4.5v2M5.5 7.5v.4" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
          </svg>
          请在手机上允许 USB 调试授权
        </div>
      </button>
      <div v-if="store.devices.length === 0" class="empty-panel">未检测到设备，请连接 USB 或使用 ADB Pair。</div>
      <div class="connection-actions">
        <div class="section-label">连接方式</div>
        <button class="connection-button" @click="store.modal = 'wireless'">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M2 4.5c1.1-1.1 2.6-1.8 4-1.8s2.9.7 4 1.8" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
            <path d="M4 7c.6-.6 1.4-1 2-1s1.4.4 2 1" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
            <circle cx="6" cy="9.5" r=".8" fill="currentColor" />
          </svg>
          USB 转无线连接
        </button>
        <button class="connection-button" @click="store.modal = 'pair'">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <rect x="1" y="2.5" width="4" height="7" rx="1" stroke="currentColor" stroke-width="1.1" />
            <rect x="7" y="2.5" width="4" height="7" rx="1" stroke="currentColor" stroke-width="1.1" />
            <path d="M5 6h2" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
          </svg>
          ADB Pair 无线配对
        </button>
      </div>
    </div>
  </section>
</template>
