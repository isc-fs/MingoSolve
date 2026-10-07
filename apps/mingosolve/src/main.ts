import '@fontsource/jost/500.css';
import '@fontsource/jost/600.css';
import '@fontsource/ibm-plex-sans/400.css';
import '@fontsource/ibm-plex-sans/500.css';
import '@fontsource/ibm-plex-sans/600.css';
import '@fontsource/ibm-plex-mono/400.css';
import '@fontsource/ibm-plex-mono/500.css';
import '@fontsource/ibm-plex-mono/600.css';
// the other styles' faces: @font-face only, a file loads when a style first uses it
import '@fontsource/source-serif-4/600.css';
import '@fontsource/source-serif-4/700.css';
import '@fontsource/source-sans-3/400.css';
import '@fontsource/source-sans-3/500.css';
import '@fontsource/source-sans-3/600.css';
import '@fontsource/source-code-pro/400.css';
import '@fontsource/source-code-pro/500.css';
import '@fontsource/source-code-pro/600.css';
import '@fontsource/barlow/400.css';
import '@fontsource/barlow/500.css';
import '@fontsource/barlow/600.css';
import '@fontsource/barlow-semi-condensed/500.css';
import '@fontsource/barlow-semi-condensed/600.css';
import '@fontsource/red-hat-display/700.css';
import '@fontsource/red-hat-text/400.css';
import '@fontsource/red-hat-text/500.css';
import '@fontsource/red-hat-text/600.css';
import '@fontsource/red-hat-mono/400.css';
import '@fontsource/red-hat-mono/500.css';
import '@fontsource/red-hat-mono/600.css';
import '@fontsource/arimo/400.css';
import '@fontsource/arimo/500.css';
import '@fontsource/arimo/600.css';
import '@fontsource/arimo/700.css';
import '@fontsource/cousine/400.css';
import '@fontsource/cousine/700.css';
import '@fontsource/geist-sans/400.css';
import '@fontsource/geist-sans/500.css';
import '@fontsource/geist-sans/600.css';
import '@fontsource/geist-mono/400.css';
import '@fontsource/geist-mono/500.css';
import '@fontsource/geist-mono/600.css';
import '@fontsource/schibsted-grotesk/400.css';
import '@fontsource/schibsted-grotesk/500.css';
import '@fontsource/schibsted-grotesk/600.css';
import '@fontsource/schibsted-grotesk/800.css';
import 'katex/dist/katex.min.css';
import './app.css';
// after app.css: a style's [data-style] rules must win over the defaults of the same specificity
import './styles/prontuario.css';
import './styles/plano.css';
import './styles/salidas.css';
import './styles/datasheet.css';
import './styles/grafito.css';
import './styles/rosa.css';
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
