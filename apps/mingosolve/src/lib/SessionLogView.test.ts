// The log view: newest first, clearing needs a confirmation, Start a mock quiz tags what follows, and the
// downloaded file is the CSV (BOM, separator chosen by the decimal-comma setting).
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';

import SessionLogView from './SessionLogView.svelte';
import { recordCopy, sessionLog, stopClock } from './sessionlog.svelte';
import { defaultSettings, settings } from './settings.svelte';

const entry = (answer: string) => ({ scriptId: 'uniform_motion', title: 'Uniform motion', inputs: [['s', '100 m']] as [string, string][], answer, rules: null, via: 'button' as const });

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    sessionLog.entries = [];
    sessionLog.quiz = null;
    sessionLog.label = '';
    stopClock();
});

it('lists the newest entry first', () => {
    recordCopy(entry('first'));
    recordCopy(entry('second'));
    render(SessionLogView);
    expect([...document.querySelectorAll('tbody td.answer')].map((c) => c.textContent)).toEqual(['second', 'first']);
});

it('Clear log asks first; Cancel keeps everything, confirming empties it', async () => {
    recordCopy(entry('12.5'));
    render(SessionLogView);
    await userEvent.click(screen.getByRole('button', { name: 'Clear log' }));
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(sessionLog.entries).toHaveLength(1);
    await userEvent.click(screen.getByRole('button', { name: 'Clear log' }));
    expect(sessionLog.entries).toHaveLength(1);
    await userEvent.click(screen.getByRole('button', { name: 'Yes, clear the log' }));
    expect(sessionLog.entries).toHaveLength(0);
    expect(screen.getByText(/Nothing yet/)).toBeTruthy();
});

it('Start a mock quiz tags the next copy with its id and shows it as running', async () => {
    render(SessionLogView);
    await userEvent.click(screen.getByRole('button', { name: 'Start a mock quiz' }));
    expect(sessionLog.quiz).not.toBeNull();
    recordCopy(entry('12.5'));
    expect(sessionLog.entries[0].quiz).toBe(sessionLog.quiz!.id);
    expect(document.body.textContent).toContain(sessionLog.quiz!.id);
    await userEvent.click(screen.getByRole('button', { name: 'End mock quiz' }));
    expect(sessionLog.quiz).toBeNull();
});

async function exported(): Promise<{ text: string; name: string }> {
    let blob: Blob | null = null;
    let name = '';
    URL.createObjectURL = (b: Blob | MediaSource) => {
        blob = b as Blob;
        return 'blob:test';
    };
    URL.revokeObjectURL = () => {};
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (this: HTMLAnchorElement) {
        name = this.download;
    });
    await userEvent.click(screen.getByRole('button', { name: 'Export CSV' }));
    click.mockRestore();
    await waitFor(() => expect(blob).not.toBeNull());
    const bytes = await new Promise<Uint8Array>((resolve) => {
        const r = new FileReader();
        r.onload = () => resolve(new Uint8Array(r.result as ArrayBuffer));
        r.readAsArrayBuffer(blob!);
    });
    expect([...bytes.slice(0, 3)]).toEqual([0xef, 0xbb, 0xbf]);
    return { text: new TextDecoder('utf-8', { ignoreBOM: true }).decode(bytes), name };
}

it('Export CSV downloads a .csv with a byte order mark, comma separated by default', async () => {
    recordCopy(entry('12.5'));
    render(SessionLogView);
    const { text, name } = await exported();
    expect(name).toMatch(/^mingosolve-session-\d{8}-\d{4}\.csv$/);
    expect(text.charCodeAt(0)).toBe(0xfeff);
    expect(text.split('\r\n')[0]).toBe('﻿timestamp,mock_quiz,question,script_id,script_title,inputs,answer,rule_year,elapsed_s,method');
});

it('with decimal commas on, the export separates with semicolons so a decimal-comma Excel opens it in columns', async () => {
    settings.decimalComma = true;
    recordCopy(entry('77,9'));
    render(SessionLogView);
    const { text } = await exported();
    const [head, row] = text.split('\r\n');
    expect(head.split(';')).toHaveLength(10);
    expect(row.split(';')[6]).toBe('77,9');
});
