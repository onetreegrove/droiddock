<script setup lang="ts">
import type { ScrcpyOptions } from '../types/app';

const props = defineProps<{ options: ScrcpyOptions }>();
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
      <select class="field-select" :value="options.videoCodec ?? 'default'" @change="patch({ videoCodec: ($event.target as HTMLSelectElement).value as ScrcpyOptions['videoCodec'] })">
        <option value="default">默认</option>
        <option value="h264">H.264</option>
        <option value="h265">H.265</option>
      </select>
    </div>
    <div class="param-row">
      <span class="param-label">视频码率</span>
      <select class="field-select" :value="options.videoBitRate ?? ''" @change="patch({ videoBitRate: ($event.target as HTMLSelectElement).value })">
        <option value="">默认</option>
        <option value="16M">16 Mbps</option>
        <option value="8M">8 Mbps</option>
        <option value="4M">4 Mbps</option>
        <option value="2M">2 Mbps</option>
      </select>
    </div>
    <div class="toggle-grid">
      <button :class="['toggle-card', { on: options.noAudio }]" @click="patch({ noAudio: !options.noAudio })">禁用音频<span>--no-audio</span></button>
      <button :class="['toggle-card', { on: options.noControl }]" @click="patch({ noControl: !options.noControl })">只看不控<span>--no-control</span></button>
      <button :class="['toggle-card', { on: options.stayAwake }]" @click="patch({ stayAwake: !options.stayAwake })">保持亮屏<span>--stay-awake</span></button>
      <button :class="['toggle-card', { on: options.turnScreenOff }]" @click="patch({ turnScreenOff: !options.turnScreenOff })">息屏投屏<span>--turn-screen-off</span></button>
      <button :class="['toggle-card', { on: options.showTouches }]" @click="patch({ showTouches: !options.showTouches })">显示触摸<span>--show-touches</span></button>
      <button :class="['toggle-card', { on: options.alwaysOnTop }]" @click="patch({ alwaysOnTop: !options.alwaysOnTop })">置顶窗口<span>--always-on-top</span></button>
      <button :class="['toggle-card', { on: options.fullscreen }]" @click="patch({ fullscreen: !options.fullscreen })">全屏<span>--fullscreen</span></button>
    </div>
  </div>
</template>
