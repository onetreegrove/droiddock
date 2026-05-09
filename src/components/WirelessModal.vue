<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';
import { buildConnectEndpoint, splitConnectEndpoint } from '../domain/wireless';

const store = useAppStore();
const ui = useUiStore();
const selectedSerial = ref(store.devices.find((device) => device.connection === 'usb' && device.state === 'device')?.serial ?? '');
const host = ref('');
const port = ref('5555');
const reconnectHost = ref('');
const reconnectPort = ref('');
const errorMessage = ref('');
const usbDevices = computed(() => store.devices.filter((device) => device.connection === 'usb' && device.state === 'device'));
const recentEndpoints = computed(() => store.appConfig?.recent_endpoints ?? []);

watch(
  () => ui.wirelessReconnectEndpoint,
  (endpoint) => {
    if (!endpoint) return;
    selectRecentEndpoint(endpoint);
  },
  { immediate: true },
);

watch(
  recentEndpoints,
  (endpoints) => {
    if (reconnectHost.value || reconnectPort.value || endpoints.length === 0) return;
    selectRecentEndpoint(endpoints[0]);
  },
  { immediate: true },
);

function selectRecentEndpoint(endpoint: string) {
  const parsed = splitConnectEndpoint(endpoint);
  reconnectHost.value = parsed.host;
  reconnectPort.value = parsed.port;
}

async function reconnect() {
  errorMessage.value = '';
  try {
    await store.adbConnect(buildConnectEndpoint(reconnectHost.value, reconnectPort.value), ui.wirelessReconnectSource);
    ui.closeModal();
  } catch (error) {
    errorMessage.value = String(error instanceof Error ? error.message : error);
  }
}

async function submit() {
  if (!selectedSerial.value || !host.value) return;
  errorMessage.value = '';
  try {
    await store.adbTcpip(selectedSerial.value, Number(port.value || 5555));
    await store.adbConnect(buildConnectEndpoint(host.value, port.value || '5555'), 'usb_tcpip');
    ui.closeModal();
  } catch (error) {
    errorMessage.value = String(error instanceof Error ? error.message : error);
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
        <div class="modal-section-title">已配对设备重连</div>
        <div class="modal-note">已完成 ADB Pair 的设备可直接填写无线调试连接端口，无需再次输入配对码</div>
        <div v-if="recentEndpoints.length > 0" class="wireless-device-list">
          <label v-for="endpoint in recentEndpoints" :key="endpoint" class="wireless-device-option">
            <input
              type="radio"
              name="recent-endpoint"
              :checked="`${reconnectHost}:${reconnectPort}` === endpoint"
              @change="selectRecentEndpoint(endpoint)"
            />
            <span><strong>最近连接</strong><span class="mono">{{ endpoint }}</span></span>
          </label>
        </div>
        <div v-else class="modal-note">暂无最近连接记录，可手动填写已配对设备的 IP 和当前连接端口</div>
        <div class="form-grid form-grid-ip">
          <label>设备 IP<input v-model="reconnectHost" class="field-input" placeholder="192.168.1.100" /></label>
          <label>连接端口<input v-model="reconnectPort" class="field-input" placeholder="39845" /></label>
        </div>
        <button class="btn btn-primary" :disabled="!reconnectHost || !reconnectPort" @click="reconnect">重连已配对设备</button>
        <div class="modal-divider"></div>
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
