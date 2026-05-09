<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '../stores/app';
import { useUiStore } from '../stores/ui';
import { buildPairRequest } from '../domain/wireless';

const store = useAppStore();
const ui = useUiStore();
const pairHost = ref('');
const pairPort = ref('');
const pairingCode = ref('');
const connectHost = ref('');
const connectPort = ref('');
const connectHostEdited = ref(false);
const errorMessage = ref('');

function updatePairHost(value: string) {
  pairHost.value = value;
  if (!connectHostEdited.value) {
    connectHost.value = value;
  }
}

function updateConnectHost(value: string) {
  connectHostEdited.value = true;
  connectHost.value = value;
}

async function submit() {
  errorMessage.value = '';
  try {
    await store.adbPair(
      buildPairRequest({
        pairHost: pairHost.value,
        pairPort: pairPort.value,
        pairingCode: pairingCode.value,
        connectHost: connectHost.value,
        connectPort: connectPort.value,
      }),
    );
    pairHost.value = '';
    pairPort.value = '';
    pairingCode.value = '';
    connectHost.value = '';
    connectPort.value = '';
    connectHostEdited.value = false;
    ui.closeModal();
  } catch (error) {
    errorMessage.value = String(error instanceof Error ? error.message : error);
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="ui.closeModal()">
    <section class="modal-card">
      <header class="modal-header">
        <div><div class="modal-title">ADB Pair 无线配对</div><div class="modal-subtitle">适用于 Android 11+ 无线调试</div></div>
        <button class="modal-close" aria-label="关闭" @click="ui.closeModal()">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M2 2l8 8M10 2L2 10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </button>
      </header>
      <div class="modal-body">
        <div class="modal-section-title">输入配对信息</div>
        <div class="modal-note">配对端口和连接端口是两个不同的端口，请仔细核对手机屏幕</div>
        <div class="form-grid">
          <label>配对 IP<input :value="pairHost" class="field-input" placeholder="192.168.1.100" @input="updatePairHost(($event.target as HTMLInputElement).value)" /></label>
          <label>配对端口<input v-model="pairPort" class="field-input" placeholder="38521" /></label>
        </div>
        <label>配对码（6 位）<input v-model="pairingCode" class="field-input code-input" maxlength="6" placeholder="123456" /></label>
        <div class="modal-note">配对码不会被保存，每次配对需要重新生成</div>
        <div class="modal-divider"></div>
        <div class="modal-section-title">连接端口</div>
        <div class="form-grid">
          <label>连接 IP<input :value="connectHost" class="field-input" placeholder="192.168.1.100" @input="updateConnectHost(($event.target as HTMLInputElement).value)" /></label>
          <label>连接端口<input v-model="connectPort" class="field-input" placeholder="39845" /></label>
        </div>
        <div v-if="errorMessage" class="modal-error">{{ errorMessage }}</div>
      </div>
      <footer class="modal-footer">
        <button class="btn btn-ghost" @click="ui.closeModal()">取消</button>
        <button class="btn btn-primary" :disabled="!pairHost || !pairPort || !pairingCode" @click="submit">执行配对并连接</button>
      </footer>
    </section>
  </div>
</template>
