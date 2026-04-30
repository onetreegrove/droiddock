<script setup lang="ts">
import type { ScrcpyOptions } from '../types/app';

const props = withDefaults(defineProps<{ options: ScrcpyOptions; variant?: 'device' | 'settings' }>(), {
  variant: 'device',
});
const emit = defineEmits<{ 'update:options': [value: ScrcpyOptions] }>();

function patch(value: Partial<ScrcpyOptions>) {
  emit('update:options', { ...props.options, ...value });
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
    <div class="toggle-grid">
      <button :class="['toggle-card', { on: options.noAudio }]" @click="patch({ noAudio: !options.noAudio })">
        <div><div class="toggle-title">禁用音频</div><span>--no-audio</span></div>
        <span :class="['toggle-switch', { on: options.noAudio }]"><span></span></span>
      </button>
      <button :class="['toggle-card', { on: options.noControl }]" @click="patch({ noControl: !options.noControl })">
        <div><div class="toggle-title">只看不控</div><span>--no-control</span></div>
        <span :class="['toggle-switch', { on: options.noControl }]"><span></span></span>
      </button>
      <button :class="['toggle-card', { on: options.stayAwake }]" @click="patch({ stayAwake: !options.stayAwake })">
        <div><div class="toggle-title">保持亮屏</div><span>--stay-awake</span></div>
        <span :class="['toggle-switch', { on: options.stayAwake }]"><span></span></span>
      </button>
      <button :class="['toggle-card', { on: options.turnScreenOff }]" @click="patch({ turnScreenOff: !options.turnScreenOff })">
        <div><div class="toggle-title">息屏投屏</div><span>--turn-screen-off</span></div>
        <span :class="['toggle-switch', { on: options.turnScreenOff }]"><span></span></span>
      </button>
      <button :class="['toggle-card', { on: options.alwaysOnTop }]" @click="patch({ alwaysOnTop: !options.alwaysOnTop })">
        <div><div class="toggle-title">置顶窗口</div><span>--always-on-top</span></div>
        <span :class="['toggle-switch', { on: options.alwaysOnTop }]"><span></span></span>
      </button>
      <button :class="['toggle-card', { on: options.showTouches }]" @click="patch({ showTouches: !options.showTouches })">
        <div><div class="toggle-title">显示触摸</div><span>--show-touches</span></div>
        <span :class="['toggle-switch', { on: options.showTouches }]"><span></span></span>
      </button>
      <button :class="['toggle-card', { on: options.fullscreen }]" @click="patch({ fullscreen: !options.fullscreen })">
        <div><div class="toggle-title">全屏</div><span>--fullscreen</span></div>
        <span :class="['toggle-switch', { on: options.fullscreen }]"><span></span></span>
      </button>
    </div>
  </div>
</template>
