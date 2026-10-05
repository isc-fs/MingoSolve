// Settings persistence through the real plugin-store client against an in-memory store: what survives an
// upgrade (older files lack newer keys), and that the autosave never writes defaults over the user's file.
import { mockIPC } from '@tauri-apps/api/mocks';
import { beforeEach, expect, it, vi } from 'vitest';

type SettingsModule = typeof import('./settings.svelte');

let disk: Record<string, unknown>;
let writes: Record<string, unknown>[];
let releaseRead: () => void;

function fakeStore(opts: { slowLoad?: boolean; noRuntime?: boolean } = {}): void {
    mockIPC(async (cmd, payload) => {
        const args = (payload ?? {}) as Record<string, unknown>;
        if (opts.noRuntime) throw new Error('no Tauri runtime');
        if (cmd === 'plugin:store|load') return 1;
        if (cmd === 'plugin:store|get') {
            // the file is open but not read yet: the window where a save would write defaults over it
            if (opts.slowLoad) await new Promise<void>((r) => (releaseRead = r));
            return [disk[String(args.key)] ?? null, String(args.key) in disk];
        }
        if (cmd === 'plugin:store|set') {
            disk[String(args.key)] = JSON.parse(JSON.stringify(args.value));
            writes.push(JSON.parse(JSON.stringify(args.value)) as Record<string, unknown>);
            return null;
        }
        if (cmd === 'plugin:store|save') return null;
        throw new Error(`unexpected ${cmd}`);
    });
}

async function fresh(): Promise<SettingsModule & typeof import('../test/autosave.svelte')> {
    vi.resetModules();
    return { ...(await import('./settings.svelte')), ...(await import('../test/autosave.svelte')) };
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

beforeEach(() => {
    disk = {};
    writes = [];
});

it('a file from an older version keeps its choices and takes defaults for everything it lacks', async () => {
    disk.all = { theme: 'light', precision: { n: 3 }, retiredOption: true };
    fakeStore();
    const m = await fresh();
    await m.loadSettings();
    expect(m.settings.theme).toBe('light');
    expect(m.settings.precision).toEqual({ kind: 'sig', n: 3 });
    expect(m.settings.rules).toBe('2027');
    expect(m.settings.autoUpdateCheck).toBe(true);
    expect(m.settings.pinned).toEqual(m.defaultSettings().pinned);
});

it('a file from before the text size setting loads at 100 %, and a chosen size is saved and read back', async () => {
    disk.all = { theme: 'dark' };
    fakeStore();
    let m = await fresh();
    const stop = m.startAutosave();
    await m.loadSettings();
    expect(m.settings.textSize).toBe(100);
    m.settings.textSize = 130;
    m.flushSync();
    await sleep(350);
    expect(writes.at(-1)).toMatchObject({ theme: 'dark', textSize: 130 });
    stop();

    m = await fresh();
    await m.loadSettings();
    expect(m.settings.textSize).toBe(130);
});

it('a hand-edited text size that is not one of the offered sizes falls back to 100 %', async () => {
    disk.all = { textSize: 250 };
    fakeStore();
    const m = await fresh();
    await m.loadSettings();
    expect(m.settings.textSize).toBe(100);
});

it('an emptied rail stays empty after a restart instead of getting the default pins back', async () => {
    disk.all = { pinned: [], recent: ['ntc'] };
    fakeStore();
    const m = await fresh();
    await m.loadSettings();
    expect(m.settings.pinned).toEqual([]);
    expect(m.settings.recent).toEqual(['ntc']);
});

it('nulls in a hand-edited file do not wipe the defaults', async () => {
    disk.all = { theme: null, precision: { kind: null, n: 6 }, decimalComma: true };
    fakeStore();
    const m = await fresh();
    await m.loadSettings();
    expect(m.settings.theme).toBe('system');
    expect(m.settings.precision).toEqual({ kind: 'sig', n: 6 });
    expect(m.settings.decimalComma).toBe(true);
});

it("changes made before the file is read don't overwrite it with defaults; later edits save once, debounced", async () => {
    disk.all = { theme: 'dark', decimalComma: true };
    fakeStore({ slowLoad: true });
    const m = await fresh();
    const stop = m.startAutosave();
    const loading = m.loadSettings();
    m.settings.calcOpen = false;
    m.flushSync();
    await sleep(300);
    expect(writes).toEqual([]);
    releaseRead();
    await loading;
    expect(m.settings.theme).toBe('dark');
    m.settings.rules = 'legacy';
    m.flushSync();
    m.settings.precision.n = 5;
    m.flushSync();
    await sleep(350);
    expect(writes).toHaveLength(1);
    expect(writes[0]).toMatchObject({ theme: 'dark', decimalComma: true, rules: 'legacy', precision: { n: 5 } });
    stop();
});

it('without a Tauri runtime (browser preview) it runs on defaults and never tries to save', async () => {
    fakeStore({ noRuntime: true });
    const m = await fresh();
    const stop = m.startAutosave();
    await m.loadSettings();
    m.settings.theme = 'dark';
    m.flushSync();
    await sleep(300);
    expect(m.settings.rules).toBe('2027');
    expect(writes).toEqual([]);
    stop();
});

it('the Past questions folds load closed from an older file, and an opened fold is saved and read back', async () => {
    disk.all = { theme: 'dark', calcOpen: true };
    fakeStore();
    let m = await fresh();
    const stop = m.startAutosave();
    await m.loadSettings();
    expect(m.settings.chainPastOpen).toBe(false);
    expect(m.settings.calcPastOpen).toBe(false);
    m.settings.chainPastOpen = true;
    m.flushSync();
    await sleep(350);
    expect(writes.at(-1)).toMatchObject({ theme: 'dark', chainPastOpen: true, calcPastOpen: false });
    stop();

    m = await fresh();
    await m.loadSettings();
    expect(m.settings.chainPastOpen).toBe(true);
    expect(m.settings.calcPastOpen).toBe(false);
});
