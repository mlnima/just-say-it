import { Field } from '../components/Fields';
import { ModelPicker } from './ModelPicker';
import { ModelRetentionSettings } from './ModelRetentionSettings';
import { SpeechDeviceSettings } from './SpeechDeviceSettings';
import { request } from '../bridge/commands';
import type { Microphone, Model, Settings, Snapshot } from '../bridge/types';

export type SettingsProps = {
  draft: Settings;
  snapshot: Snapshot;
  patch: (change: Partial<Settings>) => void;
  perform: (task: () => Promise<unknown>) => void;
};

export const AudioSettings = ({ draft, snapshot, patch, perform, microphones, models, refreshModels, refreshMicrophones }: SettingsProps & {
  microphones: Microphone[]; models: Model[]; refreshModels: () => Promise<void>; refreshMicrophones: () => Promise<void>;
}) => (
  <>
    <section className="panel">
      <div className="panel-heading"><span className="step">01</span><div><h2>Give your voice a way in</h2><p>Choose your microphone and local speech model.</p></div></div>
      <Field label="Microphone" hint="Only an explicitly selected system-default option follows device changes.">
        <select value={draft.microphone ?? ''} onChange={(event) => patch({ microphone: event.target.value || null })}>
          <option value="">Not selected</option>
          {draft.microphone && !microphones.some((mic) => mic.id === draft.microphone) && <option value={draft.microphone}>Saved device · unavailable</option>}
          {microphones.map((mic) => <option key={mic.id} value={mic.id}>{mic.label}</option>)}
        </select>
      </Field>
      <div className="inline-actions"><button type="button" className="secondary" onClick={() => perform(refreshMicrophones)}>Refresh devices</button><button type="button" className="secondary" onClick={() => perform(() => request('test_microphone', undefined))}>Test saved microphone</button></div>
      <div className="meter" role="meter" aria-label="Observed microphone level" aria-valuemin={0} aria-valuemax={1} aria-valuenow={snapshot.microphone_level}><span style={{ width: `${Math.min(100, snapshot.microphone_level * 400)}%` }} /></div>
      <ModelPicker kind="speech" value={draft.speech_model} onChange={(value) => patch({ speech_model: value })} models={models} snapshot={snapshot} perform={perform} refresh={refreshModels} />
      <SpeechDeviceSettings draft={draft} snapshot={snapshot} patch={patch} />
      <ModelRetentionSettings draft={draft} snapshot={snapshot} patch={patch} />
      <Field label="Dictation language">
        <select value={draft.language ?? ''} onChange={(event) => patch({ language: event.target.value || null })}>
          <option value="">Not selected</option>
          {snapshot.languages.map((language) => <option key={language.id} value={language.id}>{language.label}</option>)}
        </select>
      </Field>
    </section>
    <div className="note">Whisper uses up to {snapshot.defaults.inference_threads} CPU threads and can offload supported operations when GPU is selected and available. Keeping the model loaded uses memory and avoids reloading it for the next dictation; it performs no inference while idle.</div>
  </>
);
