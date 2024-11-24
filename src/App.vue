<script setup lang="ts">
import AppSidebar from './components/AppSidebar.vue';
import { Toaster } from './components/ui/sonner';
import { TooltipProvider } from './components/ui/tooltip';
import { useNetworkStore } from './stores/network';
import { handleError } from './utils/error';

const { port, startProxy, stopProxy } = useNetworkStore();

async function initProxy() {
  try {
    await stopProxy();
    await startProxy({ port: port.value });
  } catch (error) {
    handleError(error);
  }
}

initProxy();
</script>

<template>
  <TooltipProvider :delay-duration="500">
    <div class="flex h-screen w-screen bg-gray-100 pb-2 pr-2">
      <AppSidebar />

      <main class="min-w-0 flex-1">
        <RouterView />
      </main>
    </div>

    <Toaster />
  </TooltipProvider>
</template>
