// Engine errors in plain words. The engine's messages name quantities by symbol or dimension ("expected m (m)",
// "cannot show v_avg (m/s) in kg"); here they become sentences that name the field the person typed in.
// Anything not recognised is shown as the engine wrote it.

export interface ErrorField {
    name: string;
    /** What the field is called on screen. */
    label: string;
    /** What is typed in it now. */
    value: string;
}

export interface ErrorContext {
    fields?: ErrorField[];
    /** Plain name of a variable the engine mentions by symbol. */
    nameOf?: (variable: string) => string | undefined;
}

/** The engine's dimension strings (units::dims_str) for common kinds: what to call them and a valid example. */
const KINDS: Record<string, [string, string]> = {
    m: ['a length', '3 m or 250 mm'],
    kg: ['a mass', '240 kg'],
    s: ['a time', '4 s or 250 ms'],
    A: ['a current', '12 A'],
    K: ['a temperature', '25 degC or 300 K'],
    'm·s^-1': ['a speed', '15 m/s or 54 km/h'],
    'm·s^-2': ['an acceleration', '9.81 m/s**2 or 1.5 g0'],
    'm^2': ['an area', '2 m**2 or 300 cm**2'],
    'm^3': ['a volume', '600 cm**3 or 0.6 L'],
    'm·kg·s^-2': ['a force', '500 N or 2 kN'],
    'm^2·kg·s^-3': ['a power', '5 kW'],
    'm^2·kg·s^-2': ['an energy', '3 kJ or 1 kWh'],
    'm^-1·kg·s^-2': ['a pressure', '2 bar or 200 kPa'],
    'm^2·kg·s^-3·A^-1': ['a voltage', '400 V'],
    'm^2·kg·s^-3·A^-2': ['a resistance', '0.08 ohm or 80 mohm'],
    's^-1': ['a frequency or rotation speed', '50 Hz or 3000 rpm'],
    's·A': ['a charge', '20 Ah'],
    'm^-3·kg': ['a density', '1.2 kg/m**3'],
    'kg·s^-2': ['a stiffness (force per length)', '20 kN/m'],
    'm^-3·kg^-1·s^3·A^2': ['a conductivity', '56e6 S/m'],
    'm·kg·s^-3·K^-1': ['a thermal conductivity', '0.2 W/(m*K)'],
    dimensionless: ['a plain number', '0.5'],
};

function kind(dims: string, unit: string): [string, string] {
    return KINDS[dims] ?? [`a value in ${unit}`, `1 ${unit}`];
}

const cap = (s: string): string => s.charAt(0).toUpperCase() + s.slice(1);

type Say = (m: RegExpMatchArray, c: Ctx) => string;

interface Ctx {
    fields: ErrorField[];
    /** The field the error names (a "param: " prefix), if any. */
    named: ErrorField | undefined;
    /** The field to blame: the named one, or the only field satisfying `by`. */
    find: (by: (value: string) => boolean) => ErrorField | undefined;
    nameOf: (variable: string) => string | undefined;
}

const label = (f: ErrorField): string => cap(f.label);
const about = (f: ErrorField | undefined, text: string): string => (f === undefined ? text : `${label(f)}: ${text}`);

const RULES: [RegExp, Say][] = [
    [
        /^missing (\w+)$/,
        (m, c) => {
            const f = c.fields.find((x) => x.name === m[1]);
            return `${f === undefined ? m[1] : label(f)} is empty: it needs a value.`;
        },
    ],
    [
        /^(.+) is (.+), expected (.+) \((.+)\)$/,
        (m, c) => {
            const f = c.named ?? c.find((v) => v.trim() === m[1].trim());
            const [noun, example] = kind(m[4], m[3]);
            const theirs = KINDS[m[2]]?.[0] ?? m[2];
            return f === undefined
                ? `“${m[1]}” isn't ${noun}. Use something like ${example}.`
                : `${label(f)} needs ${noun}, like ${example}. “${m[1]}” is ${theirs} instead.`;
        },
    ],
    [
        /^unknown unit "(.*)"$/,
        (m, c) => about(c.named ?? c.find((v) => v.includes(m[1])), `“${m[1]}” isn't a unit we know. Try symbols like m, kg, s, km/h or kW.`),
    ],
    [
        /^cannot show (\w+) \((.+)\) in (.+)$/,
        (m, c) => `${cap(c.nameOf(m[1]) ?? m[1])} is measured in ${m[2]}, so it can't be shown in “${m[3]}”. Pick a unit of the same kind, like ${m[2]}.`,
    ],
    [
        /^unexpected end of expression$/,
        (_, c) => {
            const f = c.find((v) => /[-+*/(^]\s*$/.test(v));
            return f === undefined
                ? 'A value looks unfinished, like “3 +” or “5 /”. Check the fields for a sign at the end.'
                : `${label(f)} looks unfinished: “${f.value.trim()}” stops where a number or unit should follow.`;
        },
    ],
    [/^use \*\* for powers/, (_, c) => about(c.named ?? c.find((v) => v.includes('^')), 'write powers with **, like 2**3, not ^.')],
    [/^fractional power of a unit$/, () => 'Units can only be raised to whole powers: m**2, not m**0.5.'],
    [
        /^cannot add (.+) and (.+)$/,
        (m) => `Can't add or subtract ${kind(m[1], m[1])[0]} and ${kind(m[2], m[2])[0]}: they are different kinds of quantity. Check the units.`,
    ],
    [/^unexpected character '(.)'$/, (m, c) => about(c.named ?? c.find((v) => v.includes(m[1])), `“${m[1]}” isn't allowed in a value.`)],
    [/^number (\S+) is too large$/, (m, c) => about(c.named ?? c.find((v) => v.includes(m[1])), `the number ${m[1]} is too large for the solver.`)],
    [/^bad number (.+)$/, (m, c) => about(c.named ?? c.find((v) => v.includes(m[1])), `“${m[1]}” isn't a number.`)],
    [/^unknown function (\w+)\(\)$/, (m) => `“${m[1]}()” isn't a function we know.`],
    [/^missing \)$/, () => 'A value has an opening bracket without its closing one.'],
    [/^unexpected (?!end of|character)(.+)$/, (m) => `A value has something unexpected in it (“${m[1]}”). Look for a missing sign or a stray symbol.`],
    [/^(\w+) has no variable (\w+); it uses/, (m) => `This script has no field called ${m[2]}.`],
    [/^no finite answer/, () => 'These values give no finite answer. Look for a zero where something is divided, such as a time or a step of 0.'],
    [/^value must be positive$/, () => 'The value must be greater than zero.'],
    [/^internal error/, () => 'Something went wrong inside the solver. Try again, and tell the team if it keeps happening.'],
];

/** Turn an engine error (as caught: a string or an Error) into a sentence; unknown messages pass through. */
export function friendlyError(error: unknown, context: ErrorContext = {}): string {
    const raw = String(error).replace(/^Error: /, '');
    const fields = context.fields ?? [];
    // tools prefix their errors with the parameter ("t_team: unknown unit ..."), or with their own name
    const pre = raw.match(/^(\w+): ([\s\S]*)$/);
    const named = pre === null ? undefined : fields.find((f) => f.name === pre[1]);
    const text = pre === null ? raw : pre[2];
    const c: Ctx = {
        fields,
        named,
        find: (by) => {
            const hits = fields.filter((f) => by(f.value));
            return hits.length === 1 ? hits[0] : undefined;
        },
        nameOf: context.nameOf ?? (() => undefined),
    };
    for (const [re, say] of RULES) {
        const m = text.match(re) ?? raw.match(re);
        if (m !== null) return say(m, c);
    }
    return raw;
}
