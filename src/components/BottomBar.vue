<script setup lang="ts">
import { checkCaInstalled, installCa, startProxy } from '@/commands';
import { useNetworkStore } from '@/stores/network';
import { useProxyStore } from '@/stores/proxy';
import { useQuery } from '@tanstack/vue-query';
import { nextTick, ref, useTemplateRef } from 'vue';

const { isProxyOn, port } = useProxyStore();

const { createChannel } = useNetworkStore();

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
    await startProxy(newPort.value, createChannel());
    port.value = newPort.value;
    portPopover.value?.hide();
  } catch (error) {
    newPortInvalid.value = error as string;
  }
}
</script>

<template>
  <div
    class="text-surface-500 bg-surface-300 sticky bottom-0 flex flex-col text-xs"
  >
    <div class="flex items-center justify-between px-2 py-0.5">
      <div class="flex items-center">
        <div
          class="mr-1 ml-2 h-1.5 w-1.5 rounded-full"
          :class="[isProxyOn ? 'bg-green-700' : 'bg-red-700']"
        />
        <Button
          class="flex p-1! py-0.5! text-xs!"
          severity="secondary"
          variant="text"
          size="small"
          @click="onOpenPortPopover"
        >
          {{
            isProxyOn
              ? `Listening on 127.0.0.1:${port}`
              : 'Proxy stopped (click to restart)'
          }}
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
            :invalid="Boolean(newPortInvalid)"
            @input="newPortInvalid = ''"
            @keydown.enter="onChangePort"
          />
          <label for="change-port">Port</label>
        </FloatLabel>

        <Button size="small" @click="onChangePort">OK</Button>
      </div>
      <Message
        v-if="Boolean(newPortInvalid)"
        pt:text:class="text-xs!"
        severity="error"
        size="small"
        variant="simple"
      >
        {{ newPortInvalid }}
      </Message>
    </Popover>
  </div>
</template>
