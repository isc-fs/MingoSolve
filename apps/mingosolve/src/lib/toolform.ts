// Small rules of the tool forms: what a field starts with, and how a default is put in words.
import type { ParamInfo } from './types';

/** What the field holds before anything is typed: the Settings rule year for `rules`, the tool's default for a
 *  switch or a choice, otherwise blank (blank means "the tool's default" for text and number inputs). */
export function initialValue(p: ParamInfo, rulesYear: string): string {
    if (p.name === 'rules' && p.choices !== null) return rulesYear;
    if (p.switch) return p.default ?? '0';
    if (p.choices !== null) return p.default !== null && p.choices.some((c) => c.value === p.default) ? p.default : '';
    return '';
}

/** A default as people write it: 56e6 -> "56 000 000", 20degC -> "20 degC". */
export function describeDefault(d: string): string {
    const m = d.trim().match(/^(-?[\d.]+(?:e[-+]?\d+)?)\s*(.*)$/i);
    if (m === null) return d;
    const n = Number(m[1]);
    let num = m[1];
    if (Number.isFinite(n) && !/e/i.test(String(n))) {
        const [int, frac] = String(Math.abs(n)).split('.');
        num = `${n < 0 ? '-' : ''}${int.replace(/\B(?=(\d{3})+$)/g, ' ')}${frac === undefined ? '' : `.${frac}`}`;
    }
    return m[2] === '' ? num : `${num} ${m[2]}`;
}

/** What an empty input says: nothing for a required one (it is marked instead), else its default in words. */
export function placeholderFor(p: ParamInfo): string {
    if (p.default === null) return '';
    if (p.default === '') return 'optional';
    return `defaults to ${describeDefault(p.default)}`;
}
