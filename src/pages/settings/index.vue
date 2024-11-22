<script setup lang="ts">
import ContentCard from '@/components/ContentCard.vue';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { NumberField, NumberFieldContent, NumberFieldInput } from '@/components/ui/number-field';
import { commands } from '@/ipc';
import { useNetworkStore } from '@/stores/network';
import { handleError } from '@/utils/error';
import { ref } from 'vue';
import { toast } from 'vue-sonner';

const { port: activePort, startProxy, stopProxy } = useNetworkStore();

const configPort = ref(activePort.value);
const portAvailable = ref(true);

async function onPortChange(port: number) {
  console.log(port);
  if (Number.isNaN(port)) {
    configPort.value = activePort.value;
    return;
  }

  const available = await commands.checkPort({ port }).catch(() => false);
  portAvailable.value = available;
  if (available) {
    try {
      await stopProxy();
      await startProxy({ port });
    } catch (error) {
      handleError(error);
    }
  }
}

const certInstalling = ref(false);
async function installCert() {
  if (certInstalling.value) {
    return;
  }
  certInstalling.value = true;
  toast.promise(
    commands.installCert().finally(() => {
      certInstalling.value = false;
    }),
    {
      loading: 'Installing...',
      success: 'Certificate Installed',
      error: 'Failed to install certificate',
    },
  );
}
</script>

<template>
  <ContentCard>
    <div class="space-y-12">
      <div class="setting-item relative">
        <Label class="setting-name" for="port">
          Listening Port
          <span class="text-xs font-normal">(1024 - 65535)</span>
        </Label>
        <NumberField
          v-model="configPort"
          class="w-32"
          :min="1024"
          :max="65535"
          :format-options="{ useGrouping: false }"
          @update:model-value="onPortChange"
        >
          <NumberFieldContent>
            <NumberFieldInput />
          </NumberFieldContent>
        </NumberField>

        <div v-if="!portAvailable" class="absolute -bottom-5 text-xs text-red-500">Port not available</div>
      </div>

      <div class="setting-item">
        <Label class="setting-name">Install Certificate</Label>
        <Button class="w-40" @click="installCert">Install</Button>
      </div>
    </div>
  </ContentCard>
</template>

<style scoped>
.setting-item {
  @apply flex flex-col gap-2;

  .setting-name {
    @apply text-base font-semibold;
  }
}
</style>
