// Session log for rehearsal: one entry per copied answer, kept in its own plugin-store file (last 500), with the
// question label, the time since the problem was pasted or opened, and an optional mock-quiz id.
import { load, type Store } from '@tauri-apps/plugin-store';

export const LOG_CAP = 500;

export interface LogEntry {
    /** ISO 8601, UTC, whole seconds. */
    at: string;
    /** Mock quiz the entry belongs to, if one was running. */
    quiz: string | null;
    /** What the user typed in the sheet's "Question #" field. */
    question: string;
    scriptId: string;
    title: string;
    /** Name and value exactly as typed, units included. */
    inputs: [string, string][];
    /** The text that went to the clipboard. */
    answer: string;
    rules: string | null;
    /** Seconds since the problem was pasted or the script opened; null when no clock was running. */
    elapsedS: number | null;
    via: 'button' | 'shortcut' | 'copy-all';
}

export const sessionLog = $state({
    entries: [] as LogEntry[],
    quiz: null as { id: string; startedAt: string } | null,
    /** Current question label, shared by whatever scripts are opened for that question. */
    label: '',
});

const STORE_FILE = 'session-log.json';
const STORE_KEY = 'all';
const SAVE_DEBOUNCE_MS = 250;

let store: Store | null = null;
let loaded = false;
let loadPromise: Promise<void> | null = null;
let saveTimer: ReturnType<typeof setTimeout> | null = null;
let clockStart: number | null = null;

export function loadSessionLog(): Promise<void> {
    if (loadPromise !== null) return loadPromise;
    loadPromise = (async () => {
        try {
            store = await load(STORE_FILE);
            const stored = await store.get<{ entries?: LogEntry[]; quiz?: typeof sessionLog.quiz }>(STORE_KEY);
            if (stored !== undefined && stored !== null) {
                // entries appended before the file was read stay on top of what was on disk
                const early = sessionLog.entries;
                sessionLog.entries = [...(Array.isArray(stored.entries) ? stored.entries : []), ...early].slice(-LOG_CAP);
                if (sessionLog.quiz === null) sessionLog.quiz = stored.quiz ?? null;
            }
        } catch {
            // No Tauri runtime (plain browser preview): the log lives for this run only.
        }
        loaded = true;
        if (sessionLog.entries.length > 0) scheduleSave();
    })();
    return loadPromise;
}

export async function saveSessionLog(): Promise<void> {
    if (!loaded || store === null) return;
    await store.set(STORE_KEY, { entries: sessionLog.entries, quiz: sessionLog.quiz });
    await store.save();
}

function scheduleSave(): void {
    if (!loaded) return;
    if (saveTimer !== null) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
        saveTimer = null;
        void saveSessionLog();
    }, SAVE_DEBOUNCE_MS);
}

const isoSeconds = (ms: number): string => new Date(ms).toISOString().replace(/\.\d{3}Z$/, 'Z');

/** Start timing a question (a problem was pasted, or a script was opened with none pasted). The label is per question. */
export function startClock(): void {
    clockStart = Date.now();
    sessionLog.label = '';
}

export function stopClock(): void {
    clockStart = null;
    sessionLog.label = '';
}

export const clockRunning = (): boolean => clockStart !== null;

let lastProblem = '';

/** A paste replaces the problem; plain typing into it does not restart the clock. */
export function problemChanged(next: string): void {
    const prev = lastProblem;
    lastProblem = next;
    if (next.trim().length === 0) stopClock();
    else if (prev.trim().length === 0 || next.length - prev.length > 20 || (!next.startsWith(prev) && next.length > 20)) startClock();
}

export type CopyRecord = Pick<LogEntry, 'scriptId' | 'title' | 'inputs' | 'answer' | 'rules' | 'via'>;

export function recordCopy(r: CopyRecord): void {
    const now = Date.now();
    sessionLog.entries = [
        ...sessionLog.entries,
        {
            at: isoSeconds(now),
            quiz: sessionLog.quiz?.id ?? null,
            question: sessionLog.label.trim(),
            ...r,
            elapsedS: clockStart === null ? null : Math.max(0, Math.round((now - clockStart) / 1000)),
        },
    ].slice(-LOG_CAP);
    scheduleSave();
}

/** Reset the timer and tag every following entry with a new session id. */
export function startMockQuiz(): void {
    const now = Date.now();
    sessionLog.quiz = { id: `mock-${isoSeconds(now).replace(/[-:]/g, '').slice(0, 13)}`, startedAt: isoSeconds(now) };
    startClock();
    scheduleSave();
}

export function endMockQuiz(): void {
    sessionLog.quiz = null;
    scheduleSave();
}

export function clearLog(): void {
    sessionLog.entries = [];
    scheduleSave();
}
