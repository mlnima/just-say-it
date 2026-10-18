import { cloneElement, useId, type ReactElement } from 'react';

export const Field = ({ label, hint, children }: { label: string; hint?: string; children: ReactElement<{ id?: string; 'aria-describedby'?: string }> }) => {
  const id = useId();
  return <div className="field"><label htmlFor={id}>{label}</label>{cloneElement(children, { id, 'aria-describedby': hint ? `${id}-hint` : undefined })}{hint && <small id={`${id}-hint`}>{hint}</small>}</div>;
};

export const Switch = ({ label, description, value, onChange, disabled }: {
  label: string; description: string; value: boolean | null; onChange: (value: boolean) => void; disabled?: boolean;
}) => (
  <div className="switch-row">
    <div><strong>{label}</strong><p>{description}</p>{value === null && <small className="unset">Not configured</small>}</div>
    <button type="button" role="switch" aria-label={label} aria-checked={value === true} disabled={disabled} className="switch" data-on={value === true} onClick={() => onChange(value !== true)}><span /></button>
  </div>
);
