import { Field } from '../components/Fields';
import type { SettingsProps } from './AudioSettings';
import type { SpeechDevice } from '../bridge/types';

const deviceLabels: Record<SpeechDevice, string> = { cpu: 'CPU', gpu: 'GPU' };

export const SpeechDeviceSettings = ({ draft, snapshot, patch }: Pick<SettingsProps, 'draft' | 'snapshot' | 'patch'>) => (
  <>
    <Field label="Transcription device" hint={`When unset, transcription uses ${deviceLabels[snapshot.defaults.speech_device]}. Save to apply the device to your speech model.`}>
      <select value={draft.speech_device ?? ''} onChange={(event) => patch({ speech_device: (event.target.value || null) as SpeechDevice | null })}>
        <option value="">Not selected</option>
        <option value="cpu">CPU</option>
        <option value="gpu" disabled={!snapshot.gpu.device}>GPU{snapshot.gpu.device ? ` · ${snapshot.gpu.device.name}` : ' · unavailable'}</option>
      </select>
    </Field>
    <p className="observed" role="status">
      {!snapshot.gpu.checked && 'Checking GPU acceleration availability…'}
      {snapshot.gpu.checked && (snapshot.gpu.device ? `GPU backend initialized: ${snapshot.gpu.device.name}.` : snapshot.gpu.reason)}
    </p>
    <p className="muted">
      {snapshot.whisper_loaded && snapshot.speech_device_request
        ? `Loaded model: ${deviceLabels[snapshot.speech_device_request]}${snapshot.speech_device_request === 'gpu' ? ' offload requested. Whisper may also use CPU operations.' : '.'}`
        : 'Speech model is not loaded.'}
    </p>
  </>
);
