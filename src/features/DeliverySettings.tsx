import { useEffect, useState } from 'react';
import { Switch } from '../components/Fields';
import { request } from '../bridge/commands';
import type { SettingsProps } from './AudioSettings';

export const DeliverySettings = ({ draft, snapshot, patch, perform }: SettingsProps) => {
  const [text, setText] = useState<string | null>(null);
  useEffect(() => { if (!snapshot.pending_text) setText(null); }, [snapshot.pending_text]);
  return <>
    <section className="panel">
      <div className="panel-heading"><span className="step">03</span><div><h2>Right where you're writing</h2><p>A quiet invitation above a supported focused input.</p></div></div>
      <Switch label="Show the input button" description="Offer Dictate above detected editable fields. Clicking preserves focus. During recording, the button becomes Finish." value={draft.input_button} onChange={(input_button) => patch({ input_button })} disabled={!snapshot.input_detection_supported} />
      <div className="observed"><span className="status-dot" /><span>{snapshot.target_status}</span></div>
      <p className="muted">Detection uses Windows accessibility focus events. Full-screen games, protected fields and apps without accessible controls may not expose an input. A shortcut can still start recording; text is held for recovery when the target cannot be verified.</p>
      <div className="rule"><strong>Text insertion only.</strong><span>No automatic Enter, submission, terminal execution or clipboard replacement. Changed targets and multiline results require review.</span></div>
    </section>
    <section className="panel">
      <h2>Temporary text recovery</h2>
      <p>Keep your words when delivery is rejected or uncertain. Check the destination before inserting again.</p>
      <button type="button" className="secondary" disabled={!snapshot.pending_text} onClick={() => perform(async () => setText(await request('get_recovery', undefined)))}>Show latest text</button>
      {text !== null && snapshot.pending_text && <>
        <textarea className="recovery-text" aria-label="Temporary transcript" readOnly value={text} rows={6} />
        <p className="muted">Select and copy manually, or choose a new destination. Text expires after {snapshot.defaults.recovery_seconds / 60} minutes and clears on pause, lock or exit.</p>
        <div className="inline-actions"><button type="button" className="secondary" onClick={() => perform(() => request('control', { action: 'prepare_pending' }))}>Choose a new input</button><button type="button" className="text-button" onClick={() => perform(async () => { await request('control', { action: 'discard' }); setText(null); })}>Discard text</button></div>
      </>}
    </section>
  </>;
};
