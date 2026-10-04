// Thin invoke wrappers for src-tauri/src/rules.rs.
import { invoke } from '@tauri-apps/api/core';
import type { RuleSet } from './types';

export function ruleSets(): Promise<Record<string, RuleSet>> {
    return invoke('rule_sets');
}

export function rulesChanges(): Promise<string> {
    return invoke('rules_changes');
}
