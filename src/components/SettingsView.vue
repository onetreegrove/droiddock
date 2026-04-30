<script setup lang="ts">
import ParameterEditor from './ParameterEditor.vue';
import AppHeader from './AppHeader.vue';
import { presetLabels, presetOptions } from '../domain/scrcpyOptions';
import type { PresetId } from '../types/app';
import { useAppStore } from '../stores/app';

const store = useAppStore();

function applyPreset(id: PresetId) {
  store.globalDraftPresetId = id;
  store.globalDraftOptions = { ...presetOptions[id] };
}
</script>

<template>
  <section class="page">
    <AppHeader title="参数设置" subtitle="全局默认投屏参数，适用于所有未单独配置的设备">
      <template #actions>
        <button class="btn btn-primary" @click="store.saveDefaultOptions(store.globalDraftOptions, store.globalDraftPresetId)">保存全局默认</button>
      </template>
    </AppHeader>
    <div class="settings-body">
      <div class="section-label">快速应用预设</div>
      <div class="preset-row">
        <button v-for="(label, id) in presetLabels" :key="id" :class="['preset-chip', { on: store.globalDraftPresetId === id }]" @click="applyPreset(id as PresetId)">
          {{ label }}
        </button>
      </div>
      <div class="settings-panel">
        <div class="section-label">画面与控制参数</div>
        <ParameterEditor v-model:options="store.globalDraftOptions" />
      </div>
      <div class="error-table">
        <div><span class="mono">unauthorized</span><span>请解锁手机，并在弹窗中允许 USB 调试</span></div>
        <div><span class="mono">offline</span><span>设备已离线，请重新插拔或重连无线调试</span></div>
        <div><span class="mono">Connection refused</span><span>无线调试端口不可用，请检查 IP 和端口</span></div>
        <div><span class="mono">failed to authenticate</span><span>配对失败，请重新生成配对码</span></div>
        <div><span class="mono">device not found</span><span>设备不存在或已断开，请刷新设备列表</span></div>
      </div>
    </div>
  </section>
</template>
