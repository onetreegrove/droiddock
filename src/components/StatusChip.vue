<script setup lang="ts">
defineProps<{
  tone: 'green' | 'yellow' | 'red' | 'blue' | 'gray';
  label: string;
  dot?: boolean;
}>();
</script>

<template>
  <span :class="['chip', `chip-${tone}`]">
    <span v-if="dot" :class="['dot', `dot-${tone}`]"></span>
    {{ label }}
  </span>
</template>
