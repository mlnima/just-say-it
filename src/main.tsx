import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import './styles/theme.css';
import './styles/base.css';
import './styles/layout.css';
import './styles/controls.css';

const container = document.getElementById('root');
container && createRoot(container).render(<StrictMode><App /></StrictMode>);
