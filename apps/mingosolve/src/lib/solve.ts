// Thin invoke wrappers for src-tauri/src/solve.rs.
import { invoke } from '@tauri-apps/api/core';
import type { ChainResult, FormulaInfo, SolveResult } from './types';

export function listFormulas(): Promise<FormulaInfo[]> {
    return invoke('list_formulas');
}

export function solveFormula(
    key: string,
    given: [string, string][],
    display: Record<string, string>,
): Promise<SolveResult> {
    return invoke('solve_formula', { key, given, display });
}

export function chainFormulas(
    target: string,
    given: [string, string][],
    only: string[],
    display: Record<string, string>,
): Promise<ChainResult> {
    return invoke('chain_formulas', { target, given, only, display });
}

export function calc(expr: string): Promise<string> {
    return invoke('calc', { expr });
}
