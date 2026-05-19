<script setup lang="ts">
import { computed } from 'vue';
import appPackage from '../../package.json';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';

const store = useAppStore();
const ui = useUiStore();
const appVersion = `v${appPackage.version}`;
const runningCount = computed(() => store.sessions.filter((session) => session.status === 'running').length);
</script>

<template>
  <aside class="sidebar">
    <div class="titlebar" data-tauri-drag-region></div>
    <div class="brand" data-tauri-drag-region>
      <div class="brand-icon">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none" aria-hidden="true">
          <rect x="2.5" y="1.5" width="8" height="15" rx="2" fill="currentColor" opacity=".9" />
          <rect x="12" y="4" width="3.5" height="10" rx="1" fill="currentColor" opacity=".4" />
          <circle cx="6.5" cy="14" r="1.3" fill="#0d0f12" />
        </svg>
      </div>
      <span class="brand-name">DroidDock</span>
    </div>
    <div :class="['tool-pill', store.isToolsReady ? 'ok' : 'warn']">
      <span :class="['dot', store.isToolsReady ? 'dot-green' : 'dot-yellow']"></span>
      {{ store.isToolsReady ? 'Tools ready' : 'Tools missing' }}
    </div>
    <div class="sidebar-divider"></div>
    <nav class="nav">
      <button :class="['nav-item', { active: ui.currentPage === 'devices' }]" @click="ui.openPage('devices')">
        <span class="nav-icon">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <rect x="2" y="1" width="8" height="14" rx="1.5" stroke="currentColor" stroke-width="1.3" />
            <circle cx="6" cy="12.5" r=".8" fill="currentColor" />
            <rect x="12" y="3" width="2" height="9" rx=".8" stroke="currentColor" stroke-width="1.2" />
          </svg>
        </span>
        <span>设备</span>
      </button>
      <button :class="['nav-item', { active: ui.currentPage === 'sessions' }]" @click="ui.openPage('sessions')">
        <span class="nav-icon">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <rect x="1" y="3" width="14" height="9" rx="1.5" stroke="currentColor" stroke-width="1.3" />
            <path d="M5 13.5h6M8 12v1.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </span>
        <span>投屏总览</span>
        <span v-if="runningCount" class="nav-badge">{{ runningCount }}</span>
      </button>
      <button :class="['nav-item', { active: ui.currentPage === 'setup' }]" @click="ui.openPage('setup')">
        <span class="nav-icon">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M8 2L9.5 5.5L13 6L10.5 8.5L11 12L8 10.5L5 12L5.5 8.5L3 6L6.5 5.5L8 2Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
          </svg>
        </span>
        <span>工具配置</span>
      </button>
      <button :class="['nav-item', { active: ui.currentPage === 'settings' }]" @click="ui.openPage('settings')">
        <span class="nav-icon">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <circle cx="8" cy="8" r="2.2" stroke="currentColor" stroke-width="1.3" />
            <path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </span>
        <span>参数设置</span>
      </button>
    </nav>
    <div class="sidebar-spacer"></div>
    <div class="sidebar-footer">
      <div class="tool-version"><span>DroidDock</span><span>{{ appVersion }}</span></div>
    </div>
  </aside>
</template>
