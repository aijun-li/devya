import { Channel, invoke } from '@tauri-apps/api/core';
import { RuleDir, UpsertRuleDirReq } from './types';

export async function checkCaInstalled() {
  const installed = await invoke<boolean>('check_ca_installed');
  return installed;
}

export async function installCa() {
  await invoke('install_ca');
}

export async function startProxy(port: number, channel: Channel<string>) {
  await invoke('start_proxy', { port, channel });
}

export async function stopProxy() {
  await invoke('stop_proxy');
}

export async function checkProxyRunning() {
  const result = await invoke<{ port?: number; running_count: number }>(
    'check_proxy_running',
  );
  return result;
}

export async function getRuleDirs() {
  const result = await invoke<RuleDir[]>('get_rule_dirs');
  return result;
}

export async function upsertRuleDir(req: UpsertRuleDirReq) {
  const result = await invoke<string>('upsert_rule_dir', req);
  return result;
}
