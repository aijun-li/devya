<script setup lang="ts">
import { getRuleFiles, upsertRuleFile, deleteRuleFile } from '@/commands';
import { RuleFile } from '@/commands/types';
import { useQuery } from '@tanstack/vue-query';
import { confirm } from '@tauri-apps/plugin-dialog';
import { MenuItem } from 'primevue/menuitem';
import { TreeNode } from 'primevue/treenode';
import { computed, useTemplateRef, ref, nextTick, reactive } from 'vue';

const { data: files, refetch } = useQuery({
  queryKey: [getRuleFiles.name],
  queryFn: getRuleFiles,
});

const selectedNode = ref<TreeNode>();
const selectedKeys = computed<Record<string, boolean>>(() =>
  selectedNode.value ? { [selectedNode.value.key]: true } : {},
);
const expandedKeys = ref<Record<string, boolean>>({});

const createInputRef = useTemplateRef<any>('creating-input');
const createConfig = reactive({
  creating: false,
  parentId: undefined as number | undefined,
  name: '',
  isDir: false,
});

const menuActiveFile = reactive({
  id: undefined as number | undefined,
  isDir: false,
});

function resetMenuActiveFile() {
  Object.assign(menuActiveFile, {
    id: undefined,
    isDir: false,
  });
}

function resetCreateConfig() {
  Object.assign(createConfig, {
    creating: false,
    parentId: undefined,
    name: '',
    isDir: false,
  });
}

function focusCreateInput() {
  nextTick(() => {
    createInputRef.value?.$el?.focus?.();
  });
}

function onCreateFileStart(isDir: boolean, parentId?: number) {
  Object.assign(createConfig, {
    creating: true,
    parentId,
    name: '',
    isDir,
  });
  if (menuActiveFile.id && menuActiveFile.isDir) {
    expandedKeys.value[menuActiveFile.id] = true;
  }
  focusCreateInput();
}

function onTopCreateFileClick(isDir: boolean) {
  if (!selectedNode.value) {
    onCreateFileStart(isDir);
  } else if (selectedNode.value.isDir) {
    onCreateFileStart(isDir, selectedNode.value.id);
  } else {
    onCreateFileStart(isDir, selectedNode.value.parentId);
  }
}

async function onCreateFileCommit() {
  const { name, parentId, isDir } = createConfig;

  const newName = name.trim();
  if (!newName) {
    resetCreateConfig();
    return;
  }

  await upsertRuleFile({
    name: newName,
    parentId: parentId || undefined,
    isDir,
  });
  await refetch();

  resetCreateConfig();
}

const buildTree = (
  file: RuleFile,
  parentId?: number,
  path: number[] = [],
): TreeNode => {
  const currentPath = [...path, file.id];
  const children = file.children.map((d) =>
    buildTree(d, parentId, [...currentPath]),
  );
  if (parentId === file.id) {
    children.unshift({
      key: 'creating',
      label: '',
      path: [...currentPath, -1],
      leaf: !createConfig.isDir,
      isDir: createConfig.isDir,
      creating: true,
    });
  }
  return {
    key: String(file.id),
    id: file.id,
    parentId: file.parentId,
    label: file.name,
    path: [...currentPath],
    leaf: !file.isDir,
    isDir: file.isDir,
    children,
  };
};

const nodes = computed<TreeNode[]>(() => {
  const list = (files.value ?? []).map((file) =>
    buildTree(file, createConfig.parentId),
  );
  if (createConfig.creating && !createConfig.parentId) {
    list.unshift({
      key: 'creating',
      label: '',
      path: [-1],
      leaf: !createConfig.isDir,
      isDir: createConfig.isDir,
      creating: true,
    });
  }
  return list;
});

function onNodeSelect(node: TreeNode) {
  expandedKeys.value[node.key] = !expandedKeys.value[node.key];
  selectedNode.value = node;
}

function onNodeUnselect(node: TreeNode) {
  expandedKeys.value[node.key] = !expandedKeys.value[node.key];
}

function onNodeToggle(node: TreeNode) {
  selectedNode.value = node;
}

const dirMenuRef = useTemplateRef('dir-menu');
const dirMenuItems = computed<MenuItem[]>(() => {
  const baseItems = [
    {
      label: 'Delete',
      icon: 'delete',
      command: async () => {
        const toDeleteId = menuActiveFile.id;
        if (!toDeleteId) {
          return;
        }
        const confirmed = await confirm('This action cannot be reverted.', {
          title: 'Delete',
          kind: 'warning',
          okLabel: 'Delete',
        });
        if (confirmed) {
          await deleteRuleFile(toDeleteId);
          await refetch();
          if (selectedNode.value?.path.includes(toDeleteId)) {
            selectedNode.value = undefined;
          }
        }
      },
    },
  ] as MenuItem[];

  if (menuActiveFile.isDir) {
    baseItems.unshift(
      {
        label: 'Add File',
        icon: 'add-file',
        command: () => {
          onCreateFileStart(false, menuActiveFile.id);
          focusCreateInput();
        },
      },
      {
        label: 'Add Folder',
        icon: 'add-folder',
        command: () => {
          onCreateFileStart(true, menuActiveFile.id);
          focusCreateInput();
        },
      },
      { separator: true },
    );
  }

  return baseItems;
});
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
          @click="onTopCreateFileClick(false)"
        >
          <IconLucideFilePlus />
        </Button>
        <Button
          class="p-1! text-base!"
          icon="pi"
          severity="contrast"
          variant="text"
          size="small"
          @click="onTopCreateFileClick(true)"
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
            Object.assign(menuActiveFile, {
              id: node.id,
              isDir: node.isDir,
            });
            dirMenuRef?.show(event);
          },
        }),
      }"
      @node-select="onNodeSelect"
      @node-unselect="onNodeUnselect"
      @node-expand="onNodeToggle"
      @node-collapse="onNodeToggle"
      @keydown.esc="selectedNode = undefined"
    >
      <template #nodeicon="{ node }">
        <IconLucideFolder v-if="node.isDir" class="flex-none text-[18px]" />
        <IconLucideFile v-else class="flex-none text-[18px]" />
      </template>
      <template #nodetoggleicon="{ expanded }">
        <IconLucideChevronDown v-if="expanded" class="text-sm" />
        <IconLucideChevronRight v-else class="text-sm" />
      </template>
      <template #default="{ node }">
        <InputText
          v-if="node.creating"
          ref="creating-input"
          v-model="createConfig.name"
          class="flex h-[24px]! w-full rounded! px-1! py-0.5! text-xs"
          size="small"
          @blur="onCreateFileCommit"
          @keydown.enter="onCreateFileCommit"
          @keydown.esc="resetCreateConfig"
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
    @hide="resetMenuActiveFile"
  >
    <template #itemicon="{ item: { icon } }">
      <IconLucideFolderPlus v-if="icon === 'add-folder'" />
      <IconLucideFilePlus v-else-if="icon === 'add-file'" />
      <IconLucideTrash2 v-else-if="icon === 'delete'" />
    </template>
  </ContextMenu>
</template>
