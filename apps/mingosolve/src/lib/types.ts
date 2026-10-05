// Hand-mirrored Rust DTOs from src-tauri/src/*.rs and the fsq crate (finder::Hit, answer::Matching).
// Keep field names in sync with the serde output.

export interface VarInfo {
    name: string;
    unit: string;
    desc: string;
    signed: boolean;
    default: number | null;
    /** The name typeset (LaTeX). */
    tex: string;
    /** The unit for display (m², Ω). */
    unit_shown: string;
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

export interface SolveResult {
    found: FoundVar[];
    defaults: Shown[];
    conflicts: string[];
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
}

export interface ParamInfo {
    name: string;
    number: boolean;
    default: string | null;
}

export interface ToolInfo {
    name: string;
    doc: string;
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

export interface Found {
    hits: Hit[];
    quantities: string[];
    past: PastMatch | null;
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
