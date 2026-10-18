import { useEffect } from 'react';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow, type Theme } from '@tauri-apps/api/window';

export const useSystemTheme = () => {
  useEffect(() => {
    if (!isTauri()) return;
    const current = getCurrentWindow();
    let disposed = false;
    let revision = 0;
    let unsubscribe: (() => void) | undefined;
    const apply = (theme: Theme | null) => {
      if (!disposed) document.documentElement.dataset.theme = theme || '';
    };
    const refresh = () => {
      const requested = ++revision;
      void current.theme().then(
        (theme) => { if (requested === revision) apply(theme); },
        () => { if (requested === revision) apply(null); },
      );
    };
    void current.onThemeChanged(({ payload }) => { revision++; apply(payload); })
      .then((stop) => {
        if (disposed) stop();
        else { unsubscribe = stop; refresh(); }
      }, refresh);
    window.addEventListener('focus', refresh);
    return () => {
      disposed = true;
      unsubscribe?.();
      window.removeEventListener('focus', refresh);
    };
  }, []);
};
