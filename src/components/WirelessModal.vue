<script setup lang="ts">
import { computed, ref } from 'vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const selectedSerial = ref(store.devices.find((device) => device.connection === 'usb' && device.state === 'device')?.serial ?? '');
const host = ref('');
const port = ref('5555');
const usbDevices = computed(() => store.devices.filter((device) => device.connection === 'usb' && device.state === 'device'));

async function submit() {
  if (!selectedSerial.value || !host.value) return;
  await store.adbTcpip(selectedSerial.value, Number(port.value || 5555));
  await store.adbConnect(`${host.value}:${port.value || '5555'}`);
  store.modal = null;
}
</script>

<template>
  <div class="modal-overlay" @click.self="store.modal = null">
    <section class="modal-card">
      <header class="modal-header">
        <div><div class="modal-title">USB 转无线连接</div><div class="modal-subtitle">通过 USB 建立无线调试连接</div></div>
        <button class="btn btn-ghost" @click="store.modal = null">关闭</button>
      </header>
      <div class="modal-body">
        <label>USB 设备<select v-model="selectedSerial" class="field-select"><option v-for="device in usbDevices" :key="device.serial" :value="device.serial">{{ device.model || device.serial }}</option></select></label>
        <div class="form-grid">
          <label>手机 IP<input v-model="host" class="field-input" placeholder="192.168.1.100" /></label>
          <label>端口<input v-model="port" class="field-input" /></label>
        </div>
      </div>
      <footer class="modal-footer">
        <button class="btn btn-ghost" @click="store.modal = null">取消</button>
        <button class="btn btn-primary" :disabled="!selectedSerial || !host" @click="submit">连接</button>
      </footer>
    </section>
  </div>
</template>
