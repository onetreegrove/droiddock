<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import AppSidebar from './components/AppSidebar.vue';
import AppHeader from './components/AppHeader.vue';
import DeviceList from './components/DeviceList.vue';
import DeviceDetailPanel from './components/DeviceDetailPanel.vue';
import SessionsView from './components/SessionsView.vue';
import SetupView from './components/SetupView.vue';
import SettingsView from './components/SettingsView.vue';
import ConfirmModal from './components/ConfirmModal.vue';
import PairModal from './components/PairModal.vue';
import ReconnectModal from './components/ReconnectModal.vue';
import WirelessModal from './components/WirelessModal.vue';
import { useAppStore } from './stores/app';
import { useUiStore } from './stores/ui';

const store = useAppStore();
const ui = useUiStore();
const showExitConfirm = ref(false);
const manualRefreshing = ref(false);
let poller: number | undefined;
let unlistenClose: UnlistenFn | undefined;
let unlistenLogs: UnlistenFn | undefined;
const devicesSubtitle = computed(() => `${store.devices.length} 台已发现 · ${store.availableDeviceCount} 台可用`);

function hasTauriWindowMetadata() {
  return Boolean((window as Window & { __TAURI_INTERNALS__?: { metadata?: unknown } }).__TAURI_INTERNALS__?.metadata);
}

onMounted(async () => {
  await store.fetchAppConfig();
  await store.fetchToolStatus();
  if (!store.isToolsReady) {
    ui.openPage('setup');
  }
  await store.refreshRuntimeState();
  unlistenLogs = await store.listenSessionLogs();

  poller = window.setInterval(() => {
    void store.refreshRuntimeState();
  }, 3000);

  if (!hasTauriWindowMetadata()) return;

  unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
    const runningCount = store.sessions.filter((session) => session.status === 'running').length;
    if (runningCount === 0) return;

    event.preventDefault();
    showExitConfirm.value = true;
  });
});

async function handleExitConfirm() {
  await store.stopAllSessions();
  await getCurrentWindow().destroy();
}

async function handleManualRefresh() {
  if (manualRefreshing.value) return;
  manualRefreshing.value = true;
  try {
    await store.refreshRuntimeState();
  } finally {
    manualRefreshing.value = false;
  }
}

onUnmounted(() => {
  if (poller) window.clearInterval(poller);
  unlistenClose?.();
  unlistenLogs?.();
});
</script>

<template>
  <main class="app-shell">
    <AppSidebar />
    <section class="main">
      <Transition name="fade" mode="out-in">
        <div v-if="ui.currentPage === 'devices'" class="page" :key="'devices'">
          <AppHeader title="设备" :subtitle="devicesSubtitle">
            <template #actions>
              <button class="btn btn-ghost" @click="ui.openModal('wireless')">USB 转无线</button>
              <button class="btn btn-ghost" @click="ui.openModal('pair')">ADB Pair</button>
              <button class="btn btn-ghost refresh-button" :disabled="manualRefreshing" @click="handleManualRefresh">
                <svg
                  class="refresh-icon"
                  :class="{ spinning: manualRefreshing }"
                  width="12"
                  height="12"
                  viewBox="0 0 12 12"
                  fill="none"
                  aria-hidden="true"
                >
                  <path d="M10 5.5A4 4 0 1 1 6 2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                  <path d="M7.5 2h-1.5v1.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                刷新
              </button>
            </template>
          </AppHeader>
          <div class="devices-layout">
            <DeviceList />
            <DeviceDetailPanel />
          </div>
        </div>
        <SessionsView v-else-if="ui.currentPage === 'sessions'" :key="'sessions'" />
        <SetupView v-else-if="ui.currentPage === 'setup'" :key="'setup'" />
        <SettingsView v-else :key="'settings'" />
      </Transition>
    </section>
    <ConfirmModal
      v-if="showExitConfirm"
      title="退出 DroidDock"
      :message="`当前有 ${store.sessions.filter(s => s.status === 'running').length} 个投屏会话正在运行，是否关闭全部会话并退出？`"
      confirm-text="确认退出"
      danger
      @confirm="handleExitConfirm"
      @cancel="showExitConfirm = false"
    />
    <PairModal v-if="ui.modal === 'pair'" />
    <WirelessModal v-if="ui.modal === 'wireless'" />
    <ReconnectModal v-if="ui.modal === 'reconnect'" />
  </main>
</template>
