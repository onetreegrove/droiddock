<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import AppHeader from './AppHeader.vue';
import StatusChip from './StatusChip.vue';
import { useAppStore } from '../stores/app';
import { useToolsStore } from '../stores/tools';
import { useUiStore } from '../stores/ui';
import type { ToolDiagnostic, ToolInstallTarget, ToolKind } from '../lib/ipc/types';
import { errorUserMessage } from '../lib/ipc/errors';
import { installProgressLines, installSummary, shouldShowInstallLogDialog } from '../domain/installProgress';
import { toolActionLabel, toolHealthTone, toolSourceLabel, toolSummary } from '../domain/toolDiagnostics';

const store = useAppStore();
const toolsStore = useToolsStore();
const ui = useUiStore();
const setupError = ref<string | null>(null);
const choosingTool = ref<ToolKind | null>(null);
const installLogDialogOpen = ref(false);
let unlistenInstallProgress: (() => void) | null = null;

const toolOperationLocked = computed(() => store.busy.tools || store.busy.install || choosingTool.value !== null);
const installLocked = computed(() => store.busy.install || store.busy.tools || choosingTool.value !== null);
const visibleInstallLog = computed(() =>
  installProgressLines(Boolean(store.busy.install), store.installLogs, store.installTarget),
);
const showInstallLogDialog = computed(() =>
  shouldShowInstallLogDialog(installLogDialogOpen.value, Boolean(store.busy.install), visibleInstallLog.value),
);
const installLogSummary = computed(() =>
  installSummary(store.installStatus, store.installError, visibleInstallLog.value.length),
);

const diagnostics = computed(() => {
  const status = store.toolStatus;
  if (!status) return [];

  return [
    { ...status.adb, title: 'Android Debug Bridge' },
    { ...status.scrcpy, title: 'Screen Copy' },
  ];
});

function statusLabel(diagnostic: ToolDiagnostic) {
  return diagnostic.health === 'ready' ? '正常' : toolActionLabel(diagnostic);
}

function installButtonLabel(target: ToolInstallTarget) {
  if (store.busy.install && store.installTarget === target) return '安装中...';
  if (target === 'adb') return '安装 adb';
  if (target === 'scrcpy') return '安装 scrcpy';
  return '安装全部';
}

function installTargetLabel(target: ToolInstallTarget) {
  if (target === 'adb') return 'adb';
  if (target === 'scrcpy') return 'scrcpy';
  return 'adb + scrcpy';
}

function closeInstallLogDialog() {
  installLogDialogOpen.value = false;
  toolsStore.clearInstallLogs();
}

async function refreshTools() {
  if (toolOperationLocked.value) return;
  setupError.value = null;
  try {
    await store.fetchToolStatusStrict();
    ui.pushToast('工具状态已更新', 'success');
  } catch (error) {
    setupError.value = errorUserMessage(error);
    ui.pushToast(setupError.value, 'error');
  }
}

async function chooseToolPath(tool: ToolKind) {
  if (toolOperationLocked.value) return;
  choosingTool.value = tool;
  setupError.value = null;
  try {
    const selected = await open({
      title: `选择 ${tool} 可执行文件`,
      multiple: false,
      directory: false,
    });
    if (typeof selected === 'string') {
      await store.setToolPath(tool, selected);
      ui.pushToast(`已更新 ${tool} 路径`, 'success');
    }
  } catch (error) {
    setupError.value = errorUserMessage(error);
    ui.pushToast(setupError.value, 'error');
  } finally {
    choosingTool.value = null;
  }
}

async function clearToolPath(tool: ToolKind) {
  if (toolOperationLocked.value) return;
  setupError.value = null;
  try {
    await store.clearToolPath(tool);
    ui.pushToast(`已清除 ${tool} 手动路径`, 'success');
  } catch (error) {
    setupError.value = errorUserMessage(error);
    ui.pushToast(setupError.value, 'error');
  }
}

async function installTools(target: ToolInstallTarget) {
  if (installLocked.value) return;
  setupError.value = null;
  installLogDialogOpen.value = true;
  try {
    await store.installTools(target);
    ui.pushToast('工具安装完成', 'success');
  } catch (error) {
    setupError.value = errorUserMessage(error);
    ui.pushToast(setupError.value, 'error');
  }
}

onMounted(async () => {
  unlistenInstallProgress = await toolsStore.listenToolInstallProgress();
});

onBeforeUnmount(() => {
  unlistenInstallProgress?.();
});
</script>

<template>
  <section class="page">
    <AppHeader title="工具配置" subtitle="adb 与 scrcpy 路径管理" />
    <div class="setup-body">
      <div class="section-label">已安装工具</div>
      <div v-if="setupError" class="setup-error">{{ setupError }}</div>
      <div v-for="diagnostic in diagnostics" :key="diagnostic.kind" class="tool-card">
        <div :class="['tool-icon', diagnostic.health === 'ready' ? 'ok' : 'missing']">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <rect x="1.5" y="3.5" width="11" height="7" rx="1.2" stroke="currentColor" stroke-width="1.1" />
            <path d="M4 6.5h6M4 8.5h4" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
          </svg>
        </div>
        <div class="tool-info">
          <div class="tool-name-row"><strong class="mono">{{ diagnostic.kind }}</strong><span>{{ diagnostic.title }}</span></div>
          <div class="tool-path mono">{{ diagnostic.path || '未找到' }}</div>
          <div class="tool-tags">
            <StatusChip :tone="toolHealthTone(diagnostic)" :label="statusLabel(diagnostic)" dot />
            <StatusChip tone="gray" :label="toolSourceLabel(diagnostic.source)" />
            <StatusChip :tone="diagnostic.version ? 'blue' : 'gray'" :label="diagnostic.version || '版本未知'" />
          </div>
          <div class="tool-message">{{ toolSummary(diagnostic) }}</div>
        </div>
        <div class="tool-actions">
          <button class="btn btn-ghost compact-button" :disabled="toolOperationLocked" @click="chooseToolPath(diagnostic.kind)">手动选择</button>
          <button
            v-if="diagnostic.source === 'configured'"
            class="btn btn-ghost compact-button"
            :disabled="toolOperationLocked"
            @click="clearToolPath(diagnostic.kind)"
          >
            清除路径
          </button>
        </div>
      </div>
      <div class="install-panel">
        <div><div class="panel-title">自动安装</div><div class="hint-text">工具将安装到 ~/Library/Application Support/DroidDock/tools/</div></div>
        <div class="tool-actions">
          <button class="btn btn-ghost" :disabled="toolOperationLocked" @click="refreshTools">重新检测</button>
          <button class="btn btn-ghost" :disabled="installLocked" @click="installTools('adb')">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M6 1.5v7M3.5 6l2.5 3 2.5-3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
              <path d="M2 10.5h8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            </svg>
            {{ installButtonLabel('adb') }}
          </button>
          <button class="btn btn-ghost" :disabled="installLocked" @click="installTools('scrcpy')">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M6 1.5v7M3.5 6l2.5 3 2.5-3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
              <path d="M2 10.5h8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            </svg>
            {{ installButtonLabel('scrcpy') }}
          </button>
          <button class="btn btn-primary" :disabled="installLocked" @click="installTools('all')">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M6 1.5v7M3.5 6l2.5 3 2.5-3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
              <path d="M2 10.5h8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            </svg>
            {{ installButtonLabel('all') }}
          </button>
        </div>
      </div>
      <div class="source-panel">
        <div><span>adb 来源：</span><span class="mono">Android SDK Platform Tools (official)</span></div>
        <div><span>scrcpy 来源：</span><span class="mono">GitHub / Genymobile/scrcpy macOS arm64</span></div>
      </div>
      <div class="guide-panel">
        <div class="section-label">新手入门</div>
        <div class="guide-steps">
          <div class="guide-step"><span>1</span><div><strong>安装工具</strong><p>按需安装 adb、scrcpy，或一次安装全部</p></div></div>
          <div class="guide-step"><span>2</span><div><strong>开启开发者选项</strong><p>设置中连续点击版本号 7 次</p></div></div>
          <div class="guide-step"><span>3</span><div><strong>开启 USB 调试</strong><p>进入开发者选项打开 USB 调试</p></div></div>
          <div class="guide-step"><span>4</span><div><strong>连接并授权</strong><p>USB 连接后在手机弹窗中允许调试</p></div></div>
          <div class="guide-step"><span>5</span><div><strong>启动投屏</strong><p>在「设备」页面选择设备并点击启动</p></div></div>
        </div>
      </div>
    </div>
    <div v-if="showInstallLogDialog" class="modal-overlay" @click.self="!store.busy.install && closeInstallLogDialog()">
      <section class="modal-card log-modal install-log-modal">
        <header class="modal-header install-log-modal-header">
          <div class="install-log-heading">
            <div :class="['install-log-status-dot', installLogSummary.tone]"></div>
            <div>
              <div class="modal-title">{{ installLogSummary.title }}</div>
              <div class="modal-subtitle">目标：<span class="mono">{{ installTargetLabel(store.installTarget) }}</span></div>
            </div>
          </div>
          <button v-if="!store.busy.install" type="button" class="modal-close" aria-label="关闭" @click="closeInstallLogDialog">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M2 2l8 8M10 2L2 10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            </svg>
          </button>
        </header>
        <div class="modal-body install-log-modal-body" aria-live="polite">
          <div :class="['install-log-summary', installLogSummary.tone]">
            <span>{{ installLogSummary.title }}</span>
            <span>{{ installLogSummary.detail }}</span>
          </div>
          <div :class="['install-log', store.busy.install ? 'pending' : '']">
            <div v-for="(line, index) in visibleInstallLog" :key="`${index}-${line}`" class="install-log-line">
              <span class="install-log-index mono">{{ String(index + 1).padStart(2, '0') }}</span>
              <span class="install-log-message mono">{{ line }}</span>
            </div>
          </div>
        </div>
        <footer class="modal-footer install-log-modal-footer">
          <div class="modal-note">{{ store.busy.install ? '安装过程中请保持此窗口打开。' : '可以关闭弹窗并返回工具配置。' }}</div>
          <button v-if="!store.busy.install" type="button" class="btn btn-primary" @click="closeInstallLogDialog">关闭</button>
        </footer>
      </section>
    </div>
  </section>
</template>
