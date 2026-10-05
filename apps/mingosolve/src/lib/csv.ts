// Session-log CSV (RFC 4180): CRLF rows, fields with the delimiter, quotes or line breaks wrapped in quotes with
// quotes doubled, a UTF-8 byte order mark so Excel reads µ, Ω and ° correctly. The columns are fixed.
import type { LogEntry } from './sessionlog.svelte';

export const LOG_COLUMNS = ['timestamp', 'mock_quiz', 'question', 'script_id', 'script_title', 'inputs', 'answer', 'rule_year', 'elapsed_s', 'method'] as const;

export type Delimiter = ',' | ';';

export function csvField(value: string, delimiter: Delimiter): string {
    return value.includes(delimiter) || /["\r\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
}

/** "name=value; name=value", values exactly as typed (units included). */
export function inputsCell(inputs: [string, string][]): string {
    return inputs.map(([k, v]) => `${k}=${v}`).join('; ');
}

export function entriesToCsv(entries: LogEntry[], delimiter: Delimiter = ','): string {
    const rows = entries.map((e) => [
        e.at,
        e.quiz ?? '',
        e.question,
        e.scriptId,
        e.title,
        inputsCell(e.inputs),
        e.answer,
        e.rules ?? '',
        e.elapsedS === null ? '' : String(e.elapsedS),
        e.via,
    ]);
    const lines = [[...LOG_COLUMNS], ...rows].map((r) => r.map((f) => csvField(f, delimiter)).join(delimiter));
    return '﻿' + lines.join('\r\n') + '\r\n';
}
