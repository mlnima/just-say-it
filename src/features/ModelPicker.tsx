import { request } from '../bridge/commands';
import type { Model, Snapshot } from '../bridge/types';
import { Field } from '../components/Fields';

export const ModelPicker = ({ kind, value, onChange, models, snapshot, perform, refresh }: {
  kind: Model['kind']; value: string | null; onChange: (value: string | null) => void;
  models: Model[]; snapshot: Snapshot; perform: (task: () => Promise<unknown>) => void; refresh: () => Promise<void>;
}) => {
  const choices = models.filter((model) => model.kind === kind);
  const download = snapshot.download;
  return (
    <div className="model-picker">
      <Field label={kind === 'speech' ? 'Local Whisper model' : 'Local wake model'}>
        <select value={value ?? ''} onChange={(event) => onChange(event.target.value || null)}>
          <option value="">Not selected</option>
          {value && !choices.some((model) => model.installed_path === value) && <option value={value}>Imported · {value.split(/[\\/]/).pop()}</option>}
          {choices.filter((model) => model.installed_path).map((model) => <option key={model.id} value={model.installed_path || ''}>{model.label}</option>)}
        </select>
      </Field>
      {choices.map((model) => (
        <div className="model-download" key={model.id}>
          <div><strong>{model.label}</strong><small>{(model.bytes / 1_000_000).toFixed(1)} MB download · {model.installed_path ? 'Installed and load checked' : 'Not installed'}</small><small>{model.languages}</small><small>{model.license}</small></div>
          {!model.installed_path && <button type="button" className="secondary" disabled={!!download} onClick={() => perform(async () => { await request('install_model', { id: model.id }); await refresh(); })}>Download</button>}
        </div>
      ))}
      {download && choices.some((model) => model.id === download.id) && <div className="download-progress"><progress value={download.received} max={download.total} aria-label="Model download progress" /><button type="button" className="text-button" onClick={() => perform(() => request('cancel_download', undefined))}>Cancel download</button></div>}
      <button type="button" className="text-button" onClick={() => perform(async () => { const path = await request('import_model', { kind }); path && onChange(path); })}>Import an existing {kind === 'speech' ? 'GGML model file' : 'English wake model folder'}</button>
      <small className="model-footnote">Downloads are optional, checksum verified, and use approximately three times the download size as temporary installation space.</small>
    </div>
  );
};
