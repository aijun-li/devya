export enum TEvent {
  ProxyStarted = 'proxy-started',
  ProxyStopped = 'proxy-stopped',
}

export type TEventPayload = {
  [TEvent.ProxyStarted]: void;
  [TEvent.ProxyStopped]: void;
};
