// A fake engine for component tests: each command answers from a handler, and replies can be held back to test
// what the UI does when responses arrive late or out of order.
import { mockIPC } from '@tauri-apps/api/mocks';

type Handler = (args: Record<string, unknown>) => unknown;

export interface FakeEngine {
    calls: { cmd: string; args: Record<string, unknown> }[];
    /** Hold replies to this command until release() is called on the returned entry. */
    hold(cmd: string): { release: (i: number) => void; pending: () => number };
}

export function fakeEngine(handlers: Record<string, Handler>): FakeEngine {
    const calls: FakeEngine['calls'] = [];
    const held = new Map<string, (() => void)[]>();
    mockIPC(async (cmd, payload) => {
        const args = (payload ?? {}) as Record<string, unknown>;
        calls.push({ cmd, args });
        const queue = held.get(cmd);
        if (queue !== undefined) await new Promise<void>((resolve) => queue.push(resolve));
        const h = handlers[cmd];
        if (h === undefined) throw new Error(`unexpected command ${cmd}`);
        return h(args);
    });
    return {
        calls,
        hold(cmd) {
            const queue: (() => void)[] = [];
            held.set(cmd, queue);
            return { release: (i) => queue[i](), pending: () => queue.length };
        },
    };
}
