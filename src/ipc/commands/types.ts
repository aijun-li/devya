import { Channel } from '@tauri-apps/api/core';

export enum TauriCommand {
  StartProxy = 'start_proxy',
  StopProxy = 'stop_proxy',
  InstallCert = 'install_cert',
  CheckPort = 'check_port',
}

export enum CapturedType {
  Request = 'request',
  Response = 'response',
}

export type Captured = {
  id: string;
  type: CapturedType;
  content: string;
};

export type StartProxyReq = {
  channel: Channel<Captured>;
  port?: number;
};

export type CheckPortReq = {
  port: number;
};
