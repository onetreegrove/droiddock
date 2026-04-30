<script setup lang="ts">
import { computed } from 'vue';
import AppHeader from './AppHeader.vue';
import StatusChip from './StatusChip.vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const subtitle = computed(() => `${store.devices.length} 台已发现 · ${store.availableDeviceCount} 台可用`);

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
    <AppHeader title="设备" :subtitle="subtitle">
      <template #actions>
        <button class="btn btn-ghost" @click="store.modal = 'pair'">ADB Pair</button>
        <button class="btn btn-ghost" :disabled="store.busy.devices" @click="store.refreshDevices">
          {{ store.busy.devices ? '刷新中...' : '刷新' }}
        </button>
      </template>
    </AppHeader>
    <div class="device-list">
      <button
        v-for="device in store.devices"
        :key="device.serial"
        :class="['device-card', { selected: store.selectedSerial === device.serial }]"
        @click="store.selectedSerial = device.serial"
      >
        <div class="device-card-main">
          <div class="device-icon">▯</div>
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
        <div v-if="device.state === 'unauthorized'" class="device-warning">请在手机上允许 USB 调试授权</div>
      </button>
      <div v-if="store.devices.length === 0" class="empty-panel">未检测到设备，请连接 USB 或使用 ADB Pair。</div>
      <div class="connection-actions">
        <div class="section-label">连接方式</div>
        <button class="connection-button" @click="store.modal = 'wireless'">USB 转无线连接</button>
        <button class="connection-button" @click="store.modal = 'pair'">ADB Pair 无线配对</button>
      </div>
    </div>
  </section>
</template>
