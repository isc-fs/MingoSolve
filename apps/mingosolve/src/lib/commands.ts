// Turn an engine command line (as stored with each past question) into an "open this view with these values"
// request, so a past question or a finder hit opens pre-filled in the right view.
import { open, session, splitCommand } from './session.svelte';

export function openCommand(cmd: string, formulaKeys: Set<string>, toolNames: Set<string>): void {
    const [head, ...rest] = splitCommand(cmd);
    if (head === undefined) return;
    const values: [string, string][] = [];
    const display: Record<string, string> = {};
    const positional: string[] = [];
    for (const arg of rest) {
        const eq = arg.indexOf('=');
        if (eq > 0) {
            const k = arg.slice(0, eq);
            const v = arg.slice(eq + 1);
            if (k.startsWith('@')) display[k.slice(1)] = v;
            else values.push([k, v]);
        } else {
            positional.push(arg);
        }
    }
    if (head === 'calc') {
        session.calcInput = rest.join(' ');
        return;
    }
    if (head === 'chain' && positional.length > 0) {
        open({ view: 'chain', id: positional[0], values, display });
    } else if (formulaKeys.has(head)) {
        open({ view: 'solve', id: head, values, display });
    } else if (toolNames.has(head)) {
        open({ view: 'tools', id: head, values, positional });
    }
}
