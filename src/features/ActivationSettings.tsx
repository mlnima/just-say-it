import { Field, Switch } from '../components/Fields';
import { ModelPicker } from './ModelPicker';
import { request } from '../bridge/commands';
import type { Model } from '../bridge/types';
import type { SettingsProps } from './AudioSettings';

export const ActivationSettings = ({ draft, snapshot, patch, perform, models, refreshModels }: SettingsProps & { models: Model[]; refreshModels: () => Promise<void> }) => (
  <>
    <section className="panel">
      <div className="panel-heading"><span className="step">02</span><div><h2>Your words, your cue</h2><p>A small local detector listens for your phrase. Whisper stays idle.</p></div></div>
      <ModelPicker kind="wake" value={draft.wake_model} onChange={(value) => patch({ wake_model: value })} models={models} snapshot={snapshot} perform={perform} refresh={refreshModels} />
      <Field label="Wake phrase" hint="English phrases only with the bundled wake model. Uncommon phrases of several words reduce incidental triggers.">
        <input value={draft.wake_phrase ?? ''} placeholder="For example: hello little notebook" maxLength={80} onChange={(event) => patch({ wake_phrase: event.target.value || null })} />
      </Field>
      <Switch label="Listen for my wake phrase" description="Keeps microphone capture and the keyword detector active when ready. Other voices or speaker audio can trigger it." value={draft.wake_enabled} onChange={(wake_enabled) => patch({ wake_enabled })} />
      <div className="inline-actions"><button type="button" className="secondary" onClick={() => perform(() => request('test_phrase', undefined))}>Test saved phrase</button><span className="muted">Observed detections: {snapshot.phrase_detections}</span></div>
      <p className="callout">Say your phrase, pause briefly, then speak when the input button says “Finish”. Immediate speech without that pause is not supported yet.</p>
    </section>
    <section className="panel">
      <h2>A keyboard route, too</h2>
      <div className="two-columns">
        <Field label="Toggle recording" hint="Press once to start and again to finish."><input value={draft.toggle_shortcut ?? ''} placeholder="Control+Shift+Space" onChange={(event) => patch({ toggle_shortcut: event.target.value || null })} /></Field>
        <Field label="Cancel recording" hint="Discards the active utterance."><input value={draft.cancel_shortcut ?? ''} placeholder="Control+Shift+Backspace" onChange={(event) => patch({ cancel_shortcut: event.target.value || null })} /></Field>
      </div>
      <p className="muted">Actual registration: {snapshot.shortcuts_registered ? 'Toggle registered' : 'Toggle not registered'}. Tray Finish and Cancel are also available.</p>
    </section>
    <section className="panel">
      <h2>Know when to stop</h2>
      <Field label="Stop phrase" hint="Recognized only as an isolated command surrounded by silence."><input value={draft.stop_phrase ?? ''} placeholder="Choose a phrase different from your wake phrase" onChange={(event) => patch({ stop_phrase: event.target.value || null })} /></Field>
      <Switch label="Finish with my stop phrase" description="Uses the selected local keyword model during recording." value={draft.stop_enabled} onChange={(stop_enabled) => patch({ stop_enabled })} />
      <div className="two-columns">
        <Field label="Silence timeout (seconds)" hint={`Unset: ${snapshot.defaults.silence_seconds}s for wake sessions; toggle sessions finish manually.`}><input type="number" min="0.5" max="30" step="0.5" value={draft.silence_seconds ?? ''} placeholder="Not configured" onChange={(event) => patch({ silence_seconds: event.target.value ? Number(event.target.value) : null })} /></Field>
        <Field label="Recording limit (seconds)" hint={`Runtime limit when unset: ${snapshot.defaults.recording_limit_seconds}s.`}><input type="number" min="5" max="120" value={draft.recording_limit_seconds ?? ''} placeholder="Not configured" onChange={(event) => patch({ recording_limit_seconds: event.target.value ? Number(event.target.value) : null })} /></Field>
      </div>
      <Field label="No-speech timeout (seconds)" hint={`Runtime timeout when unset: ${snapshot.defaults.no_speech_seconds}s.`}><input type="number" min="1" max="30" value={draft.no_speech_seconds ?? ''} placeholder="Not configured" onChange={(event) => patch({ no_speech_seconds: event.target.value ? Number(event.target.value) : null })} /></Field>
    </section>
  </>
);
