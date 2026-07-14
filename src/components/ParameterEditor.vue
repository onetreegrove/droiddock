<script setup lang="ts">
import { computed } from 'vue';
import { backgroundColorErrorMessage, defaultScrcpyCapabilities, nextWindowAspectRatioLockValue } from '../domain/scrcpyOptions';
import type { ScrcpyCapabilities, ScrcpyOptions } from '../types/app';

const props = withDefaults(defineProps<{ options: ScrcpyOptions; variant?: 'device' | 'settings'; capabilities?: ScrcpyCapabilities }>(), {
  variant: 'device',
  capabilities: () => defaultScrcpyCapabilities,
});
const emit = defineEmits<{ 'update:options': [value: ScrcpyOptions] }>();

const backgroundColor = computed(() => props.options.backgroundColor ?? '');
const backgroundColorError = computed(() => backgroundColorErrorMessage(backgroundColor.value));

function patch(value: Partial<ScrcpyOptions>) {
  emit('update:options', { ...props.options, ...value });
}

function patchBackgroundColor(value: string) {
  patch({ backgroundColor: value.trim() ? value : undefined });
}
</script>

<template>
  <div class="parameter-editor">
    <div class="param-row">
      <span class="param-label">最大分辨率</span>
      <select class="field-select" :value="options.maxSize ?? 1920" @change="patch({ maxSize: Number(($event.target as HTMLSelectElement).value) })">
        <option :value="1920">1920 (推荐)</option>
        <option :value="1280">1280</option>
        <option :value="1024">1024</option>
      </select>
    </div>
    <div class="param-row">
      <span class="param-label">最大帧率</span>
      <select class="field-select" :value="options.maxFps ?? 60" @change="patch({ maxFps: Number(($event.target as HTMLSelectElement).value) })">
        <option :value="60">60 fps</option>
        <option :value="45">45 fps</option>
        <option :value="30">30 fps</option>
      </select>
    </div>
    <div class="param-row">
      <span class="param-label">视频编码</span>
      <select
        class="field-select"
        :value="options.videoCodec ?? 'default'"
        @change="patch({ videoCodec: ($event.target as HTMLSelectElement).value as ScrcpyOptions['videoCodec'] })"
      >
        <option value="default">默认</option>
        <option value="h264">H.264</option>
        <option value="h265">H.265</option>
      </select>
    </div>
    <div class="param-row">
      <span class="param-label">视频码率</span>
      <select class="field-select" :value="options.videoBitRate ?? ''" @change="patch({ videoBitRate: ($event.target as HTMLSelectElement).value })">
        <option value="">默认</option>
        <option value="8M">8 Mbps</option>
        <option value="4M">4 Mbps</option>
        <option value="2M">2 Mbps</option>
      </select>
    </div>
    <div class="param-row">
      <span class="param-label">背景色</span>
      <div class="field-stack">
        <input
          :class="['field-input', { invalid: backgroundColorError }]"
          :value="backgroundColor"
          :disabled="!capabilities.supportsBackgroundColor"
          placeholder="#234567"
          @input="patchBackgroundColor(($event.target as HTMLInputElement).value)"
        />
        <div v-if="backgroundColorError && capabilities.supportsBackgroundColor" class="field-error">{{ backgroundColorError }}</div>
      </div>
    </div>
    <div class="toggle-groups">
      <div class="toggle-group">
        <div class="toggle-group-title">窗口</div>
        <div class="toggle-grid">
          <button :class="['toggle-card', { on: options.alwaysOnTop }]" @click="patch({ alwaysOnTop: !options.alwaysOnTop })">
            <div><div class="toggle-title">置顶窗口</div><span>--always-on-top</span></div>
            <span :class="['toggle-switch', { on: options.alwaysOnTop }]"><span></span></span>
          </button>
          <button :class="['toggle-card', { on: options.fullscreen }]" @click="patch({ fullscreen: !options.fullscreen })">
            <div><div class="toggle-title">全屏</div><span>--fullscreen</span></div>
            <span :class="['toggle-switch', { on: options.fullscreen }]"><span></span></span>
          </button>
          <button
            :class="['toggle-card', { on: options.windowAspectRatioLock !== false }]"
            :disabled="!capabilities.supportsWindowAspectRatioLock"
            @click="patch({ windowAspectRatioLock: nextWindowAspectRatioLockValue(options.windowAspectRatioLock) })"
          >
            <div><div class="toggle-title">锁定窗口比例</div><span>{{ capabilities.supportsWindowAspectRatioLock ? '避免黑边' : '需要 scrcpy 4.0+' }}</span></div>
            <span :class="['toggle-switch', { on: options.windowAspectRatioLock !== false }]"><span></span></span>
          </button>
        </div>
      </div>
      <div class="toggle-group">
        <div class="toggle-group-title">控制</div>
        <div class="toggle-grid">
          <button :class="['toggle-card', { on: options.noControl }]" @click="patch({ noControl: !options.noControl })">
            <div><div class="toggle-title">只看不控</div><span>--no-control</span></div>
            <span :class="['toggle-switch', { on: options.noControl }]"><span></span></span>
          </button>
          <button :class="['toggle-card', { on: options.stayAwake }]" @click="patch({ stayAwake: !options.stayAwake })">
            <div><div class="toggle-title">插电时保持唤醒</div><span>--stay-awake</span></div>
            <span :class="['toggle-switch', { on: options.stayAwake }]"><span></span></span>
          </button>
          <button
            :class="['toggle-card', { on: options.keepActive }]"
            :disabled="!capabilities.supportsKeepActive"
            @click="patch({ keepActive: !options.keepActive })"
          >
            <div><div class="toggle-title">保持设备活跃</div><span>{{ capabilities.supportsKeepActive ? '不修改系统设置' : '需要 scrcpy 4.0+' }}</span></div>
            <span :class="['toggle-switch', { on: options.keepActive }]"><span></span></span>
          </button>
          <button :class="['toggle-card', { on: options.showTouches }]" @click="patch({ showTouches: !options.showTouches })">
            <div><div class="toggle-title">显示触摸</div><span>--show-touches</span></div>
            <span :class="['toggle-switch', { on: options.showTouches }]"><span></span></span>
          </button>
        </div>
      </div>
      <div class="toggle-group">
        <div class="toggle-group-title">设备</div>
        <div class="toggle-grid">
          <button :class="['toggle-card', { on: options.noAudio }]" @click="patch({ noAudio: !options.noAudio })">
            <div><div class="toggle-title">禁用音频</div><span>--no-audio</span></div>
            <span :class="['toggle-switch', { on: options.noAudio }]"><span></span></span>
          </button>
          <button :class="['toggle-card', { on: options.turnScreenOff }]" @click="patch({ turnScreenOff: !options.turnScreenOff })">
            <div><div class="toggle-title">息屏投屏</div><span>--turn-screen-off</span></div>
            <span :class="['toggle-switch', { on: options.turnScreenOff }]"><span></span></span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
