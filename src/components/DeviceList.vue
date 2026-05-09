<script setup lang="ts">
import StatusChip from './StatusChip.vue';
import { wirelessSourceLabel } from '../domain/wireless';
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
            @click.stop="ui.openWirelessReconnect(device.endpoint, device.wireless_source || 'manual')"
          >
            重连
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
  </section>
</template>
