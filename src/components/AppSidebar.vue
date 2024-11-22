<script setup lang="ts">
import { ArrowDownUp, ListTodo, Settings } from 'lucide-vue-next';
import { RouterLink } from 'vue-router';

const menuItems = [
  { title: 'Network', link: '/network', icon: ArrowDownUp },
  { title: 'Rules', link: '/rules', icon: ListTodo },
  { title: 'Settings', link: '/settings', icon: Settings },
];
</script>

<template>
  <div class="flexflex-col px-2 py-4">
    <h1 class="pl-4 text-xl font-bold">Devya</h1>

    <div class="mt-8 flex flex-col gap-2 text-sm">
      <RouterLink
        v-for="item in menuItems"
        :key="item.title"
        class="flex w-full items-center rounded-lg p-4 pr-8 font-semibold hover:bg-gray-200"
        active-class="bg-gray-200"
        :to="item.link"
      >
        <component :is="item.icon" class="mr-4" />
        {{ item.title }}
      </RouterLink>
    </div>
  </div>
</template>
