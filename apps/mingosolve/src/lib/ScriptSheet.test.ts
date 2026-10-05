// The script sheet against a fake engine: what it sends, which root it puts in the answer slab, and how it copes
// with replies that arrive late. The real engine behind these commands is covered by the IPC tests and e2e/.
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import ScriptSheet from './ScriptSheet.svelte';
import { catalog } from './catalog.svelte';
import { openCommand, openScript, session } from './session.svelte';
import { defaultSettings, settings } from './settings.svelte';
import { fakeEngine } from '../test/ipc';
import type { FormulaInfo, SolveResult, ToolInfo } from './types';

const motion: FormulaInfo = {
    key: 'uniform_motion',
    title: 'Uniform motion',
    eqs: ['v = s/t'],
    tex: ['v = \\frac{s}{t}'],
    tags: [],
    notes: '',
    vars: ['s', 't', 'v'].map((name) => ({ name, unit: 'm', desc: name, signed: false, default: null, tex: name, unit_shown: '' })),
};
const battery: FormulaInfo = {
    key: 'battery_load',
    title: 'Battery under load',
    eqs: ['V_oc = N_s*V_cell', 'P = (V_oc - I*R_pack)*I'],
    tex: [],
    tags: [],
    notes: '',
    vars: ['N_s', 'V_cell', 'R_pack', 'P', 'I', 'V_oc'].map((name) => ({ name, unit: '', desc: name, signed: false, default: null, tex: name, unit_shown: '' })),
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

function field(name: string): HTMLInputElement {
    return document.querySelector<HTMLInputElement>(`label.field[data-var="${name}"] input`)!;
}

const slab = () => document.querySelector('.answer .a-value')?.textContent;

/** v = s/t with a given s and t, as the engine would answer. */
function solveMotion(args: Record<string, unknown>): SolveResult {
    const g = Object.fromEntries(args.given as [string, string][]);
    const v = parseFloat(g.s) / parseFloat(g.t);
    return { found: Number.isFinite(v) ? [{ name: 'v', desc: 'v', values: [v], shown: [`${v} m/s`] }] : [], defaults: [], conflicts: [] };
}

const common = {
    script_examples: () => [],
    format_answer: (a: Record<string, unknown>) => String(a.value),
    match_options: () => ({ options: [], warning: null }),
};

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    catalog.scripts = new Map([
        ['uniform_motion', { id: 'uniform_motion', kind: 'formula', title: motion.title, aliases: '', topic: null, formula: motion }],
        ['battery_load', { id: 'battery_load', kind: 'formula', title: battery.title, aliases: '', topic: null, formula: battery }],
        ['event_score', { id: 'event_score', kind: 'tool', title: 'Event score', aliases: '', topic: null, tool: score }],
    ]);
    session.problemFills = {};
    session.sheet = null;
});

describe('live solving', () => {
    it('never lets a late reply for an earlier input overwrite the answer for the current one', async () => {
        const engine = fakeEngine({ ...common, solve_formula: solveMotion });
        const solves = engine.hold('solve_formula');
        openScript('uniform_motion', [['s', '100 m'], ['t', '10 s']]);
        render(ScriptSheet);
        await waitFor(() => expect(solves.pending()).toBe(1));
        const t = field('t');
        // one edit = one new solve; typing key by key on a slow machine would add solves for "1", "16"...
        await fireEvent.input(t, { target: { value: '20 s' } });
        await waitFor(() => expect(solves.pending()).toBe(2));
        // the engine answers the current input first, then the stale one
        solves.release(1);
        await waitFor(() => expect(slab()).toBe('5 m/s'));
        solves.release(0);
        await new Promise((r) => setTimeout(r, 50));
        expect(slab()).toBe('5 m/s');
    });

    it('copies the answer on screen even when the formatting of an earlier answer replies last', async () => {
        const engine = fakeEngine({ ...common, solve_formula: solveMotion, format_answer: (a) => String(a.value).replace('.', ',') });
        const formats = engine.hold('format_answer');
        const copies: string[] = [];
        Object.defineProperty(navigator, 'clipboard', { value: { writeText: async (t: string) => void copies.push(t) }, configurable: true });
        openScript('uniform_motion', [['s', '100 m'], ['t', '8 s']]);
        render(ScriptSheet);
        await waitFor(() => expect(formats.pending()).toBe(1));
        const t = field('t');
        // one edit = one new solve; typing key by key on a slow machine would add solves for "1", "16"...
        await fireEvent.input(t, { target: { value: '16 s' } });
        await waitFor(() => expect(slab()).toBe('6.25 m/s'));
        formats.release(formats.pending() - 1);
        formats.release(0);
        await new Promise((r) => setTimeout(r, 50));
        // by class: jsdom can't compute accessible names across KaTeX's MathML (real browsers can; see e2e/)
        await userEvent.click(document.querySelector<HTMLButtonElement>('.answer .copy')!);
        expect(copies).toEqual(['6,25']);
    });

    it('sends only the filled fields, and the unit the answer should be shown in', async () => {
        const engine = fakeEngine({ ...common, solve_formula: solveMotion });
        openScript('uniform_motion', [['s', '100 m'], ['t', '10 s']]);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('10 m/s'));
        await userEvent.type(screen.getByPlaceholderText(/km\/h/), 'km/h');
        await waitFor(() => {
            const last = engine.calls.filter((c) => c.cmd === 'solve_formula').at(-1)!.args;
            expect(last.given).toEqual([
                ['s', '100 m'],
                ['t', '10 s'],
            ]);
            expect(last.display).toMatchObject({ v: 'km/h' });
        });
    });
});

describe('checking the answer against pasted options', () => {
    it('sends the answer as shown, with its unit, so options in kN or ms can be compared', async () => {
        const engine = fakeEngine({
            ...common,
            solve_formula: solveMotion,
            match_options: () => ({
                options: [
                    { text: 'a) 36 km/h', value: 10, rel_diff: 0, best: true },
                    { text: 'b) 5 kg', value: null, rel_diff: null, best: false },
                ],
                warning: 'skipped, the unit is not the answer\'s: b) 5 kg',
            }),
        });
        openScript('uniform_motion', [['s', '100 m'], ['t', '10 s']]);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('10 m/s'));
        await userEvent.click(document.querySelector<HTMLButtonElement>('.check button')!);
        await fireEvent.input(document.querySelector('textarea.input')!, { target: { value: 'a) 36 km/h\nb) 5 kg' } });
        await waitFor(() => expect(document.querySelector('.opts li.best')?.textContent).toContain('a) 36 km/h'));
        const call = engine.calls.filter((c) => c.cmd === 'match_options').at(-1)!;
        expect(call.args).toEqual({ answer: '10 m/s', options: 'a) 36 km/h\nb) 5 kg' });
        expect(document.querySelector('.note-warn')?.textContent).toContain('b) 5 kg');
    });
});

describe('which root the answer slab shows', () => {
    const twoRoots: SolveResult = {
        found: [
            { name: 'V_oc', desc: '', values: [391.4], shown: ['391.4 V'] },
            { name: 'I', desc: '', values: [4814.6, 77.887], shown: ['4814.6 A', '77.887 A'] },
        ],
        defaults: [],
        conflicts: [],
    };

    it('a worked example shows the root that matches its official answer, with the other root mentioned', async () => {
        fakeEngine({ ...common, solve_formula: () => twoRoots });
        openCommand('battery_load N_s=103 V_cell=3.8V R_pack=0.08ohm P=30kW', 77.9);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('77.887 A'));
        expect(document.querySelector('.a-note')?.textContent).toContain('4814.6 A');
    });

    it('a pasted problem shows the variable it asks for, not the first one solved', async () => {
        fakeEngine({ ...common, solve_formula: () => twoRoots });
        session.problemFills = { battery_load: { values: [['N_s', '103']], target: 'I' } };
        openScript('battery_load');
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('4814.6 A'));
        expect(field('N_s').value).toBe('103');
    });

    it('with nothing to go on, it shows the first solved variable', async () => {
        fakeEngine({ ...common, solve_formula: () => twoRoots });
        openScript('battery_load', [['N_s', '103']]);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('391.4 V'));
    });
});

describe('tool scripts', () => {
    it("a worked example's positional arguments fill the parameters in order and run at once", async () => {
        const engine = fakeEngine({ ...common, run_tool: () => '41.117' });
        openCommand('event_score skidpad 5.6 5.1 rules=legacy', 41.117);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('41.117'));
        const run = engine.calls.find((c) => c.cmd === 'run_tool')!.args;
        expect(Object.fromEntries(run.args as [string, string][])).toEqual({
            event: 'skidpad',
            t_team: '5.6',
            t_min: '5.1',
            rules: 'legacy',
        });
    });

    it('a worked example whose answer comes from the defaults alone still runs (e.g. skidpad layout, Q99)', async () => {
        const engine = fakeEngine({ ...common, run_tool: () => '18.25' });
        openCommand('event_score', 18.25);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('18.25'));
        expect(engine.calls.filter((c) => c.cmd === 'run_tool')).toHaveLength(1);
    });

    it('opening a bare tool waits for Run instead of erroring on the empty required fields, and uses the rule year from Settings', async () => {
        settings.rules = '2026';
        const engine = fakeEngine({ ...common, run_tool: () => '1' });
        openScript('event_score');
        render(ScriptSheet);
        await waitFor(() => expect(field('rules').value).toBe('2026'));
        await new Promise((r) => setTimeout(r, 50));
        expect(engine.calls.some((c) => c.cmd === 'run_tool')).toBe(false);
    });
});

describe('sheet state', () => {
    it('keeps what was typed when the view is left and reopened, but a new open starts clean', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        openScript('uniform_motion');
        const first = render(ScriptSheet);
        await userEvent.type(field('s'), '42 m');
        await waitFor(() => expect(session.sheet?.values.s).toBe('42 m'));
        first.unmount();
        const back = render(ScriptSheet);
        expect(field('s').value).toBe('42 m');
        back.unmount();
        openScript('uniform_motion');
        render(ScriptSheet);
        expect(field('s').value).toBe('');
    });
});

describe('values taken from the pasted problem', () => {
    const marker = (name: string) => document.querySelector(`label.field[data-var="${name}"] .from`);

    it('marks only the fields the problem filled, with the source text, and drops the mark when the field is edited', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        session.problemFills = { uniform_motion: { values: [['s', '100 m']], target: null } };
        openScript('uniform_motion');
        render(ScriptSheet);
        expect(field('s').value).toBe('100 m');
        expect(marker('s')?.textContent).toContain('from the problem');
        expect(marker('s')?.getAttribute('title')).toContain('100 m');
        expect(marker('t')).toBeNull();
        await userEvent.type(field('t'), '5 s');
        expect(marker('t')).toBeNull();
        expect(marker('s')).not.toBeNull();
        await userEvent.type(field('s'), '0');
        expect(marker('s')).toBeNull();
    });

    it('values loaded from a worked example never carry the mark', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        session.problemFills = { uniform_motion: { values: [['s', '100 m']], target: null } };
        openCommand('uniform_motion s=75m t=3.8s');
        render(ScriptSheet);
        expect(field('s').value).toBe('75m');
        expect(document.querySelector('.from')).toBeNull();
    });
});

function clipboardSpy(): string[] {
    const copies: string[] = [];
    Object.defineProperty(navigator, 'clipboard', { value: { writeText: async (t: string) => void copies.push(t) }, configurable: true });
    return copies;
}

const status = () => document.querySelector('[role="status"]')?.textContent ?? '';
const ctrlEnter = '{Control>}{Enter}{/Control}';

describe('copy shortcut', () => {
    it('Ctrl+Enter copies the current answer from a field and from the options box, and says so', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        const copies = clipboardSpy();
        openScript('uniform_motion', [['s', '100 m'], ['t', '8 s']]);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('12.5 m/s'));
        await waitFor(() => expect(document.querySelector('.copy')?.textContent).toContain('Copy 12.5'));
        field('t').focus();
        await userEvent.keyboard(ctrlEnter);
        await waitFor(() => expect(status()).toContain('Copied 12.5'));
        expect(copies).toEqual(['12.5']);
        expect(document.querySelector('.copy')?.textContent).toContain('Copied');

        await userEvent.click(document.querySelector<HTMLButtonElement>('.check button')!);
        const box = document.querySelector<HTMLTextAreaElement>('textarea.input')!;
        box.focus();
        await userEvent.keyboard(ctrlEnter);
        await waitFor(() => expect(copies).toEqual(['12.5', '12.5']));
        // the shortcut never types into the box
        expect(box.value).toBe('');
    });

    it('does nothing while there is no answer', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        const copies = clipboardSpy();
        openScript('uniform_motion');
        render(ScriptSheet);
        field('s').focus();
        await userEvent.keyboard(ctrlEnter);
        expect(copies).toEqual([]);
    });
});

describe('clearing', () => {
    it('Esc wipes nothing, in a field or in the options box', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        openScript('uniform_motion', [['s', '100 m'], ['t', '10 s']]);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('10 m/s'));
        await userEvent.click(document.querySelector<HTMLButtonElement>('.check button')!);
        const box = document.querySelector<HTMLTextAreaElement>('textarea.input')!;
        await userEvent.type(box, 'a) 36 km/h');
        await userEvent.keyboard('{Escape}');
        field('s').focus();
        await userEvent.keyboard('{Escape}');
        expect(field('s').value).toBe('100 m');
        expect(field('t').value).toBe('10 s');
        expect(box.value).toBe('a) 36 km/h');
        expect(document.querySelector('.undo')).toBeNull();
    });

    it('Clear then Undo restores values, display unit, the picked root and the options exactly', async () => {
        const twoRoots: SolveResult = {
            found: [
                { name: 'V_oc', desc: '', values: [391.4], shown: ['391.4 V'] },
                { name: 'I', desc: '', values: [4814.6, 77.887], shown: ['4814.6 A', '77.887 A'] },
            ],
            defaults: [],
            conflicts: [],
        };
        fakeEngine({ ...common, solve_formula: () => twoRoots });
        openCommand('battery_load N_s=103 V_cell=3.8V R_pack=0.08ohm P=30kW', 77.9);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('77.887 A'));
        // a root the sheet would not choose by itself, a unit, and pasted options
        await userEvent.click(document.querySelector<HTMLButtonElement>('.res[data-var="V_oc"]')!);
        await waitFor(() => expect(slab()).toBe('391.4 V'));
        await userEvent.type(screen.getByPlaceholderText(/km\/h/), 'kV');
        await userEvent.click(document.querySelector<HTMLButtonElement>('.check button')!);
        await userEvent.type(document.querySelector('textarea.input')!, 'a) 391 V');
        const before = { n: field('N_s').value, v: field('V_cell').value, p: field('P').value };

        // by text: jsdom can't compute accessible names across KaTeX's MathML
        const button = (text: string) => [...document.querySelectorAll('button')].find((b) => b.textContent?.trim() === text)!;
        await userEvent.click(button('Clear'));
        expect(field('N_s').value).toBe('');
        expect(field('P').value).toBe('');
        await waitFor(() => expect(slab()).toBe('77.887 A'));

        await userEvent.click(button('Undo clear'));
        expect({ n: field('N_s').value, v: field('V_cell').value, p: field('P').value }).toEqual(before);
        await waitFor(() => expect(slab()).toBe('391.4 V'));
        expect((screen.getByPlaceholderText(/km\/h/) as HTMLInputElement).value).toBe('kV');
        expect(document.querySelector<HTMLTextAreaElement>('textarea.input')!.value).toBe('a) 391 V');
        expect(document.querySelector('.undo')).toBeNull();
    });

    it('the Undo offer goes away by itself after a few seconds', async () => {
        vi.useFakeTimers({ shouldAdvanceTime: true });
        try {
            fakeEngine({ ...common, solve_formula: solveMotion });
            openScript('uniform_motion', [['s', '100 m']]);
            render(ScriptSheet);
            await fireEvent.click(screen.getByRole('button', { name: 'Clear' }));
            expect(document.querySelector('.undo')).not.toBeNull();
            await vi.advanceTimersByTimeAsync(8100);
            expect(document.querySelector('.undo')).toBeNull();
        } finally {
            vi.useRealTimers();
        }
    });
});

const nodal = '12.5\n  I1 = 3 A\n  I2 = 4 A';

describe('multi-line tool output', () => {
    it('shows the first number as the value and the rest line by line, labelled with the tool title; Copy takes the number, Copy all the text', async () => {
        fakeEngine({ ...common, run_tool: () => nodal });
        const copies = clipboardSpy();
        openCommand('event_score skidpad 5.5 5.0');
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('12.5'));
        expect(document.querySelector('.a-label')?.textContent).toContain('Event score');
        expect(document.querySelector('.a-extra')?.textContent).toBe('  I1 = 3 A\n  I2 = 4 A');
        await waitFor(() => expect(document.querySelector('.copy')?.textContent).toContain('Copy 12.5'));
        await userEvent.click(document.querySelector<HTMLButtonElement>('.copy')!);
        await userEvent.click(screen.getByRole('button', { name: 'Copy all' }));
        await waitFor(() => expect(copies).toHaveLength(2));
        expect(copies).toEqual(['12.5', nodal]);
        expect(status()).toContain('3 lines');
    });

    it('a text-only output stays one block with no number to copy separately', async () => {
        fakeEngine({ ...common, run_tool: () => 'insulation test 1000 V\nclearance 2 mm' });
        openCommand('event_score skidpad 5.5 5.0');
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('.a-value.block')?.textContent).toBe('insulation test 1000 V\nclearance 2 mm'));
        expect(screen.queryByRole('button', { name: 'Copy all' })).toBeNull();
    });
});

describe('rule years', () => {
    /** The skidpad score for 5.5 / 5.0 s under each rule set (2027 26.73, legacy 40.53), scaled by t_team. */
    const base: Record<string, number> = { '2027': 26.73, '2026': 26.73, legacy: 40.53 };
    const byYear = (a: Record<string, unknown>) => {
        const g = Object.fromEntries(a.args as [string, string][]);
        if (!(g.rules in base)) throw new Error(`unknown rules ${g.rules}`);
        return String(+(base[g.rules] * (parseFloat(g.t_team) / 5.5)).toFixed(2));
    };
    /** Nearest option wins, like the engine's matcher. */
    const nearest = (a: Record<string, unknown>) => {
        const ans = parseFloat(String(a.answer));
        const options = String(a.options)
            .split('\n')
            .map((text) => {
                const value = parseFloat(text.replace(/^\w\)\s*/, ''));
                return { text, value, rel_diff: Math.abs(value - ans) / ans, best: false };
            });
        options.reduce((b, o) => (o.rel_diff < b.rel_diff ? o : b)).best = true;
        return { options, warning: null };
    };
    const paste = async (text: string) => {
        await userEvent.click(document.querySelector<HTMLButtonElement>('.check button')!);
        await fireEvent.input(document.querySelector('textarea.input')!, { target: { value: text } });
    };

    it('names the rule set behind the answer and lists what the other years give', async () => {
        fakeEngine({ ...common, run_tool: byYear });
        openCommand('event_score skidpad 5.5 5.0');
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('26.73'));
        expect(document.querySelector('.answer .pill')?.textContent).toBe('rules 2027');
        await waitFor(() => expect(document.querySelector('.a-years')?.textContent).toBe('Other rules: 2026: 26.73 · legacy: 40.53'));
    });

    it('skips a year that errors', async () => {
        fakeEngine({
            ...common,
            run_tool: (a) => {
                if ((a.args as [string, string][]).some(([k, v]) => k === 'rules' && v === '2026')) throw new Error('no such rules');
                return byYear(a);
            },
        });
        openCommand('event_score skidpad 5.5 5.0');
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('.a-years')?.textContent).toBe('Other rules: legacy: 40.53'));
    });

    it('says when pasted options fit only the legacy rules', async () => {
        fakeEngine({ ...common, run_tool: byYear, match_options: nearest });
        openCommand('event_score skidpad 5.5 5.0');
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('.a-years')).not.toBeNull());
        await paste('a) 40.5\nb) 55.0');
        await waitFor(() => expect(document.querySelector('.note-year')?.textContent).toContain('The legacy rules match option a) 40.5; this question may use the old key.'));
    });

    it('stays quiet when the selected year already matches an option', async () => {
        fakeEngine({ ...common, run_tool: byYear, match_options: nearest });
        openCommand('event_score skidpad 5.5 5.0');
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('.a-years')).not.toBeNull());
        await paste('a) 26.7\nb) 40.5');
        await waitFor(() => expect(document.querySelector('.opts li.best')).not.toBeNull());
        await new Promise((r) => setTimeout(r, 80));
        expect(document.querySelector('.note-year')).toBeNull();
    });

    it('never shows replies of an earlier run once a newer one is on screen', async () => {
        const engine = fakeEngine({ ...common, run_tool: byYear });
        const runs = engine.hold('run_tool');
        openCommand('event_score skidpad 5.5 5.0');
        render(ScriptSheet);
        // run 1 = main + two other years; run 2 (t_team edited, Run pressed) = three more
        await waitFor(() => expect(runs.pending()).toBe(3));
        await fireEvent.input(field('t_team'), { target: { value: '11' } });
        await userEvent.click(screen.getByRole('button', { name: /^Run/ }));
        await waitFor(() => expect(runs.pending()).toBe(6));
        [3, 4, 5].forEach(runs.release);
        await waitFor(() => expect(slab()).toBe('53.46'));
        await waitFor(() => expect(document.querySelector('.a-years')?.textContent).toBe('Other rules: 2026: 53.46 · legacy: 81.06'));
        [0, 1, 2].forEach(runs.release);
        await new Promise((r) => setTimeout(r, 60));
        expect(slab()).toBe('53.46');
        expect(document.querySelector('.a-years')?.textContent).toBe('Other rules: 2026: 53.46 · legacy: 81.06');
    });
});

describe('answer slab placement', () => {
    it('is the sticky slab when there is a solution, and absent otherwise', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        openScript('uniform_motion');
        render(ScriptSheet);
        expect(document.querySelector('.answer')).toBeNull();
        await fireEvent.input(field('s'), { target: { value: '100 m' } });
        await fireEvent.input(field('t'), { target: { value: '10 s' } });
        await waitFor(() => expect(document.querySelector('.answer')?.classList.contains('sticky')).toBe(true));
    });
});
