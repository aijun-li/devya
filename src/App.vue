<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';

const greetMsg = ref<boolean>();

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  try {
    greetMsg.value = await invoke<boolean>('check_ca_installed');
  } catch (e) {
    console.error(e);
  }
}

async function installCA() {
  try {
    greetMsg.value = await invoke('install_ca');
  } catch (e) {
    console.error(e);
  }
}
</script>

<template>
  <main
    class="flex h-screen w-screen flex-col items-center justify-center gap-2"
  >
    <div v-if="greetMsg !== undefined">{{ greetMsg }}</div>
    <Button @click="greet">Greet</Button>
    <Button @click="installCA">Install CA</Button>
  </main>
</template>
