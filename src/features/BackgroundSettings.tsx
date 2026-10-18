import { Switch } from '../components/Fields';
import type { SettingsProps } from './AudioSettings';
import { request } from '../bridge/commands';

export const BackgroundSettings = ({ draft, snapshot, patch, perform }: SettingsProps) => (
  <>
    <section className="panel">
      <div className="panel-heading"><span className="step">04</span><div><h2>There when you need it</h2><p>Runs in your desktop session, with controls in the tray.</p></div></div>
      <Switch label="Launch at login" description="Start in the tray for the current user. Starting the app never starts a dictation recording." value={draft.launch_at_login} onChange={(launch_at_login) => patch({ launch_at_login })} />
      <p className="muted">Observed startup registration: {snapshot.startup_registered === null ? 'Unknown' : snapshot.startup_registered ? 'Registered' : 'Not registered'}.</p>
      <div className="inline-actions">
        <button type="button" className="secondary" aria-describedby="restart-description" onClick={() => perform(() => request('control', { action: 'restart' }))}>Restart application</button>
        <button type="button" className="secondary" onClick={() => perform(() => request('control', { action: 'quit' }))}>Quit application</button>
      </div>
      <p id="restart-description" className="muted">Restart the whole app from saved settings for a fresh startup. The current recording, temporary recovery text and unsaved settings changes are cleared.</p>
      <div className="rule"><strong>Close the window, keep your flow.</strong><span>Closing settings destroys its webview. Native tray and dictation controls continue running. Choose Quit from the tray to exit.</span></div>
    </section>
    <section className="panel">
      <h2>Private, by default</h2>
      <p>Wake recognition and speech transcription run on your device. After model installation, neither needs a network connection.</p>
      <dl className="diagnostics">
        <div><dt>Microphone capture</dt><dd>{snapshot.microphone_open ? 'Open' : 'Closed'}</dd></div>
        <div><dt>Wake detector</dt><dd>{snapshot.wake_listening ? 'Listening locally' : 'Not listening'}</dd></div>
        <div><dt>Whisper model memory</dt><dd>{snapshot.whisper_loaded ? 'Loaded' : 'Unloaded'}</dd></div>
        <div><dt>Transcription runs this session</dt><dd>{snapshot.inference_runs}</dd></div>
        <div><dt>Cloud processing</dt><dd>None</dd></div>
      </dl>
      <p className="muted">No recording history, transcript logs, analytics or automatic uploads. Pause releases microphone capture and stays in effect across restarts.</p>
    </section>
  </>
);
