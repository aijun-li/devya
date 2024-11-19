<script setup lang="ts">
import { commands } from '@/ipc';
import { ref } from 'vue';
import { Label } from './ui/label';
import { NumberField, NumberFieldContent, NumberFieldInput } from './ui/number-field';

const model = defineModel<number>({ required: true });

const portAvailable = ref(true);

async function onPortChange(port: number) {
  portAvailable.value = await commands.checkPort({
    port,
  });
}
</script>

<template>
  <NumberField
    v-model="model"
    :min="1024"
    :max="65535"
    :format-options="{ useGrouping: false }"
    @update:model-value="onPortChange"
  >
    <Label class="font-bold text-xs">Port: 1024 - 65535</Label>
    <NumberFieldContent>
      <NumberFieldInput />
    </NumberFieldContent>
  </NumberField>

  <div v-if="!portAvailable" class="text-red-500 text-xs mt-1">Port not available</div>
</template>
