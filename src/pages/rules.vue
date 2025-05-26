<script setup lang="ts">
import { getRuleDirs, upsertRuleDir } from '@/commands';
import { useQuery } from '@tanstack/vue-query';
import { Splitter } from 'primevue';
import { TreeNode } from 'primevue/treenode';
import { computed, nextTick, ref, useTemplateRef } from 'vue';

const { data: dirs, refetch } = useQuery({
  queryKey: [getRuleDirs.name],
  queryFn: getRuleDirs,
});

const creatingDir = ref(false);
const newDirName = ref('');
const creatingInputRef = useTemplateRef<any>('creating-input');

function onCreateDirClick() {
  creatingDir.value = true;
  newDirName.value = '';
  nextTick(() => {
    console.log(creatingInputRef.value);
    creatingInputRef.value?.$el?.focus?.();
  });
}

async function onCreateDir() {
  const newName = newDirName.value.trim();
  if (!newName) {
    creatingDir.value = false;
    return;
  }
  await upsertRuleDir({ name: newName });
  await refetch();
  creatingDir.value = false;
}

const nodes = computed(() => {
  const list = (dirs.value ?? []).map((item) => ({
    key: String(item.id),
    label: item.name,
  })) as TreeNode[];

  if (creatingDir.value) {
    list.push({
      key: 'new',
      label: 'New',
      creatingDir: true,
    });
  }

  return list;
});

const selectedKeys = ref<Record<string, boolean>>({});
const expandedKeys = ref<Record<string, boolean>>({});

function onNodeSelect(node: TreeNode) {
  expandedKeys.value[node.key] = !expandedKeys.value[node.key];
  selectedKeys.value = { [node.key]: true };
}

function onNodeUnselect(node: TreeNode) {
  expandedKeys.value[node.key] = !expandedKeys.value[node.key];
}

function onNodeToggle(node: TreeNode) {
  selectedKeys.value = { [node.key]: true };
}
</script>

<template>
  <Splitter class="h-full" :gutter-size="6">
    <SplitterPanel :size="30" :min-size="20">
      <PanelCard class="p-2">
        <div class="flex items-center justify-between">
          <span class="font-semibold">Rules</span>
          <div class="flex items-center gap-1">
            <Button
              class="p-1!"
              icon="pi"
              severity="contrast"
              variant="text"
              size="small"
            >
              <IconLucideFilePlus />
            </Button>
            <Button
              class="p-1!"
              icon="pi"
              severity="contrast"
              variant="text"
              size="small"
              @click="onCreateDirClick"
            >
              <IconLucideFolderPlus />
            </Button>
          </div>
        </div>
        <div class="pt-2">
          <Tree
            :value="nodes"
            selection-mode="single"
            v-model:expanded-keys="expandedKeys"
            :selection-keys="selectedKeys"
            pt:root:class="p-0!"
            @node-select="onNodeSelect"
            @node-unselect="onNodeUnselect"
            @node-expand="onNodeToggle"
            @node-collapse="onNodeToggle"
          >
            <template #nodeicon>
              <IconLucideFolder class="flex-none text-sm" />
            </template>
            <template #nodetoggleicon="{ expanded }">
              <IconLucideChevronDown v-if="expanded" class="text-sm" />
              <IconLucideChevronRight v-else class="text-sm" />
            </template>
            <template #default="{ node }">
              <InputText
                v-if="node.creatingDir"
                ref="creating-input"
                v-model="newDirName"
                class="py-1! text-xs"
                size="small"
                @blur="onCreateDir"
                @keydown.enter="onCreateDir"
                @keydown.esc="creatingDir = false"
              />
              <template v-else>{{ node.label }}</template>
            </template>
          </Tree>
        </div>
      </PanelCard>
    </SplitterPanel>

    <SplitterPanel :size="70" :min-size="60">
      <PanelCard class="p-2">
        <div>213</div>
      </PanelCard>
    </SplitterPanel>
  </Splitter>
</template>
