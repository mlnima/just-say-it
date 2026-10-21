import { Field } from '../components/Fields';
import type { Settings } from '../bridge/types';
import type { SettingsProps } from './AudioSettings';

const durationLabel = (seconds: number) => {
  if (seconds === 0) return 'Immediately';
  const minutes = seconds / 60;
  return minutes < 60 ? `${minutes} minutes` : `${minutes / 60} ${minutes === 60 ? 'hour' : 'hours'}`;
};

const retentionFromValue = (value: string): Settings['model_retention'] =>
  value === 'never' ? 'never' : { idle_seconds: Number(value) };

export const ModelRetentionSettings = ({ draft, snapshot, patch }: Pick<SettingsProps, 'draft' | 'snapshot' | 'patch'>) => (
  <>
    <Field label="Unload speech model after" hint="Never unload is the default and prepares the selected model at startup. A timeout releases it after inactivity. Listening follows your activation settings.">
      <select
        value={draft.model_retention === 'never' ? 'never' : draft.model_retention.idle_seconds}
        onChange={(event) => patch({ model_retention: retentionFromValue(event.target.value) })}
      >
        <option value="never">Never unload</option>
        {snapshot.defaults.model_idle_options_seconds.map((seconds) => <option key={seconds} value={seconds}>{durationLabel(seconds)}</option>)}
      </select>
    </Field>
    <p className="observed">Speech model: {snapshot.whisper_loaded ? 'Loaded in memory' : 'Not loaded'}</p>
  </>
);
