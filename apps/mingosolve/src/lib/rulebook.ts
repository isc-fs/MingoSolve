// Thin invoke wrappers for src-tauri/src/rulebook.rs, plus the native file picker for the rules PDF.
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { RuleHit, RulebookStatus } from './types';

export function loadRulebook(path: string, year: string): Promise<RulebookStatus> {
    return invoke('load_rulebook', { path, year });
}

export function rulebookStatus(): Promise<RulebookStatus[]> {
    return invoke('rulebook_status');
}

export function searchRules(query: string, year: string, limit: number): Promise<RuleHit[]> {
    return invoke('search_rules', { query, year, limit });
}

export function removeRulebook(year: string): Promise<void> {
    return invoke('remove_rulebook', { year });
}

/** The PDF the user picks, or null when they cancel. */
export async function pickRulebookPdf(): Promise<string | null> {
    const picked = await open({ multiple: false, directory: false, filters: [{ name: 'PDF', extensions: ['pdf'] }] });
    return typeof picked === 'string' ? picked : null;
}
