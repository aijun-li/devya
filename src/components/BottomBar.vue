<script setup lang="ts">
import { checkCaInstalled, installCa, startProxy } from '@/commands';
import { useProxyStore } from '@/stores/proxy';
import { useQuery } from '@tanstack/vue-query';
import { nextTick, ref, useTemplateRef } from 'vue';

const { isProxyOn, port } = useProxyStore();

const { data: caInstalled, refetch: reCheckCa } = useQuery({
  queryKey: [checkCaInstalled.name],
  queryFn: checkCaInstalled,
});

async function onHttpsClick() {
  if (caInstalled.value) {
    return;
  }
  await installCa();
  await reCheckCa();
}

const newPort = ref(port.value);
const newPortInvalid = ref('');

const portPopover = useTemplateRef('port-popover');
const portInput = useTemplateRef<any>('port-input');
// const restartPopover = useTemplateRef('restart-popover');

async function onOpenPortPopover(event: Event) {
  newPort.value = port.value;
  newPortInvalid.value = '';
  portPopover.value?.toggle(event);
  await nextTick();
  portInput.value?.$refs?.input?.$el?.focus?.();
}

async function onChangePort() {
  if (!newPort.value || newPort.value < 1 || newPort.value > 65535) {
    newPortInvalid.value = '';
    return;
  }
  try {
    await startProxy(newPort.value);
    port.value = newPort.value;
  } catch (error) {
    newPortInvalid.value = (error as Error).message;
  }
}
</script>

<template>
  <div
    class="text-surface-500 sticky bottom-0 flex flex-col bg-stone-300 text-xs"
  >
    <div class="flex items-center justify-between px-2 py-0.5">
      <div class="flex items-center">
        <div
          class="mr-1 ml-2 h-1.5 w-1.5 rounded-full"
          :class="[isProxyOn ? 'bg-green-700' : 'bg-red-700']"
        />
        <Button
          v-if="isProxyOn"
          class="flex p-1! py-0.5! text-xs!"
          severity="secondary"
          variant="text"
          size="small"
          @click="onOpenPortPopover"
        >
          Listening on 127.0.0.1:{{ port }}
        </Button>
        <Button
          v-else
          class="flex p-1! py-0.5! text-xs!"
          severity="secondary"
          variant="text"
          size="small"
        >
          Proxy stopped
        </Button>
      </div>

      <Button
        class="flex items-center gap-1! p-1! text-xs!"
        :class="[caInstalled ? 'text-green-700!' : 'cursor-pointer']"
        variant="text"
        size="small"
        :disabled="caInstalled"
        @click="onHttpsClick"
      >
        <IconMdiHttps />
        TLS {{ caInstalled ? 'Enabled' : 'Disabled' }}
      </Button>
    </div>

    <Popover ref="port-popover">
      <div class="flex items-center gap-1">
        <FloatLabel variant="on">
          <InputNumber
            ref="port-input"
            v-model="newPort"
            input-id="change-port"
            :use-grouping="false"
            :min="1"
            :max="65535"
            placeholder="1-65535"
            size="small"
            pt:pcInputText:root:class="w-[120px]"
            :invalid="Boolean(newPortInvalid)"
            @input="newPortInvalid = ''"
          />
          <label for="change-port">Change Port</label>
        </FloatLabel>

        <Button size="small" @click="onChangePort">OK</Button>
      </div>
      <Message v-if="Boolean(newPortInvalid)" severity="error" size="small">
        {{ newPortInvalid }}
      </Message>
    </Popover>

    <Popover ref="restart-popover">
      <div>restart</div>
    </Popover>
  </div>
</template>
