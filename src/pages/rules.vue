<script setup lang="ts">
import { getRuleDirs, upsertRuleDir } from '@/commands';
import { useQuery } from '@tanstack/vue-query';
import { Splitter } from 'primevue';
import { MenuItem } from 'primevue/menuitem';
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
    leaf: false,
  })) as TreeNode[];

  if (creatingDir.value) {
    list.push({
      key: 'new',
      label: 'New',
      creatingDir: true,
      leaf: false,
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

const menuActiveDirId = ref<string>();
const dirMenuRef = useTemplateRef('dir-menu');
const dirMenuItems: MenuItem[] = [
  {
    label: 'Delete',
    command: () => {
      console.log('delete', menuActiveDirId.value);
    },
  },
];
</script>

<template>
  <Splitter class="h-full" :gutter-size="6">
    <SplitterPanel :size="30" :min-size="20">
      <PanelCard class="flex flex-col p-2">
        <div class="flex items-center justify-between">
          <span class="font-semibold">Rules</span>
          <div class="flex items-center gap-1">
            <Button
              class="p-1! text-base!"
              icon="pi"
              severity="contrast"
              variant="text"
              size="small"
            >
              <IconLucideFilePlus />
            </Button>
            <Button
              class="p-1! text-base!"
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

        <div class="flex-1 pt-2">
          <Tree
            :value="nodes"
            selection-mode="single"
            v-model:expanded-keys="expandedKeys"
            :selection-keys="selectedKeys"
            :pt="{
              root: 'p-0!',
              nodeContent: 'outline-none!',
              node: {
                oncontextmenu: (event: Event) => {
                  event.preventDefault();
                  dirMenuRef?.show(event);
                },
              },
            }"
            @node-select="onNodeSelect"
            @node-unselect="onNodeUnselect"
            @node-expand="onNodeToggle"
            @node-collapse="onNodeToggle"
            @keydown.esc="selectedKeys = {}"
          >
            <template #nodeicon>
              <IconLucideFolder class="flex-none text-[18px]" />
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
                class="px-1! py-0.5! text-xs"
                size="small"
                @blur="onCreateDir"
                @keydown.enter="onCreateDir"
                @keydown.esc="creatingDir = false"
              />
              <span v-else class="relative -top-[1px] ml-1 text-sm">
                {{ node.label }}
              </span>
            </template>
          </Tree>
        </div>

        <ContextMenu ref="dir-menu" :model="dirMenuItems" />
      </PanelCard>
    </SplitterPanel>

    <SplitterPanel :size="70" :min-size="60">
      <PanelCard class="p-2">
        <div>213</div>
      </PanelCard>
    </SplitterPanel>
  </Splitter>
</template>
