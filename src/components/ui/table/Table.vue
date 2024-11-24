<script setup lang="ts">
import { cn } from '@/utils/shadcn';
import type { HTMLAttributes } from 'vue';
import { ScrollArea } from '../scroll-area';

const props = defineProps<{
  class?: HTMLAttributes['class'];
  wrapperClass?: HTMLAttributes['class'];
}>();
</script>

<template>
  <ScrollArea as-child>
    <div class="relative w-full" :class="wrapperClass">
      <table :class="cn('w-full caption-bottom text-sm', props.class)">
        <slot />
      </table>
    </div>
  </ScrollArea>
</template>
