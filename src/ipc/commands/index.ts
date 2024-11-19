import { invoke } from '@tauri-apps/api/core';
import { CheckPortReq, StartProxyReq, TauriCommand } from './types';

export function startProxy(data: StartProxyReq) {
  return invoke<void>(TauriCommand.StartProxy, data);
}

export function stopProxy() {
  return invoke<void>(TauriCommand.StopProxy);
}

export function installCert() {
  return invoke<void>(TauriCommand.InstallCert);
}

export function checkPort(data: CheckPortReq) {
  return invoke<boolean>(TauriCommand.CheckPort, data);
}
