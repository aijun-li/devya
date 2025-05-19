import { Event, listen } from '@tauri-apps/api/event';
import { TEvent, TEventPayload } from './types';

export function listenEvent<T extends TEvent>(
  event: T,
  cb: (event: Event<TEventPayload>) => void,
) {
  return listen(event, cb);
}

export async function listenEvents(listeners: {
  [T in TEvent]: (event: Event<TEventPayload[T]>) => void;
}) {
  const unlisteners = await Promise.all(
    Object.entries(listeners).map(async ([event, cb]) => {
      const unlisten = await listenEvent(event as any, cb as any);
      return unlisten;
    }),
  );

  return () => {
    unlisteners.forEach((unlisten) => unlisten());
  };
}
