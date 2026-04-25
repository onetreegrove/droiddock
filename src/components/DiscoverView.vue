<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();

const pairStep = ref(1);
const pairIp = ref('');
const pairPort = ref('');
const pairCode = ref('');
const connectPort = ref('');

const resetPair = async () => {
  try {
    await store.adbPair({
      host: pairIp.value,
      pair_port: parseInt(pairPort.value),
      pairing_code: pairCode.value,
      connect_port: connectPort.value ? parseInt(connectPort.value) : null
    });
    pairStep.value = 1;
    pairIp.value = '';
    pairPort.value = '';
    pairCode.value = '';
    connectPort.value = '';
  } catch (e) {
    // Error logged in store
  }
};

const nextStep = () => {
  if (pairStep.value < 3) pairStep.value++;
};
</script>

<template>
  <div class="view-content">
    <h1>发现与配对</h1>
    <p>通过 USB 连接 Android 设备或使用无线调试（Android 11+）。</p>

    <div class="card">
      <h2>USB 设备发现</h2>
      <div v-if="store.devices.length === 0" class="empty-state">
        <p>未检测到 USB 设备。请插入手机并开启 USB 调试。</p>
        <button class="primary" @click="store.refreshDevices">刷新列表</button>
      </div>
      <div v-else class="usb-list">
        <div v-for="device in store.devices.filter(d => d.connection === 'usb')" :key="device.serial" class="usb-item">
          <div>
            <strong>{{ device.model || '未知设备' }}</strong>
            <span class="serial">{{ device.serial }}</span>
          </div>
          <span v-if="device.state === 'unauthorized'" class="warn-text">未授权 - 请查看手机屏幕</span>
          <span v-else class="ok-text">就绪</span>
        </div>
      </div>
    </div>

    <div class="card">
      <h2>无线配对 (Android 11+)</h2>
      <div v-if="pairStep === 1" class="wizard-step">
        <span class="wizard-label">1. 输入配对地址</span>
        <div style="display: flex; gap: 8px;">
          <input v-model="pairIp" class="wizard-input" placeholder="IP 地址 (例如 192.168.1.5)" />
          <input v-model="pairPort" class="wizard-input" style="width: 100px;" placeholder="端口" />
        </div>
        <button class="primary" style="margin-top: 16px;" @click="nextStep" :disabled="!pairIp || !pairPort">下一步</button>
      </div>

      <div v-if="pairStep === 2" class="wizard-step">
        <span class="wizard-label">2. 输入配对码</span>
        <input v-model="pairCode" class="wizard-input" placeholder="6 位配对码" maxlength="6" />
        <div style="margin-top: 16px; display: flex; gap: 8px;">
          <button class="secondary" @click="pairStep = 1">上一步</button>
          <button class="primary" @click="nextStep" :disabled="!pairCode">下一步</button>
        </div>
      </div>

      <div v-if="pairStep === 3" class="wizard-step">
        <span class="wizard-label">3. 输入连接端口</span>
        <input v-model="connectPort" class="wizard-input" placeholder="连接端口（通常与配对端口不同）" />
        <div style="margin-top: 16px; display: flex; gap: 8px;">
          <button class="secondary" @click="pairStep = 2">上一步</button>
          <button class="primary" @click="resetPair">配对并连接</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty-state {
  text-align: center;
  padding: 20px 0;
}

.usb-list {
  display: grid;
  gap: 12px;
}

.usb-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
}

.serial {
  display: block;
  font-size: 11px;
  color: var(--muted);
}

.warn-text { color: var(--amber); font-size: 12px; font-weight: 600; }
.ok-text { color: var(--green); font-size: 12px; font-weight: 600; }
</style>
