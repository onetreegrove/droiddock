<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAppStore } from '../stores/app';

const props = defineProps<{
  serial: string;
}>();

const store = useAppStore();
const device = computed(() => store.devices.find(d => d.serial === props.serial));
const session = computed(() => store.activeSession(props.serial));

const showLogs = ref(false);
const activePreset = ref('daily');

const presets = [
  { id: 'daily', name: '日常使用', desc: '1920px, 60fps, 禁用音频', options: { maxSize: 1920, maxFps: 60, noAudio: true, stayAwake: true } },
  { id: 'wireless', name: '低带宽模式', desc: '1024px, 30fps, 2M 码率', options: { maxSize: 1024, maxFps: 30, videoBitRate: '2M', noAudio: true } },
  { id: 'presentation', name: '演示模式', desc: '1920px, 显示触摸, 窗口置顶', options: { maxSize: 1920, showTouches: true, alwaysOnTop: true, stayAwake: true } },
  { id: 'view-only', name: '仅查看模式', desc: '禁用控制, 高画质', options: { maxSize: 1920, noControl: true, stayAwake: true } },
];

const startMirror = async () => {
  const preset = presets.find(p => p.id === activePreset.value);
  if (!preset) return;
  try {
    await store.startMirror(props.serial, preset.options);
  } catch (e) {
    // Error logged in store
  }
};

const stopMirror = async () => {
  if (session.value) {
    await store.stopMirror(session.value.session_id);
  }
};

const switchToWireless = async () => {
  await store.adbTcpip(props.serial);
};

const sessionLogs = computed(() => {
  if (session.value) {
    return store.sessionLogs[session.value.session_id] || [];
  }
  return [];
});
</script>

<template>
  <div v-if="device" class="view-content">
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;">
      <h1>{{ device.model || '未知设备' }}</h1>
      <span :class="['badge', `badge-${device.connection}`]">{{ device.connection === 'usb' ? 'USB' : '无线' }}</span>
    </div>
    <p>序列号: {{ device.serial }} | 产品: {{ device.product }}</p>

    <div class="btn-group">
      <button v-if="!session" class="primary" @click="startMirror">启动投屏</button>
      <button v-else class="primary danger" @click="stopMirror">停止投屏</button>
      
      <button class="secondary" @click="store.refreshDevices">刷新状态</button>
      <button v-if="device.connection === 'usb'" class="secondary" @click="switchToWireless">切换到无线连接</button>
    </div>

    <div v-if="session" class="card" style="border-color: var(--green);">
      <div style="display: flex; align-items: center; justify-content: space-between;">
        <strong>正在投屏中</strong>
        <span style="font-size: 12px; color: var(--green);">PID: {{ session.pid }}</span>
      </div>
    </div>

    <h2>投屏预设</h2>
    <div class="preset-grid">
      <button 
        v-for="p in presets" 
        :key="p.id" 
        :class="['preset-item', { active: activePreset === p.id }]"
        @click="activePreset = p.id"
        :disabled="!!session"
      >
        <strong>{{ p.name }}</strong>
        <span>{{ p.desc }}</span>
      </button>
    </div>

    <div class="log-panel">
      <div class="log-header" @click="showLogs = !showLogs">
        <h2>{{ session ? '会话日志' : '系统日志' }}</h2>
        <span>{{ showLogs ? '▼' : '▶' }}</span>
      </div>
      <div v-if="showLogs" class="log-content">
        <template v-if="session">
          <div v-for="(l, i) in sessionLogs" :key="i" :class="l.level" style="margin-bottom: 2px;">
            [{{ new Date(l.timestamp * 1000).toLocaleTimeString() }}] {{ l.message }}
          </div>
        </template>
        <template v-else>
          {{ store.logs.join('\n') }}
        </template>
      </div>
    </div>
  </div>
  <div v-else class="view-content">
    <p>设备未找到或已断开。</p>
  </div>
</template>
