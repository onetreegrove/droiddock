<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import AppSidebar from './components/AppSidebar.vue';
import AppHeader from './components/AppHeader.vue';
import DeviceList from './components/DeviceList.vue';
import DeviceDetailPanel from './components/DeviceDetailPanel.vue';
import SessionsView from './components/SessionsView.vue';
import SetupView from './components/SetupView.vue';
import SettingsView from './components/SettingsView.vue';
import PairModal from './components/PairModal.vue';
import WirelessModal from './components/WirelessModal.vue';
import { useAppStore } from './stores/app';

const store = useAppStore();
let poller: number | undefined;
let unlistenClose: UnlistenFn | undefined;
const devicesSubtitle = computed(() => `${store.devices.length} 台已发现 · ${store.availableDeviceCount} 台可用`);

onMounted(async () => {
  await store.fetchAppConfig();
  await store.fetchToolStatus();
  await store.refreshDevices();
  await store.refreshSessions();

  poller = window.setInterval(async () => {
    await store.refreshDevices();
    await store.refreshSessions();
    for (const session of store.sessions) {
      if (session.status === 'running') {
        await store.fetchSessionLogs(session.session_id);
      }
    }
  }, 3000);

  unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
    const runningCount = store.sessions.filter((session) => session.status === 'running').length;
    if (runningCount === 0) return;

    event.preventDefault();
    const shouldStop = window.confirm(`当前有 ${runningCount} 个投屏会话正在运行，是否关闭全部会话并退出 DroidDock？`);
    if (!shouldStop) return;

    await store.stopAllSessions();
    await getCurrentWindow().destroy();
  });
});

onUnmounted(() => {
  if (poller) window.clearInterval(poller);
  unlistenClose?.();
});
</script>

<template>
  <main class="app-shell">
    <AppSidebar />
    <section class="main">
      <div v-if="store.currentPage === 'devices'" class="page">
        <AppHeader title="设备" :subtitle="devicesSubtitle">
          <template #actions>
            <button class="btn btn-ghost" @click="store.modal = 'pair'">ADB Pair</button>
            <button class="btn btn-ghost" :disabled="store.busy.devices" @click="store.refreshDevices">
              <svg v-if="!store.busy.devices" width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M10 5.5A4 4 0 1 1 6 2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                <path d="M7.5 2h-1.5v1.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
              {{ store.busy.devices ? '刷新中...' : '刷新' }}
            </button>
          </template>
        </AppHeader>
        <div class="devices-layout">
          <DeviceList />
          <DeviceDetailPanel />
        </div>
      </div>
      <SessionsView v-else-if="store.currentPage === 'sessions'" />
      <SetupView v-else-if="store.currentPage === 'setup'" />
      <SettingsView v-else />
    </section>
    <PairModal v-if="store.modal === 'pair'" />
    <WirelessModal v-if="store.modal === 'wireless'" />
  </main>
</template>
