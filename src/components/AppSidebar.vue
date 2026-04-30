<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const runningCount = computed(() => store.sessions.filter((session) => session.status === 'running').length);
</script>

<template>
  <aside class="sidebar">
    <div class="titlebar"></div>
    <div class="brand">
      <div class="brand-icon">D</div>
      <span class="brand-name">DroidDock</span>
    </div>
    <div :class="['tool-pill', store.isToolsReady ? 'ok' : 'warn']">
      <span :class="['dot', store.isToolsReady ? 'dot-green' : 'dot-yellow']"></span>
      {{ store.isToolsReady ? 'Tools ready' : 'Tools missing' }}
    </div>
    <div class="sidebar-divider"></div>
    <nav class="nav">
      <button :class="['nav-item', { active: store.currentPage === 'devices' }]" @click="store.currentPage = 'devices'">
        设备
      </button>
      <button :class="['nav-item', { active: store.currentPage === 'sessions' }]" @click="store.currentPage = 'sessions'">
        <span>投屏会话</span>
        <span v-if="runningCount" class="nav-badge">{{ runningCount }}</span>
      </button>
      <button :class="['nav-item', { active: store.currentPage === 'setup' }]" @click="store.currentPage = 'setup'">
        工具配置
      </button>
      <button :class="['nav-item', { active: store.currentPage === 'settings' }]" @click="store.currentPage = 'settings'">
        参数设置
      </button>
    </nav>
    <div class="sidebar-spacer"></div>
    <div class="sidebar-footer">
      <div class="tool-version"><span class="mono">adb</span><span class="mono">{{ store.toolStatus?.adb_version || '-' }}</span></div>
      <div class="tool-version"><span class="mono">scrcpy</span><span class="mono">{{ store.toolStatus?.scrcpy_version || '-' }}</span></div>
    </div>
  </aside>
</template>
