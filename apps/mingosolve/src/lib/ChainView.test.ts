// The Chain view against a fake engine: past questions fill the form from the real command lines of
// data/examples.toml, and the target picker finds a variable by what it is called in words. The engine side (that
// every one of these commands reproduces its official answer) is in src-tauri/src/ipc_tests.rs.
import { parse } from 'smol-toml';
import examplesToml from '../../../../data/examples.toml?raw';
import { render, screen, waitFor, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';

import ChainView from './ChainView.svelte';
import { catalog, type Script } from './catalog.svelte';
import { session, splitCommand } from './session.svelte';
import { defaultSettings, settings } from './settings.svelte';
import { fakeEngine } from '../test/ipc';
import type { ChainResult, FormulaInfo, WorkedExample } from './types';

const rows = (parse(examplesToml) as unknown as { example: (WorkedExample & { what?: string })[] }).example;
const chainRows: WorkedExample[] = rows
    .filter((e) => e.cmd.startsWith('chain '))
    .map((e) => ({ id: e.id, what: e.what ?? '', cmd: e.cmd, answer: e.answer }));

function formula(key: string, title: string, vars: [string, string, string][], tags: string[] = []): Script {
    const f: FormulaInfo = {
        key,
        title,
        eqs: [],
        tex: [],
        tags,
        notes: '',
        vars: vars.map(([name, desc, tex]) => ({ name, desc, unit: 'm/s', signed: false, hint: null, default: null, tex, unit_shown: 'm/s', dims: 'm·s^-1' })),
    };
    return { id: key, kind: 'formula', title, aliases: '', topic: null, formula: f };
}

const reached: ChainResult = { steps: [], defaults: [], reached: true, target: '74.7 km/h', known: [], conflicts: [], mapped: [], problems: [] };

const common = { chain_examples: () => chainRows, chain_formulas: () => reached };

/** The "Past questions" fold, found by its button (it is closed until the person opens it). */
const pastButton = (): HTMLElement => screen.getByRole('button', { name: 'Past questions' });
async function openPast(): Promise<HTMLElement> {
    const section = await screen.findByRole('region', { name: 'Past questions' });
    if (pastButton().getAttribute('aria-expanded') !== 'true') await userEvent.click(pastButton());
    return section;
}

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    const lib = [
        formula('cornering', 'Cornering: ay = v^2 / R, lap time, yaw rate', [['v', '(final) velocity', 'v'], ['ay', 'lateral acceleration', 'a_y'], ['R_c', 'corner radius', 'R_c']], ['dynamics', 'cornering']),
        formula('rollover', 'Rollover limit of a rigid car', [['ay', 'lateral acceleration', 'a_y'], ['h_cg', 'CG height', 'h_{cg}'], ['t_tr', 'track width', 't_{tr}']], ['dynamics', 'rollover']),
        formula('speed', 'Average speed: v_avg = s / t', [['v_avg', 'average speed over a distance', 'v_{avg}'], ['s', 'distance travelled', 's'], ['t', 'time', 't']], ['kinematics', 'speed']),
        formula('kinematics', 'Constant acceleration', [['v', '(final) velocity', 'v'], ['v0', 'initial velocity', 'v_0'], ['a', 'acceleration', 'a']]),
    ];
    catalog.scripts = new Map(lib.map((s) => [s.id, s]));
    catalog.ready = true;
    session.chain = { target: '', given: '', only: '', shownIn: '', run: 0 };
    session.activeView = 'chain';
});

describe('past questions', () => {
    it('lists every chain question of the bank and loads one into the form and runs it', async () => {
        const engine = fakeEngine(common);
        render(ChainView);
        const section = await openPast();
        expect(within(section).getAllByRole('button')).toHaveLength(chainRows.length + 1);
        expect(chainRows.length).toBeGreaterThan(5);

        // Q515 (rollover speed, shown in km/h): hand-split from its command line in examples.toml
        await userEvent.click(within(section).getByRole('button', { name: /^Q515 ·/ }));
        await waitFor(() => expect(engine.calls.some((c) => c.cmd === 'chain_formulas')).toBe(true));
        const call = engine.calls.find((c) => c.cmd === 'chain_formulas')!;
        expect(call.args).toEqual({
            target: 'v',
            given: [['h_cg', '0.205m'], ['R_c', '14.5m'], ['t_tr', '1.24m']],
            only: [],
            display: { v: 'km/h' },
        });
        expect((screen.getByRole('combobox', { name: 'Target' }) as HTMLInputElement).value).toBe('v');
        expect((screen.getByLabelText('Show in') as HTMLInputElement).value).toBe('km/h');
        expect((screen.getByLabelText(/Known values/) as HTMLTextAreaElement).value).toBe('h_cg = 0.205m\nR_c = 14.5m\nt_tr = 1.24m');
        await waitFor(() => expect(document.querySelector('.answer .a-value')?.textContent).toBe('74.7 km/h'));
    });

    it('every chain question sends exactly its own target, known values and display unit', async () => {
        const engine = fakeEngine(common);
        render(ChainView);
        const section = await openPast();
        for (const ex of chainRows) {
            engine.calls.length = 0;
            await userEvent.click(within(section).getByRole('button', { name: new RegExp(`^Q${ex.id}\\b`) }));
            await waitFor(() => expect(engine.calls.filter((c) => c.cmd === 'chain_formulas')).toHaveLength(1));
            const [, ...args] = splitCommand(ex.cmd);
            const target = args.find((a) => !a.includes('='))!;
            const sent = engine.calls.find((c) => c.cmd === 'chain_formulas')!.args as { target: string; given: [string, string][]; display: Record<string, string> };
            expect(sent.target, ex.cmd).toBe(target);
            expect(sent.given.map(([k, v]) => `${k}=${v}`), ex.cmd).toEqual(args.filter((a) => a.includes('=') && !a.startsWith('@')));
            expect(sent.display[target] ?? '', ex.cmd).toBe(args.find((a) => a.startsWith(`@${target}=`))?.split('=')[1] ?? '');
        }
    });
});

describe('past questions fold', () => {
    it('is closed at first, opens and closes with the keyboard, and the chips are only reachable while open', async () => {
        fakeEngine(common);
        render(ChainView);
        await screen.findByRole('region', { name: 'Past questions' });
        const head = pastButton();
        expect(head.getAttribute('aria-expanded')).toBe('false');
        const body = document.getElementById(head.getAttribute('aria-controls')!)!;
        expect(body.hidden).toBe(true);
        expect(screen.queryByRole('button', { name: /^Q515 ·/ })).toBeNull();

        head.focus();
        await userEvent.keyboard('{Enter}');
        expect(head.getAttribute('aria-expanded')).toBe('true');
        expect(body.hidden).toBe(false);
        expect(screen.getByRole('button', { name: /^Q515 ·/ })).toBeTruthy();

        await userEvent.keyboard(' ');
        expect(head.getAttribute('aria-expanded')).toBe('false');
        expect(screen.queryByRole('button', { name: /^Q515 ·/ })).toBeNull();
    });

    it('remembers its state: the choice lives in the settings, so a new view opens the way it was left', async () => {
        fakeEngine(common);
        const first = render(ChainView);
        await screen.findByRole('region', { name: 'Past questions' });
        await userEvent.click(pastButton());
        expect(settings.chainPastOpen).toBe(true);
        first.unmount();

        render(ChainView);
        await screen.findByRole('region', { name: 'Past questions' });
        expect(pastButton().getAttribute('aria-expanded')).toBe('true');
        expect(screen.getByRole('button', { name: /^Q515 ·/ })).toBeTruthy();
    });
});

describe('names typed the way people write them', () => {
    const typeAndRun = async (given: string): Promise<void> => {
        await userEvent.type(screen.getByRole('combobox', { name: 'Target' }), 's');
        await userEvent.type(screen.getByLabelText(/Known values/), given);
        await userEvent.click(screen.getByRole('button', { name: 'Chain' }));
    };

    it('sends the names as typed and says, quietly, what each was read as', async () => {
        const engine = fakeEngine({
            ...common,
            chain_formulas: () => ({
                ...reached,
                target: '40 m',
                mapped: [
                    { from: 'v_i', to: 'v0', desc: 'initial velocity' },
                    { from: 'v_f', to: 'v', desc: 'final velocity' },
                ],
            }),
        });
        render(ChainView);
        await typeAndRun('v_i = 0{Enter}v_f = 20 m/s{Enter}t = 4 s');
        await waitFor(() => expect(document.querySelector('.answer .a-value')?.textContent).toBe('40 m'));
        expect(engine.calls.find((c) => c.cmd === 'chain_formulas')!.args.given).toEqual([['v_i', '0'], ['v_f', '20 m/s'], ['t', '4 s']]);
        const notes = within(screen.getByRole('list', { name: 'Names read as' })).getAllByRole('listitem').map((l) => l.textContent);
        expect(notes).toEqual(['v_i → v0 (initial velocity)', 'v_f → v (final velocity)']);
        expect(document.querySelector('.note-bad')).toBeNull();
    });

    it('an unknown name offers its likely meanings; clicking one rewrites that line and chains again', async () => {
        const engine = fakeEngine({
            ...common,
            chain_formulas: (args) =>
                (args.given as [string, string][]).some(([n]) => n === 'v_intial')
                    ? {
                          ...reached,
                          reached: false,
                          target: null,
                          problems: [
                              {
                                  name: 'v_intial',
                                  kind: 'unknown',
                                  choices: [
                                      { name: 'v0', desc: 'initial velocity' },
                                      { name: 'v', desc: 'final velocity' },
                                  ],
                              },
                          ],
                      }
                    : { ...reached, target: '40 m' },
        });
        render(ChainView);
        await typeAndRun('v_intial = 0{Enter}t = 4 s');
        const note = await screen.findByText(/^Unknown name “v_intial”\./);
        expect(note.textContent?.replace(/\s+/g, ' ').trim()).toBe('Unknown name “v_intial”. Did you mean v0 (initial velocity) or v (final velocity)?');
        // nothing was solved, so no "no path" verdict is shown
        expect(document.querySelector('.answer')).toBeNull();

        await userEvent.click(within(note).getByRole('button', { name: 'v0 (initial velocity)' }));
        await waitFor(() => expect(document.querySelector('.answer .a-value')?.textContent).toBe('40 m'));
        expect((screen.getByLabelText(/Known values/) as HTMLTextAreaElement).value).toBe('v0 = 0\nt = 4 s');
        expect(engine.calls.filter((c) => c.cmd === 'chain_formulas').at(-1)!.args.given).toEqual([['v0', '0'], ['t', '4 s']]);
        expect(screen.queryByText(/Unknown name/)).toBeNull();
    });

    it('an ambiguous name lists its meanings instead of guessing', async () => {
        fakeEngine({
            ...common,
            chain_formulas: () => ({
                ...reached,
                reached: false,
                target: null,
                problems: [
                    {
                        name: 'r',
                        kind: 'ambiguous',
                        choices: [
                            { name: 'R_c', desc: 'corner radius' },
                            { name: 'r_w', desc: 'wheel dynamic radius' },
                        ],
                    },
                ],
            }),
        });
        render(ChainView);
        await typeAndRun('r = 14.5 m');
        const note = await screen.findByText(/^“r” could mean several variables\./);
        expect(within(note).getAllByRole('button').map((b) => b.textContent)).toEqual(['R_c (corner radius)', 'r_w (wheel dynamic radius)']);
    });
});

describe('target picker', () => {
    it('finds a variable by a word of its description and by words of the formulas that use it', async () => {
        fakeEngine(common);
        render(ChainView);
        const box = screen.getByRole('combobox', { name: 'Target' });
        for (const query of ['velocity', 'rollover speed']) {
            await userEvent.clear(box);
            await userEvent.type(box, query);
            const names = within(screen.getByRole('listbox', { name: 'Variables' })).getAllByRole('option').map((o) => o.textContent ?? '');
            expect(names.some((t) => t.includes('— (final) velocity [m/s]')), query).toBe(true);
        }
    });

    it('is a keyboard combobox: arrows move, Enter picks the symbol, a second Enter runs', async () => {
        const engine = fakeEngine(common);
        render(ChainView);
        const box = screen.getByRole('combobox', { name: 'Target' });
        await userEvent.type(box, 'velocity');
        expect(box.getAttribute('aria-expanded')).toBe('true');
        const options = screen.getAllByRole('option');
        expect(box.getAttribute('aria-activedescendant')).toBe(options[0].id);
        await userEvent.keyboard('{ArrowDown}');
        expect(box.getAttribute('aria-activedescendant')).toBe(options[1].id);
        await userEvent.keyboard('{ArrowUp}{Enter}');
        expect((box as HTMLInputElement).value).toBe('v');
        expect(box.getAttribute('aria-expanded')).toBe('false');
        expect(engine.calls.filter((c) => c.cmd === 'chain_formulas')).toHaveLength(0);
        await userEvent.keyboard('{Enter}');
        await waitFor(() => expect(engine.calls.filter((c) => c.cmd === 'chain_formulas')).toHaveLength(1));
    });

    it('types the symbol of the target in the result and keeps what it was run for', async () => {
        fakeEngine(common);
        render(ChainView);
        session.chain = { ...session.chain, target: 'v', given: 'h_cg = 0.205 m', run: 1 };
        await waitFor(() => expect(document.querySelector('.answer .a-label .katex')).not.toBeNull());
        await userEvent.clear(screen.getByRole('combobox', { name: 'Target' }));
        expect(document.querySelector('.answer .a-label .katex')).not.toBeNull();
    });
});
