// Hand-mirrored Rust DTOs from src-tauri/src/*.rs and the fsq crate (finder::Hit, answer::Matching).
// Keep field names in sync with the serde output.

export interface VarInfo {
    name: string;
    unit: string;
    desc: string;
    signed: boolean;
    default: number | null;
}

export interface FormulaInfo {
    key: string;
    title: string;
    eqs: string[];
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

export interface ExampleInfo {
    id: number;
    what: string;
    cmd: string;
    answer: number;
    warning: string | null;
}

/** Parsed TOML rule set: nested tables of numbers, strings and arrays. */
export type RuleSet = Record<string, unknown>;
