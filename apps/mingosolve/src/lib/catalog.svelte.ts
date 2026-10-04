// Everything the engine can solve, loaded once: formulas, tools and the topic library, indexed by script id.
import { listFormulas } from './solve';
import { listTools } from './tools';
import { listTopics } from './topics';
import type { FormulaInfo, TopicInfo, ToolInfo } from './types';

export interface Script {
    id: string;
    kind: 'formula' | 'tool';
    title: string;
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
    for (const t of topics) for (const s of t.scripts) topicOf.set(s.id, t);
    const scripts = new Map<string, Script>();
    for (const f of formulas) {
        scripts.set(f.key, { id: f.key, kind: 'formula', title: f.title, topic: topicOf.get(f.key) ?? null, formula: f });
    }
    for (const t of tools) {
        const title = topics.flatMap((x) => x.scripts).find((s) => s.id === t.name)?.title ?? t.name;
        scripts.set(t.name, { id: t.name, kind: 'tool', title, topic: topicOf.get(t.name) ?? null, tool: t });
    }
    catalog.topics = topics;
    catalog.scripts = scripts;
    catalog.ready = true;
}

/** Scripts whose id, title, topic or variable descriptions contain every word of `query`. */
export function searchScripts(query: string, limit = 30): Script[] {
    const words = query.toLowerCase().split(/\s+/).filter((w) => w.length > 0);
    const all = [...catalog.scripts.values()];
    if (words.length === 0) return all.slice(0, limit);
    return all
        .filter((s) => {
            const extra = s.formula?.vars.map((v) => `${v.name} ${v.desc}`).join(' ') ?? s.tool?.doc ?? '';
            const hay = `${s.id.replaceAll('_', ' ')} ${s.title} ${s.topic?.name ?? ''} ${s.formula?.tags.join(' ') ?? ''} ${extra}`.toLowerCase();
            return words.every((w) => hay.includes(w));
        })
        .slice(0, limit);
}

/** Short label for compact lists: the script id in words ("battery_load" -> "Battery load"). */
export function shortName(id: string): string {
    const words = id.replaceAll('_', ' ');
    return words.charAt(0).toUpperCase() + words.slice(1);
}
