// Session log persistence (real plugin-store client over an in-memory store), the 500-entry cap, the clock and
// mock-quiz rules, and the CSV export parsed back with an independent RFC 4180 reader.
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { LogEntry } from './sessionlog.svelte';

type Log = typeof import('./sessionlog.svelte');
type Csv = typeof import('./csv');

let disk: Record<string, unknown>;
let writes: number;

function fakeStore(noRuntime = false): void {
    mockIPC(async (cmd, payload) => {
        const args = (payload ?? {}) as Record<string, unknown>;
        if (noRuntime) throw new Error('no Tauri runtime');
        if (cmd === 'plugin:store|load') return 1;
        if (cmd === 'plugin:store|get') return [disk[String(args.key)] ?? null, String(args.key) in disk];
        if (cmd === 'plugin:store|set') {
            disk[String(args.key)] = JSON.parse(JSON.stringify(args.value));
            writes += 1;
            return null;
        }
        if (cmd === 'plugin:store|save') return null;
        throw new Error(`unexpected ${cmd}`);
    });
}

async function fresh(): Promise<Log & Csv> {
    vi.resetModules();
    return { ...(await import('./sessionlog.svelte')), ...(await import('./csv')) };
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const copy = (over: Partial<Parameters<Log['recordCopy']>[0]> = {}) => ({
    scriptId: 'uniform_motion',
    title: 'Uniform motion',
    inputs: [['s', '100 m']] as [string, string][],
    answer: '12.5',
    rules: null,
    via: 'button' as const,
    ...over,
});

beforeEach(() => {
    disk = {};
    writes = 0;
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date('2027-01-12T10:00:00Z'));
});
afterEach(() => vi.useRealTimers());

describe('persistence', () => {
    it('entries and the running mock quiz survive a restart', async () => {
        fakeStore();
        const a = await fresh();
        await a.loadSessionLog();
        a.startMockQuiz();
        vi.setSystemTime(new Date('2027-01-12T10:02:00Z'));
        a.sessionLog.label = 'Q3';
        a.recordCopy(copy({ inputs: [['V', '3,8 V'], ['R', '0.08 Ω']], answer: '47,5' }));
        await sleep(350);
        expect(writes).toBe(1);

        const b = await fresh();
        await b.loadSessionLog();
        expect(b.sessionLog.entries).toEqual([
            {
                at: '2027-01-12T10:02:00Z',
                quiz: 'mock-20270112T1000',
                question: 'Q3',
                scriptId: 'uniform_motion',
                title: 'Uniform motion',
                inputs: [['V', '3,8 V'], ['R', '0.08 Ω']],
                answer: '47,5',
                rules: null,
                elapsedS: 120,
                via: 'button',
            },
        ]);
        expect(b.sessionLog.quiz).toEqual({ id: 'mock-20270112T1000', startedAt: '2027-01-12T10:00:00Z' });
    });

    it('a copy made before the file was read is kept next to what was on disk, not lost or overwritten', async () => {
        const old: LogEntry = { at: '2027-01-11T09:00:00Z', quiz: null, question: 'Q1', scriptId: 'x', title: 'X', inputs: [], answer: '1', rules: null, elapsedS: 5, via: 'button' };
        disk.all = { entries: [old], quiz: null };
        fakeStore();
        const m = await fresh();
        m.recordCopy(copy({ answer: '2' }));
        await m.loadSessionLog();
        expect(m.sessionLog.entries.map((e) => e.answer)).toEqual(['1', '2']);
        await sleep(350);
        expect((disk.all as { entries: LogEntry[] }).entries.map((e) => e.answer)).toEqual(['1', '2']);
    });

    it('keeps only the last 500 entries, in memory and on disk', async () => {
        fakeStore();
        const m = await fresh();
        await m.loadSessionLog();
        for (let i = 1; i <= 505; i++) m.recordCopy(copy({ answer: String(i) }));
        expect(m.sessionLog.entries).toHaveLength(500);
        expect(m.sessionLog.entries[0].answer).toBe('6');
        expect(m.sessionLog.entries[499].answer).toBe('505');
        await sleep(350);
        expect((disk.all as { entries: LogEntry[] }).entries).toHaveLength(500);
    });

    it('clearing the log empties the file too', async () => {
        fakeStore();
        const m = await fresh();
        await m.loadSessionLog();
        m.recordCopy(copy());
        await sleep(350);
        m.clearLog();
        await sleep(350);
        expect((disk.all as { entries: LogEntry[] }).entries).toEqual([]);
    });

    it('without a Tauri runtime it keeps the log in memory and never saves', async () => {
        fakeStore(true);
        const m = await fresh();
        await m.loadSessionLog();
        m.recordCopy(copy());
        await sleep(350);
        expect(m.sessionLog.entries).toHaveLength(1);
        expect(writes).toBe(0);
    });
});

describe('the clock and the mock quiz', () => {
    it('elapsed time counts from the paste; typing into the problem does not restart it, pasting another does', async () => {
        fakeStore();
        const m = await fresh();
        const pasted = 'A car of mass 240 kg drives the skidpad with a friction coefficient of 1.4, what is the speed?';
        m.problemChanged(pasted);
        vi.setSystemTime(new Date('2027-01-12T10:00:30Z'));
        m.problemChanged(pasted + ' ');
        vi.setSystemTime(new Date('2027-01-12T10:01:00Z'));
        m.recordCopy(copy());
        expect(m.sessionLog.entries[0].elapsedS).toBe(60);

        m.problemChanged('Another, entirely different problem about a battery pack of 103 cells.');
        vi.setSystemTime(new Date('2027-01-12T10:01:10Z'));
        m.recordCopy(copy());
        expect(m.sessionLog.entries[1].elapsedS).toBe(10);

        m.problemChanged('');
        m.recordCopy(copy());
        expect(m.sessionLog.entries[2].elapsedS).toBeNull();
    });

    it('Start a mock quiz resets the timer and tags later entries only; ending it stops the tagging', async () => {
        fakeStore();
        const m = await fresh();
        m.startClock();
        vi.setSystemTime(new Date('2027-01-12T10:05:00Z'));
        m.recordCopy(copy({ answer: 'before' }));
        m.startMockQuiz();
        vi.setSystemTime(new Date('2027-01-12T10:05:45Z'));
        m.recordCopy(copy({ answer: 'during' }));
        m.endMockQuiz();
        m.recordCopy(copy({ answer: 'after' }));
        const [before, during, after] = m.sessionLog.entries;
        expect([before.elapsedS, during.elapsedS]).toEqual([300, 45]);
        expect([before.quiz, during.quiz, after.quiz]).toEqual([null, 'mock-20270112T1005', null]);
    });
});

/** RFC 4180 reader, written separately from the writer: quoted fields, doubled quotes, line breaks inside quotes. */
function parseCsv(text: string, delimiter: string): string[][] {
    const rows: string[][] = [];
    let row: string[] = [];
    let field = '';
    let quoted = false;
    for (let i = 0; i < text.length; i++) {
        const c = text[i];
        if (quoted) {
            if (c === '"' && text[i + 1] === '"') {
                field += '"';
                i++;
            } else if (c === '"') quoted = false;
            else field += c;
        } else if (c === '"') quoted = true;
        else if (c === delimiter) {
            row.push(field);
            field = '';
        } else if (c === '\r' && text[i + 1] === '\n') {
            row.push(field);
            rows.push(row);
            row = [];
            field = '';
            i++;
        } else field += c;
    }
    if (field !== '' || row.length > 0) {
        row.push(field);
        rows.push(row);
    }
    return rows;
}

describe('CSV export', () => {
    const nasty: LogEntry[] = [
        { at: '2027-01-12T10:01:23Z', quiz: 'mock-20270112T1000', question: 'Q7', scriptId: 'battery_load', title: 'Battery under load', inputs: [['N_s', '103'], ['R_pack', '0.08 Ω'], ['T', '60 °C']], answer: '77,887', rules: null, elapsedS: 83, via: 'button' },
        { at: '2027-01-12T10:02:00Z', quiz: null, question: 'say "hi", Q8', scriptId: 'event_score', title: 'Dynamic event score', inputs: [['event', 'skid, pad'], ['C', '1800 µF']], answer: '41.117\n  I1 = 3 A\n  "x" = 1,5', rules: 'legacy', elapsedS: null, via: 'copy-all' },
        { at: '2027-01-12T10:03:00Z', quiz: null, question: '', scriptId: 'a', title: 'Ünïcode — µ Ω °C', inputs: [], answer: '-5.2', rules: null, elapsedS: 0, via: 'shortcut' },
    ];
    const expected = (e: LogEntry, inputs: string): string[] => [e.at, e.quiz ?? '', e.question, e.scriptId, e.title, inputs, e.answer, e.rules ?? '', e.elapsedS === null ? '' : String(e.elapsedS), e.via];

    for (const delimiter of [',', ';'] as const) {
        it(`parses back to the original fields with "${delimiter}": commas, quotes, line breaks, µ Ω °C`, async () => {
            const m = await fresh();
            const text = m.entriesToCsv(nasty, delimiter);
            expect(text.startsWith('﻿')).toBe(true);
            expect(text.endsWith('\r\n')).toBe(true);
            const rows = parseCsv(text.slice(1), delimiter);
            expect(rows[0]).toEqual([...m.LOG_COLUMNS]);
            expect(rows.slice(1)).toEqual([
                expected(nasty[0], 'N_s=103; R_pack=0.08 Ω; T=60 °C'),
                expected(nasty[1], 'event=skid, pad; C=1800 µF'),
                expected(nasty[2], ''),
            ]);
            for (const r of rows) expect(r).toHaveLength(10);
        });
    }

    it('a decimal-comma answer stays in one column', async () => {
        const m = await fresh();
        const rows = parseCsv(m.entriesToCsv([nasty[0]]).slice(1), ',');
        expect(rows[1][6]).toBe('77,887');
        expect(rows[1]).toHaveLength(10);
    });

    it('timestamps are ISO 8601 UTC to the second', async () => {
        const m = await fresh();
        expect(parseCsv(m.entriesToCsv(nasty).slice(1), ',').slice(1).map((r) => r[0])).toEqual(nasty.map((e) => e.at));
        expect(nasty.every((e) => !Number.isNaN(Date.parse(e.at)))).toBe(true);
    });
});
