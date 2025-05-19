<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { startProxy } from './commands';
import { listenEvents } from './events';
import { TEvent } from './events/types';
import { useProxyStore } from './stores/proxy';

const { proxyOnCount } = useProxyStore();

let unlisten: (() => void) | undefined;
onMounted(async () => {
  unlisten = await listenEvents({
    [TEvent.ProxyStarted]: () => {
      proxyOnCount.value++;
    },
    [TEvent.ProxyStopped]: () => {
      proxyOnCount.value--;
    },
  });

  startProxy(7777);
});
onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="flex h-screen w-screen flex-col bg-stone-200">
    {{ proxyOnCount }}
    <TitleBar />

    <div class="flex min-h-0 flex-1 overflow-auto">
      <Sidebar />

      <div class="h-full min-w-0 flex-1 overflow-auto pt-1 pr-1.5 pb-1.5 pl-0">
        <RouterView />
      </div>
    </div>

    <BottomBar />
  </div>
</template>
