<script setup lang="ts">
defineProps<{
  title: string;
  subtitle?: string;
}>();
</script>

<template>
  <header class="page-header" data-tauri-drag-region>
    <div data-tauri-drag-region>
      <div class="page-title" data-tauri-drag-region>{{ title }}</div>
      <div v-if="subtitle" class="page-subtitle" data-tauri-drag-region>{{ subtitle }}</div>
    </div>
    <div class="header-actions">
      <slot name="actions"></slot>
    </div>
  </header>
</template>
