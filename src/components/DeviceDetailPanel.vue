<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import CommandPreview from './CommandPreview.vue';
import DeviceSessionPanel from './DeviceSessionPanel.vue';
import ParameterEditor from './ParameterEditor.vue';
import StatusChip from './StatusChip.vue';
import { buildScrcpyCommand, presetLabels, presetOptions } from '../domain/scrcpyOptions';
import { deviceIpAddress, wirelessSourceLabel } from '../domain/wireless';
import type { PresetId, ScrcpyOptions } from '../types/app';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const editorOptions = ref<ScrcpyOptions>({});
const activePreset = ref<PresetId>('daily');
const device = computed(() => store.selectedDevice);
const hasDeviceOptions = computed(() => Boolean(device.value && store.deviceOptionEntry(device.value.serial)));
const displaySession = computed(() => (device.value ? store.displaySession(device.value.serial) : null));
const isMirroring = computed(() => displaySession.value?.status === 'running');
const ipAddress = computed(() => (device.value ? deviceIpAddress(device.value.serial) : null));
const command = computed(() => (device.value ? buildScrcpyCommand(device.value.serial, editorOptions.value) : 'scrcpy'));
const canReconnectAndLaunch = computed(
  () => Boolean(device.value?.connection === 'wireless' && device.value?.presence === 'offline' && device.value?.endpoint && store.isToolsReady),
);
const canLaunch = computed(
  () =>
    !isMirroring.value &&
    (Boolean(device.value?.presence === 'online' && device.value?.state === 'device' && store.isToolsReady) ||
      canReconnectAndLaunch.value),
);
const launchButtonText = computed(() => (canReconnectAndLaunch.value ? '重连投屏' : '启动投屏'));
const launchHint = computed(() => {
  if (!device.value) return '请选择设备';
  if (!store.isToolsReady) return '请先完成工具配置';
  if (device.value.presence === 'offline') {
    if (canReconnectAndLaunch.value) return '确认无线调试地址后重连并启动投屏';
    return device.value.connection === 'wireless' ? '缺少无线连接地址，请重新配对或通过 USB 转无线' : '设备当前不在线，插入 USB 后会自动刷新';
  }
  if (device.value.state === 'unauthorized') return '请先在手机上允许 USB 调试授权';
  if (device.value.state === 'offline') return '设备已离线，请重新连接';
  return '';
});
const connectionLabel = computed(() => {
  if (!device.value) return '-';
  if (device.value.connection === 'usb') return 'USB';
  return wirelessSourceLabel(device.value.wireless_source);
});

watch(
  () => [ui.selectedSerial, store.appConfig] as const,
  ([serial]) => {
    if (!serial) return;
    editorOptions.value = { ...store.effectiveOptions(serial) };
    activePreset.value = store.deviceOptionEntry(serial)?.presetId ?? store.appConfig?.default_preset_id ?? 'daily';
  },
  { immediate: true },
);

function applyPreset(presetId: PresetId) {
  activePreset.value = presetId;
  editorOptions.value = { ...presetOptions[presetId] };
}

async function saveForDevice() {
  if (!device.value) return;
  await store.saveDeviceOptions(device.value.serial, editorOptions.value, activePreset.value);
}

async function resetToGlobal() {
  if (!device.value) return;
  await store.clearDeviceOptions(device.value.serial);
  editorOptions.value = { ...store.effectiveOptions(device.value.serial) };
}

async function launch() {
  if (!device.value) return;
  if (canReconnectAndLaunch.value && device.value.endpoint) {
    ui.openReconnectModal(device.value.serial, device.value.endpoint, device.value.wireless_source, true);
    return;
  }
  store.sessionDraftOptions[device.value.serial] = { ...editorOptions.value };
  await store.startMirror(device.value.serial, editorOptions.value);
}

async function handleEditAlias() {
  if (!device.value) return;
  const newAlias = window.prompt('输入设备别名', device.value.alias || device.value.model || '');
  if (newAlias === null) return;
  await store.saveDeviceAlias(device.value.serial, newAlias.trim() || null);
}
</script>

<template>
  <section v-if="device" class="device-detail">
    <div class="device-hero-block">
      <div class="device-hero">
        <div class="hero-icon">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
            <rect x="3.5" y="1.5" width="13" height="17" rx="2.5" stroke="currentColor" stroke-width="1.3" />
            <circle cx="10" cy="15.5" r="1.2" fill="currentColor" />
            <rect x="6.5" y="4" width="7" height="1" rx=".5" fill="currentColor" opacity=".4" />
          </svg>
        </div>
        <div class="device-hero-info">
          <div class="hero-title-row">
            <div class="hero-title">{{ device.alias || device.display_name || device.model || '未知设备' }}</div>
            <button class="btn-icon-sm" title="编辑别名" @click="handleEditAlias">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                <path d="M2.5 11.5v-2l6-6 2 2-6 6h-2zM9.5 2.5l2 2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
              </svg>
            </button>
          </div>
          <div class="hero-chips">
            <StatusChip
              :tone="device.state === 'device' ? 'green' : device.state === 'unauthorized' ? 'yellow' : 'red'"
              :label="device.presence === 'offline' ? '离线' : device.state === 'device' ? '可用' : device.state === 'unauthorized' ? '待授权' : '离线'"
              dot
            />
            <StatusChip tone="gray" :label="connectionLabel" />
          </div>
        </div>
      </div>
      <div v-if="device.presence === 'offline'" class="device-warning">
        {{ device.connection === 'wireless' ? '设备当前不在线，可先重连无线调试。' : '设备当前不在线，插入 USB 后会自动刷新。' }}
        <button
          v-if="device.connection === 'wireless' && device.endpoint"
          class="btn btn-ghost compact-button"
          @click.stop="ui.openReconnectModal(device.serial, device.endpoint, device.wireless_source)"
        >
          重连
        </button>
      </div>
      <div class="metadata-grid">
        <span>Serial</span><span class="mono">{{ device.serial }}</span>
        <span>型号</span><span>{{ device.model || '-' }}</span>
        <span>IP</span><span class="mono">{{ ipAddress || '-' }}</span>
      </div>
    </div>
    <DeviceSessionPanel :device="device" :session="displaySession" />
    <section class="detail-section">
      <div class="section-head">
        <div>
          <div class="section-title">投屏参数</div>
          <StatusChip :tone="hasDeviceOptions ? 'blue' : 'gray'" :label="hasDeviceOptions ? '使用设备独立设置' : '使用全局默认'" />
        </div>
        <div class="inline-actions">
          <select class="field-select compact-preset" :value="activePreset" @change="applyPreset(($event.target as HTMLSelectElement).value as PresetId)">
            <option v-for="(label, id) in presetLabels" :key="id" :value="id">{{ label }}</option>
          </select>
          <button class="btn btn-ghost compact-button" @click="saveForDevice">保存</button>
          <button class="btn btn-ghost compact-button" :disabled="!hasDeviceOptions" @click="resetToGlobal">重置</button>
        </div>
      </div>
      <ParameterEditor v-model:options="editorOptions" variant="device" />
      <CommandPreview :command="command" />
    </section>
    <footer class="launch-bar" :class="{ disabled: !canLaunch }">
      <button class="btn btn-primary launch-button" :disabled="!canLaunch" :title="launchHint || launchButtonText" @click="launch">
        <svg width="13" height="13" viewBox="0 0 13 13" fill="none" aria-hidden="true">
          <path d="M3 2L11 6.5L3 11V2Z" fill="currentColor" />
        </svg>
        {{ launchButtonText }}
      </button>
    </footer>
  </section>
  <section v-else class="device-detail empty-detail">
    <div>
      <div class="empty-title">未发现设备</div>
      <div class="empty-copy">请连接 USB 设备，或使用 ADB Pair 添加无线设备。</div>
    </div>
  </section>
</template>
