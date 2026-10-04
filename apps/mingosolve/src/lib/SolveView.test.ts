// The Solve view's finder: the matches and the values scripts take from the problem must follow the text on
// screen, whatever order the engine replies in.
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';

import SolveView from './SolveView.svelte';
import { catalog } from './catalog.svelte';
import { session } from './session.svelte';
import { fakeEngine } from '../test/ipc';
import type { Found, FormulaInfo, Hit } from './types';

const hit = (id: string, prefill: [string, string][]): Hit => ({ kind: 'formula', id, title: id, score: 1, prefill, target: null, warning: null });

const finds: Record<string, Found> = {
    skidpad: { hits: [hit('cornering_downforce', [['m', '240 kg']])], quantities: ['240 kg'] },
    discharge: { hits: [hit('ts_discharge', [['V_0', '396 V']])], quantities: ['396 V'] },
};

beforeEach(() => {
    catalog.scripts = new Map();
    session.script = null;
    session.problem = '';
    session.problemFills = {};
    session.sheet = null;
});

it('shows the matches for the problem on screen when an earlier search replies last', async () => {
    const engine = fakeEngine({ find_question: (a) => finds[String(a.text).split(' ')[0]] });
    const replies = engine.hold('find_question');
    render(SolveView);
    session.problem = 'skidpad car of 240 kg';
    await waitFor(() => expect(replies.pending()).toBe(1));
    session.problem = 'discharge from 396 V';
    await waitFor(() => expect(replies.pending()).toBe(2));
    replies.release(1);
    await waitFor(() => expect(screen.getByLabelText('Scripts that fit').textContent).toContain('Ts discharge'));
    replies.release(0);
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.getByLabelText('Scripts that fit').textContent).not.toContain('Cornering downforce');
    expect(Object.keys(session.problemFills)).toEqual(['ts_discharge']);
});

it('clearing the problem drops what scripts would take from it', async () => {
    fakeEngine({ find_question: (a) => finds[String(a.text).split(' ')[0]] });
    render(SolveView);
    session.problem = 'skidpad car of 240 kg';
    await waitFor(() => expect(session.problemFills).toHaveProperty('cornering_downforce'));
    session.problem = '';
    await waitFor(() => expect(session.problemFills).toEqual({}));
});

it('a search still running when the problem is cleared does not bring its matches back', async () => {
    const engine = fakeEngine({ find_question: (a) => finds[String(a.text).split(' ')[0]] });
    const replies = engine.hold('find_question');
    render(SolveView);
    session.problem = 'skidpad car of 240 kg';
    await waitFor(() => expect(replies.pending()).toBe(1));
    session.problem = '';
    await new Promise((r) => setTimeout(r, 250));
    replies.release(0);
    await new Promise((r) => setTimeout(r, 50));
    expect(session.problemFills).toEqual({});
});

describe('which detected values the open script uses', () => {
    const motion: FormulaInfo = {
        key: 'uniform_motion',
        title: 'Uniform motion',
        eqs: ['v = s/t'],
        tex: ['v = \\frac{s}{t}'],
        tags: [],
        notes: '',
        vars: ['s', 't', 'v'].map((name) => ({ name, unit: 'm', desc: name, signed: false, default: null, tex: name, unit_shown: '' })),
    };
    const chip = (text: string) => [...document.querySelectorAll('.chips .chip')].find((c) => c.textContent?.includes(text))!;

    async function openWithProblem() {
        fakeEngine({
            find_question: () => ({ hits: [hit('uniform_motion', [['s', '75 m']])], quantities: ['75 m', '3.8 s', '9.81 m/s²'] }),
            solve_formula: () => ({ found: [], defaults: [], conflicts: [] }),
            script_examples: () => [],
            format_answer: () => '',
            match_options: () => ({ options: [], warning: null }),
        });
        catalog.scripts = new Map([['uniform_motion', { id: 'uniform_motion', kind: 'formula', title: motion.title, aliases: '', topic: null, formula: motion }]]);
        render(SolveView);
        session.problem = 'a car covers 75 m in 3.8 s, g = 9.81 m/s^2';
        await waitFor(() => expect(screen.getByLabelText('Scripts that fit').querySelector('.match')).not.toBeNull());
        await userEvent.click(screen.getByLabelText('Scripts that fit').querySelector('.match')!);
    }

    it('checks the values the sheet holds and marks the rest as not used, for the eye and for screen readers', async () => {
        await openWithProblem();
        await waitFor(() => expect(chip('75 m').textContent).toContain('used in this script'));
        expect(chip('3.8 s').textContent).toContain('not used');
        expect(chip('9.81').textContent).toContain('not used');
        expect(chip('3.8 s').classList.contains('unused')).toBe(true);
        expect(chip('75 m').classList.contains('used')).toBe(true);
    });

    it('follows the sheet: typing a leftover value into a field uses it, overwriting a filled one frees its chip', async () => {
        await openWithProblem();
        await waitFor(() => expect(chip('75 m').classList.contains('used')).toBe(true));
        const t = document.querySelector<HTMLInputElement>('label.field[data-var="t"] input')!;
        await userEvent.type(t, '3.8 s');
        await waitFor(() => expect(chip('3.8 s').classList.contains('used')).toBe(true));
        const s = document.querySelector<HTMLInputElement>('label.field[data-var="s"] input')!;
        await userEvent.clear(s);
        await userEvent.type(s, '80 m');
        await waitFor(() => expect(chip('75 m').textContent).toContain('not used'));
    });

    it('shows no used or unused state while no script is open', async () => {
        fakeEngine({ find_question: () => ({ hits: [], quantities: ['75 m'] }) });
        render(SolveView);
        session.problem = 'a car covers 75 m';
        await waitFor(() => expect(chip('75 m')).toBeDefined());
        expect(chip('75 m').classList.contains('unused')).toBe(false);
        expect(chip('75 m').textContent).not.toContain('not used');
    });
});
