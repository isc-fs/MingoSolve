// Engine errors in plain words. Every raw string below was produced by the engine itself (`fsq` CLI on bad
// inputs, 2026-10-05; "internal error" comes from src-tauri/src/lib.rs), so these are the texts people really get.
import { describe, expect, it } from 'vitest';

import { friendlyError, type ErrorContext, type ErrorField } from './errors';

const f = (name: string, label: string, value: string): ErrorField => ({ name, label, value });
const speed: ErrorContext = {
    fields: [f('s', 'distance', '3kg'), f('t', 'time', '2s')],
    nameOf: (v) => ({ v_avg: 'average speed', s: 'distance' })[v],
};

const cases: [string, string, string, ErrorContext][] = [
    ['unit of the wrong kind, field found by its text', '3kg is kg, expected m (m)', 'Distance needs a length, like 3 m or 250 mm. “3kg” is a mass instead.', speed],
    ['unit of the wrong kind, field unknown', '3kg is kg, expected m (m)', '“3kg” isn\'t a length. Use something like 3 m or 250 mm.', {}],
    [
        'a speed given where a volume goes',
        '54km/h is m·s^-1, expected m**3 (m^3)',
        'Engine size needs a volume, like 600 cm**3 or 0.6 L. “54km/h” is a speed instead.',
        { fields: [f('V_d', 'engine size', '54km/h')] },
    ],
    [
        'a conductivity given a time (real engine text)',
        '5s is s, expected S/m (m^-3·kg^-1·s^3·A^2)',
        'Electrical conductivity needs a conductivity, like 56e6 S/m. “5s” is a time instead.',
        { fields: [f('sigma_c', 'electrical conductivity', '5s')] },
    ],
    [
        'a kind the table does not know falls back to the unit asked for (same engine format)',
        '5s is s, expected J/K (m^2·kg·s^-2·K^-1)',
        '“5s” isn\'t a value in J/K. Use something like 1 J/K.',
        {},
    ],
    ['unknown unit inside a value', 'unknown unit "xyz"', 'Distance: “xyz” isn\'t a unit we know. Try symbols like m, kg, s, km/h or kW.', { fields: [f('s', 'distance', '3xyz'), f('t', 'time', '2s')] }],
    [
        'unknown unit in a tool parameter, named by the engine',
        't_team: unknown unit "abc"',
        'Your corrected time: “abc” isn\'t a unit we know. Try symbols like m, kg, s, km/h or kW.',
        { fields: [f('t_team', 'your corrected time', 'abc')] },
    ],
    ['unknown unit typed in the Show in box', 'unknown unit "xyz"', 'Show in: “xyz” isn\'t a unit we know. Try symbols like m, kg, s, km/h or kW.', { fields: [f('s', 'distance', '3m'), f('@v_avg', 'Show in', 'xyz')] }],
    [
        'display unit of another kind',
        'cannot show v_avg (m/s) in kg',
        'Average speed is measured in m/s, so it can\'t be shown in “kg”. Pick a unit of the same kind, like m/s.',
        speed,
    ],
    ['display unit of another kind, variable unknown to the page', 'cannot show v_avg (m/s) in kg', 'V_avg is measured in m/s, so it can\'t be shown in “kg”. Pick a unit of the same kind, like m/s.', {}],
    ['value that stops short', 'unexpected end of expression', 'Distance looks unfinished: “3m/” stops where a number or unit should follow.', { fields: [f('s', 'distance', '3m/ '), f('t', 'time', '2s')] }],
    ['value that stops short, field unclear', 'unexpected end of expression', 'A value looks unfinished, like “3 +” or “5 /”. Check the fields for a sign at the end.', speed],
    ['caret for a power', 'use ** for powers (^ is not allowed)', 'Distance: write powers with **, like 2**3, not ^.', { fields: [f('s', 'distance', '3^2m')] }],
    ['fractional power of a unit', 'fractional power of a unit', 'Units can only be raised to whole powers: m**2, not m**0.5.', {}],
    ['adding different kinds', 'cannot add m and s', 'Can\'t add or subtract a length and a time: they are different kinds of quantity. Check the units.', {}],
    ['stray character', "unexpected character '@'", 'Distance: “@” isn\'t allowed in a value.', { fields: [f('s', 'distance', '3@')] }],
    ['huge number', 'number 1e999 is too large', 'Distance: the number 1e999 is too large for the solver.', { fields: [f('s', 'distance', '1e999m')] }],
    ['a bare number glued to a unit', 'unexpected number 1', 'A value has something unexpected in it (“number 1”). Look for a missing sign or a stray symbol.', {}],
    ['tool parameter left empty', 'event_score: missing t_team', 'Your corrected time is empty: it needs a value.', { fields: [f('t_team', 'your corrected time', '')] }],
    ['bad number in a circuit', 'bad number x', 'Circuit: “x” isn\'t a number.', { fields: [f('netlist', 'circuit', 'R1 1 2 x')] }],
    ['field the script does not have', 'speed has no variable foo; it uses v_avg, s, t', 'This script has no field called foo.', {}],
    ['division by zero', 'no finite answer (division by zero or overflow)', 'These values give no finite answer. Look for a zero where something is divided, such as a time or a step of 0.', {}],
    ['value that must be positive', 'value must be positive', 'The value must be greater than zero.', {}],
    ['internal failure', 'internal error: worker panicked', 'Something went wrong inside the solver. Try again, and tell the team if it keeps happening.', {}],
];

describe('friendlyError turns what the engine says into a sentence that names the field', () => {
    it.each(cases)('%s', (_, raw, expected, context) => {
        expect(friendlyError(raw, context)).toBe(expected);
    });

    it('accepts the Error object the invoke call rejects with', () => {
        expect(friendlyError(new Error('cannot add m and s'))).toContain('Can\'t add or subtract');
    });

    it('leaves messages it does not know exactly as the engine wrote them', () => {
        for (const raw of ['above 600 V: TS max is 600 V (EV 4.1.1)', 'mode for 2027: clearance | creepage | coated', 'rules 2027: no tmax_pmin.bogus']) {
            expect(friendlyError(raw, speed)).toBe(raw);
        }
    });

    it('never blames a field when two fields could be at fault', () => {
        const two: ErrorContext = { fields: [f('s', 'distance', '3xyz'), f('t', 'time', '2xyz')] };
        expect(friendlyError('unknown unit "xyz"', two)).toBe('“xyz” isn\'t a unit we know. Try symbols like m, kg, s, km/h or kW.');
    });
});
