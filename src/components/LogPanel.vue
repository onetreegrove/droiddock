<script setup lang="ts">
import type { SessionLogLine } from '../types/app';

defineProps<{ lines: SessionLogLine[] }>();
</script>

<template>
  <div class="log-panel">
    <div v-for="line in lines" :key="`${line.timestamp}-${line.message}`" :class="['log-line', line.level]">
      [{{ new Date(line.timestamp * 1000).toLocaleTimeString() }}] {{ line.message }}
    </div>
  </div>
</template>
