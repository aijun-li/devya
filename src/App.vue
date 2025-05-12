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

    <div class="text-surface-500 sticky bottom-0 flex flex-col text-xs">
      <Divider class="m-0!" />

      <div class="flex items-center justify-between px-2 py-0.5">
        <div class="flex items-center">
          <div class="mx-2 h-1.5 w-1.5 rounded-full bg-green-700" />
          <SecondaryButton class="p-1! text-xs!" text size="small"
            >Listening on 127.0.0.1:8899</SecondaryButton
          >
          <ToggleSwitch :model-value="true" class="scale-70" />
        </div>
        <SecondaryButton
          class="flex items-center gap-1! p-1! text-xs!"
          :class="[caInstalled ? 'text-green-700!' : 'cursor-pointer']"
          text
          size="small"
          :disabled="caInstalled"
          @click="onHttpsClick"
        >
          <IconMdiHttps />
          TLS {{ caInstalled ? 'Enabled' : 'Disabled' }}
        </SecondaryButton>
      </div>
    </div>
  </main>
</template>
