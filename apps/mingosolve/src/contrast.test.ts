// WCAG contrast of the token pairs the interface actually draws, computed from the real custom properties in
// app.css (nothing is hard-coded here): text needs 4.5:1 (1.4.3), focus rings and UI boundaries 3:1 (1.4.11).
// Glass is translucent, so every pair is checked over the worst case behind it: the opaque [data-solid] surface,
// the glass over the bare ground, and the glass over each glow of the lit backdrop.
import { describe, expect, it } from 'vitest';

import { readFileSync } from 'node:fs';

// vitest runs from apps/mingosolve
const css = readFileSync('src/app.css', 'utf8');

type Tokens = Record<string, string>;
type Rgba = { r: number; g: number; b: number; a: number };

function parseBlocks(source: string): { selectors: string[]; decls: Tokens }[] {
    const text = source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/--grain-image:[^\n]*\n/, '');
    const out: { selectors: string[]; decls: Tokens }[] = [];
    for (const m of text.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
        const decls: Tokens = {};
        for (const d of m[2].matchAll(/(--[\w-]+)\s*:\s*((?:[^;(]|\([^)]*(?:\([^)]*\)[^)]*)*\))*);/g)) decls[d[1]] = d[2].trim();
        out.push({ selectors: m[1].split(',').map((s) => s.trim()), decls });
    }
    return out;
}

/** The custom properties in force for a theme: plain :root rules, then the theme's own, in source order. */
function tokens(theme: 'dark' | 'light'): Tokens {
    const own = `:root[data-theme='${theme}']`;
    const all: Tokens = {};
    for (const b of parseBlocks(css)) {
        if (b.selectors.includes(':root') || b.selectors.includes(own)) Object.assign(all, b.decls);
    }
    return all;
}

function resolve(t: Tokens, name: string, seen = 0): string {
    if (seen > 10) throw new Error(`token loop at ${name}`);
    const raw = t[name];
    if (raw === undefined) throw new Error(`token ${name} not found in app.css`);
    return raw.replace(/var\((--[\w-]+)\)/g, (_, n: string) => resolve(t, n, seen + 1));
}

function parseColor(v: string): Rgba {
    const s = v.trim();
    let m = s.match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
    if (m !== null) {
        const h = m[1].length === 3 ? [...m[1]].map((c) => c + c).join('') : m[1];
        return { r: parseInt(h.slice(0, 2), 16), g: parseInt(h.slice(2, 4), 16), b: parseInt(h.slice(4, 6), 16), a: 1 };
    }
    m = s.match(/^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*(?:,\s*([\d.]+)\s*)?\)$/);
    if (m !== null) return { r: +m[1], g: +m[2], b: +m[3], a: m[4] === undefined ? 1 : +m[4] };
    throw new Error(`cannot parse colour ${v}`);
}

/** Every colour literal inside a value (the stops of a gradient), in order. */
function colorsIn(value: string): Rgba[] {
    return [...value.matchAll(/#[0-9a-f]{6}\b|#[0-9a-f]{3}\b|rgba?\([^)]*\)/gi)].map((m) => parseColor(m[0]));
}

const over = (fg: Rgba, bg: Rgba): Rgba => ({
    r: fg.r * fg.a + bg.r * (1 - fg.a),
    g: fg.g * fg.a + bg.g * (1 - fg.a),
    b: fg.b * fg.a + bg.b * (1 - fg.a),
    a: 1,
});

function luminance(c: Rgba): number {
    const f = (v: number) => {
        const x = v / 255;
        return x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4;
    };
    return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b);
}

function ratio(a: Rgba, b: Rgba): number {
    const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
    return (hi + 0.05) / (lo + 0.05);
}

/** The opaque colours a glass panel can sit on. */
function surfaces(t: Tokens): Rgba[] {
    const ground = parseColor(resolve(t, '--ground'));
    const glass = parseColor(resolve(t, '--glass'));
    const strong = parseColor(resolve(t, '--glass-strong'));
    const glows = colorsIn(resolve(t, '--glow'));
    const backdrops = [ground, ...glows.map((g) => over(g, ground))];
    return [parseColor(resolve(t, '--glass-solid')), ...backdrops.flatMap((b) => [over(glass, b), over(strong, b)])];
}

/** The answer slab: its gradient stops over the solid layer under them. */
function slab(t: Tokens): Rgba[] {
    const solid = parseColor(resolve(t, '--glass-solid'));
    return colorsIn(resolve(t, '--answer-bg')).map((c) => over(c, solid));
}

/** Text token on a (possibly translucent) fill over each surface: the worst ratio. */
function worst(t: Tokens, fg: string, fill: string | null, on: Rgba[], opacity = 1): number {
    const fgc = parseColor(resolve(t, fg));
    let min = Infinity;
    for (const s of on) {
        const bg = fill === null ? s : over(parseColor(resolve(t, fill)), s);
        const text = over({ ...fgc, a: fgc.a * opacity }, bg);
        min = Math.min(min, ratio(text, bg));
    }
    return min;
}

const themes = ['dark', 'light'] as const;

describe.each(themes)('%s theme', (theme) => {
    const t = tokens(theme);
    const glass = surfaces(t);
    const sl = slab(t);

    it('reads the real tokens (a typo in a name would throw, not pass)', () => {
        expect(resolve(t, '--text').length).toBeGreaterThan(0);
        expect(sl.length).toBeGreaterThanOrEqual(2);
        expect(glass.length).toBeGreaterThan(4);
    });

    it.each([
        ['--text', null],
        ['--text-2', null],
        ['--muted', null],
        ['--ink-accent', null],
        ['--var', null],
        ['--muted', '--well'],
        ['--muted', '--field'],
        ['--ink-accent', '--accent-soft'],
        ['--good', '--good-soft'],
        ['--warn', '--warn-soft'],
        ['--bad', '--bad-soft'],
        ['--text', '--bad-soft'],
        ['--text-2', '--hover'],
        ['--text', '--selected'],
    ])('text %s on %s meets 4.5:1 over every glass surface', (fg, fill) => {
        expect(worst(t, fg, fill, glass)).toBeGreaterThanOrEqual(4.5);
    });

    it('input placeholder text (muted, at the opacity the stylesheet gives it) meets 4.5:1 on the field', () => {
        const opacity = Number(css.match(/\.input::placeholder\s*\{[^}]*opacity:\s*([\d.]+)/)?.[1] ?? '1');
        expect(worst(t, '--muted', '--field', glass, opacity)).toBeGreaterThanOrEqual(4.5);
    });

    it('the key hint inside a primary button (inherits the button text) is readable on the accent fill', () => {
        const accent = parseColor(resolve(t, '--accent'));
        expect(ratio(parseColor(resolve(t, '--on-accent')), accent)).toBeGreaterThanOrEqual(4.5);
        // and the stylesheet really makes the hint inherit instead of using the muted colour
        expect(css).toMatch(/\.btn-primary kbd\s*\{[^}]*color:\s*inherit/);
    });

    it('the gold Copy button text is readable on its gold fill', () => {
        expect(ratio(parseColor(resolve(t, '--on-gold')), parseColor(resolve(t, '--isc-gold')))).toBeGreaterThanOrEqual(4.5);
    });

    it.each(['--answer-label', '--answer-text', '--answer-note'])('%s meets 4.5:1 on every stop of the answer slab', (fg) => {
        expect(worst(t, fg, null, sl)).toBeGreaterThanOrEqual(4.5);
    });

    it('the Copy button focus ring is visible (3:1) against the slab it is drawn on, and the button against the slab', () => {
        expect(worst(t, '--answer-focus', null, sl)).toBeGreaterThanOrEqual(3);
        const gold = parseColor(resolve(t, '--isc-gold'));
        expect(Math.min(...sl.map((s) => ratio(gold, s)))).toBeGreaterThanOrEqual(3);
    });

    it('the ordinary focus ring (--field-focus) is visible (3:1) on every glass surface and on a field', () => {
        expect(worst(t, '--field-focus', null, glass)).toBeGreaterThanOrEqual(3);
        expect(worst(t, '--field-focus', '--field', glass)).toBeGreaterThanOrEqual(3);
    });
});
