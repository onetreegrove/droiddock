<script setup lang="ts">
import { computed } from 'vue';
import StatusChip from './StatusChip.vue';
import type { ManagedDevice } from '../types/app';

const props = defineProps<{
  device: ManagedDevice;
  ipAddress: string | null;
  connectionLabel: string;
}>();

const emit = defineEmits<{
  (e: 'edit-alias'): void;
}>();

const displayName = computed(() => props.device.alias || props.device.display_name || props.device.model || '未知设备');
const networkLabel = computed(() => {
  if (props.device.connection !== 'wireless') return null;
  return props.device.endpoint || props.ipAddress || null;
});
const stateLabel = computed(() => {
  if (props.device.presence === 'offline') return '离线';
  if (props.device.state === 'device') return '可用';
  if (props.device.state === 'unauthorized') return '待授权';
  return '离线';
});
const stateTone = computed(() => {
  if (props.device.presence === 'offline') return 'red';
  if (props.device.state === 'device') return 'green';
  if (props.device.state === 'unauthorized') return 'yellow';
  return 'red';
});
</script>

<template>
  <div class="device-hero-new">
    <div class="hero-icon-box" :title="connectionLabel">
      <svg width="28" height="28" viewBox="0 0 20 20" fill="none" aria-hidden="true">
        <rect x="3.5" y="1.5" width="13" height="17" rx="2.5" stroke="currentColor" stroke-width="1.3" />
        <circle cx="10" cy="15.5" r="1.2" fill="currentColor" />
        <rect x="6.5" y="4" width="7" height="1" rx=".5" fill="currentColor" opacity=".4" />
      </svg>
      <div class="connection-badge" aria-hidden="true">
        <svg
          v-if="device.connection === 'usb'"
          width="10"
          height="10"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
        >
          <path d="M12 2v20M2 12h20" />
        </svg>
        <svg v-else width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
          <path
            d="M5 12.55a11 11 0 0 1 14.08 0M1.42 9a16 16 0 0 1 21.16 0M8.59 16.11a6 6 0 0 1 6.82 0M12 20h.01"
          />
        </svg>
      </div>
    </div>

    <div class="hero-info">
      <div class="hero-primary-row">
        <h1 class="hero-alias" :title="`${displayName}，点击修改别名`" @click="emit('edit-alias')">
          {{ displayName }}
        </h1>
        <button class="btn-icon-sm" title="编辑别名" @click="emit('edit-alias')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <path
              d="M2.5 11.5v-2l6-6 2 2-6 6h-2zM9.5 2.5l2 2"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linecap="round"
            />
          </svg>
        </button>
        <StatusChip :tone="stateTone" :label="stateLabel" dot />
      </div>

      <div class="hero-secondary-row">
        <span class="secondary-item">{{ device.model || '-' }}</span>
        <span class="dot-separator">·</span>
        <span class="secondary-item">{{ connectionLabel }}</span>
        <template v-if="networkLabel">
          <span class="dot-separator">·</span>
          <span class="secondary-item mono">{{ networkLabel }}</span>
        </template>
        <span class="dot-separator">·</span>
        <span class="secondary-item mono">{{ device.serial }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.device-hero-new {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 20px;
}

.hero-icon-box {
  position: relative;
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border: 1px solid var(--border2);
  border-radius: 12px;
  background: var(--bg3);
  color: var(--acc);
}

.connection-badge {
  position: absolute;
  right: -4px;
  bottom: -4px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: 2px solid var(--bg);
  border-radius: 999px;
  background: var(--bg5);
  color: var(--t2);
}

.hero-info {
  flex: 1;
  min-width: 0;
}

.hero-primary-row {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  margin-bottom: 5px;
}

.hero-alias {
  min-width: 0;
  margin: 0;
  overflow: hidden;
  font-size: 22px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}

.hero-primary-row :deep(.chip) {
  flex: 0 0 auto;
}

.hero-secondary-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 8px;
  color: var(--t3);
  font-size: 13px;
  line-height: 1.5;
}

.secondary-item {
  min-width: 0;
  max-width: 100%;
  overflow-wrap: anywhere;
}

.dot-separator {
  opacity: 0.5;
}

@media (max-width: 760px) {
  .device-hero-new {
    align-items: flex-start;
    gap: 14px;
  }

  .hero-primary-row {
    flex-wrap: wrap;
  }
}
</style>
