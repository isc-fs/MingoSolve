import '@fontsource/jost/500.css';
import '@fontsource/jost/600.css';
import '@fontsource/ibm-plex-sans/400.css';
import '@fontsource/ibm-plex-sans/500.css';
import '@fontsource/ibm-plex-sans/600.css';
import '@fontsource/ibm-plex-mono/400.css';
import '@fontsource/ibm-plex-mono/500.css';
import '@fontsource/ibm-plex-mono/600.css';
import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app');
if (target === null) {
    throw new Error('#app target not found in index.html');
}

mount(App, { target });
