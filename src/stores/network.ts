import { withRefs } from '@/utils/pinia';
import { Channel } from '@tauri-apps/api/core';
import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useNetworkStore = withRefs(
  defineStore('network', () => {
    const list = ref<string[]>([]);
    const channel = ref<Channel<string>>();

    function createChannel() {
      const newChannel = new Channel<string>();

      newChannel.onmessage = (message) => {
        console.log('message', message);
        list.value.push(message);
      };

      channel.value = newChannel; // update the channel ref to the new channel, so that we can close it later on in the fetc

      return newChannel;
    }

    return {
      list,
      createChannel,
    };
  }),
);
