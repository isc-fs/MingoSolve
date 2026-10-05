// Persistent settings: one reactive object, loaded from the plugin-store JSON in the OS app-config folder and
// saved by a debounced autosave effect that no-ops until the first load finished.
import { load, type Store } from '@tauri-apps/plugin-store';

export type RuleYear = '2027' | '2026' | 'legacy';
export const RULE_YEARS: RuleYear[] = ['2027', '2026', 'legacy'];
export type Theme = 'system' | 'dark' | 'light';
/** Text size in percent of the browser's default; the root font size and every rem-based size follow it. */
export const TEXT_SIZES = [90, 100, 115, 130] as const;
export type TextSize = (typeof TEXT_SIZES)[number];

export interface Settings {
    /** Night glass (dark), Paper glass (light), or follow the OS. */
    theme: Theme;
    /** Opaque surfaces instead of glass (also applied when the OS asks to reduce transparency). */
    solid: boolean;
    textSize: TextSize;
    /** Script ids pinned to the rail, and the most recently opened ones. */
    pinned: string[];
    recent: string[];
    calcOpen: boolean;
    /** Rule set used by scoring tools unless a tool call names another. */
    rules: RuleYear;
    precision: { kind: 'sig' | 'decimals'; n: number };
    /** Copy answers with a decimal comma (most FS-Quiz input fields accept both; some quizzes ask for commas). */
    decimalComma: boolean;
    /** Look for a newer version when the app starts. The manual button in Settings works either way. */
    autoUpdateCheck: boolean;
}

export function defaultSettings(): Settings {
    return {
        theme: 'system',
        solid: false,
        textSize: 100,
        pinned: ['battery_load', 'cornering_downforce', 'event_score'],
        recent: [],
        calcOpen: true,
        rules: '2027',
        precision: { kind: 'sig', n: 4 },
        decimalComma: false,
        autoUpdateCheck: true,
    };
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
                if (!(TEXT_SIZES as readonly number[]).includes(settings.textSize)) settings.textSize = 100;
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
