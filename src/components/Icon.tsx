type Props = { name: 'mic' | 'wave' | 'cursor' | 'power' | 'sliders' | 'arrow' | 'check'; size?: number };

export const Icon = ({ name, size = 20 }: Props) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.65" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    {name === 'mic' && <><rect x="8" y="2" width="8" height="13" rx="4" /><path d="M5 10v2a7 7 0 0 0 14 0v-2M12 19v3M8 22h8" /></>}
    {name === 'wave' && <path d="M3 10v4m4-8v12m5-16v20m5-15v10m4-7v4" />}
    {name === 'cursor' && <><path d="m5 3 14 10-7 1-3 7zM16 17l4 4" /></>}
    {name === 'power' && <><path d="M12 2v9M5 5a9 9 0 1 0 14 0" /></>}
    {name === 'sliders' && <><path d="M4 6h16M4 12h16M4 18h16" /><path d="M8 3v6m8 0v6m-6 0v6" /></>}
    {name === 'arrow' && <path d="M5 12h14m-6-6 6 6-6 6" />}
    {name === 'check' && <path d="m5 12 4 4L19 6" />}
  </svg>
);
