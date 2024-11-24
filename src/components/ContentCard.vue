<script setup lang="ts">
import { ScrollArea } from './ui/scroll-area';
import { Separator } from './ui/separator';

interface Props {
  withScroll?: boolean;
  contentClass?: string;
}

const { withScroll = true } = defineProps<Props>();

const slots = defineSlots<{ default: () => unknown; header?: () => unknown }>();
</script>

<template>
  <div class="flex h-full flex-col rounded-lg bg-white shadow">
    <template v-if="slots.header">
      <div class="px-4 py-2 font-semibold">
        <slot name="header" />
      </div>
      <Separator />
    </template>

    <ScrollArea v-if="withScroll" class="flex-1 p-4" :class="contentClass">
      <slot />
    </ScrollArea>
    <div v-else class="min-h-0 flex-1 p-4" :class="contentClass">
      <slot />
    </div>
  </div>
</template>
