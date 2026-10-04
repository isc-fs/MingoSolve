// Thin invoke wrappers for src-tauri/src/finder.rs.
import { invoke } from '@tauri-apps/api/core';
import type { Found, Matching, Precision } from './types';

export function findQuestion(text: string): Promise<Found> {
    return invoke('find_question', { text });
}

/** `answer` is the shown answer with its unit ("77.887 A"), so options written in other units can be compared. */
export function matchOptions(answer: string, options: string): Promise<Matching> {
    return invoke('match_options', { answer, options });
}

export function formatAnswer(value: number, precision: Precision, decimalComma: boolean): Promise<string> {
    return invoke('format_answer', { value, precision, decimalComma });
}
