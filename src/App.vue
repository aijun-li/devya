<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { checkProxyRunning, startProxy } from './commands';
import { listenEvents } from './events';
import { TEvent } from './events/types';
import { useNetworkStore } from './stores/network';
import { useProxyStore } from './stores/proxy';

const { proxyOnCount, port } = useProxyStore();
const { createChannel } = useNetworkStore();

let unlisten: (() => void) | undefined;
onMounted(async () => {
  const { port: runningPort, running_count } = await checkProxyRunning();

  proxyOnCount.value = running_count;
  port.value = runningPort;

  unlisten = await listenEvents({
    [TEvent.ProxyStarted]: () => {
      proxyOnCount.value++;
    },
    [TEvent.ProxyStopped]: () => {
      proxyOnCount.value--;
    },
  });

  if (running_count <= 0) {
    startProxy(7777, createChannel()).then(() => {
      port.value = 7777;
    });
  }
});
onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="bg-surface-200 flex h-screen w-screen flex-col">
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
