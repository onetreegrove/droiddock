<script setup lang="ts">
import { onMounted } from 'vue';
import { useAppStore } from './stores/app';
import Sidebar from './components/Sidebar.vue';
import DiscoverView from './components/DiscoverView.vue';
import DeviceDetailView from './components/DeviceDetailView.vue';
import SettingsView from './components/SettingsView.vue';

const store = useAppStore();

onMounted(async () => {
  await store.fetchToolStatus();
  await store.refreshDevices();
  await store.refreshSessions();
  
  if (!store.selectedSerial) {
    store.selectedSerial = 'discover';
  }

  // Poll for devices and sessions
  setInterval(async () => {
    await store.refreshDevices();
    await store.refreshSessions();
    
    // Fetch logs for active sessions
    for (const session of store.sessions) {
      if (session.status === 'running') {
        await store.fetchSessionLogs(session.session_id);
      }
    }
  }, 3000);
});
</script>

<template>
  <main class="app-shell">
    <Sidebar />

    <section class="main-area">
      <header class="top-bar">
        <div class="status-indicator">
          <span>工具状态：</span>
          <div :class="['dot', { ready: store.isToolsReady }]"></div>
        </div>
        <button class="secondary" @click="store.refreshDevices" :disabled="store.loading">
          {{ store.loading ? '刷新中...' : '刷新设备列表' }}
        </button>
      </header>

      <div class="view-container">
        <DiscoverView v-if="store.selectedSerial === 'discover'" />
        <SettingsView v-else-if="store.selectedSerial === 'settings'" />
        <DeviceDetailView v-else-if="store.selectedSerial" :serial="store.selectedSerial" />
        <div v-else class="view-content">
          <h1>选择设备</h1>
          <p>请在侧边栏选择一个设备，或前往“发现与配对”连接新设备。</p>
        </div>
      </div>
    </section>
  </main>
</template>

<style>
.view-container {
  height: 100%;
  overflow: hidden;
}
</style>
