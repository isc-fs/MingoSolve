// WCAG contrast of the token pairs the interface actually draws, computed from the real custom properties in
// app.css and src/styles/*.css for every style in both modes (nothing is hard-coded here): text needs 4.5:1
// (1.4.3), focus rings and UI boundaries 3:1 (1.4.11).
// Glass is translucent, so every pair is checked over the worst case behind it: the opaque [data-solid] surface,
// the glass over the bare ground, and the glass over each glow of the lit backdrop.
import { describe, expect, it } from 'vitest';

import css from './app.css?raw';
import { STYLES } from './lib/styles';

const styleFiles = Object.fromEntries(
    Object.entries(import.meta.glob<string>('./styles/*.css', { query: '?raw', import: 'default', eager: true })).map(([f, text]) => [
        f.replace('./styles/', '').replace('.css', ''),
        text,
    ]),
);

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

/** Whether a token selector (`:root`, `[data-style]`, `[data-style='x'][data-theme='y']`...) applies to the root of a
 *  window in this style and mode; any other selector (`:root[data-solid] .glass`) is not a token block. */
function applies(selector: string, style: string, theme: string): boolean {
    const m = selector.match(/^(?::root)?((?:\[data-(?:style|theme)(?:='[\w-]+')?\])*)$/);
    if (m === null) return false;
    for (const [, attr, value] of m[1].matchAll(/\[data-(style|theme)(?:='([\w-]+)')?\]/g)) {
        if (value !== undefined && value !== (attr === 'style' ? style : theme)) return false;
    }
    return true;
}

/** The custom properties in force for a style and theme: app.css, then the style files, in source order. */
function tokens(style: string, theme: 'dark' | 'light'): Tokens {
    const all: Tokens = {};
    for (const source of [css, ...Object.values(styleFiles)]) {
        for (const b of parseBlocks(source)) {
            if (b.selectors.some((sel) => applies(sel, style, theme))) Object.assign(all, b.decls);
        }
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

const cases = STYLES.flatMap((s) => (['dark', 'light'] as const).map((theme) => [s.id, theme] as const));

it('every style file is registered and read here, and every registered style other than ISC has a file', () => {
    for (const text of Object.values(styleFiles)) expect(text.length).toBeGreaterThan(0);
    expect(Object.keys(styleFiles).sort()).toEqual(STYLES.filter((s) => s.id !== 'isc').map((s) => s.id).sort());
});

describe.each(cases)('%s style, %s mode', (style, theme) => {
    const t = tokens(style, theme);
    const glass = surfaces(t);
    const sl = slab(t);

    it('reads the real tokens (a typo in a name would throw, not pass)', () => {
        expect(resolve(t, '--text').length).toBeGreaterThan(0);
        expect(sl.length).toBeGreaterThanOrEqual(1);
        // the opaque surface, then glass and strong glass over the ground (and over each glow, when the style has any)
        expect(glass.length).toBeGreaterThanOrEqual(3);
    });

    it.each([
        ['--text', null],
        ['--heading', null],
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
        ['--muted', '--selected'],
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

    it('the answer slab is an image layer (a plain colour before the last background layer voids the whole rule)', () => {
        expect(resolve(t, '--answer-bg')).toMatch(/gradient\(/);
    });

    it('the unit field inside the slab: its text and placeholder read on it', () => {
        const field = (on: Rgba) => over(parseColor(resolve(t, '--slab-field')), on);
        const opacity = Number(css.match(/\.input::placeholder\s*\{[^}]*opacity:\s*([\d.]+)/)?.[1] ?? '1');
        for (const s of sl) {
            const f = field(s);
            const muted = parseColor(resolve(t, '--slab-field-muted'));
            expect(ratio(over(parseColor(resolve(t, '--slab-field-text')), f), f)).toBeGreaterThanOrEqual(4.5);
            expect(ratio(over({ ...muted, a: muted.a * opacity }, f), f)).toBeGreaterThanOrEqual(4.5);
        }
    });

    it('slab shadows are colours, never an image token (an image in box-shadow voids the whole shadow)', () => {
        for (const name of ['--slab-ring', '--slab-shadow', '--shadow']) expect(resolve(t, name)).not.toMatch(/gradient\(/);
    });

    it('the answer figures meet 4.5:1 on their plate (the bib, the flap) or on the slab itself', () => {
        const plate = resolve(t, '--value-bg');
        const on = plate === 'none' ? sl : sl.flatMap((s) => colorsIn(plate).map((c) => over(c, s)));
        expect(worst(t, '--value-text', null, on)).toBeGreaterThanOrEqual(4.5);
    });

    it('the answer label meets 4.5:1 on its cell (the datasheet symbol cell) or on the slab itself', () => {
        const cell = resolve(t, '--slab-cell');
        const on = cell === 'none' ? sl : sl.flatMap((s) => colorsIn(cell).map((c) => over(c, s)));
        expect(worst(t, '--answer-label', null, on)).toBeGreaterThanOrEqual(4.5);
    });

    it('the Copy button text is readable on its fill', () => {
        expect(ratio(parseColor(resolve(t, '--copy-text')), parseColor(resolve(t, '--copy-bg')))).toBeGreaterThanOrEqual(4.5);
    });

    it.each(['--answer-label', '--answer-text', '--answer-note'])('%s meets 4.5:1 on every stop of the answer slab', (fg) => {
        expect(worst(t, fg, null, sl)).toBeGreaterThanOrEqual(4.5);
    });

    it('the Copy button focus ring is visible (3:1) against the slab it is drawn on, and the button against the slab', () => {
        expect(worst(t, '--answer-focus', null, sl)).toBeGreaterThanOrEqual(3);
        const copy = parseColor(resolve(t, '--copy-bg'));
        expect(Math.min(...sl.map((s) => ratio(copy, s)))).toBeGreaterThanOrEqual(3);
    });

    it('the ordinary focus ring (--field-focus) is visible (3:1) on every glass surface and on a field', () => {
        expect(worst(t, '--field-focus', null, glass)).toBeGreaterThanOrEqual(3);
        expect(worst(t, '--field-focus', '--field', glass)).toBeGreaterThanOrEqual(3);
    });
});
