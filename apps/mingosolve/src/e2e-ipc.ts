// End-to-end test mode only (imported when the frontend is built with VITE_IPC_BRIDGE). App commands go to the
// test bridge, which runs the real Rust command functions (src-tauri/examples/ipc_bridge.rs); plugin calls the
// browser can't make get small fakes. The settings store persists to localStorage so a page reload tests the
// app's real load / autosave / merge logic.
import { Channel } from '@tauri-apps/api/core';
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';

export function installBridge(bridge: string): void {
    mockWindows('main');
    let nextRid = 1;
    const stores = new Map<number, string>();
    const read = (path: string): Record<string, unknown> => JSON.parse(localStorage.getItem(`e2e-store:${path}`) ?? '{}');
    mockIPC(async (cmd, payload) => {
        const args = (payload ?? {}) as Record<string, unknown>;
        if (cmd === 'plugin:store|load') {
            const rid = nextRid++;
            stores.set(rid, String(args.path));
            return rid;
        }
        if (cmd === 'plugin:store|get') {
            const data = read(stores.get(Number(args.rid))!);
            const key = String(args.key);
            return [data[key] ?? null, key in data];
        }
        if (cmd === 'plugin:store|set') {
            const path = stores.get(Number(args.rid))!;
            const data = read(path);
            data[String(args.key)] = args.value;
            localStorage.setItem(`e2e-store:${path}`, JSON.stringify(data));
            return null;
        }
        if (cmd === 'plugin:store|save') return null;
        if (cmd === 'plugin:app|version') return '0.0.0-e2e';
        if (cmd === 'plugin:updater|check') return fakeUpdate(args);
        if (cmd === 'plugin:updater|download_and_install') return fakeDownload(args);
        if (cmd === 'plugin:process|restart') return null;
        if (cmd.startsWith('plugin:window|')) return null;
        if (cmd.startsWith('plugin:')) throw new Error(`e2e: unmocked plugin call ${cmd}`);
        const res = await fetch(`${bridge}/invoke/${cmd}`, { method: 'POST', body: JSON.stringify(args) });
        const body: unknown = await res.json();
        if (!res.ok) throw body;
        return body;
    });
}

// Specs set localStorage['e2e-update'] = 'available' before load to make the updater find a newer version.
// The shapes follow @tauri-apps/plugin-updater: check() resolves metadata, downloadAndInstall() sends Started /
// Progress / Finished events on a Channel.
function fakeUpdate(_args: Record<string, unknown>): unknown {
    if (localStorage.getItem('e2e-update') !== 'available') return null;
    return {
        rid: 9001,
        available: true,
        currentVersion: '0.0.0-e2e',
        version: '9.9.9',
        date: '2026-10-01T00:00:00Z',
        body: 'Fake release notes',
        rawJson: {},
    };
}

async function fakeDownload(args: Record<string, unknown>): Promise<null> {
    if (localStorage.getItem('e2e-update-fail') === '1') throw new Error('e2e: download failed');
    const onEvent = args.onEvent as Channel<unknown>;
    onEvent.onmessage({ event: 'Started', data: { contentLength: 1000 } });
    onEvent.onmessage({ event: 'Progress', data: { chunkLength: 400 } });
    onEvent.onmessage({ event: 'Finished' });
    return null;
}
