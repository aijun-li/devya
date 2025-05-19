<script setup lang="ts">
import { checkCaInstalled, installCa } from '@/commands';
import { useProxyStore } from '@/stores/proxy';
import { useQuery } from '@tanstack/vue-query';

const { isProxyOn } = useProxyStore();

const { data: caInstalled, refetch: reCheckCa } = useQuery({
  queryKey: [checkCaInstalled.name],
  queryFn: checkCaInstalled,
});

async function onHttpsClick() {
  if (caInstalled.value) {
    return;
  }
  await installCa();
  await reCheckCa();
}
</script>

<template>
  <div
    class="text-surface-500 sticky bottom-0 flex flex-col bg-stone-300 text-xs"
  >
    <div class="flex items-center justify-between px-2 py-0.5">
      <div class="flex items-center">
        <div
          class="mr-1 ml-2 h-1.5 w-1.5 rounded-full"
          :class="[isProxyOn ? 'bg-green-700' : 'bg-red-700']"
        />
        <Button
          class="p-1! py-0.5! text-xs!"
          severity="secondary"
          variant="text"
          size="small"
        >
          {{ isProxyOn ? 'Listening on 127.0.0.1:7777' : 'Proxy stopped' }}
        </Button>
      </div>
      <Button
        class="flex items-center gap-1! p-1! text-xs!"
        :class="[caInstalled ? 'text-green-700!' : 'cursor-pointer']"
        variant="text"
        size="small"
        :disabled="caInstalled"
        @click="onHttpsClick"
      >
        <IconMdiHttps />
        TLS {{ caInstalled ? 'Enabled' : 'Disabled' }}
      </Button>
    </div>
  </div>
</template>
