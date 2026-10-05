// The Help view lists every shortcut, and the main ones really do what the table says when the keys are pressed
// in the whole app (key events on the window, as the webview delivers them).
import { mockIPC } from '@tauri-apps/api/mocks';
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';

import App from '../App.svelte';
import HelpView from './HelpView.svelte';
import { session } from './session.svelte';
import { SHORTCUTS, keysFor } from './shortcuts';

beforeEach(() => {
    session.activeView = 'solve';
    session.paletteOpen = false;
    session.script = null;
    session.problem = '';
});

it('Help lists every shortcut with the keys for this platform and what it does', () => {
    render(HelpView);
    const rows = [...document.querySelectorAll<HTMLElement>('tr[data-shortcut]')];
    expect(rows.map((r) => r.dataset.shortcut)).toEqual(SHORTCUTS.map((s) => s.id));
    for (const s of SHORTCUTS) {
        const row = document.querySelector(`tr[data-shortcut="${s.id}"]`)!;
        // jsdom reports Linux: Ctrl labels, never the Mac glyphs
        expect(row.querySelector('kbd')?.textContent).toBe(keysFor(s, 'linux'));
        expect(row.textContent).toContain(s.does);
    }
    expect(screen.getByRole('heading', { level: 1, name: 'Help' })).toBeTruthy();
    expect(screen.getAllByRole('heading', { level: 2 }).map((h) => h.textContent)).toEqual([
        'Quick start',
        'Finding scripts',
        'Reading the answer',
        'Checking options',
        'Known-wrong keys',
        'Shortcuts',
        'Rehearsing',
        'If the app fails',
    ]);
});

it('Help gives the fallback commands from the install guide', () => {
    render(HelpView);
    const text = document.body.textContent ?? '';
    expect(text).toContain('cargo run --release');
    expect(text).toContain('cd legacy/python && uv sync && uv run fsq');
});

function startApp(): void {
    vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener: () => {} }));
    mockIPC(async (cmd) => {
        if (cmd === 'plugin:store|load') return 1;
        if (cmd === 'plugin:store|get') return [null, false];
        if (['list_formulas', 'list_tools', 'list_topics'].includes(cmd)) return [];
        return null;
    });
    const internals = (window as unknown as { __TAURI_INTERNALS__: Record<string, unknown> }).__TAURI_INTERNALS__;
    internals.metadata = { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } };
    render(App);
}

it('Ctrl+K opens and closes the palette; Ctrl+/ opens Help and the same keys go back', async () => {
    startApp();
    await waitFor(() => expect(document.querySelector('.rail')).not.toBeNull());
    await waitFor(() => expect(screen.queryByText('Loading…')).toBeNull());

    await userEvent.keyboard('{Control>}k{/Control}');
    expect(screen.getByRole('dialog', { name: 'Find a script' })).toBeTruthy();
    await userEvent.keyboard('{Escape}');
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    await userEvent.keyboard('{Control>}/{/Control}');
    expect(screen.getByRole('heading', { level: 1, name: 'Help' })).toBeTruthy();
    expect(session.activeView).toBe('help');
    await userEvent.keyboard('{Control>}/{/Control}');
    expect(screen.queryByRole('heading', { level: 1, name: 'Help' })).toBeNull();
    expect(session.activeView).toBe('solve');

    // from Settings it returns to Settings, and an open palette is closed by it
    session.activeView = 'settings';
    await userEvent.keyboard('{Control>}k{/Control}');
    await userEvent.keyboard('{Control>}/{/Control}');
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(session.activeView).toBe('help');
    await userEvent.keyboard('{Control>}/{/Control}');
    expect(session.activeView).toBe('settings');
});

it('the Help button in the rail opens the same view', async () => {
    startApp();
    await waitFor(() => expect(screen.queryByText('Loading…')).toBeNull());
    await userEvent.click(screen.getByRole('button', { name: /^Help/ }));
    expect(screen.getByRole('heading', { level: 1, name: 'Help' })).toBeTruthy();
});
