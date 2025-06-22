<script setup lang="ts">
import { getRuleContent, updateRuleContent } from '@/commands';
import { useDebounceFn } from '@vueuse/core';
import { Splitter } from 'primevue';
import { ref } from 'vue';

const selectedFileId = ref<number>();
const selectedContent = ref<string>('');

async function onFileSelect(id: number) {
  if (selectedFileId.value === id) {
    return;
  }
  selectedFileId.value = id;
  selectedContent.value = await getRuleContent(id);
}

const onContentUpdate = useDebounceFn(async (content: string) => {
  if (!selectedFileId.value) {
    return;
  }
  await updateRuleContent({ id: selectedFileId.value, content });
}, 800);
</script>

<template>
  <Splitter class="h-full" :gutter-size="6">
    <SplitterPanel :size="30" :min-size="20">
      <PanelCard class="flex flex-col p-2">
        <RuleFileTree @file-select="onFileSelect" />
      </PanelCard>
    </SplitterPanel>

    <SplitterPanel :size="70" :min-size="60">
      <PanelCard class="px-1 py-2">
        <RuleEditor
          v-if="selectedFileId"
          v-model="selectedContent"
          @update:model-value="onContentUpdate"
        />
        <div
          v-else
          class="flex h-full items-center justify-center gap-2 text-xl font-medium"
        >
          <IconLucideMoveLeft class="text-[28px]" />
          Select a file
        </div>
      </PanelCard>
    </SplitterPanel>
  </Splitter>
</template>
