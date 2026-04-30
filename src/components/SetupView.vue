<script setup lang="ts">
import AppHeader from './AppHeader.vue';
import StatusChip from './StatusChip.vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
</script>

<template>
  <section class="page">
    <AppHeader title="工具配置" subtitle="adb 与 scrcpy 路径管理" />
    <div class="setup-body">
      <div class="section-label">已安装工具</div>
      <div class="tool-card">
        <div><strong class="mono">adb</strong><div class="tool-path mono">{{ store.toolStatus?.adb_path || '未找到' }}</div></div>
        <StatusChip :tone="store.toolStatus?.adb_ok ? 'green' : 'red'" :label="store.toolStatus?.adb_version || '缺失'" />
      </div>
      <div class="tool-card">
        <div><strong class="mono">scrcpy</strong><div class="tool-path mono">{{ store.toolStatus?.scrcpy_path || '未找到' }}</div></div>
        <StatusChip :tone="store.toolStatus?.scrcpy_ok ? 'green' : 'red'" :label="store.toolStatus?.scrcpy_version || '缺失'" />
      </div>
      <div class="install-panel">
        <div><div class="panel-title">自动安装</div><div class="hint-text">工具将安装到 ~/Library/Application Support/DroidDock/tools/</div></div>
        <button class="btn btn-ghost" disabled>自动安装待接入</button>
      </div>
      <div class="guide-panel">
        <div class="section-label">新手入门</div>
        <ol class="guide-list">
          <li>安装工具：点击自动安装或手动配置路径。</li>
          <li>开启开发者选项：设置中连续点击版本号 7 次。</li>
          <li>开启 USB 调试：进入开发者选项打开 USB 调试。</li>
          <li>连接并授权：USB 连接后在手机弹窗中允许调试。</li>
          <li>启动投屏：在设备页选择设备并点击启动。</li>
        </ol>
      </div>
    </div>
  </section>
</template>
