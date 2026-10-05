import '@fontsource/jost/500.css';
import '@fontsource/jost/600.css';
import '@fontsource/ibm-plex-sans/400.css';
import '@fontsource/ibm-plex-sans/500.css';
import '@fontsource/ibm-plex-sans/600.css';
import '@fontsource/ibm-plex-mono/400.css';
import '@fontsource/ibm-plex-mono/500.css';
import '@fontsource/ibm-plex-mono/600.css';
import 'katex/dist/katex.min.css';
import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app');
if (target === null) {
    throw new Error('#app target not found in index.html');
}

// End-to-end tests build with VITE_IPC_BRIDGE so the UI talks to the real engine over a local test bridge; in
// normal builds the condition is a compile-time constant and the e2e module is not bundled.
const ready = import.meta.env.VITE_IPC_BRIDGE
    ? import('./e2e-ipc').then((m) => m.installBridge(import.meta.env.VITE_IPC_BRIDGE!))
    : Promise.resolve();

void ready.then(() => mount(App, { target }));
