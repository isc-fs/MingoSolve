// Unit and component tests (jsdom). The end-to-end journeys against the real engine are in e2e/ (Playwright).
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig({
    plugins: [svelte(), svelteTesting()],
    // tests read the engine's data (data/examples.toml) at the repo root
    server: { fs: { allow: ['../..'] } },
    test: {
        environment: 'jsdom',
        include: ['src/**/*.test.ts'],
        setupFiles: ['src/test/setup.ts'],
    },
});
