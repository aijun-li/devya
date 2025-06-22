<script setup lang="ts">
import * as monaco from 'monaco-editor';
import { loader, VueMonacoEditor } from '@guolao/vue-monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

const model = defineModel<string>({ required: true, default: '' });

self.MonacoEnvironment = {
  getWorker() {
    return new editorWorker();
  },
};

loader.config({ monaco });

monaco.languages.register({
  id: 'rule',
});

monaco.languages.setMonarchTokensProvider('rule', {
  tokenizer: {
    root: [
      [/#.*$/, 'comment'],
      [/\b(?:http|https|wss|ws):\/\//, 'keyword'],
      [/:\d+$/, 'number'],
      [/\*\.[a-zA-Z0-9-]+\.[a-zA-Z]+/, 'variable'],
    ],
  },
});

const monacoOptions: monaco.editor.IStandaloneEditorConstructionOptions = {
  minimap: { enabled: false },
  scrollbar: { vertical: 'hidden' },
};
</script>

<template>
  <VueMonacoEditor
    v-model:value="model"
    language="rule"
    :options="monacoOptions"
  />
</template>
