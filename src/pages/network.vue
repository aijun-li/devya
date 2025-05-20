<script setup lang="ts">
import { useNetworkStore } from '@/stores/network';
import { computed } from 'vue';

const { list } = useNetworkStore();

const data = computed(() =>
  list.value.map((item, index) => {
    const url = new URL(item);

    return {
      id: index + 1,
      method: 'GET',
      protocol: url.protocol,
      host: url.hostname,
      path: url.pathname,
      code: 200,
    };
  }),
);
</script>

<template>
  <Splitter class="h-full" :gutter-size="6">
    <SplitterPanel :size="70" :min-size="60">
      <div class="bg-surface-0 h-full overflow-hidden rounded-lg">
        <DataTable class="text-sm" :value="data" striped-rows size="small">
          <Column field="id" header="ID"></Column>
          <Column field="method" header="Method"></Column>
          <Column field="protocol" header="Protocol"></Column>
          <Column field="host" header="Host"></Column>
          <Column field="path" header="Path"></Column>
          <Column field="code" header="Code"></Column>
        </DataTable>
      </div>
    </SplitterPanel>

    <SplitterPanel :size="30" :min-size="20">
      <div class="bg-surface-0 h-full overflow-hidden rounded-lg"></div>
    </SplitterPanel>
  </Splitter>
</template>
