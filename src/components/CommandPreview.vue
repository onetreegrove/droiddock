<script setup lang="ts">
defineProps<{ command: string }>();
</script>

<template>
  <div class="command-preview">
    <div class="command-label">命令预览</div>
    <div class="command-text mono">{{ command }}</div>
  </div>
</template>
