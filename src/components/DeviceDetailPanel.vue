<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import CommandPreview from './CommandPreview.vue';
import DeviceHero from './DeviceHero.vue';
import DeviceSessionPanel from './DeviceSessionPanel.vue';
import DeviceStatusBanner from './DeviceStatusBanner.vue';
import ParameterEditor from './ParameterEditor.vue';
import StatusChip from './StatusChip.vue';
import { launchAvailability } from '../domain/deviceDetail';
import { buildScrcpyCommand, presetLabels, presetOptions } from '../domain/scrcpyOptions';
import { deviceIpAddress, wirelessSourceLabel } from '../domain/wireless';
import type { PresetId, ScrcpyOptions } from '../types/app';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const editorOptions = ref<ScrcpyOptions>({});
const activePreset = ref<PresetId>('daily');
const aliasModalOpen = ref(false);
const aliasDraft = ref('');
const aliasError = ref('');
const device = computed(() => store.selectedDevice);
const hasDeviceOptions = computed(() => Boolean(device.value && store.deviceOptionEntry(device.value.serial)));
const displaySession = computed(() => (device.value ? store.displaySession(device.value.serial) : null));
const isMirroring = computed(() => displaySession.value?.status === 'running');
const ipAddress = computed(() => (device.value ? deviceIpAddress(device.value.serial) : null));
const command = computed(() => (device.value ? buildScrcpyCommand(device.value.serial, editorOptions.value) : 'scrcpy'));
const launchState = computed(() => launchAvailability(device.value, isMirroring.value, store.isToolsReady));
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

watch(
  () => ui.selectedSerial,
  () => {
    aliasModalOpen.value = false;
    aliasError.value = '';
  },
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
  store.sessionDraftOptions[device.value.serial] = { ...editorOptions.value };
  await store.startMirror(device.value.serial, editorOptions.value);
}

function handleEditAlias() {
  if (!device.value) return;
  aliasDraft.value = device.value.alias || device.value.display_name || device.value.model || '';
  aliasError.value = '';
  aliasModalOpen.value = true;
}

async function saveAlias() {
  if (!device.value) return;
  aliasError.value = '';
  try {
    await store.saveDeviceAlias(device.value.serial, aliasDraft.value.trim() || null);
    aliasModalOpen.value = false;
  } catch (error) {
    aliasError.value = String(error instanceof Error ? error.message : error);
  }
}
</script>

<template>
  <section v-if="device" class="device-detail">
    <div class="device-hero-block">
      <DeviceHero
        :device="device"
        :ip-address="ipAddress"
        :connection-label="connectionLabel"
        @edit-alias="handleEditAlias"
      />
      <DeviceStatusBanner :device="device" />
    </div>
    <DeviceSessionPanel :device="device" :session="displaySession" :launch-state="launchState" @launch="launch" />
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
    <div v-if="aliasModalOpen" class="modal-overlay" @click.self="aliasModalOpen = false">
      <form class="modal-card alias-modal" @submit.prevent="saveAlias">
        <header class="modal-header">
          <div>
            <div class="modal-title">编辑设备别名</div>
            <div class="modal-subtitle">给这台设备取一个更容易识别的名称</div>
          </div>
          <button type="button" class="modal-close" aria-label="关闭" @click="aliasModalOpen = false">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M2 2l8 8M10 2L2 10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            </svg>
          </button>
        </header>
        <div class="modal-body">
          <label>
            设备别名
            <input v-model="aliasDraft" class="field-input" maxlength="40" placeholder="例如：客厅手机" />
          </label>
          <div class="modal-note">留空后会恢复为设备型号或系统名称。</div>
          <div v-if="aliasError" class="modal-error">{{ aliasError }}</div>
        </div>
        <footer class="modal-footer">
          <button type="button" class="btn btn-ghost" @click="aliasModalOpen = false">取消</button>
          <button class="btn btn-primary" type="submit">保存别名</button>
        </footer>
      </form>
    </div>
  </section>
  <section v-else class="device-detail empty-detail">
    <div>
      <div class="empty-title">未发现设备</div>
      <div class="empty-copy">请连接 USB 设备，或使用 ADB Pair 添加无线设备。</div>
    </div>
  </section>
</template>
