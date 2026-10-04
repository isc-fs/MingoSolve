// Thin invoke wrappers for src-tauri/src/finder.rs.
import { invoke } from '@tauri-apps/api/core';
import type { Found, Matching, Precision } from './types';

export function findQuestion(text: string): Promise<Found> {
    return invoke('find_question', { text });
}

export function matchOptions(value: number, options: string): Promise<Matching> {
    return invoke('match_options', { value, options });
}

export function formatAnswer(value: number, precision: Precision, decimalComma: boolean): Promise<string> {
    return invoke('format_answer', { value, precision, decimalComma });
}
