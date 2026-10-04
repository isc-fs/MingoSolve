// Session state shared across views: the open script and its pre-filled values, the pasted problem, the palette,
// the calculator input. Pinned and recent scripts live in settings (persisted).
import type { ViewId } from './stores';
import { settings } from './settings.svelte';

export interface OpenScript {
    id: string;
    /** Values to pre-fill (variable or parameter name, value as typed). */
    values: [string, string][];
    display?: Record<string, string>;
    /** Positional tool arguments (from worked examples). */
    positional?: string[];
    /** Bumped on every open so the sheet resets even when the same script is reopened. */
    nonce: number;
}

export const session = $state({
    activeView: 'solve' as ViewId,
    script: null as OpenScript | null,
    problem: '',
    paletteOpen: false,
    /** Topic to expand when the Topics view opens. */
    topic: null as string | null,
    calcInput: '',
    /** Values each script can take from the pasted problem (finder pre-fill), by script id. */
    problemFills: {} as Record<string, [string, string][]>,
});

let nonce = 0;

/** Open a script; with no values given, it takes whatever the pasted problem fills in for it. */
export function openScript(id: string, values: [string, string][] = [], extra: Partial<OpenScript> = {}): void {
    if (values.length === 0 && extra.positional === undefined) values = session.problemFills[id] ?? [];
    nonce += 1;
    session.script = { id, values, ...extra, nonce };
    session.activeView = 'solve';
    settings.recent = [id, ...settings.recent.filter((r) => r !== id)].slice(0, 8);
}

export function togglePin(id: string): void {
    settings.pinned = settings.pinned.includes(id)
        ? settings.pinned.filter((p) => p !== id)
        : [...settings.pinned, id];
}

/** Split an engine command line: whitespace separates, double quotes group. */
export function splitCommand(line: string): string[] {
    const out: string[] = [];
    let cur = '';
    let quoted = false;
    let any = false;
    for (const c of line) {
        if (c === '"') {
            quoted = !quoted;
            any = true;
        } else if (/\s/.test(c) && !quoted) {
            if (any || cur.length > 0) {
                out.push(cur);
                cur = '';
                any = false;
            }
        } else {
            cur += c;
        }
    }
    if (any || cur.length > 0) out.push(cur);
    return out;
}

/** Open a worked example (an engine command such as "battery_load N_s=103 P=30kW @I=A") in its script. */
export function openCommand(cmd: string): void {
    const [head, ...rest] = splitCommand(cmd);
    if (head === undefined) return;
    if (head === 'calc') {
        session.calcInput = rest.join(' ');
        return;
    }
    const values: [string, string][] = [];
    const display: Record<string, string> = {};
    const positional: string[] = [];
    for (const arg of rest) {
        const eq = arg.indexOf('=');
        if (eq > 0) {
            const k = arg.slice(0, eq);
            if (k.startsWith('@')) display[k.slice(1)] = arg.slice(eq + 1);
            else values.push([k, arg.slice(eq + 1)]);
        } else {
            positional.push(arg);
        }
    }
    if (head === 'chain') return;
    openScript(head, values, { display, positional });
}
