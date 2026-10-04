// Runs the settings autosave effect outside a component (imported after vi.resetModules, so it shares the fresh
// Svelte runtime with the settings module under test).
import { flushSync } from 'svelte';
import { registerAutosaveEffect } from '../lib/settings.svelte';

export function startAutosave(): () => void {
    return $effect.root(() => registerAutosaveEffect());
}

export { flushSync };
