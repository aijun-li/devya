<script setup lang="ts">
import { platform } from '@tauri-apps/plugin-os';

const isMacOS = platform() === 'macos';
</script>

<template>
  <div class="flex min-h-[24px]">
    <div class="flex-1" data-tauri-drag-region></div>
    <WindowControls v-if="!isMacOS" />
  </div>
</template>
