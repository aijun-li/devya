import { withRefs } from '@/utils/pinia';
import { defineStore } from 'pinia';

export const useProxyStore = withRefs(
  defineStore('proxy', {
    state: () => ({
      proxyOnCount: 0,
      port: undefined as number | undefined,
    }),
    getters: {
      isProxyOn: (state) => state.proxyOnCount > 0,
    },
  }),
);
