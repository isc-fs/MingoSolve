// The Solve view's finder: the matches and the values scripts take from the problem must follow the text on
// screen, whatever order the engine replies in.
import { render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, expect, it } from 'vitest';

import SolveView from './SolveView.svelte';
import { catalog } from './catalog.svelte';
import { session } from './session.svelte';
import { fakeEngine } from '../test/ipc';
import type { Found, Hit } from './types';

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
