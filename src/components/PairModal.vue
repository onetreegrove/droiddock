<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '../stores/app';

const store = useAppStore();
const host = ref('');
const pairPort = ref('');
const pairingCode = ref('');
const connectPort = ref('');

async function submit() {
  await store.adbPair({
    host: host.value,
    pair_port: Number(pairPort.value),
    pairing_code: pairingCode.value,
    connect_port: connectPort.value ? Number(connectPort.value) : null,
  });
  host.value = '';
  pairPort.value = '';
  pairingCode.value = '';
  connectPort.value = '';
  store.modal = null;
}
</script>

<template>
  <div class="modal-overlay" @click.self="store.modal = null">
    <section class="modal-card">
      <header class="modal-header">
        <div><div class="modal-title">ADB Pair 无线配对</div><div class="modal-subtitle">适用于 Android 11+ 无线调试</div></div>
        <button class="btn btn-ghost" @click="store.modal = null">关闭</button>
      </header>
      <div class="modal-body">
        <div class="modal-note">配对端口和连接端口通常不同，请按手机屏幕分别填写。</div>
        <div class="form-grid">
          <label>配对 IP<input v-model="host" class="field-input" placeholder="192.168.1.100" /></label>
          <label>配对端口<input v-model="pairPort" class="field-input" placeholder="38521" /></label>
        </div>
        <label>配对码（6 位）<input v-model="pairingCode" class="field-input code-input" maxlength="6" placeholder="123456" /></label>
        <div class="form-grid">
          <label>连接 IP<input :value="host" class="field-input" disabled /></label>
          <label>连接端口<input v-model="connectPort" class="field-input" placeholder="39845" /></label>
        </div>
      </div>
      <footer class="modal-footer">
        <button class="btn btn-ghost" @click="store.modal = null">取消</button>
        <button class="btn btn-primary" :disabled="!host || !pairPort || !pairingCode" @click="submit">执行配对并连接</button>
      </footer>
    </section>
  </div>
</template>
