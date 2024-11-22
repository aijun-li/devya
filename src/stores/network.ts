import { StoreName } from '@/const';
import { commands } from '@/ipc';
import { Captured, CapturedType } from '@/ipc/commands/types';
import { withRefs } from '@/utils/pinia';
import { Channel } from '@tauri-apps/api/core';
import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useNetworkStore = withRefs(
  defineStore(StoreName.Network, () => {
    const port = ref(7777);
    const requestList = ref<Captured[]>([]);

    const channel = new Channel<Captured>();
    channel.onmessage = (message) => {
      if (message.type === CapturedType.Request) {
        requestList.value.push(message);
      } else {
        const target = requestList.value.find((item) => item.id === message.id);
        if (target) {
          target.content += ` -> ${message.content}`;
        }
      }
    };

    async function startProxy(data: { port: number }) {
      await commands.startProxy({
        port: data.port,
        channel,
      });
      port.value = data.port;
    }

    async function stopProxy() {
      await commands.stopProxy();
    }

    return {
      port,
      requestList,

      startProxy,
      stopProxy,
    };
  }),
);
