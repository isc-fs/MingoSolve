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
        vars: vars.map(([name, desc, tex]) => ({ name, desc, unit: 'm/s', signed: false, hint: null, default: null, tex, unit_shown: 'm/s' })),
    };
    return { id: key, kind: 'formula', title, aliases: '', topic: null, formula: f };
}

const reached: ChainResult = { steps: [], defaults: [], reached: true, target: '74.7 km/h', known: [], conflicts: [] };

const common = { chain_examples: () => chainRows, chain_formulas: () => reached };

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
        const section = await screen.findByRole('region', { name: 'Past questions' });
        expect(within(section).getAllByRole('button')).toHaveLength(chainRows.length);
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
        const section = await screen.findByRole('region', { name: 'Past questions' });
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
