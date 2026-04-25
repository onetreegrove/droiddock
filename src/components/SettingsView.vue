<script setup lang="ts">
import { useAppStore } from '../stores/app';

const store = useAppStore();
</script>

<template>
  <div class="view-content">
    <h1>设置</h1>
    <p>管理工具路径和应用偏好。</p>

    <div class="card">
      <h2>工具状态</h2>
      <div class="tool-list">
        <div class="tool-item">
          <div class="tool-info">
            <strong>ADB (Android 调试桥)</strong>
            <span>{{ store.toolStatus?.adb_path || '未找到' }}</span>
          </div>
          <span :class="['badge', store.toolStatus?.adb_path ? 'badge-wireless' : 'badge-offline']">
            {{ store.toolStatus?.adb_version || '缺失' }}
          </span>
        </div>

        <div class="tool-item">
          <div class="tool-info">
            <strong>scrcpy</strong>
            <span>{{ store.toolStatus?.scrcpy_path || '未找到' }}</span>
          </div>
          <span :class="['badge', store.toolStatus?.scrcpy_path ? 'badge-wireless' : 'badge-offline']">
            {{ store.toolStatus?.scrcpy_version || '缺失' }}
          </span>
        </div>
      </div>
      
      <div style="margin-top: 24px; display: flex; gap: 12px;">
        <button class="primary" @click="store.fetchToolStatus">刷新工具状态</button>
        <button class="secondary">自动安装 (Apple Silicon)</button>
        <button class="secondary">手动配置</button>
      </div>
    </div>

    <div class="card">
      <h2>关于 DroidDock</h2>
      <p>版本 0.1.0 (Alpha)</p>
      <p>专为 macOS Apple Silicon 构建。基于 Tauri 和 Vue 3。</p>
    </div>
  </div>
</template>

<style scoped>
.tool-list {
  display: grid;
  gap: 16px;
}

.tool-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border: 1px solid var(--line);
  border-radius: 8px;
}

.tool-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tool-info span {
  font-family: ui-monospace, monospace;
  font-size: 11px;
  color: var(--muted);
}
</style>
