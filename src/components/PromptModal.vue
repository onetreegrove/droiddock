<script setup lang="ts">
import { ref, onMounted } from 'vue';

const props = defineProps<{
  title: string;
  message: string;
  defaultValue?: string;
  confirmText?: string;
  cancelText?: string;
  placeholder?: string;
}>();

const emit = defineEmits(['confirm', 'cancel']);
const value = ref(props.defaultValue || '');
const inputRef = ref<HTMLInputElement | null>(null);

onMounted(() => {
  inputRef.value?.focus();
  inputRef.value?.select();
});

function handleConfirm() {
  emit('confirm', value.value);
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('cancel')">
    <div class="modal-card prompt-modal">
      <div class="modal-header">
        <div class="modal-title">{{ title }}</div>
        <button class="modal-close" @click="emit('cancel')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M10.5 3.5L3.5 10.5M3.5 3.5L10.5 10.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
      <div class="modal-body">
        <p class="modal-note">{{ message }}</p>
        <input
          ref="inputRef"
          v-model="value"
          class="field-input"
          :placeholder="placeholder"
          @keyup.enter="handleConfirm"
        />
      </div>
      <div class="modal-footer">
        <button class="btn btn-ghost" @click="emit('cancel')">{{ cancelText || '取消' }}</button>
        <button class="btn btn-primary" @click="handleConfirm">
          {{ confirmText || '确定' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.prompt-modal {
  width: 400px;
}
.modal-note {
  margin-bottom: 12px;
}
</style>
