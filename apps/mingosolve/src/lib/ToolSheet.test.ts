// The tool form in plain words (labels, units, selects, switches, help, summary) and the hints and friendly errors
// the sheet shows. The engine is a fake: what the sheet sends and shows is what is tested.
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';

import ScriptSheet from './ScriptSheet.svelte';
import { catalog } from './catalog.svelte';
import { openCommand, openScript, session } from './session.svelte';
import { defaultSettings, settings } from './settings.svelte';
import { fakeEngine } from '../test/ipc';
import { corrected, score } from '../test/tools';
import type { FormulaInfo } from './types';

const hinted: FormulaInfo = {
    key: 'cornering',
    title: 'Cornering',
    eqs: ['a_y = v**2/R'],
    tex: [],
    tags: [],
    notes: '',
    vars: [
        { name: 'a_y', unit: 'm/s**2', desc: 'lateral acceleration', hint: 'Type 1.5g0 for 1.5 g.', signed: false, default: null, tex: 'a_y', unit_shown: 'm/s²' },
        { name: 'R', unit: 'm', desc: 'a corner radius that is described at some length so the label wraps', hint: null, signed: false, default: null, tex: 'R', unit_shown: 'm' },
    ],
};

const common = {
    script_examples: () => [],
    format_answer: (a: Record<string, unknown>) => String(a.value),
    match_options: () => ({ options: [], warning: null }),
};

const slab = () => document.querySelector('.answer .a-value')?.textContent;
const box = (name: string) => document.querySelector<HTMLElement>(`label.field[data-var="${name}"]`)!;
const field = (name: string) => box(name).querySelector<HTMLInputElement>('input:not([type="checkbox"])');
const select = (name: string) => box(name).querySelector<HTMLSelectElement>('select')!;
const checkbox = (name: string) => box(name).querySelector<HTMLInputElement>('input[type="checkbox"]')!;
const run = () => userEvent.click(screen.getByRole('button', { name: /^Run/ }));
type Engine = { calls: { cmd: string; args: Record<string, unknown> }[] };
const runs = (engine: Engine) => engine.calls.filter((c) => c.cmd === 'run_tool').map((c) => Object.fromEntries(c.args.args as [string, string][]));
/** The run for what the form holds (a run also asks the other rule years, with their own `rules`). */
const toolArgs = (engine: Engine) => runs(engine).filter((a) => a.rules === select('rules').value).at(-1)!;

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    catalog.scripts = new Map([
        ['event_score', { id: 'event_score', kind: 'tool', title: 'Event score', aliases: '', topic: null, tool: score }],
        ['corrected_time', { id: 'corrected_time', kind: 'tool', title: 'Corrected time', aliases: '', topic: null, tool: corrected }],
        ['cornering', { id: 'cornering', kind: 'formula', title: hinted.title, aliases: '', topic: null, formula: hinted }],
    ]);
    session.problemFills = {};
    session.sheet = null;
});

describe('the tool form in plain words', () => {
    const open = async () => {
        const engine = fakeEngine({ ...common, run_tool: () => '41.117' });
        openScript('event_score');
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('form.fields')).not.toBeNull());
        return engine;
    };

    it('shows the label and the unit of each field, never the parameter name', async () => {
        await open();
        expect(box('t_team').textContent).toContain('Your corrected time');
        expect(box('t_team').querySelector('.unit')?.textContent).toBe('s');
        expect(box('kappa').querySelector('.unit')?.textContent).toBe('S/m');
        for (const name of ['t_team', 't_min', 'pmax', 'kappa']) expect(box(name).querySelector('.flabel')?.textContent).not.toContain(name);
    });

    it('offers a choice as a select with readable options, and sends the value picked', async () => {
        const engine = await open();
        const event = select('event');
        expect([...event.options].map((o) => o.textContent)).toContain('Skidpad');
        expect(field('event')).toBeNull();
        await userEvent.selectOptions(event, 'endurance');
        await userEvent.type(field('t_team')!, '1672');
        await userEvent.type(field('t_min')!, '1345');
        await run();
        await waitFor(() => expect(slab()).toBe('41.117'));
        expect(toolArgs(engine)).toMatchObject({ event: 'endurance', t_team: '1672', t_min: '1345' });
    });

    it('shows an on/off setting as a checkbox that sends 1 or 0', async () => {
        const engine = await open();
        expect(field('finish')).toBeNull();
        const finish = checkbox('finish');
        expect(finish.checked).toBe(true);
        await userEvent.click(finish);
        await run();
        await waitFor(() => expect(toolArgs(engine).finish).toBe('0'));
        await userEvent.click(finish);
        await run();
        await waitFor(() => expect(toolArgs(engine).finish).toBe('1'));
    });

    it('starts the rules year at the Settings year and keeps the others one pick away', async () => {
        settings.rules = 'legacy';
        const engine = await open();
        expect(select('rules').value).toBe('legacy');
        await userEvent.selectOptions(select('rules'), '2027');
        await run();
        await waitFor(() => expect(toolArgs(engine).rules).toBe('2027'));
        // the other years are asked with the select's alternatives, for the comparison under the answer
        await waitFor(() => expect(new Set(runs(engine).slice(-3).map((a) => a.rules))).toEqual(new Set(['legacy', '2026', '2027'])));
    });

    it('writes the help under the field (not only in a tooltip), and marks only the required fields', async () => {
        await open();
        const help = box('t_min').querySelector('.hint');
        expect(help?.textContent).toBe('The best corrected time of any team in this event.');
        expect(help?.closest('[title]')).toBeNull();
        const required = [...document.querySelectorAll('label.field')].filter((l) => l.querySelector('.req') !== null).map((l) => l.getAttribute('data-var'));
        expect(required).toEqual(['event', 't_team', 't_min']);
        expect(field('t_team')!.getAttribute('aria-required')).toBe('true');
        expect(field('pmax')!.getAttribute('aria-required')).toBe('false');
    });

    it('puts the default in the empty input in plain words', async () => {
        await open();
        expect(field('kappa')!.placeholder).toBe('defaults to 56 000 000');
        expect(field('pmax')!.placeholder).toBe('defaults to 0');
        expect(field('t_team')!.placeholder).toBe('');
    });

    it('opens with a plain sentence and keeps the technical description behind Details', async () => {
        await open();
        expect(document.querySelector('p.doc')?.textContent).toBe(score.summary);
        const details = document.querySelector<HTMLDetailsElement>('details.details')!;
        expect(details.open).toBe(false);
        expect(details.textContent).toContain('rules=legacy reproduces older quiz keys');
        await userEvent.click(details.querySelector('summary')!);
        expect(details.open).toBe(true);
    });

    it('a worked example fills the select and the checkbox from its command, and sends them back', async () => {
        const engine = fakeEngine({ ...common, run_tool: () => '65.1348' });
        openCommand('event_score endurance 1672 1345 legacy finish=0', 65.1348);
        render(ScriptSheet);
        await waitFor(() => expect(slab()).toBe('65.1348'));
        expect(select('event').value).toBe('endurance');
        expect(select('rules').value).toBe('legacy');
        expect(checkbox('finish').checked).toBe(false);
        expect(toolArgs(engine)).toMatchObject({ event: 'endurance', rules: 'legacy', finish: '0' });
    });

    it('opening a tool whose choice has a default still waits for Run', async () => {
        const engine = fakeEngine({ ...common, run_tool: () => '1' });
        openScript('corrected_time');
        render(ScriptSheet);
        await waitFor(() => expect(select('event').value).toBe('autocross'));
        await new Promise((r) => setTimeout(r, 50));
        expect(engine.calls.some((c) => c.cmd === 'run_tool')).toBe(false);
    });

    it('Clear puts every field back to where it started, the rules year included', async () => {
        settings.rules = '2026';
        await open();
        await userEvent.selectOptions(select('rules'), 'legacy');
        await userEvent.click(checkbox('finish'));
        await userEvent.type(field('t_team')!, '5.6');
        await userEvent.click(screen.getByRole('button', { name: 'Clear' }));
        expect(select('rules').value).toBe('2026');
        expect(checkbox('finish').checked).toBe(true);
        expect(field('t_team')!.value).toBe('');
    });
});

describe('typing hints and engine errors in the sheet', () => {
    it('shows a typing hint under its field, outside the label, and nothing under fields without one', async () => {
        fakeEngine({ ...common, solve_formula: () => ({ found: [], defaults: [], conflicts: [] }) });
        openScript('cornering');
        render(ScriptSheet);
        const hint = box('a_y').querySelector('.hint');
        expect(hint?.textContent).toBe('Type 1.5g0 for 1.5 g.');
        expect(box('a_y').querySelector('.flabel')?.textContent).not.toContain('1.5g0');
        expect(box('R').querySelector('.hint')).toBeNull();
        expect(box('R').querySelector('.flabel')?.textContent).toContain('described at some length so the label wraps');
    });

    it("explains a unit mismatch by naming the field (the engine's own text for 3 kg where a length goes)", async () => {
        fakeEngine({
            ...common,
            solve_formula: () => {
                throw '3kg is kg, expected m (m)';
            },
        });
        openScript('cornering', [['R', '3kg']]);
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('.note-bad')?.textContent).toContain('needs a length'));
        const text = document.querySelector('.note-bad')!.textContent!;
        expect(text).toContain('A corner radius');
        expect(text).not.toContain('expected m');
    });

    it("explains a tool's missing value by the label of the field", async () => {
        fakeEngine({
            ...common,
            run_tool: () => {
                throw 'event_score: missing t_team';
            },
        });
        openScript('event_score');
        render(ScriptSheet);
        await run();
        await waitFor(() => expect(document.querySelector('.note-bad')?.textContent).toContain('Your corrected time is empty'));
    });
});
