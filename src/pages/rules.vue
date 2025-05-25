<script setup lang="ts">
import { getRuleDirs, upsertRuleDir } from '@/commands';
import { useQuery } from '@tanstack/vue-query';
import { TreeNode } from 'primevue/treenode';
import { computed, ref } from 'vue';

const { data: dirs, refetch } = useQuery({
  queryKey: [getRuleDirs.name],
  queryFn: getRuleDirs,
});

async function onCreateDir() {
  await upsertRuleDir({ name: 'test' });
  await refetch();
}

const nodes = computed(() => {
  const list = dirs.value ?? [];

  return list.map((item) => ({
    key: String(item.id),
    label: item.name,
    children: [
      {
        key: item.id + '11',
        label: 'test',
      },
    ],
  }));
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
  <div class="h-full">
    <PanelCard class="p-2">
      <div class="flex items-center justify-between">
        <span>Rules</span>
        <div class="flex items-center gap-1">
          <Button icon="pi" severity="contrast" variant="text">
            <IconLucideFilePlus />
          </Button>
          <Button
            icon="pi"
            severity="contrast"
            variant="text"
            @click="onCreateDir"
          >
            <IconLucideFolderPlus />
          </Button>
        </div>
      </div>
      <div>
        <Tree
          :value="nodes"
          selection-mode="single"
          v-model:expanded-keys="expandedKeys"
          :selection-keys="selectedKeys"
          @node-select="onNodeSelect"
          @node-unselect="onNodeUnselect"
          @node-expand="onNodeToggle"
          @node-collapse="onNodeToggle"
        >
          <template #nodeicon>
            <IconLucideFolder />
          </template>
          <template #nodetoggleicon="{ expanded }">
            <IconLucideChevronDown v-if="expanded" />
            <IconLucideChevronRight v-else />
          </template>
        </Tree>
      </div>
    </PanelCard>
  </div>
</template>
