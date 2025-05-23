<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { checkProxyRunning, startProxy } from './commands';
import { DEFAULT_PORT } from './const';
import { listenEvents } from './events';
import { TEvent } from './events/types';
import { useSettings } from './hooks/use-settings';
import { useNetworkStore } from './stores/network';
import { useProxyStore } from './stores/proxy';

const { proxyOnCount, updateProxyPort } = useProxyStore();
const { createChannel } = useNetworkStore();

const { getSettings } = useSettings();

let unlisten: (() => void) | undefined;
onMounted(async () => {
  const { port: runningPort, running_count } = await checkProxyRunning();

  proxyOnCount.value = running_count;
  updateProxyPort(runningPort);

  unlisten = await listenEvents({
    [TEvent.ProxyStarted]: () => {
      proxyOnCount.value++;
    },
    [TEvent.ProxyStopped]: () => {
      proxyOnCount.value--;
    },
  });

  if (running_count <= 0) {
    const initPort = (await getSettings('port')) ?? DEFAULT_PORT;
    startProxy(initPort, createChannel()).then(() => {
      updateProxyPort(initPort);
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
