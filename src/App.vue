<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { commands } from '@/ipc';
import { Captured, CapturedType } from '@/ipc/commands/types';
import { Channel } from '@tauri-apps/api/core';
import { Pencil, ShieldPlus } from 'lucide-vue-next';
import { ref } from 'vue';
import { toast } from 'vue-sonner';
import PortInput from './components/PortInput.vue';
import { Popover, PopoverContent, PopoverTrigger } from './components/ui/popover';
import { Toaster } from './components/ui/sonner';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from './components/ui/tooltip';
import { handleError } from './utils/error';

const port = ref(7777);
const proxyOn = ref(false);

const list = ref<Captured[]>([]);

const newPort = ref(port.value);
async function onPortChange(open: boolean) {
  if (open) {
    newPort.value = port.value;
  } else {
    const available = await commands.checkPort({ port: newPort.value });
    if (!available || newPort.value === port.value) {
      return;
    }
    if (proxyOn.value) {
      await commands.stopProxy();
      await commands.startProxy({
        port: newPort.value,
        channel,
      });
    }
    port.value = newPort.value;
  }
}

const channel = new Channel<Captured>();
channel.onmessage = (message) => {
  if (message.type === CapturedType.Request) {
    list.value.push(message);
  } else {
    const target = list.value.find((item) => item.id === message.id);
    if (target) {
      target.content += ` -> ${message.content}`;
    }
  }
};

async function startProxy() {
  await commands.startProxy({
    port: port.value,
    channel,
  });
  proxyOn.value = true;
}

async function stopProxy() {
  await commands.stopProxy();
  proxyOn.value = false;
  list.value.length = 0;
}

async function toggleProxy() {
  if (proxyOn.value) {
    await stopProxy();
  } else {
    await startProxy();
  }
}

async function installCert() {
  toast.promise(commands.installCert(), {
    loading: 'Installing...',
    success: 'Certificate Installed',
    error: handleError,
  });
}
</script>

<template>
  <TooltipProvider :delay-duration="500">
    <div class="w-screen h-screen p-4 flex flex-col gap-4">
      <div class="flex gap-4">
        <div class="border rounded-md flex items-center px-4 h-9 w-full">
          <div class="mr-2 w-1.5 h-1.5 rounded-full" :class="[proxyOn ? 'bg-green-500' : 'bg-zinc-300']" />

          <div class="flex-1">Proxying on 127.0.0.1:{{ port }}</div>

          <div class="flex gap-1">
            <Popover @update:open="onPortChange">
              <PopoverTrigger>
                <Tooltip>
                  <TooltipTrigger class="config-button">
                    <Pencil :size="14" />
                  </TooltipTrigger>
                  <TooltipContent>Modify Port</TooltipContent>
                </Tooltip>
              </PopoverTrigger>
              <PopoverContent class="w-fit">
                <PortInput v-model="newPort" />
              </PopoverContent>
            </Popover>

            <Tooltip>
              <TooltipTrigger class="config-button" @click="installCert">
                <ShieldPlus :size="14" />
              </TooltipTrigger>
              <TooltipContent>Install Certificate</TooltipContent>
            </Tooltip>
          </div>
        </div>

        <Button class="min-w-28" :variant="proxyOn ? 'destructive' : 'default'" @click="toggleProxy">
          {{ proxyOn ? 'Stop Proxy' : 'Start Proxy' }}
        </Button>
      </div>
      <ul class="flex-1 overflow-auto list-disc list-inside pr-4 -mr-4">
        <li v-for="item in list" :key="item.id" class="break-all">{{ item.content }}</li>
      </ul>
    </div>

    <Toaster />
  </TooltipProvider>
</template>

<style scoped>
.config-button {
  @apply rounded-full hover:bg-zinc-100 size-6 flex items-center justify-center;

  cursor: pointer;
}
</style>
