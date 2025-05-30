import { Channel, invoke } from '@tauri-apps/api/core';
import { RuleFile, UpsertRuleFileReq } from './types';

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

export async function getRuleFiles() {
  const result = await invoke<RuleFile[]>('get_rule_files');
  return result;
}

export async function upsertRuleFile(req: UpsertRuleFileReq) {
  const result = await invoke<string>('upsert_rule_file', req);
  return result;
}

export async function deleteRuleFile(id: number) {
  const result = await invoke('delete_rule_file', { id });
  return result;
}
