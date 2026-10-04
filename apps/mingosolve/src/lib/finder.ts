// Thin invoke wrappers for src-tauri/src/finder.rs.
import { invoke } from '@tauri-apps/api/core';
import type { ExampleInfo, Hit, Matching, Precision } from './types';

export function findQuestion(text: string): Promise<Hit[]> {
    return invoke('find_question', { text });
}

export function matchOptions(value: number, options: string): Promise<Matching> {
    return invoke('match_options', { value, options });
}

export function formatAnswer(value: number, precision: Precision, decimalComma: boolean): Promise<string> {
    return invoke('format_answer', { value, precision, decimalComma });
}

export function listExamples(): Promise<ExampleInfo[]> {
    return invoke('list_examples');
}
