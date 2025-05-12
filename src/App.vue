<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query';
import Column from 'primevue/column';
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

const mockData = Array.from({ length: 5 }).map((_, index) => ({
  id: index + 1,
  method: 'GET',
  protocol: 'https',
  host: 'www.google.com',
  path: '/search',
  code: 200,
}));
</script>

<template>
  <main class="flex h-screen w-screen flex-col">
    <div class="flex flex-1 overflow-auto">
      <div
        class="border-surface-200 flex h-full w-13 flex-col items-center gap-4 border-r border-solid py-4"
      >
        <ContrastButton icon="pi" variant="text" raised>
          <IconLucideSquareActivity />
        </ContrastButton>
        <ContrastButton icon="pi" variant="text">
          <IconLucideFileJson />
        </ContrastButton>
        <ContrastButton icon="pi" variant="text">
          <IconLucideSettings />
        </ContrastButton>
      </div>

      <div class="h-full flex-1 overflow-auto p-2">
        <DataTable :value="mockData" striped-rows>
          <Column field="id" header="ID"></Column>
          <Column field="method" header="Method"></Column>
          <Column field="protocol" header="Protocol"></Column>
          <Column field="host" header="Host"></Column>
          <Column field="path" header="Path"></Column>
          <Column field="code" header="Code"></Column>
        </DataTable>
      </div>
    </div>

    <div
      class="text-surface-500 border-surface-200 sticky bottom-0 flex flex-col border-t border-solid text-xs"
    >
      <div class="flex items-center justify-between px-2 py-0.5">
        <div class="flex items-center">
          <div class="mx-2 h-1.5 w-1.5 rounded-full bg-green-700" />
          <SecondaryButton class="p-1! text-xs!" variant="text" size="small">
            Listening on 127.0.0.1:8899
          </SecondaryButton>
          <ToggleSwitch :model-value="true" class="scale-70" />
        </div>
        <SecondaryButton
          class="flex items-center gap-1! p-1! text-xs!"
          :class="[caInstalled ? 'text-green-700!' : 'cursor-pointer']"
          variant="text"
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
