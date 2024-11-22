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
  <ContentCard>
    <Table>
      <TableHeader class="sticky top-0">
        <TableRow>
          <TableHead>Status</TableHead>
          <TableHead>Protocol</TableHead>
          <TableHead>Hostname</TableHead>
          <TableHead>Pathname</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-for="item in list" :key="item.id">
          <TableCell>{{ item.status }}</TableCell>
          <TableCell>{{ item.protocol }}</TableCell>
          <TableCell>{{ item.hostname }}</TableCell>
          <TableCell>{{ item.pathname }}</TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </ContentCard>
</template>
