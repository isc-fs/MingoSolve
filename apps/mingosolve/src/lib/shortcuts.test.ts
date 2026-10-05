// The search shortcut is written once, by platform.ts: macOS gets ⌘K, Windows and Linux get Ctrl K. Every text that
// mentions it must come from there, so the helper is replaced here and each place must show the replacement.
import { render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('./platform', async (original) => ({ ...(await original<typeof import('./platform')>()), searchShortcut: () => 'KEYS' }));

import Rail from './Rail.svelte';
import ScriptSheet from './ScriptSheet.svelte';
import SettingsView from './SettingsView.svelte';
import SolveView from './SolveView.svelte';
import { catalog } from './catalog.svelte';
import { session } from './session.svelte';
import { fakeEngine } from '../test/ipc';

beforeEach(() => {
    catalog.scripts = new Map();
    session.script = null;
    session.problem = '';
    session.sheet = null;
});

describe('the search shortcut in texts', () => {
    it('is the platform shortcut in the rail, the empty sheet and the Solve hint', async () => {
        fakeEngine({ find_question: () => ({ hits: [], quantities: [], past: null }) });
        const rail = render(Rail);
        expect(rail.container.querySelector('.search kbd')?.textContent).toBe('KEYS');
        rail.unmount();

        const sheet = render(ScriptSheet);
        expect(sheet.container.querySelector('.empty kbd')?.textContent).toBe('KEYS');
        sheet.unmount();

        render(SolveView);
        session.problem = 'something nothing matches';
        await waitFor(() => expect(screen.getByLabelText('Scripts that fit').textContent).toContain('No script fits yet'));
        expect(screen.getByLabelText('Scripts that fit').querySelector('kbd')?.textContent).toBe('KEYS');
    });

    it('is the platform shortcut in the Settings about text', () => {
        fakeEngine({});
        const { container } = render(SettingsView);
        const keys = [...container.querySelectorAll('kbd')].map((k) => k.textContent);
        expect(keys).toContain('KEYS');
        expect(container.textContent).not.toMatch(/⌘K|Ctrl K/);
    });
});
