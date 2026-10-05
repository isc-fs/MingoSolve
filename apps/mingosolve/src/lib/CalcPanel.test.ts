// The calculator panel against a fake engine: past questions, copy feedback and error display. That each past
// question's expression evaluates to its official answer is checked against the real engine in ipc_tests.rs.
import { parse } from 'smol-toml';
import examplesToml from '../../../../data/examples.toml?raw';
import { render, screen, waitFor, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import CalcPanel from './CalcPanel.svelte';
import { session } from './session.svelte';
import { defaultSettings, settings } from './settings.svelte';
import { fakeEngine } from '../test/ipc';
import type { WorkedExample } from './types';

const calcRows: WorkedExample[] = (parse(examplesToml) as unknown as { example: (WorkedExample & { what?: string })[] }).example
    .filter((e) => e.cmd.startsWith('calc '))
    .map((e) => ({ id: e.id, what: e.what ?? '', cmd: e.cmd, answer: e.answer }));

let copies: string[];

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    settings.calcOpen = true;
    session.calcInput = '';
    copies = [];
    Object.defineProperty(navigator, 'clipboard', { value: { writeText: async (t: string) => void copies.push(t) }, configurable: true });
});

describe('past questions', () => {
    it('clicking one loads its expression and evaluates it', async () => {
        const engine = fakeEngine({ calc_examples: () => calcRows, calc: () => '3.54 kW' });
        render(CalcPanel);
        await userEvent.click(screen.getByText('Past questions'));
        const q = calcRows[0];
        await userEvent.click(await screen.findByRole('button', { name: new RegExp(`^Q${q.id}\\b`) }));
        const expr = q.cmd.slice('calc '.length);
        expect((screen.getByRole('textbox', { name: 'Expression' }) as HTMLInputElement).value).toBe(expr);
        await waitFor(() => expect(engine.calls.filter((c) => c.cmd === 'calc')).toHaveLength(1));
        expect(engine.calls.find((c) => c.cmd === 'calc')!.args).toEqual({ expr });
        expect(await screen.findByText('3.54 kW')).toBeTruthy();
        expect(calcRows.length).toBeGreaterThan(5);
    });

    it('is a fold closed by default, with the state announced and remembered', async () => {
        fakeEngine({ calc_examples: () => calcRows });
        const first = render(CalcPanel);
        const head = screen.getByRole('button', { name: 'Past questions' });
        expect(head.getAttribute('aria-expanded')).toBe('false');
        await userEvent.click(head);
        expect(head.getAttribute('aria-expanded')).toBe('true');
        expect(settings.calcPastOpen).toBe(true);
        first.unmount();
        render(CalcPanel);
        expect(screen.getByRole('button', { name: 'Past questions' }).getAttribute('aria-expanded')).toBe('true');
        expect(await screen.findByRole('button', { name: new RegExp(`^Q${calcRows[0].id}\\b`) })).toBeTruthy();
    });

    it('does not ask the engine for the list until the disclosure is opened', () => {
        const engine = fakeEngine({ calc_examples: () => calcRows });
        render(CalcPanel);
        expect(engine.calls).toHaveLength(0);
    });
});

describe('results', () => {
    it('copying an entry says Copied (visibly and to a screen reader) and copies the number only', async () => {
        fakeEngine({ calc: () => '12.5 kW' });
        render(CalcPanel);
        const box = screen.getByRole('textbox', { name: 'Expression' });
        await userEvent.type(box, '2*6.25kW{Enter}');
        const entry = await screen.findByRole('button', { name: /12\.5 kW/ });
        await userEvent.click(entry);
        await waitFor(() => expect(within(entry).getByText('Copied')).toBeTruthy());
        expect(copies).toEqual(['12.5']);
        expect(document.querySelector('[aria-live="polite"]')!.textContent).toBe('Copied');
    });

    it('Copied goes away again', async () => {
        vi.useFakeTimers({ shouldAdvanceTime: true });
        try {
            fakeEngine({ calc: () => '1 m' });
            render(CalcPanel);
            await userEvent.type(screen.getByRole('textbox', { name: 'Expression' }), '1m{Enter}');
            await userEvent.click(await screen.findByRole('button', { name: /1 m/ }));
            await waitFor(() => expect(document.querySelector('.copied')).not.toBeNull());
            vi.advanceTimersByTime(2000);
            await waitFor(() => expect(document.querySelector('.copied')).toBeNull());
            expect(document.querySelector('[aria-live="polite"]')!.textContent).toBe('');
        } finally {
            vi.useRealTimers();
        }
    });

    it('an engine error reads as a message, without the "error:" prefix, and is not offered for copying', async () => {
        fakeEngine({ calc: () => 'error: result is length, not kg' });
        render(CalcPanel);
        await userEvent.type(screen.getByRole('textbox', { name: 'Expression' }), '3m -> kg{Enter}');
        expect(await screen.findByText('result is length, not kg')).toBeTruthy();
        expect(screen.queryByText(/error:/)).toBeNull();
        expect(screen.queryByRole('button', { name: /result is length/ })).toBeNull();
    });
});
