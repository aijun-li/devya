import { invoke } from '@tauri-apps/api/core';

export async function checkCaInstalled() {
  const installed = await invoke<boolean>('check_ca_installed');
  return installed;
}

export async function installCa() {
  await invoke('install_ca');
}

export async function startProxy(port: number) {
  await invoke('start_proxy', { port });
}

export async function checkProxyRunning() {
  const result = await invoke<{ port?: number; running_count: number }>(
    'check_proxy_running',
  );
  return result;
}
