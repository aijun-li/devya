<script setup lang="ts">
import { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMounted, onUnmounted, ref } from 'vue';

const maximized = ref(false);

const appWindow = getCurrentWindow();

let unlistenResize: UnlistenFn | undefined;
onMounted(async () => {
  unlistenResize = await appWindow.onResized(async () => {
    const isMaximized = await appWindow.isMaximized();
    maximized.value = isMaximized;
  });
});
onUnmounted(() => {
  unlistenResize?.();
});

function onMinimize() {
  appWindow.minimize();
}

function onMaximize() {
  appWindow.toggleMaximize().then(() => {
    maximized.value = !maximized.value;
  });
}

function onClose() {
  appWindow.close();
}
</script>

<template>
  <div>
    <button
      class="inline-flex h-8 w-[46px] cursor-default items-center justify-center rounded-none bg-transparent text-black/90 hover:bg-black/[.05] active:bg-black/[.03] dark:text-white dark:hover:bg-white/[.06] dark:active:bg-white/[.04]"
      @click="onMinimize"
    >
      <IconFluentMinimize16Regular />
    </button>
    <button
      class="inline-flex h-8 w-[46px] cursor-default items-center justify-center rounded-none bg-transparent text-black/90 hover:bg-black/[.05] active:bg-black/[.03] dark:text-white dark:hover:bg-white/[.06] dark:active:bg-white/[.04]"
      @click="onMaximize"
    >
      <IconFluentMaximize16Regular v-if="!maximized" />
      <IconFluentSquareMultiple16Regular v-else />
    </button>
    <button
      class="inline-flex h-8 w-[46px] cursor-default items-center justify-center rounded-none bg-transparent text-black/90 hover:bg-[#c42b1c] hover:text-white active:bg-[#c42b1c]/90 dark:text-white"
      @click="onClose"
    >
      <IconFluentDismiss16Regular />
    </button>
  </div>
</template>
