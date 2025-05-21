<script setup lang="ts">
import { useNetworkStore } from '@/stores/network';
import { computed, nextTick, ref, useTemplateRef, watchEffect } from 'vue';

interface NetworkItem {
  id: number;
  method: string;
  protocol: string;
  host: string;
  path: string;
  code: number;
}

const { list } = useNetworkStore();

const data = computed<NetworkItem[]>(() =>
  list.value.map((item, index) => {
    const url = new URL(item);

    return {
      id: index + 1,
      method: 'GET',
      protocol: url.protocol.slice(0, -1),
      host: url.hostname,
      path: url.pathname,
      code: 200,
    };
  }),
);

const activeItem = ref<NetworkItem | null>(null);

const splitter = useTemplateRef('splitter');

watchEffect(() => {
  if (activeItem.value) {
    nextTick(() => {
      splitter.value?.resetState();
    });
  }
});
</script>

<template>
  <Splitter ref="splitter" class="h-full" :gutter-size="6">
    <SplitterPanel :size="70" :min-size="50">
      <div class="bg-surface-0 h-full overflow-hidden rounded-lg">
        <DataTable
          class="text-sm"
          :value="data"
          v-model:selection="activeItem"
          :virtual-scroller-options="{ itemSize: 33 }"
          scrollable
          scroll-height="flex"
          data-key="id"
          selection-mode="single"
          striped-rows
          resizable-columns
          size="small"
        >
          <Column field="id" header="ID"></Column>
          <Column field="method" header="Method"></Column>
          <Column field="protocol" header="Protocol"></Column>
          <Column field="host" header="Host"></Column>
          <Column class="max-w-[300px]" field="path" header="Path"></Column>
          <Column field="code" header="Code"></Column>
        </DataTable>
      </div>
    </SplitterPanel>

    <SplitterPanel v-if="activeItem" :size="30" :min-size="30">
      <div class="bg-surface-0 h-full overflow-hidden rounded-lg p-2">
        <div>{{ activeItem.method }}</div>
        <div>{{ activeItem.protocol }}</div>
        <div>{{ activeItem.host }}</div>
        <div class="break-all">{{ activeItem.path }}</div>
        <div>{{ activeItem.code }}</div>
      </div>
    </SplitterPanel>
  </Splitter>
</template>
