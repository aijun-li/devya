<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import IconLucideFileJson from '~icons/lucide/file-json.vue';
import IconLucideSettings from '~icons/lucide/settings.vue';
import IconLucideSquareActivity from '~icons/lucide/square-activity.vue';

const groups = [
  [
    {
      key: 'network',
      icon: IconLucideSquareActivity,
      path: '/network',
    },
    {
      key: 'rules',
      icon: IconLucideFileJson,
      path: '/rules',
    },
  ],
  [
    {
      key: 'settings',
      icon: IconLucideSettings,
      path: '/settings',
    },
  ],
];

const route = useRoute();
const router = useRouter();
</script>

<template>
  <div class="flex h-full flex-col justify-between px-1 py-4">
    <div
      v-for="(group, index) in groups"
      :key="index"
      class="flex flex-col items-center gap-4"
    >
      <Button
        v-for="config in group"
        :key="config.key"
        icon="pi"
        severity="contrast"
        variant="text"
        :raised="route.path === config.path"
        @click="router.push(config.path)"
      >
        <component :is="config.icon" />
      </Button>
    </div>
  </div>
</template>
