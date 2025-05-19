import { withRefs } from '@/utils/pinia';
import { defineStore } from 'pinia';

export const useProxyStore = withRefs(
  defineStore('proxy', {
    state: () => ({
      proxyOnCount: 0,
    }),
    getters: {
      isProxyOn: (state) => state.proxyOnCount > 0,
    },
  }),
);
