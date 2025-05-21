import { withRefs } from '@/utils/pinia';
import { Channel } from '@tauri-apps/api/core';
import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useNetworkStore = withRefs(
  defineStore('network', () => {
    const list = ref<string[]>([]);

    function createChannel() {
      const channel = new Channel<string>((message) => {
        list.value.push(message);
      });

      return channel;
    }

    return {
      list,
      createChannel,
    };
  }),
);
