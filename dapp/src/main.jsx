import { Buffer } from 'buffer';
window.Buffer = Buffer;
globalThis.Buffer = Buffer;

import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';

createRoot(document.getElementById('root')).render(<App />);
