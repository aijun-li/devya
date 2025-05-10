<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';

const greetMsg = ref('');
const name = ref('devya');

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke('greet', { name: name.value });
}
</script>

<template>
  <main
    class="flex h-screen w-screen flex-col items-center justify-center gap-2"
  >
    <div>{{ greetMsg }}</div>
    <Button @click="greet">Greet</Button>
  </main>
</template>
