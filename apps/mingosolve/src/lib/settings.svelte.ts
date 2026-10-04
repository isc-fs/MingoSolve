// Persistent settings (MingoCAN pattern): one reactive object, loaded from the plugin-store JSON in the OS
// app-config folder, saved by a debounced autosave effect that no-ops until the first load finished.
import { load, type Store } from '@tauri-apps/plugin-store';

export type RuleYear = '2027' | '2026' | 'legacy';

export interface Settings {
    /** Rule set used by scoring tools unless a tool call names another. */
    rules: RuleYear;
    precision: { kind: 'sig' | 'decimals'; n: number };
    /** Copy answers with a decimal comma (most FS-Quiz input fields accept both; some quizzes ask for commas). */
    decimalComma: boolean;
}

export function defaultSettings(): Settings {
    return { rules: '2027', precision: { kind: 'sig', n: 4 }, decimalComma: false };
}

export const settings = $state<Settings>(defaultSettings());

const STORE_FILE = 'settings.json';
const STORE_KEY = 'all';
const SAVE_DEBOUNCE_MS = 250;

let store: Store | null = null;
let loaded = false;
let saveTimer: ReturnType<typeof setTimeout> | null = null;
let loadPromise: Promise<void> | null = null;

export function loadSettings(): Promise<void> {
    if (loadPromise !== null) return loadPromise;
    loadPromise = (async () => {
        try {
            store = await load(STORE_FILE);
            const stored = await store.get<Partial<Settings>>(STORE_KEY);
            if (stored !== undefined && stored !== null) {
                mergeInto(settings, stored);
            }
        } catch {
            // No Tauri runtime (plain browser preview): run on defaults.
        }
        loaded = true;
    })();
    return loadPromise;
}

export async function saveSettings(): Promise<void> {
    if (!loaded || store === null) return;
    await store.set(STORE_KEY, settings);
    await store.save();
}

function scheduleSave(): void {
    if (!loaded) return;
    if (saveTimer !== null) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
        saveTimer = null;
        void saveSettings();
    }, SAVE_DEBOUNCE_MS);
}

/** Call once from App.svelte: any settings change triggers a debounced save. */
export function registerAutosaveEffect(): void {
    $effect(() => {
        JSON.stringify(settings);
        scheduleSave();
    });
}

function mergeInto<T extends object>(target: T, source: Partial<T>): void {
    for (const key in source) {
        const incoming = source[key];
        if (incoming === undefined || incoming === null) continue;
        const current = (target as Record<string, unknown>)[key];
        if (
            typeof incoming === 'object' &&
            !Array.isArray(incoming) &&
            typeof current === 'object' &&
            current !== null &&
            !Array.isArray(current)
        ) {
            mergeInto(current as object, incoming as object);
        } else {
            (target as Record<string, unknown>)[key] = incoming;
        }
    }
}
