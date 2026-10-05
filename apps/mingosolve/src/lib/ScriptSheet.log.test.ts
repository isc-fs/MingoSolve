// The session log as the sheet feeds it: each way of copying (button, Ctrl+Enter, Copy all) appends exactly one
// entry holding what was typed, what reached the clipboard, the rule year and the time since the problem arrived.
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import ScriptSheet from './ScriptSheet.svelte';
import { catalog } from './catalog.svelte';
import { openCommand, openScript, session } from './session.svelte';
import { sessionLog, startClock, stopClock } from './sessionlog.svelte';
import { defaultSettings, settings } from './settings.svelte';
import { fakeEngine } from '../test/ipc';
import type { FormulaInfo, SolveResult, ToolInfo } from './types';

const motion: FormulaInfo = {
    key: 'uniform_motion',
    title: 'Uniform motion',
    eqs: ['v = s/t'],
    tex: [],
    tags: [],
    notes: '',
    vars: ['s', 't', 'v'].map((name) => ({ name, unit: 'm', desc: name, signed: false, default: null, tex: name, unit_shown: '', dims: 'm' })),
};
const score: ToolInfo = {
    name: 'event_score',
    doc: 'Dynamic event score',
    params: [
        { name: 'event', number: false, default: null },
        { name: 't_team', number: true, default: null },
        { name: 't_min', number: true, default: null },
        { name: 'rules', number: false, default: '2027' },
    ],
};

const solveMotion = (args: Record<string, unknown>): SolveResult => {
    const g = Object.fromEntries(args.given as [string, string][]);
    const v = parseFloat(g.s) / parseFloat(g.t);
    return { found: Number.isFinite(v) ? [{ name: 'v', desc: 'v', values: [v], shown: [`${v} m/s`] }] : [], defaults: [], conflicts: [] };
};
const common = {
    script_examples: () => [],
    format_answer: (a: Record<string, unknown>) => String(a.value),
    match_options: () => ({ options: [], warning: null }),
};

function clipboardSpy(): string[] {
    const copies: string[] = [];
    Object.defineProperty(navigator, 'clipboard', { value: { writeText: async (t: string) => void copies.push(t) }, configurable: true });
    return copies;
}
const slab = () => document.querySelector('.answer .a-value')?.textContent;
const labelBox = () => document.querySelector<HTMLInputElement>('.qlabel input')!;
const ctrlEnter = '{Control>}{Enter}{/Control}';

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    catalog.scripts = new Map([
        ['uniform_motion', { id: 'uniform_motion', kind: 'formula', title: motion.title, aliases: '', topic: null, formula: motion }],
        ['event_score', { id: 'event_score', kind: 'tool', title: 'Event score', aliases: '', topic: null, tool: score }],
    ]);
    session.problemFills = {};
    session.sheet = null;
    session.problem = '';
    sessionLog.entries = [];
    sessionLog.quiz = null;
    stopClock();
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date('2027-01-12T10:00:00Z'));
});
afterEach(() => vi.useRealTimers());

async function openMotion(): Promise<string[]> {
    fakeEngine({ ...common, solve_formula: solveMotion });
    const copies = clipboardSpy();
    startClock();
    openScript('uniform_motion', [['s', '100 m'], ['t', '8 s']]);
    render(ScriptSheet);
    await waitFor(() => expect(slab()).toBe('12.5 m/s'));
    await waitFor(() => expect(document.querySelector('.copy')?.textContent).toContain('Copy 12.5'));
    return copies;
}

describe('every way of copying appends one entry', () => {
    it('the Copy button: inputs as typed, copied text, elapsed time from the problem', async () => {
        const copies = await openMotion();
        vi.setSystemTime(new Date('2027-01-12T10:01:23Z'));
        await userEvent.click(document.querySelector<HTMLButtonElement>('.copy')!);
        await waitFor(() => expect(sessionLog.entries).toHaveLength(1));
        expect(copies).toEqual(['12.5']);
        expect(sessionLog.entries[0]).toEqual({
            at: '2027-01-12T10:01:23Z',
            quiz: null,
            question: '',
            scriptId: 'uniform_motion',
            title: 'Uniform motion',
            inputs: [['s', '100 m'], ['t', '8 s']],
            answer: '12.5',
            rules: null,
            elapsedS: 83,
            via: 'button',
        });
    });

    it('Ctrl+Enter, from a field, appends one entry and no more', async () => {
        await openMotion();
        (document.querySelector('label.field[data-var="t"] input') as HTMLInputElement).focus();
        await userEvent.keyboard(ctrlEnter);
        await waitFor(() => expect(sessionLog.entries).toHaveLength(1));
        await new Promise((r) => setTimeout(r, 30));
        expect(sessionLog.entries).toHaveLength(1);
        expect(sessionLog.entries[0]).toMatchObject({ via: 'shortcut', answer: '12.5' });
    });

    it('Copy all logs the whole multi-line text, and the rule year of a tool run', async () => {
        const nodal = '41.117\n  detail line';
        fakeEngine({ ...common, run_tool: () => nodal });
        const copies = clipboardSpy();
        startClock();
        openCommand('event_score skidpad 5.6 5.1 rules=legacy', 41.117);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('41.117'));
        await userEvent.click(screen.getByRole('button', { name: 'Copy all' }));
        await waitFor(() => expect(sessionLog.entries).toHaveLength(1));
        expect(copies).toEqual([nodal]);
        expect(sessionLog.entries[0]).toMatchObject({
            scriptId: 'event_score',
            answer: nodal,
            rules: 'legacy',
            via: 'copy-all',
            inputs: [['event', 'skidpad'], ['t_team', '5.6'], ['t_min', '5.1'], ['rules', 'legacy']],
        });
    });

    it('a copy that the clipboard refuses is not logged', async () => {
        await openMotion();
        Object.defineProperty(navigator, 'clipboard', { value: { writeText: () => Promise.reject(new Error('denied')) }, configurable: true });
        await userEvent.click(document.querySelector<HTMLButtonElement>('.copy')!);
        await waitFor(() => expect(document.querySelector('[role="status"]')?.textContent).toContain('Copy failed'));
        expect(sessionLog.entries).toEqual([]);
    });

    it('without a running clock the elapsed time is empty, not zero', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        clipboardSpy();
        openScript('uniform_motion', [['s', '100 m'], ['t', '8 s']]);
        stopClock();
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('.copy')?.textContent).toContain('Copy 12.5'));
        await userEvent.click(document.querySelector<HTMLButtonElement>('.copy')!);
        await waitFor(() => expect(sessionLog.entries).toHaveLength(1));
        expect(sessionLog.entries[0].elapsedS).toBeNull();
    });
});

describe('the question label', () => {
    it('what is typed in "Question #" goes on the entry and stays for the next script of the same question', async () => {
        const copies = await openMotion();
        await fireEvent.input(labelBox(), { target: { value: ' Q7 ' } });
        await userEvent.click(document.querySelector<HTMLButtonElement>('.copy')!);
        await waitFor(() => expect(sessionLog.entries).toHaveLength(1));
        expect(sessionLog.entries[0].question).toBe('Q7');

        // another script opened for the same question keeps the label
        openScript('uniform_motion', [['s', '50 m'], ['t', '5 s']]);
        await waitFor(() => expect(slab()).toBe('10 m/s'));
        expect(labelBox().value).toBe(' Q7 ');
        await waitFor(() => expect(document.querySelector('.copy')?.textContent).toContain('Copy 10'));
        await userEvent.click(document.querySelector<HTMLButtonElement>('.copy')!);
        await waitFor(() => expect(sessionLog.entries).toHaveLength(2));
        expect(sessionLog.entries[1]).toMatchObject({ question: 'Q7', answer: '10' });
        expect(copies).toEqual(['12.5', '10']);
    });
});
