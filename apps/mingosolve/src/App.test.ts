// Quiz-day safety: with "Check for updates automatically" off, starting the app must not contact the update
// server (the manual button in Settings is the only way); with it on (the default), it checks once.
import { mockIPC } from '@tauri-apps/api/mocks';
import { beforeEach, expect, it, vi } from 'vitest';

let disk: Record<string, unknown>;
let calls: string[];
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
    const { render, waitFor } = await import('@testing-library/svelte');
    const { default: App } = await import('./App.svelte');
    render(App);
    await waitFor(() => expect(calls).toContain('list_topics'));
    await new Promise((r) => setTimeout(r, 400)); // lets the settings autosave settle before the mock is cleared
}

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
