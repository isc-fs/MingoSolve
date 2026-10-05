import { clearMocks } from '@tauri-apps/api/mocks';
import { afterEach } from 'vitest';

afterEach(() => clearMocks());

// jsdom has no matchMedia; theme.svelte.ts reads it at import time
if (typeof window.matchMedia !== "function") {
    window.matchMedia = ((query: string) => ({ matches: false, media: query, addEventListener: () => {}, removeEventListener: () => {} })) as typeof window.matchMedia;
}
