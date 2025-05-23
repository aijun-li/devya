<script setup lang="ts">
import { appDataDir } from '@tauri-apps/api/path';
import { revealItemInDir } from '@tauri-apps/plugin-opener';

async function openDataDir() {
  const dirPath = await appDataDir();
  revealItemInDir(dirPath);
}
</script>

<template>
  <div class="h-full">
    <PanelCard class="p-2">
      <div>Settings</div>
      <div>
        <Button @click="openDataDir">Open Data Dir</Button>
      </div>
    </PanelCard>
  </div>
</template>
