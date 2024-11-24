<script setup lang="ts">
import ContentCard from '@/components/ContentCard.vue';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { useNetworkStore } from '@/stores/network';
import { computed } from 'vue';

const { requestList } = useNetworkStore();

const list = computed(() =>
  requestList.value.map((item) => {
    const { protocol, hostname, port, pathname } = new URL(item.content);
    const parsed = decodeURIComponent(pathname).split(' -> ');

    return {
      id: item.id,
      protocol,
      hostname,
      port,
      pathname: parsed[0],
      status: (parsed[1] ?? '').split(' ', 1)[0],
    };
  }),
);
</script>

<template>
  <ContentCard class="relative" content-class="pt-2" :with-scroll="false">
    <Table class="table-fixed" wrapper-class="h-full">
      <TableHeader>
        <TableRow class="sticky top-0 border-none bg-white [&_th]:shadow-[inset_0_-1px_hsl(var(--border))]">
          <TableHead class="w-16">Status</TableHead>
          <TableHead class="w-20">Protocol</TableHead>
          <TableHead class="w-64">Hostname</TableHead>
          <TableHead>Pathname</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-for="item in list" :key="item.id">
          <TableCell>{{ item.status }}</TableCell>
          <TableCell>{{ item.protocol }}</TableCell>
          <TableCell class="truncate">{{ item.hostname }}</TableCell>
          <TableCell class="truncate">{{ item.pathname }}</TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </ContentCard>
</template>
