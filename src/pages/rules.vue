<script setup lang="ts">
import { deleteRuleDir, getRuleDirs, upsertRuleDir } from '@/commands';
import { RuleDir } from '@/commands/types';
import { useQuery } from '@tanstack/vue-query';
import { Splitter } from 'primevue';
import { MenuItem } from 'primevue/menuitem';
import { TreeNode } from 'primevue/treenode';
import { computed, nextTick, ref, useTemplateRef } from 'vue';

const { data: dirs, refetch } = useQuery({
  queryKey: [getRuleDirs.name],
  queryFn: getRuleDirs,
});

const selectedKey = ref<string>();
const selectedKeys = computed<Record<string, boolean>>(() =>
  selectedKey.value ? { [selectedKey.value]: true } : {},
);
const expandedKeys = ref<Record<string, boolean>>({});

const creatingDir = ref<number>();
const newDirName = ref('');
const creatingInputRef = useTemplateRef<any>('creating-input');

function onCreateDirClick() {
  creatingDir.value = Number(selectedKey.value) || 0;
  if (creatingDir.value) {
    expandedKeys.value[creatingDir.value] = true;
  }
  newDirName.value = '';
  nextTick(() => {
    console.log(creatingInputRef.value);
    creatingInputRef.value?.$el?.focus?.();
  });
}

async function onCreateDir() {
  const newName = newDirName.value.trim();
  if (!newName) {
    creatingDir.value = undefined;
    return;
  }
  await upsertRuleDir({
    name: newName,
    parentId: creatingDir.value || undefined,
  });
  await refetch();
  creatingDir.value = undefined;
}

const buildTree = (dir: RuleDir, creatingId?: number): TreeNode => {
  const children = dir.dirs.map((d) => buildTree(d, creatingId));
  if (creatingId === dir.id) {
    children.unshift({
      key: 'creating',
      label: '',
      leaf: false,
      creating: true,
    });
  }
  return {
    key: String(dir.id),
    label: dir.name,
    leaf: false,
    children,
  };
};

const nodes = computed<TreeNode[]>(() => {
  const list = (dirs.value ?? []).map((dir) =>
    buildTree(dir, creatingDir.value),
  );
  if (creatingDir.value === 0) {
    list.unshift({
      key: 'creating',
      label: '',
      leaf: false,
      creating: true,
    });
  }
  return list;
});

function onNodeSelect(node: TreeNode) {
  expandedKeys.value[node.key] = !expandedKeys.value[node.key];
  selectedKey.value = node.key;
}

function onNodeUnselect(node: TreeNode) {
  expandedKeys.value[node.key] = !expandedKeys.value[node.key];
}

function onNodeToggle(node: TreeNode) {
  selectedKey.value = node.key;
}

const menuActiveDirId = ref<number>();
const dirMenuRef = useTemplateRef('dir-menu');
const dirMenuItems: MenuItem[] = [
  {
    label: 'Add Folder',
    command: async () => {
      if (!menuActiveDirId.value) {
        return;
      }
      newDirName.value = '';
      creatingDir.value = menuActiveDirId.value;
      expandedKeys.value[menuActiveDirId.value] = true;
      nextTick(() => {
        creatingInputRef.value?.$el?.focus?.();
      });
    },
  },
  { separator: true },
  {
    label: 'Delete',
    command: async () => {
      if (!menuActiveDirId.value) {
        return;
      }
      await deleteRuleDir(menuActiveDirId.value);
      await refetch();
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
              node: ({ context: { node } }) => ({
                oncontextmenu: (event: Event) => {
                  menuActiveDirId = Number(node.key);
                  dirMenuRef?.show(event);
                },
              }),
            }"
            @node-select="onNodeSelect"
            @node-unselect="onNodeUnselect"
            @node-expand="onNodeToggle"
            @node-collapse="onNodeToggle"
            @keydown.esc="selectedKey = undefined"
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
                v-if="node.creating"
                ref="creating-input"
                v-model="newDirName"
                class="px-1! py-0.5! text-xs"
                size="small"
                @blur="onCreateDir"
                @keydown.enter="onCreateDir"
                @keydown.esc="creatingDir = undefined"
                @click.stop=""
              />
              <span v-else class="relative -top-[1px] ml-1 text-sm">
                {{ node.label }}
              </span>
            </template>
          </Tree>
        </div>

        <ContextMenu
          ref="dir-menu"
          :model="dirMenuItems"
          @hide="menuActiveDirId = undefined"
        />
      </PanelCard>
    </SplitterPanel>

    <SplitterPanel :size="70" :min-size="60">
      <PanelCard class="p-2">
        <div>213</div>
      </PanelCard>
    </SplitterPanel>
  </Splitter>
</template>
