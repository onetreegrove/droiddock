<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import AppSidebar from './components/AppSidebar.vue';
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
});

onUnmounted(() => {
  if (poller) window.clearInterval(poller);
});
</script>

<template>
  <main class="app-shell">
    <AppSidebar />
    <section class="main">
      <div v-if="store.currentPage === 'devices'" class="page">
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
