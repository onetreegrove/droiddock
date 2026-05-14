<script setup lang="ts">
import { computed } from 'vue';
import { useUiStore } from '../stores/ui';
import type { ManagedDevice } from '../types/app';

type BannerTone = 'warn' | 'error';
type BannerAction = 'reconnect' | null;

const props = defineProps<{
  device: ManagedDevice;
}>();

const ui = useUiStore();

const banner = computed<{
  tone: BannerTone;
  title: string;
  message: string;
  action: BannerAction;
} | null>(() => {
  if (props.device.presence === 'offline') {
    if (props.device.connection === 'wireless') {
      return {
        tone: 'error',
        title: '无线设备已离线',
        message: props.device.endpoint
          ? '手机无线调试端口可能已变化。可先重连恢复连接，或使用底部“重连投屏”继续启动。'
          : '缺少保存的无线连接地址，请重新配对或通过 USB 转无线后再启动投屏。',
        action: props.device.endpoint ? 'reconnect' : null,
      };
    }

    return {
      tone: 'error',
      title: 'USB 设备已离线',
      message: '请重新插入 USB 数据线，确认手机已解锁并允许 USB 调试，设备列表会自动刷新。',
      action: null,
    };
  }

  if (props.device.state === 'unauthorized') {
    return {
      tone: 'warn',
      title: '设备待授权',
      message: '请解锁手机，在 USB 调试授权弹窗中勾选“一律允许使用这台电脑进行调试”，然后点击允许。',
      action: null,
    };
  }

  return null;
});

function handleReconnect() {
  if (!props.device.endpoint) return;
  ui.openReconnectModal(props.device.serial, props.device.endpoint, props.device.wireless_source, false);
}
</script>

<template>
  <div v-if="banner" class="status-banner" :class="banner.tone">
    <div class="banner-icon" aria-hidden="true">
      <svg
        v-if="banner.tone === 'warn'"
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path
          d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0zM12 9v4M12 17h.01"
        />
      </svg>
      <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
    </div>
    <div class="banner-content">
      <div class="banner-title">{{ banner.title }}</div>
      <div class="banner-message">{{ banner.message }}</div>
    </div>
    <button v-if="banner.action === 'reconnect'" class="btn btn-ghost compact-button banner-btn" @click="handleReconnect">
      重连
    </button>
  </div>
</template>

<style scoped>
.status-banner {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  margin-bottom: 18px;
  border: 1px solid transparent;
  border-radius: 10px;
}

.status-banner.warn {
  border-color: rgba(251, 191, 36, 0.2);
  background: var(--yellow-d);
  color: var(--yellow);
}

.status-banner.error {
  border-color: rgba(248, 113, 113, 0.2);
  background: var(--red-d);
  color: var(--red);
}

.banner-icon {
  flex: 0 0 auto;
  padding-top: 2px;
}

.banner-content {
  flex: 1;
  min-width: 0;
}

.banner-title {
  margin-bottom: 2px;
  font-weight: 600;
}

.banner-message {
  color: var(--t1);
  font-size: 13px;
  line-height: 1.5;
}

.banner-btn {
  flex: 0 0 auto;
  align-self: center;
  color: inherit;
}

@media (max-width: 760px) {
  .status-banner {
    flex-wrap: wrap;
  }

  .banner-btn {
    margin-left: 30px;
  }
}
</style>
