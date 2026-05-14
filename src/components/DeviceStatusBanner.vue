<script setup lang="ts">
import { computed } from 'vue';
import { statusBanner } from '../domain/deviceDetail';
import type { ManagedDevice } from '../types/app';

const props = defineProps<{
  device: ManagedDevice;
}>();

const banner = computed(() => statusBanner(props.device));
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

@media (max-width: 760px) {
  .status-banner {
    flex-wrap: wrap;
  }
}
</style>
