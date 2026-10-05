// How scripts get opened: from worked examples (engine command lines), from the rail with the pasted problem's
// values, and what is remembered.
import { parse } from 'smol-toml';
import examplesToml from '../../../../data/examples.toml?raw';
import { beforeEach, describe, expect, it } from 'vitest';

import { openCommand, openScript, session, splitCommand } from './session.svelte';
import { defaultSettings, settings } from './settings.svelte';

const examples = (parse(examplesToml) as {
    example: { id: number; cmd: string; answer: number }[];
}).example;

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    session.script = null;
    session.problemFills = {};
    session.calcInput = '';
});

describe('worked examples', () => {
    it('every past-question command opens its script with every argument kept', () => {
        const opened = examples.filter((e) => !/^(chain|calc) /.test(e.cmd));
        expect(opened.length).toBeGreaterThan(100);
        for (const e of opened) {
            openCommand(e.cmd, e.answer);
            const s = session.script!;
            const [head, ...args] = e.cmd.split(/\s+/);
            expect(s.id, e.cmd).toBe(head);
            expect(s.answer).toBe(e.answer);
            const rebuilt = [
                ...s.values.map(([k, v]) => `${k}=${v}`),
                ...Object.entries(s.display ?? {}).map(([k, v]) => `@${k}=${v}`),
                ...(s.positional ?? []),
            ];
            expect(rebuilt.sort(), e.cmd).toEqual([...args].sort());
        }
    });

    it('a calculator example goes to the calculator and leaves the open script alone', () => {
        openScript('battery_load', [['N_s', '103']]);
        const before = session.script;
        openCommand('calc (14.4V/(0.4ohm+0.1ohm))**2*0.1ohm -> W');
        expect(session.calcInput).toBe('(14.4V/(0.4ohm+0.1ohm))**2*0.1ohm -> W');
        expect(session.script).toBe(before);
    });

    it('a chain example fills the Chain form (target, one known per line, show-in unit, tag filter) and switches to it', () => {
        session.activeView = 'solve';
        const before = session.chain.run;
        openCommand('chain v h_cg=0.205m R_c=14.5m @v=km/h @h_cg=mm only=dynamics,cg');
        expect(session.chain).toEqual({
            target: 'v',
            given: 'h_cg = 0.205m\nR_c = 14.5m',
            only: 'dynamics, cg',
            shownIn: 'km/h',
            run: before + 1,
        });
        expect(session.activeView).toBe('chain');
    });

    it('a calculator example opens the panel and asks it to evaluate', () => {
        settings.calcOpen = false;
        const before = session.calcRun;
        openCommand('calc 3600/(6000/0.0005)**0.25');
        expect(settings.calcOpen).toBe(true);
        expect(session.calcRun).toBe(before + 1);
    });

    it('quotes group a multi-word argument such as a netlist', () => {
        expect(splitCommand('nodal "R1 1 0 2; R2 1 2 3" @V=mV')).toEqual(['nodal', 'R1 1 0 2; R2 1 2 3', '@V=mV']);
        expect(splitCommand('tool ""  x')).toEqual(['tool', '', 'x']);
    });
});

describe('opening a script with the pasted problem', () => {
    beforeEach(() => {
        session.problemFills = { battery_load: { values: [['N_s', '103'], ['P', '30 kW']], target: 'I' } };
    });

    it('takes the values the problem gives it, and the variable it asks for', () => {
        openScript('battery_load');
        expect(session.script).toMatchObject({ values: [['N_s', '103'], ['P', '30 kW']], target: 'I' });
    });

    it("a worked example's own values win over the problem's", () => {
        openCommand('battery_load N_s=96 V_cell=4.2V', 1);
        expect(session.script!.values).toEqual([['N_s', '96'], ['V_cell', '4.2V']]);
        expect(session.script!.target).toBeUndefined();
    });

    it('every open is a fresh sheet, even for the script already open', () => {
        openScript('battery_load');
        const first = session.script!.nonce;
        openScript('battery_load');
        expect(session.script!.nonce).not.toBe(first);
    });
});

it('recent scripts: newest first, no repeats, at most eight', () => {
    for (const id of ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'b']) openScript(id);
    expect(settings.recent).toEqual(['b', 'i', 'h', 'g', 'f', 'e', 'd', 'c']);
});
