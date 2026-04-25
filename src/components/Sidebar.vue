<script setup lang="ts">
import { useAppStore } from '../stores/app';

const store = useAppStore();

const selectDevice = (serial: string) => {
  store.selectedSerial = serial;
};

const selectDiscover = () => {
  store.selectedSerial = 'discover';
};

const selectSettings = () => {
  store.selectedSerial = 'settings';
};
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-section">
      <div class="sidebar-title">导航</div>
      <nav class="sidebar-nav">
        <button 
          class="sidebar-item" 
          :class="{ active: store.selectedSerial === 'discover' }"
          @click="selectDiscover"
        >
          <span class="icon">🔍</span>
          发现与配对
        </button>
      </nav>
    </div>

    <div class="sidebar-section">
      <div class="sidebar-title">已连接设备</div>
      <nav class="sidebar-nav">
        <button 
          v-for="device in store.devices" 
          :key="device.serial"
          class="sidebar-item"
          :class="{ active: store.selectedSerial === device.serial }"
          @click="selectDevice(device.serial)"
        >
          <span class="icon">{{ device.connection === 'usb' ? '🔌' : '📡' }}</span>
          {{ device.model || device.serial }}
        </button>
        <div v-if="store.devices.length === 0" class="sidebar-item" style="opacity: 0.5; font-style: italic;">
          未连接设备
        </div>
      </nav>
    </div>

    <div class="sidebar-footer">
      <button 
        class="sidebar-item" 
        :class="{ active: store.selectedSerial === 'settings' }"
        @click="selectSettings"
      >
        <span class="icon">⚙️</span>
        设置
      </button>
    </div>
  </aside>
</template>

<style scoped>
.icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}
</style>
