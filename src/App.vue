<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query';
import { checkCaInstalled, installCa } from './commands';

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
  <main class="flex h-screen w-screen flex-col">
    <div class="flex flex-1 items-center justify-center overflow-auto">
      <Button>123</Button>
    </div>

    <div class="sticky bottom-0 flex items-center justify-between px-2 py-1">
      <div>Listening on 127.0.0.1:8899</div>
      <div
        class="flex items-center gap-2"
        :class="[caInstalled ? 'text-green-600' : 'cursor-pointer']"
        @click="onHttpsClick"
      >
        <IconMdiHttps />
      </div>
    </div>
  </main>
</template>
