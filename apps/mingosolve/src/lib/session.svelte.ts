// Session-only state shared across views: navigation requests ("open this formula with these values"), the
// calculator input, and the per-question answer log. Nothing here is persisted.
import type { ViewId } from './stores';

export interface OpenRequest {
    view: ViewId;
    /** Formula key, tool name or chain target. */
    id: string;
    values: [string, string][];
    display?: Record<string, string>;
    /** Positional tool arguments (from past-question commands). */
    positional?: string[];
}

export interface LogEntry {
    question: number;
    what: string;
    inputs: string;
    answer: string;
}

export const session = $state({
    activeView: 'solve' as ViewId,
    request: null as OpenRequest | null,
    calcInput: '',
    log: [] as LogEntry[],
    nextQuestion: 1,
    focusSearch: 0,
});

export function open(req: OpenRequest): void {
    session.request = req;
    session.activeView = req.view;
}

export function logAnswer(what: string, inputs: string, answer: string): void {
    session.log.push({ question: session.nextQuestion, what, inputs, answer });
    session.nextQuestion += 1;
}

/** Split a CLI line like the engine does: whitespace separates, double quotes group. */
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
