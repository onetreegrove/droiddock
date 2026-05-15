<script setup lang="ts">
import { useUiStore } from '../stores/ui';

const ui = useUiStore();
</script>

<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast-list">
        <div v-for="toast in ui.toasts" :key="toast.id" :class="['toast-item', `toast-${toast.type}`]">
          <div class="toast-icon">
            <svg v-if="toast.type === 'success'" width="14" height="14" viewBox="0 0 12 12" fill="none">
              <path d="M2.5 6L5 8.5L9.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <svg v-else-if="toast.type === 'error'" width="14" height="14" viewBox="0 0 12 12" fill="none">
              <path d="M2 2L10 10M10 2L2 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            <svg v-else width="14" height="14" viewBox="0 0 12 12" fill="none">
              <circle cx="6" cy="6" r="4.5" stroke="currentColor" stroke-width="1.2" />
              <path d="M6 4V6.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
              <circle cx="6" cy="8.5" r="0.5" fill="currentColor" />
            </svg>
          </div>
          <div class="toast-message">{{ toast.message }}</div>
          <button class="toast-close" @click="ui.removeToast(toast.id)">
            <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
              <path d="M2 2L10 10M10 2L2 10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            </svg>
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 1000;
  display: flex;
  flex-direction: column;
  gap: 10px;
  pointer-events: none;
}

.toast-item {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 240px;
  max-width: 400px;
  padding: 12px 14px;
  border-radius: 10px;
  background: var(--bg3);
  border: 1px solid var(--border2);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  color: var(--t1);
  font-size: 13px;
}

.toast-success {
  border-color: rgba(74, 222, 128, 0.3);
  background: rgba(13, 18, 14, 0.9);
}
.toast-success .toast-icon { color: var(--green); }

.toast-error {
  border-color: rgba(248, 113, 113, 0.3);
  background: rgba(18, 13, 13, 0.9);
}
.toast-error .toast-icon { color: var(--red); }

.toast-info {
  border-color: rgba(61, 217, 235, 0.3);
  background: rgba(13, 17, 18, 0.9);
}
.toast-info .toast-icon { color: var(--acc); }

.toast-icon {
  flex-shrink: 0;
  display: grid;
  place-items: center;
}

.toast-message {
  flex: 1;
  line-height: 1.4;
}

.toast-close {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: grid;
  place-items: center;
  background: transparent;
  color: var(--t3);
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.toast-close:hover {
  background: var(--bg5);
  color: var(--t1);
}

.toast-list-enter-active,
.toast-list-leave-active {
  transition: all 0.3s ease;
}
.toast-list-enter-from {
  opacity: 0;
  transform: translateX(30px);
}
.toast-list-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}
</style>
