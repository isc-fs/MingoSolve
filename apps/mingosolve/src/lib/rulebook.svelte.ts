// Which rulebooks are loaded, shared by the Rules view and the palette. Failures read as "none loaded".
import { rulebookStatus } from './rulebook';
import type { RulebookStatus } from './types';

/** Years the Rules view offers; each is a separate PDF. */
export const RULE_YEARS = ['2027', '2026', '2025'];

export const rulebooks = $state({ loaded: [] as RulebookStatus[] });

export async function refreshRulebooks(): Promise<void> {
    try {
        rulebooks.loaded = (await rulebookStatus()) ?? [];
    } catch {
        rulebooks.loaded = [];
    }
}

/** The year to search: the preferred one if loaded, else the newest loaded one; null when nothing is loaded. */
export function searchYear(preferred: string): string | null {
    const years = rulebooks.loaded.map((r) => r.year);
    if (years.includes(preferred)) return preferred;
    return years.length === 0 ? null : [...years].sort().at(-1)!;
}

/** Cut `text` into plain and highlighted parts by `[start, end)` UTF-16 spans. */
export function highlight(text: string, spans: [number, number][]): { text: string; mark: boolean }[] {
    const parts: { text: string; mark: boolean }[] = [];
    let at = 0;
    for (const [s, e] of [...spans].sort((a, b) => a[0] - b[0])) {
        if (s < at || e <= s) continue;
        if (s > at) parts.push({ text: text.slice(at, s), mark: false });
        parts.push({ text: text.slice(s, e), mark: true });
        at = e;
    }
    if (at < text.length) parts.push({ text: text.slice(at), mark: false });
    return parts;
}
