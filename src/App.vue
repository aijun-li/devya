<script setup lang="ts">
import { ref } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Captured, CapturedType } from '@/types/command'

const port = ref(7777);
const proxyOn = ref(false);

const list = ref<Captured[]>([]);

const channel = new Channel<Captured>();
channel.onmessage = (message) => {
  if (message.type === CapturedType.Request) {
    list.value.push(message);
  } else {
    const target = list.value.find(item => item.id === message.id);
    if (target) {
      target.content += ` -> ${message.content}`
    }
  }
};

async function startProxy() {
  await invoke("start_proxy", {
    channel,
  });
  proxyOn.value = true;
}

async function stopProxy() {
  await emit("stop_proxy");
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
</script>

<template>
  <div class="w-screen h-screen p-4 flex flex-col gap-4">
    <div class="flex gap-4">
      <Input v-model="port" placeholder="Port" disabled />
      <Button :variant="proxyOn ? 'destructive' : 'default'" @click="toggleProxy">
        {{ proxyOn ? "Stop Proxy" : "Start Proxy" }}
      </Button>
    </div>
    <ul class="flex-1 overflow-auto list-disc list-inside">
      <li v-for="item in list" :key="item.id">{{ item.content }}</li>
    </ul>
  </div>
</template>
