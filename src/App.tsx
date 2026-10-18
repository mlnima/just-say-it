import { useEffect, useState } from 'react';
import { useRuntime } from './bridge/useRuntime';
import { useSystemTheme } from './bridge/useSystemTheme';
import { request } from './bridge/commands';
import type { Action, Microphone, Model, Settings } from './bridge/types';
import { Icon } from './components/Icon';
import { AudioSettings } from './features/AudioSettings';
import { ActivationSettings } from './features/ActivationSettings';
import { DeliverySettings } from './features/DeliverySettings';
import { BackgroundSettings } from './features/BackgroundSettings';
import logo from '../assets/icon.svg';

const sections = [
  { id: 'audio', title: 'Audio & model', icon: 'mic' },
  { id: 'activation', title: 'Activation', icon: 'wave' },
  { id: 'delivery', title: 'Input & recovery', icon: 'cursor' },
  { id: 'background', title: 'Background & privacy', icon: 'power' },
] as const;

export const App = () => {
  useSystemTheme();
  const { snapshot, error, setError, refresh } = useRuntime();
  const [section, setSection] = useState<string>('audio');
  const [draft, setDraft] = useState<Settings | null>(null);
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState(false);
  const [microphones, setMicrophones] = useState<Microphone[]>([]);
  const [models, setModels] = useState<Model[]>([]);
  const refreshModels = async () => setModels(await request('get_models', undefined));
  const refreshMicrophones = async () => setMicrophones(await request('list_microphones', undefined));

  useEffect(() => { if (snapshot && !dirty) setDraft(snapshot.settings); }, [snapshot, dirty]);
  useEffect(() => {
    if (!snapshot) return;
    Promise.all([refreshMicrophones(), refreshModels()]).catch((error: unknown) => setError(String(error)));
  }, [!!snapshot, setError]);
  useEffect(() => {
    if (snapshot && !snapshot.download) refreshModels().catch((error: unknown) => setError(String(error)));
  }, [snapshot?.download?.id, setError]);
  useEffect(() => { if (snapshot?.app_name) document.title = `${snapshot.app_name} · Settings`; }, [snapshot?.app_name]);

  const perform = (task: () => Promise<unknown>) => {
    setError(null);
    setBusy(true);
    task().catch((error: unknown) => setError(String(error))).finally(() => setBusy(false));
  };
  const patch = (change: Partial<Settings>) => { setDraft((draft) => draft && ({ ...draft, ...change })); setDirty(true); };
  const action = (action: Action) => perform(() => request('control', { action }));

  if (!snapshot || !draft) return <main className="connection"><img src={logo} width="64" height="64" alt="" /><h1>Dictation settings</h1><p role="status">{error || 'Connecting to the desktop runtime…'}</p></main>;
  const props = { draft, snapshot, patch, perform };
  const phase = snapshot.phase.replaceAll('_', ' ');
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand"><img src={logo} width="38" height="38" alt="" /><div><strong>{snapshot.app_name}</strong><span>A little less typing.</span></div></div>
        <div className="nav-label">YOUR WORKSPACE</div>
        <nav aria-label="Settings sections">{sections.map((item) => <button key={item.id} type="button" aria-current={section === item.id ? 'page' : undefined} onClick={() => setSection(item.id)}><Icon name={item.icon} /><span>{item.title}</span>{section === item.id && <span className="nav-marker" />}</button>)}</nav>
        <div className="sidebar-bottom"><Icon name="sliders" /><strong>Local by design</strong><p>Your voice stays on this computer. Whisper works only when you ask.</p><span className="local-label"><span />No cloud processing</span></div>
        <span className="version">Desktop initialization · 0.1</span>
      </aside>
      <div className="workspace">
        <header className="topbar"><span>Settings <span className="slash">/</span> {sections.find((item) => item.id === section)?.title}</span><span className="runtime-pill" data-active={snapshot.microphone_open}><span />{snapshot.microphone_open ? 'Microphone open' : 'Microphone closed'}</span></header>
        <main className="main-content">
          <div className="page-intro"><div><span className="eyebrow">A VOICE FOR YOUR EVERYDAY</span><h1>Less typing.<br /><em>More flow.</em></h1><p>Set your cue. Keep your focus.<br />Write in the place you're already working.</p></div><div className="voice-mark" aria-hidden="true"><span /><span /><span /><span /><span /><span /><span /></div></div>
          <div className="runtime-card" role="status" aria-live="polite"><span className="state-symbol"><Icon name={snapshot.phase === 'recording' ? 'mic' : 'wave'} size={22} /></span><div><strong>{phase}</strong><p>{snapshot.message}</p></div><button type="button" className="secondary" disabled={busy} onClick={() => action(snapshot.settings.paused ? 'resume' : 'pause')}>{snapshot.settings.paused ? 'Resume' : 'Pause'}</button></div>
          {error && <div className="error" role="alert">{error}<button type="button" aria-label="Dismiss error" onClick={() => setError(null)}>×</button></div>}
          <fieldset disabled={busy} className="settings-fields">
            {section === 'audio' && <AudioSettings {...props} microphones={microphones} models={models} refreshModels={refreshModels} refreshMicrophones={refreshMicrophones} />}
            {section === 'activation' && <ActivationSettings {...props} models={models} refreshModels={refreshModels} />}
            {section === 'delivery' && <DeliverySettings {...props} />}
            {section === 'background' && <BackgroundSettings {...props} />}
          </fieldset>
        </main>
        <footer className="save-bar"><span>{dirty ? 'You have unsaved changes.' : 'Showing saved configuration and observed runtime status.'}</span><div>{dirty && <button type="button" className="text-button" disabled={busy} onClick={() => { setDraft(snapshot.settings); setDirty(false); }}>Discard changes</button>}<button type="button" className="primary" disabled={busy || !dirty} onClick={() => perform(async () => { await request('save_settings', { settings: draft }); await refresh(); setDirty(false); })}>{busy ? 'Applying…' : 'Save settings'}<Icon name="arrow" size={17} /></button></div></footer>
      </div>
    </div>
  );
};
