import { useCallback, useEffect, useState } from 'react';
import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { request } from './commands';
import type { Snapshot } from './types';

export const useRuntime = () => {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const accept = useCallback((next: Snapshot) => {
    setSnapshot((current) => !current || next.revision >= current.revision ? next : current);
  }, []);
  const refresh = useCallback(async () => {
    try { accept(await request('get_snapshot', undefined)); }
    catch (error) { setError(String(error)); }
  }, [accept]);

  useEffect(() => {
    if (!isTauri()) {
      setError('Open the desktop application to configure real devices and local models. The browser does not have access to the dictation runtime.');
      return;
    }
    let disposed = false;
    let unsubscribe: (() => void) | undefined;
    listen<Snapshot>('runtime-status', ({ payload }) => { if (!disposed) accept(payload); })
      .then((stop) => { if (disposed) stop(); else { unsubscribe = stop; void refresh(); } })
      .catch((error: unknown) => setError(String(error)));
    const reconnect = () => { if (document.visibilityState === 'visible') void refresh(); };
    document.addEventListener('visibilitychange', reconnect);
    return () => { disposed = true; unsubscribe?.(); document.removeEventListener('visibilitychange', reconnect); };
  }, [accept, refresh]);

  return { snapshot, error, setError, refresh };
};
