// How defaults are put in words and what a field starts with. Defaults come from the engine (src/tools.rs).
import { describe, expect, it } from 'vitest';

import { describeDefault, initialValue, placeholderFor } from './toolform';
import type { ParamInfo } from './types';

const param = (over: Partial<ParamInfo>): ParamInfo => ({ name: 'x', number: true, default: null, label: 'X', unit: null, help: null, choices: null, switch: false, ...over });

describe('describeDefault', () => {
    it.each([
        ['56e6', '56 000 000'],
        ['10000', '10 000'],
        ['120', '120'],
        ['1.225', '1.225'],
        ['9.81', '9.81'],
        ['6e6', '6 000 000'],
        ['-1200.5', '-1 200.5'],
        ['20degC', '20 degC'],
        ['3', '3'],
    ])('%s is written %s', (given, said) => {
        expect(describeDefault(given)).toBe(said);
    });
});

describe('placeholderFor', () => {
    it('says nothing for a required field, "optional" for a blank default, else the default in words', () => {
        expect(placeholderFor(param({ default: null }))).toBe('');
        expect(placeholderFor(param({ default: '' }))).toBe('optional');
        expect(placeholderFor(param({ default: '56e6' }))).toBe('defaults to 56 000 000');
    });
});

describe('initialValue', () => {
    const years = [
        { value: 'legacy', label: 'legacy' },
        { value: '2027', label: '2027' },
    ];

    it('takes the Settings year for the rules parameter', () => {
        expect(initialValue(param({ name: 'rules', default: '', choices: years }), '2027')).toBe('2027');
    });

    it('starts a switch at its default, and a choice at its default when that is one of the choices', () => {
        expect(initialValue(param({ switch: true, default: '1' }), '2027')).toBe('1');
        expect(initialValue(param({ switch: true, default: '0' }), '2027')).toBe('0');
        const modes = [
            { value: 'creepage', label: 'c' },
            { value: 'coated', label: 'k' },
        ];
        expect(initialValue(param({ choices: modes, default: 'creepage' }), '2027')).toBe('creepage');
        expect(initialValue(param({ choices: modes, default: null }), '2027')).toBe('');
    });

    it('leaves text and number inputs blank so the tool applies its own default', () => {
        expect(initialValue(param({ default: '56e6' }), '2027')).toBe('');
    });
});
