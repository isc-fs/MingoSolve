// Everything the engine can solve, loaded once: formulas, tools and the topic library, indexed by script id.
import { listFormulas } from './solve';
import { listTools } from './tools';
import { listTopics } from './topics';
import type { FormulaInfo, ScriptRef, TopicInfo, ToolInfo } from './types';

export interface Script {
    id: string;
    kind: 'formula' | 'tool';
    title: string;
    /** Extra search words (data/topics.toml [aliases]). */
    aliases: string;
    topic: TopicInfo | null;
    formula?: FormulaInfo;
    tool?: ToolInfo;
}

export const catalog = $state({
    ready: false,
    topics: [] as TopicInfo[],
    scripts: new Map<string, Script>(),
});

export async function loadCatalog(): Promise<void> {
    if (catalog.ready) return;
    const [formulas, tools, topics] = await Promise.all([listFormulas(), listTools(), listTopics()]);
    const topicOf = new Map<string, TopicInfo>();
    const refOf = new Map<string, ScriptRef>();
    for (const t of topics) {
        for (const s of t.scripts) {
            topicOf.set(s.id, t);
            refOf.set(s.id, s);
        }
    }
    const scripts = new Map<string, Script>();
    for (const f of formulas) {
        scripts.set(f.key, { id: f.key, kind: 'formula', title: f.title, aliases: refOf.get(f.key)?.aliases ?? '', topic: topicOf.get(f.key) ?? null, formula: f });
    }
    for (const t of tools) {
        const ref = refOf.get(t.name);
        scripts.set(t.name, { id: t.name, kind: 'tool', title: ref?.title ?? t.name, aliases: ref?.aliases ?? '', topic: topicOf.get(t.name) ?? null, tool: t });
    }
    catalog.topics = topics;
    catalog.scripts = scripts;
    catalog.ready = true;
}

/** Ranked search over every script: by name (id, title, variable symbols), then aliases and tags, variable
 *  descriptions, topic. Prefixes count ("susp"), one typo is forgiven in words of 4+ letters ("aceleration"), and
 *  scripts that match every word come first and those missing one word follow, so one extra word never empties
 *  the list. An empty query lists pinned, then recent, then everything in library order. */
export function searchScripts(query: string, limit = 30, prefer: string[] = []): Script[] {
    const all = [...catalog.scripts.values()];
    // split exactly like the index, so "friction-limited" is two words on both sides
    const queryWords = words(query).filter((w) => !STOP.has(w));
    const preferRank = (id: string) => {
        const i = prefer.indexOf(id);
        return i < 0 ? prefer.length : i;
    };
    if (queryWords.length === 0) {
        return [...all].sort((a, b) => preferRank(a.id) - preferRank(b.id)).slice(0, limit);
    }
    const whole = query.trim().toLowerCase().replaceAll(' ', '_');
    const scored = all.map((s, order) => {
        const fields = indexOf(s);
        let matched = 0;
        let score = s.id === whole ? 500 : 0;
        for (const w of queryWords) {
            let best = 0;
            for (const [tokens, weight] of fields) {
                for (const t of tokens) best = Math.max(best, weight * closeness(w, t));
            }
            if (best > 0) matched += 1;
            score += best;
        }
        return { s, matched, score, order };
    });
    // partial matches stay (one extra word never empties the list) but only those missing at most one word
    const best = Math.max(0, ...scored.map((r) => r.matched));
    return scored
        .filter((r) => r.matched > 0 && r.matched >= best - 1)
        .sort((a, b) => b.matched - a.matched || b.score - a.score || preferRank(a.s.id) - preferRank(b.s.id) || a.order - b.order)
        .slice(0, limit)
        .map((r) => r.s);
}

const STOP = new Set(['a', 'an', 'the', 'of', 'and', 'or', 'for', 'to', 'in', 'on', 'with', 'from', 'at', 'by', 'is', 'what', 'how']);

const index = new WeakMap<Script, [string[], number][]>();

function words(text: string): string[] {
    return text.toLowerCase().split(/[^a-z0-9_]+/).filter((w) => w.length > 0);
}

function indexOf(s: Script): [string[], number][] {
    let f = index.get(s);
    if (f === undefined) {
        const names = [s.id, ...words(s.id.replaceAll('_', ' ')), ...words(s.title), ...(s.formula?.vars.map((v) => v.name.toLowerCase()) ?? [])];
        const aliases = [...words(s.aliases), ...(s.formula?.tags.flatMap(words) ?? [])];
        const descs = s.formula?.vars.flatMap((v) => words(v.desc)) ?? words(s.tool?.doc ?? '');
        f = [
            [names, 100],
            [aliases, 70],
            [descs, 25],
            [words(s.topic?.name ?? ''), 15],
        ];
        index.set(s, f);
    }
    return f;
}

/** How well a query word matches one token: exact 1, prefix 0.6, inside the token 0.3, one typo 0.25. */
function closeness(w: string, t: string): number {
    if (w === t) return 1;
    if (w.length >= 2 && t.startsWith(w)) return 0.6;
    if (w.length >= 3 && t.includes(w)) return 0.3;
    if (w.length >= 4 && t.length >= 4 && withinOneEdit(w, t)) return 0.25;
    return 0;
}

/** True when one insertion, deletion, substitution or swap of neighbours turns a into b. */
function withinOneEdit(a: string, b: string): boolean {
    if (Math.abs(a.length - b.length) > 1) return false;
    let i = 0;
    while (i < a.length && i < b.length && a[i] === b[i]) i++;
    if (a.length === b.length) {
        if (a.slice(i + 1) === b.slice(i + 1)) return true;
        return a[i] === b[i + 1] && a[i + 1] === b[i] && a.slice(i + 2) === b.slice(i + 2);
    }
    return a.length > b.length ? a.slice(i + 1) === b.slice(i) : a.slice(i) === b.slice(i + 1);
}

/** A formula's variable symbols ("v · s · t"), how people recognise it at a glance; empty for tools. */
export function symbols(s: Script, max = 6): string {
    const names = s.formula?.vars.map((v) => v.name) ?? [];
    return names.slice(0, max).join(' · ') + (names.length > max ? ' …' : '');
}

/** Short label for compact lists: the script id in words ("battery_load" -> "Battery load"). */
export function shortName(id: string): string {
    const words = id.replaceAll('_', ' ');
    return words.charAt(0).toUpperCase() + words.slice(1);
}
