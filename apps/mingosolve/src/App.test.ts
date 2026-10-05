// Quiz-day safety: with "Check for updates automatically" off, starting the app must not contact the update
// server (the manual button in Settings is the only way); with it on (the default), it checks once.
import { mockIPC } from '@tauri-apps/api/mocks';
import { screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

let disk: Record<string, unknown>;
let calls: string[];
let unmountApps: () => void = () => {};
let updateOffered: boolean;

function fakeApp(): void {
    mockIPC(async (cmd, payload) => {
        const args = (payload ?? {}) as Record<string, unknown>;
        calls.push(cmd);
        switch (cmd) {
            case 'plugin:store|load':
                return 1;
            case 'plugin:store|get':
                return [disk[String(args.key)] ?? null, String(args.key) in disk];
            case 'plugin:store|set':
                disk[String(args.key)] = args.value;
                return null;
            case 'plugin:updater|check':
                return updateOffered
                    ? { rid: 7, available: true, currentVersion: '0.5.0', version: '0.5.1', date: null, body: '', rawJson: {} }
                    : null;
            case 'list_formulas':
            case 'list_tools':
            case 'list_topics':
                return [];
            default:
                return null;
        }
    });
    // the theme effect asks for the current window, which the Tauri runtime describes at startup
    const internals = (window as unknown as { __TAURI_INTERNALS__: Record<string, unknown> }).__TAURI_INTERNALS__;
    internals.metadata = { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } };
}

async function startApp(): Promise<void> {
    vi.resetModules();
    const { render, cleanup } = await import('@testing-library/svelte');
    unmountApps = cleanup;
    const { default: App } = await import('./App.svelte');
    render(App);
    await waitFor(() => expect(calls).toContain('list_topics'));
    await new Promise((r) => setTimeout(r, 400)); // lets the settings autosave settle before the mock is cleared
}

// each start re-imports the app, so the previous instance has to be unmounted through its own testing-library copy
afterEach(() => unmountApps());

beforeEach(() => {
    vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener: () => {} }));
    disk = {};
    calls = [];
    updateOffered = false;
});

it('does not contact the update server on startup when automatic checks are off', async () => {
    disk.all = { autoUpdateCheck: false };
    fakeApp();
    await startApp();
    expect(calls).not.toContain('plugin:updater|check');
});

it('checks once on startup by default', async () => {
    fakeApp();
    await startApp();
    expect(calls.filter((c) => c === 'plugin:updater|check')).toHaveLength(1);
});

// ---- the guided tour ----

const tourDialog = () => screen.queryByRole('dialog', { name: /./ });
const savedTourDone = () => (disk.all as { tourDone?: boolean } | undefined)?.tourDone;

it('shows the tour on first launch, and once it is skipped it is saved and the next launch starts clean', async () => {
    fakeApp();
    await startApp();
    await waitFor(() => expect(tourDialog()).not.toBeNull());
    expect(screen.getByText(/^Step 1 of 7$/)).toBeTruthy();
    await userEvent.keyboard('{Escape}');
    expect(tourDialog()).toBeNull();
    await waitFor(() => expect(savedTourDone()).toBe(true));

    unmountApps();
    await startApp();
    expect(tourDialog()).toBeNull();
});

it('saves tourDone when the tour is finished with Done', async () => {
    fakeApp();
    await startApp();
    await waitFor(() => expect(tourDialog()).not.toBeNull());
    for (let i = 0; i < 6; i++) await userEvent.keyboard('{ArrowRight}');
    await userEvent.click(screen.getByRole('button', { name: 'Done' }));
    expect(tourDialog()).toBeNull();
    await waitFor(() => expect(savedTourDone()).toBe(true));
});

it('a settings file from before the tour existed (no tourDone) shows it once', async () => {
    disk.all = { theme: 'dark', pinned: ['battery_load'], autoUpdateCheck: false };
    fakeApp();
    await startApp();
    await waitFor(() => expect(tourDialog()).not.toBeNull());
    await userEvent.keyboard('{Escape}');
    await waitFor(() => expect(savedTourDone()).toBe(true));
    expect((disk.all as { theme: string }).theme).toBe('dark');

    unmountApps();
    await startApp();
    expect(tourDialog()).toBeNull();
});

it('does not start for someone who already did it; Take the tour in Help starts it again and Escape hands focus back', async () => {
    disk.all = { tourDone: true };
    fakeApp();
    await startApp();
    expect(tourDialog()).toBeNull();
    const { session } = await import('./lib/session.svelte');
    session.activeView = 'help';
    const button = await screen.findByRole('button', { name: 'Take the tour' });
    button.focus();
    await userEvent.keyboard('{Enter}');
    await waitFor(() => expect(tourDialog()).not.toBeNull());
    expect(screen.getByText(/^Step 1 of 7$/)).toBeTruthy();
    await userEvent.keyboard('{Escape}');
    expect(tourDialog()).toBeNull();
    // the tour walked to Solve and back, so Help was drawn again: focus is on the new Take the tour button
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Take the tour' })));
});

it('Take the tour in Settings starts it too', async () => {
    disk.all = { tourDone: true };
    fakeApp();
    await startApp();
    const { session } = await import('./lib/session.svelte');
    session.activeView = 'settings';
    await userEvent.click(await screen.findByRole('button', { name: 'Take the tour' }));
    await waitFor(() => expect(tourDialog()).not.toBeNull());
});

it('shows an offered update inside the main column, below the title-bar zone, and Later dismisses it', async () => {
    updateOffered = true;
    fakeApp();
    await startApp();
    const { screen, waitFor } = await import('@testing-library/svelte');
    const userEvent = (await import('@testing-library/user-event')).default;
    const banner = await waitFor(() => screen.getByText(/Update available/).closest('[role="status"]') as HTMLElement);
    expect(banner.textContent).toContain('v0.5.1');
    // not above the shell (where it was drawn under the macOS window buttons): a child of <main>
    expect(banner.closest('main')).not.toBeNull();
    expect(banner.closest('.shell')).not.toBeNull();
    await userEvent.setup().click(screen.getByRole('button', { name: 'Later' }));
    expect(screen.queryByText(/Update available/)).toBeNull();
});
