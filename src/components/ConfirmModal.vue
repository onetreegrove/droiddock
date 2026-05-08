<script setup lang="ts">
defineProps<{
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}>();

const emit = defineEmits(['confirm', 'cancel']);
</script>

<template>
  <div class="modal-overlay" @click.self="emit('cancel')">
    <div class="modal-card confirm-modal">
      <div class="modal-header">
        <div class="modal-title">{{ title }}</div>
        <button class="modal-close" @click="emit('cancel')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M10.5 3.5L3.5 10.5M3.5 3.5L10.5 10.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
      <div class="modal-body">
        <p class="modal-message">{{ message }}</p>
      </div>
      <div class="modal-footer">
        <button class="btn btn-ghost" @click="emit('cancel')">{{ cancelText || '取消' }}</button>
        <button :class="['btn', danger ? 'btn-danger' : 'btn-primary']" @click="emit('confirm')">
          {{ confirmText || '确认' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.confirm-modal {
  width: 400px;
}
.modal-message {
  font-size: 14px;
  line-height: 1.6;
  color: var(--t2);
  margin: 0;
}
</style>
