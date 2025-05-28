<script setup lang="ts">
import { getRuleDirs, upsertRuleDir, deleteRuleDir } from '@/commands';
import { RuleDir } from '@/commands/types';
import { useQuery } from '@tanstack/vue-query';
import { MenuItem } from 'primevue/menuitem';
import { TreeNode } from 'primevue/treenode';
import { computed, useTemplateRef, ref, nextTick } from 'vue';

const { data: dirs, refetch } = useQuery({
  queryKey: [getRuleDirs.name],
  queryFn: getRuleDirs,
});

const selectedKey = ref<string>();
const selectedKeys = computed<Record<string, boolean>>(() =>
  selectedKey.value ? { [selectedKey.value]: true } : {},
);
const expandedKeys = ref<Record<string, boolean>>({});

const creatingDirParent = ref<number>();
const newDirName = ref('');
const creatingInputRef = useTemplateRef<any>('creating-input');

function onCreateDirClick() {
  creatingDirParent.value = Number(selectedKey.value) || 0;
  if (creatingDirParent.value) {
    expandedKeys.value[creatingDirParent.value] = true;
  }
  newDirName.value = '';
  nextTick(() => {
    creatingInputRef.value?.$el?.focus?.();
  });
}

async function onCreateDirCommit() {
  const newName = newDirName.value.trim();
  if (!newName) {
    creatingDirParent.value = undefined;
    return;
  }
  await upsertRuleDir({
    name: newName,
    parentId: creatingDirParent.value || undefined,
  });
  await refetch();
  creatingDirParent.value = undefined;
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
    buildTree(dir, creatingDirParent.value),
  );
  if (creatingDirParent.value === 0) {
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
    label: 'Add File',
    icon: 'add-file',
    command: () => {},
  },
  {
    label: 'Add Folder',
    icon: 'add-folder',
    command: () => {
      if (!menuActiveDirId.value) {
        return;
      }
      newDirName.value = '';
      creatingDirParent.value = menuActiveDirId.value;
      expandedKeys.value[menuActiveDirId.value] = true;
      nextTick(() => {
        creatingInputRef.value?.$el?.focus?.();
      });
    },
  },
  { separator: true },
  {
    label: 'Delete',
    icon: 'delete',
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
  <div>
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

    <Tree
      :value="nodes"
      selection-mode="single"
      v-model:expanded-keys="expandedKeys"
      :selection-keys="selectedKeys"
      :pt="{
        root: 'p-0!',
        nodeContent: 'outline-none! px-1! py-0.5!',
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
          class="flex h-[24px]! w-full rounded! px-1! py-0.5! text-xs"
          size="small"
          @blur="onCreateDirCommit"
          @keydown.enter="onCreateDirCommit"
          @keydown.esc="creatingDirParent = undefined"
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
    :pt="{
      root: 'min-w-[150px]!',
      itemLabel: 'text-sm',
    }"
    @hide="menuActiveDirId = undefined"
  >
    <template #itemicon="{ item: { icon } }">
      <IconLucideFolderPlus v-if="icon === 'add-folder'" />
      <IconLucideFilePlus v-else-if="icon === 'add-file'" />
      <IconLucideTrash2 v-else-if="icon === 'delete'" />
    </template>
  </ContextMenu>
</template>
