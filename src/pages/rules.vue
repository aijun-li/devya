<script setup lang="ts">
import { Splitter } from 'primevue';
</script>

<template>
  <Splitter class="h-full" :gutter-size="6">
    <SplitterPanel :size="30" :min-size="20">
      <PanelCard class="flex flex-col p-2">
        <RuleFileTree />
      </PanelCard>
    </SplitterPanel>

    <SplitterPanel :size="70" :min-size="60">
      <PanelCard class="px-1 py-2">
        <RuleEditor />
      </PanelCard>
    </SplitterPanel>
  </Splitter>
</template>
