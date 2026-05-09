<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';
import AppHeader from './AppHeader.vue';
import SessionCard from './SessionCard.vue';

const store = useAppStore();
const ui = useUiStore();
const runningSessions = computed(() => store.sessions.filter(s => s.status === 'running'));

const toolSteps = computed(() => [
  {
    name: 'ADB (Android Debug Bridge)',
    ok: store.toolStatus?.adb_ok,
    version: store.toolStatus?.adb_version,
    path: store.toolStatus?.adb_path,
    desc: '用于发现和管理 Android 设备的底层驱动工具。'
  },
  {
    name: 'Scrcpy (Screen Copy)',
    ok: store.toolStatus?.scrcpy_ok,
    version: store.toolStatus?.scrcpy_version,
    path: store.toolStatus?.scrcpy_path,
    desc: '高性能屏幕镜像引擎，支持低延迟投屏。'
  }
]);
</script>

<template>
  <div class="page">
    <AppHeader title="首页" subtitle="欢迎使用 DroidDock 控制台" />
    
    <div class="dashboard-body">
      <!-- 工具状态概览 -->
      <section class="dashboard-section">
        <div class="section-label">核心组件状态</div>
        <div class="tool-grid">
          <div v-for="tool in toolSteps" :key="tool.name" :class="['dashboard-card', 'tool-status-card', { ok: tool.ok }]">
            <div class="tool-status-icon">
              <svg v-if="tool.ok" width="20" height="20" viewBox="0 0 20 20" fill="none">
                <path d="M16 6L8.5 13.5L4 9" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              <svg v-else width="20" height="20" viewBox="0 0 20 20" fill="none">
                <path d="M12 8L8 12M8 8L12 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
                <circle cx="10" cy="10" r="7" stroke="currentColor" stroke-width="2"/>
              </svg>
            </div>
            <div class="tool-status-content">
              <div class="tool-status-title">
                <strong>{{ tool.name.split(' (')[0] }}</strong>
                <span v-if="tool.version" class="mono">{{ tool.version }}</span>
              </div>
              <p class="tool-status-desc">{{ tool.desc }}</p>
              <div v-if="!tool.ok" class="tool-status-action">
                <button class="btn btn-primary btn-sm" @click="ui.openPage('setup')">立即配置</button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- 快速操作 -->
      <section class="dashboard-section">
        <div class="section-label">快速连接</div>
        <div class="action-grid">
          <button class="dashboard-card action-card" @click="ui.openModal('wireless')">
            <div class="action-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M5 12.55a11 11 0 0 1 14.08 0M1.42 9a16 16 0 0 1 21.16 0M8.53 16.11a6 6 0 0 1 6.95 0M12 20h.01" />
              </svg>
            </div>
            <div class="action-info">
              <div class="action-title">USB 转无线</div>
              <div class="action-desc">将已连接的 USB 设备切换至 Wi-Fi 模式</div>
            </div>
          </button>
          <button class="dashboard-card action-card" @click="ui.openModal('pair')">
            <div class="action-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="2" width="18" height="20" rx="2" />
                <path d="M12 18h.01M8 6h8" />
              </svg>
            </div>
            <div class="action-info">
              <div class="action-title">ADB Pair</div>
              <div class="action-desc">通过配对码连接 Android 11+ 无线设备</div>
            </div>
          </button>
        </div>
      </section>

      <!-- 活跃会话预览 -->
      <section v-if="runningSessions.length > 0" class="dashboard-section">
        <div class="section-label">当前运行中的投屏 ({{ runningSessions.length }})</div>
        <div class="session-grid">
          <SessionCard v-for="session in runningSessions" :key="session.session_id" :session="session" />
        </div>
      </section>
      
      <!-- 引导提示 -->
      <section v-else class="dashboard-section empty-state-section">
        <div class="dashboard-card empty-guide-card">
          <div class="empty-guide-icon">
            <svg width="48" height="48" viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M24 4v40M4 24h40M10 10l28 28M38 10L10 38" opacity="0.1" />
              <rect x="14" y="8" width="20" height="32" rx="4" />
              <path d="M24 34h.01M20 14h8" />
            </svg>
          </div>
          <div class="empty-guide-content">
            <h3>准备好开始了吗？</h3>
            <p>连接你的 Android 设备到电脑，或者使用上方的无线连接向导。</p>
            <button class="btn btn-primary" @click="ui.openPage('devices')">查看设备列表</button>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.dashboard-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.dashboard-section {
  display: flex;
  flex-direction: column;
}

.dashboard-card {
  background: var(--bg4);
  border: 1px solid var(--border1);
  border-radius: 16px;
  padding: 20px;
  transition: all 0.2s ease;
}

.tool-grid, .action-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
  margin-top: 12px;
}

.tool-status-card {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.tool-status-card.ok {
  border-color: rgba(74, 222, 128, 0.2);
}

.tool-status-icon {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  background: var(--bg5);
  color: var(--t3);
}

.ok .tool-status-icon {
  background: var(--green-d);
  color: var(--green);
}

.tool-status-content {
  flex: 1;
}

.tool-status-title {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 4px;
}

.tool-status-title strong {
  font-size: 15px;
}

.tool-status-desc {
  font-size: 13px;
  color: var(--t3);
  line-height: 1.5;
  margin: 0;
}

.tool-status-action {
  margin-top: 12px;
}

.action-card {
  display: flex;
  align-items: center;
  gap: 16px;
  text-align: left;
  cursor: pointer;
  border: 1px solid var(--border1);
}

.action-card:hover {
  background: var(--bg5);
  border-color: var(--acc);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.action-icon {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  display: grid;
  place-items: center;
  background: var(--acc-d);
  color: var(--acc);
  flex-shrink: 0;
}

.action-title {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 2px;
}

.action-desc {
  font-size: 12px;
  color: var(--t3);
}

.session-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 16px;
  margin-top: 12px;
}

.empty-guide-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 48px;
  background: linear-gradient(to bottom, var(--bg3), var(--bg4));
  border-style: dashed;
  border-width: 2px;
}

.empty-guide-icon {
  margin-bottom: 20px;
  color: var(--acc);
}

.empty-guide-content h3 {
  margin: 0 0 8px;
  font-size: 20px;
}

.empty-guide-content p {
  color: var(--t3);
  margin-bottom: 24px;
}

.btn-sm {
  min-height: 28px;
  padding: 4px 12px;
  font-size: 12px;
}
</style>
