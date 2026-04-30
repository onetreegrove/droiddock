<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog';
import AppHeader from './AppHeader.vue';
import StatusChip from './StatusChip.vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();

function archLabel(arch: string | null | undefined) {
  if (!arch) return '架构未知';
  const normalized = arch.toLowerCase();
  if (normalized.includes('arm64')) return 'arm64';
  if (normalized.includes('universal binary')) return 'universal';
  if (normalized.includes('shell script') || normalized.includes('text executable')) return 'script';
  return '架构不兼容';
}

async function chooseToolPath(tool: 'adb' | 'scrcpy') {
  const selected = await open({
    title: `选择 ${tool} 可执行文件`,
    multiple: false,
    directory: false,
  });
  if (typeof selected === 'string') {
    await store.setToolPath(tool, selected);
  }
}

async function installTools() {
  const result = await store.installTools();
  window.alert(`安装完成\nadb: ${result.adb_path}\nscrcpy: ${result.scrcpy_path}`);
}
</script>

<template>
  <section class="page">
    <AppHeader title="工具配置" subtitle="adb 与 scrcpy 路径管理" />
    <div class="setup-body">
      <div class="section-label">已安装工具</div>
      <div class="tool-card">
        <div :class="['tool-icon', store.toolStatus?.adb_ok ? 'ok' : 'missing']">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <rect x="1.5" y="3.5" width="11" height="7" rx="1.2" stroke="currentColor" stroke-width="1.1" />
            <path d="M4 6.5h6M4 8.5h4" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
          </svg>
        </div>
        <div class="tool-info">
          <div class="tool-name-row"><strong class="mono">adb</strong><span>Android Debug Bridge</span></div>
          <div class="tool-path mono">{{ store.toolStatus?.adb_path || '未找到' }}</div>
          <div class="tool-tags">
            <StatusChip :tone="store.toolStatus?.adb_ok ? 'blue' : 'red'" :label="store.toolStatus?.adb_version || '缺失'" />
            <StatusChip :tone="store.toolStatus?.adb_ok ? 'gray' : 'red'" :label="archLabel(store.toolStatus?.adb_arch)" />
            <StatusChip :tone="store.toolStatus?.adb_ok ? 'green' : 'red'" :label="store.toolStatus?.adb_ok ? '正常' : '缺失'" />
          </div>
        </div>
        <button class="btn btn-ghost compact-button" @click="chooseToolPath('adb')">手动选择</button>
      </div>
      <div class="tool-card">
        <div :class="['tool-icon', store.toolStatus?.scrcpy_ok ? 'ok' : 'missing']">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <rect x="1.5" y="3.5" width="11" height="7" rx="1.2" stroke="currentColor" stroke-width="1.1" />
            <path d="M4 6.5h6M4 8.5h4" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
          </svg>
        </div>
        <div class="tool-info">
          <div class="tool-name-row"><strong class="mono">scrcpy</strong><span>Screen Copy</span></div>
          <div class="tool-path mono">{{ store.toolStatus?.scrcpy_path || '未找到' }}</div>
          <div class="tool-tags">
            <StatusChip :tone="store.toolStatus?.scrcpy_ok ? 'blue' : 'red'" :label="store.toolStatus?.scrcpy_version || '缺失'" />
            <StatusChip :tone="store.toolStatus?.scrcpy_ok ? 'gray' : 'red'" :label="archLabel(store.toolStatus?.scrcpy_arch)" />
            <StatusChip :tone="store.toolStatus?.scrcpy_ok ? 'green' : 'red'" :label="store.toolStatus?.scrcpy_ok ? '正常' : '缺失'" />
          </div>
        </div>
        <button class="btn btn-ghost compact-button" @click="chooseToolPath('scrcpy')">手动选择</button>
      </div>
      <div class="install-panel">
        <div><div class="panel-title">自动安装</div><div class="hint-text">工具将安装到 ~/Library/Application Support/DroidDock/tools/</div></div>
        <button class="btn btn-primary" :disabled="store.busy.install" @click="installTools">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M6 1.5v7M3.5 6l2.5 3 2.5-3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
            <path d="M2 10.5h8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
          {{ store.busy.install ? '安装中...' : '自动安装全部' }}
        </button>
      </div>
      <div class="source-panel">
        <div><span>adb 来源：</span><span class="mono">Android SDK Platform Tools (official)</span></div>
        <div><span>scrcpy 来源：</span><span class="mono">GitHub / Genymobile/scrcpy macOS arm64</span></div>
      </div>
      <div class="guide-panel">
        <div class="section-label">新手入门</div>
        <div class="guide-steps">
          <div class="guide-step"><span>1</span><div><strong>安装工具</strong><p>点击「自动安装全部」或手动配置路径</p></div></div>
          <div class="guide-step"><span>2</span><div><strong>开启开发者选项</strong><p>设置中连续点击版本号 7 次</p></div></div>
          <div class="guide-step"><span>3</span><div><strong>开启 USB 调试</strong><p>进入开发者选项打开 USB 调试</p></div></div>
          <div class="guide-step"><span>4</span><div><strong>连接并授权</strong><p>USB 连接后在手机弹窗中允许调试</p></div></div>
          <div class="guide-step"><span>5</span><div><strong>启动投屏</strong><p>在「设备」页面选择设备并点击启动</p></div></div>
        </div>
      </div>
    </div>
  </section>
</template>
