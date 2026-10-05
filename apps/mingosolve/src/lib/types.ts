// Hand-mirrored Rust DTOs from src-tauri/src/*.rs and the fsq crate (finder::Hit, answer::Matching).
// Keep field names in sync with the serde output.

export interface VarInfo {
    name: string;
    unit: string;
    desc: string;
    /** How to type the value, shown under the field (`hint` in data/formulas). */
    hint: string | null;
    signed: boolean;
    default: number | null;
    /** The name typeset (LaTeX). */
    tex: string;
    /** The unit for display (m², Ω). */
    unit_shown: string;
    /** What the unit measures (engine `dims_key`); equal to a FormatHint's `dims` when the question's unit fits. */
    dims: string;
}

export interface FormulaInfo {
    key: string;
    title: string;
    eqs: string[];
    /** The equations typeset (LaTeX), same order as eqs. */
    tex: string[];
    tags: string[];
    notes: string;
    vars: VarInfo[];
}

export interface FoundVar {
    name: string;
    desc: string;
    values: number[];
    shown: string[];
}

export interface Shown {
    name: string;
    shown: string;
}

/** A typed name the engine read as another: `v_i` as `v0` (initial velocity). */
export interface NameMap {
    from: string;
    to: string;
    desc: string;
}

export interface NameChoice {
    name: string;
    desc: string;
}

/** A typed name the engine could not use: unknown (with suggestions) or ambiguous (with its possible meanings). */
export interface NameProblem {
    name: string;
    kind: 'unknown' | 'ambiguous';
    choices: NameChoice[];
}

export interface SolveResult {
    found: FoundVar[];
    defaults: Shown[];
    conflicts: string[];
    mapped: NameMap[];
}

export interface ChainStep {
    formula: string;
    var: string;
    shown: string;
}

export interface ChainResult {
    steps: ChainStep[];
    defaults: Shown[];
    reached: boolean;
    target: string | null;
    known: string[];
    conflicts: string[];
    /** Typed names (target included) the engine read as other names. */
    mapped: NameMap[];
    /** Names it could not use; when there are any, nothing was solved. */
    problems: NameProblem[];
}

export interface Choice {
    value: string;
    label: string;
}

export interface ParamInfo {
    name: string;
    number: boolean;
    default: string | null;
    /** Plain-English name (data/tool_params.toml). */
    label: string;
    /** Unit a bare number is read in, shown next to the input. */
    unit: string | null;
    help: string | null;
    /** A select with these values (blank = leave to the tool's default). */
    choices: Choice[] | null;
    /** An on/off checkbox, sent as 1 or 0. */
    switch: boolean;
}

export interface ToolInfo {
    name: string;
    /** The technical description, kept behind "Details". */
    doc: string;
    /** One plain sentence. */
    summary: string;
    params: ParamInfo[];
}

export interface Hit {
    kind: 'formula' | 'tool' | 'example';
    id: string;
    title: string;
    score: number;
    prefill: [string, string][];
    /** Variable the question asks for, when the finder could tell. */
    target: string | null;
    warning: string | null;
}

export interface OptionMatch {
    text: string;
    value: number | null;
    rel_diff: number | null;
    best: boolean;
}

export interface Matching {
    options: OptionMatch[];
    warning: string | null;
}

export type Precision = { sig: number } | { decimals: number };

export interface PastExample {
    cmd: string;
    answer: number;
}

/** A pasted question recognised as one of the FS-Quiz bank (fingerprint match, no question text shipped). */
export interface PastMatch {
    id: number;
    /** Quizzes it appeared in, oldest first ("FSG 2023 EV"). */
    quizzes: string[];
    /** Official answer when it is a number with a unit. */
    answer: string | null;
    similarity: number;
    runner_up: number;
    /** Weak match or another question fits as well: say "probably". */
    probable: boolean;
    /** The numbers match the bank question, so the official answer applies as is. */
    same_numbers: boolean;
    example: PastExample | null;
    /** Why the official key is known to be wrong, when it is. */
    known_key: string | null;
}

/** How a pasted question wants its answer (engine format_hint::FormatHint). */
export interface FormatHint {
    rounding: Precision | null;
    /** Display unit in the engine's syntax ("km/h", "N*m"). */
    unit: string | null;
    /** The unit as the question wrote it ("kilonewtons"). */
    unit_label: string | null;
    dims: string | null;
    /** The asked quantity when named next to the unit ("average lap speed"). */
    quantity: string | null;
}

export interface Found {
    hits: Hit[];
    quantities: string[];
    past: PastMatch | null;
    format: FormatHint | null;
}

export interface ScriptRef {
    id: string;
    kind: 'formula' | 'tool';
    title: string;
    /** Extra search words (data/topics.toml [aliases]). */
    aliases: string;
}

export interface TopicInfo {
    id: string;
    name: string;
    blurb: string;
    hue: 'green' | 'teal' | 'gold';
    scripts: ScriptRef[];
}

export interface WorkedExample {
    id: number;
    what: string;
    cmd: string;
    answer: number;
}

export interface RulebookStatus {
    year: string;
    /** File name of the PDF it was loaded from. */
    source: string;
    /** Seconds since the Unix epoch. */
    loaded_at: number;
    pages: number;
    entries: number;
}

export interface RuleHit {
    year: string;
    id: string;
    title: string;
    page: number;
    snippet: string;
    /** `[start, end)` of matched words in `snippet`, in UTF-16 units. */
    matches: [number, number][];
}
