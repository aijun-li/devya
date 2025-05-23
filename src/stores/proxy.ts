import { useSettings } from '@/hooks/use-settings';
import { withRefs } from '@/utils/pinia';
import { defineStore } from 'pinia';
import { computed, readonly, ref } from 'vue';

export const useProxyStore = withRefs(
  defineStore('proxy', () => {
    const { setSettings } = useSettings();

    const proxyOnCount = ref(0);
    const port = ref<number>();

    const isProxyOn = computed(() => proxyOnCount.value > 0);

    function updateProxyPort(newPort?: number) {
      port.value = newPort;
      if (newPort) {
        setSettings('port', newPort);
      }
    }

    return {
      isProxyOn,
      port: readonly(port),
      proxyOnCount,
      updateProxyPort,
    };
  }),
);
