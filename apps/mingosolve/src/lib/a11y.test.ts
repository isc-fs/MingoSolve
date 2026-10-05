// Accessibility behaviour that jsdom can check: solved values as visible text, what is announced and how, the
// palette's focus trap, arrow keys in radio groups, and the narrow-window layout. Colour contrast is in
// ../contrast.test.ts; real focus rings, zoom and layout at several widths are in e2e/accessibility.spec.ts.
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import CalcPanel from './CalcPanel.svelte';
import Palette from './Palette.svelte';
import Rail from './Rail.svelte';
import ScriptSheet from './ScriptSheet.svelte';
import SettingsView from './SettingsView.svelte';
import { catalog } from './catalog.svelte';
import { layout } from './layout.svelte';
import { openScript, session } from './session.svelte';
import { defaultSettings, settings } from './settings.svelte';
import { fakeEngine } from '../test/ipc';
import { score } from '../test/tools';
import type { FormulaInfo, SolveResult } from './types';

const motion: FormulaInfo = {
    key: 'uniform_motion',
    title: 'Uniform motion',
    eqs: ['v = s/t'],
    tex: ['v = \\frac{s}{t}'],
    tags: [],
    notes: '',
    vars: [
        ['s', 'm'],
        ['t', 's'],
        ['v', 'm/s'],
    ].map(([name, unit]) => ({ name, unit, desc: name, signed: false, hint: null, default: null, tex: name, unit_shown: '', dims: '' })),
};

const common = {
    script_examples: () => [],
    format_answer: (a: Record<string, unknown>) => String(a.value),
    match_options: () => ({ options: [], warning: null }),
};

/** v = s/t as the engine would answer. */
function solveMotion(args: Record<string, unknown>): SolveResult {
    const g = Object.fromEntries(args.given as [string, string][]);
    const v = parseFloat(g.s) / parseFloat(g.t);
    return { found: Number.isFinite(v) ? [{ name: 'v', desc: 'v', values: [v], shown: [`${v} m/s`] }] : [], defaults: [], conflicts: [] };
}

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    catalog.scripts = new Map([
        ['uniform_motion', { id: 'uniform_motion', kind: 'formula', title: motion.title, aliases: '', topic: null, formula: motion }],
        ['event_score', { id: 'event_score', kind: 'tool', title: 'Event score', aliases: '', topic: null, tool: score }],
    ]);
    session.sheet = null;
    session.script = null;
    session.paletteOpen = false;
    session.activeView = 'solve';
    layout.narrow = false;
    layout.calcPeek = false;
});

afterEach(() => cleanup());

const field = (name: string) => document.querySelector<HTMLInputElement>(`label.field[data-var="${name}"] input`)!;

describe('solved values in a formula sheet', () => {
    it('shows a solved value as visible text with its unit next to the field, not only as a placeholder', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        openScript('uniform_motion', [['s', '100 m'], ['t', '8 s']]);
        render(ScriptSheet);
        // hand derivation: 100 m / 8 s = 12.5 m/s
        await waitFor(() => expect(document.querySelector('[data-solved="v"]')?.textContent).toContain('= 12.5 m/s'));
        expect(field('v').value).toBe('');
        expect(field('v').placeholder).not.toContain('12.5');
        // fields that are not solved carry no result text
        expect(document.querySelector('[data-solved="s"]')).toBeNull();
    });

    it('stops showing the result once the user types a value of their own in that field', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        openScript('uniform_motion', [['s', '100 m'], ['t', '8 s']]);
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('[data-solved="v"]')).not.toBeNull());
        await fireEvent.input(field('v'), { target: { value: '12.5 m/s' } });
        expect(document.querySelector('[data-solved="v"]')).toBeNull();
    });
});

describe('announcements in the sheet', () => {
    it('has a polite status region from the start that then contains the answer', async () => {
        fakeEngine({ ...common, solve_formula: solveMotion });
        openScript('uniform_motion');
        render(ScriptSheet);
        const live = document.querySelector<HTMLElement>('[data-live="answer"]')!;
        expect(live.getAttribute('role')).toBe('status');
        expect(live.textContent).toBe('');
        await fireEvent.input(field('s'), { target: { value: '100 m' } });
        await fireEvent.input(field('t'), { target: { value: '8 s' } });
        await waitFor(() => expect(live.textContent).toBe('v = 12.5 m/s'));
    });

    it('reads a formula error in a polite status region (it appears while the user types), not an alert', async () => {
        fakeEngine({
            ...common,
            solve_formula: () => {
                throw 'boom';
            },
        });
        openScript('uniform_motion', [['s', '100 m']]);
        render(ScriptSheet);
        await waitFor(() => expect(document.querySelector('.note-bad')).not.toBeNull());
        const region = document.querySelector('.note-bad')!.closest('[role="status"]');
        expect(region).not.toBeNull();
        expect(screen.queryByRole('alert')).toBeNull();
    });

    it('reports a failed tool run (an explicit Run) as an alert', async () => {
        fakeEngine({
            ...common,
            run_tool: () => {
                throw 'event_score: missing t_team';
            },
        });
        openScript('event_score');
        render(ScriptSheet);
        await userEvent.click(screen.getByRole('button', { name: /^Run/ }));
        const alert = await screen.findByRole('alert');
        expect(alert.textContent).toContain("Can't solve that");
    });
});

describe('palette', () => {
    it('keeps Tab and Shift+Tab inside the dialog and announces problem mode at six words', async () => {
        fakeEngine({});
        render(Palette);
        session.paletteOpen = true;
        const box = await screen.findByRole('combobox', { name: 'Search scripts' });
        await waitFor(() => expect(document.activeElement).toBe(box));
        const status = document.querySelector('p[role="status"]')!;
        await userEvent.type(box, 'one two three four five');
        expect(status.textContent).not.toContain('problem');
        await userEvent.type(box, ' six');
        await waitFor(() => expect(status.textContent).toContain('Searching as a problem'));

        const go = screen.getByRole('button', { name: /Find the scripts for this problem/ });
        // two stops: the box and the button; Tab from the last returns to the first, Shift+Tab from the first to the last
        await userEvent.tab();
        expect(document.activeElement).toBe(go);
        await userEvent.tab();
        expect(document.activeElement).toBe(box);
        await userEvent.tab({ shift: true });
        expect(document.activeElement).toBe(go);
    });

    it('Escape closes it and puts focus back on what opened it', async () => {
        fakeEngine({});
        const opener = document.body.appendChild(document.createElement('button'));
        opener.focus();
        render(Palette);
        session.paletteOpen = true;
        await waitFor(() => expect(screen.getByRole('combobox', { name: 'Search scripts' })).toBe(document.activeElement));
        await userEvent.keyboard('{Escape}');
        await waitFor(() => expect(session.paletteOpen).toBe(false));
        expect(document.activeElement).toBe(opener);
        opener.remove();
    });
});

describe('theme and text size radio groups', () => {
    it('are one Tab stop and move with the arrow keys, selecting as they go', async () => {
        render(SettingsView, { update: null });
        const theme = screen.getByRole('radiogroup', { name: 'Theme' });
        const radios = () => [...theme.querySelectorAll<HTMLElement>('[role="radio"]')];
        expect(radios().map((r) => r.tabIndex)).toEqual([0, -1, -1]);
        radios()[0].focus();
        await userEvent.keyboard('{ArrowRight}');
        expect(settings.theme).toBe('dark');
        expect(document.activeElement).toBe(radios()[1]);
        expect(radios().map((r) => r.tabIndex)).toEqual([-1, 0, -1]);
        await userEvent.keyboard('{ArrowLeft}{ArrowLeft}');
        // wraps from the first to the last
        expect(settings.theme).toBe('light');
        expect(document.activeElement).toBe(radios()[2]);
    });

    it('text size offers 90, 100, 115 and 130 %, and an arrow key changes the setting', async () => {
        render(SettingsView, { update: null });
        const group = screen.getByRole('radiogroup', { name: 'Text size' });
        const radios = [...group.querySelectorAll<HTMLElement>('[role="radio"]')];
        expect(radios.map((r) => r.textContent?.trim())).toEqual(['90 %', '100 %', '115 %', '130 %']);
        radios[1].focus();
        await userEvent.keyboard('{ArrowRight}');
        expect(settings.textSize).toBe(115);
    });
});

describe('rail landmarks and narrow mode', () => {
    it('names its navigation landmarks and marks the current view, in the bottom group too', () => {
        session.activeView = 'settings';
        render(Rail);
        expect(screen.getByRole('navigation', { name: 'Views' })).toBeTruthy();
        expect(screen.getByRole('navigation', { name: 'Help and settings' })).toBeTruthy();
        expect(screen.queryByRole('complementary')).toBeNull();
        expect(screen.getByRole('button', { name: /Settings/ }).getAttribute('aria-current')).toBe('page');
        expect(screen.getByRole('button', { name: /^Solve/ }).hasAttribute('aria-current')).toBe(false);
    });

    it('is icon-only in a narrow window yet every button keeps its name and a tooltip', () => {
        layout.narrow = true;
        const { container } = render(Rail);
        expect(container.querySelector('.rail')!.classList.contains('compact')).toBe(true);
        for (const label of ['Solve', 'Topics', 'Chain', 'Rules', 'Help', 'Settings']) {
            const b = screen.getByRole('button', { name: new RegExp(`^${label}`) });
            expect(b.title).toContain(label);
        }
        expect(screen.getByRole('button', { name: /Find a script/ }).title).toContain('Find a script');
    });
});

describe('calculator in narrow windows', () => {
    it('folds into its button below the breakpoint and returns to the user\'s own choice above it', async () => {
        settings.calcOpen = true;
        render(CalcPanel);
        expect(screen.queryByRole('complementary', { name: 'Calculator' })).not.toBeNull();
        layout.narrow = true;
        await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Calculator' })).toBeNull());
        // opening it on purpose while narrow works without touching the saved preference
        await userEvent.click(screen.getByRole('button', { name: 'Show calculator' }));
        expect(screen.queryByRole('complementary', { name: 'Calculator' })).not.toBeNull();
        expect(settings.calcOpen).toBe(true);
        await userEvent.click(screen.getByRole('button', { name: 'Hide calculator' }));
        expect(settings.calcOpen).toBe(true);
        layout.narrow = false;
        await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Calculator' })).not.toBeNull());
    });

    it('stays closed above the breakpoint when the user closed it', async () => {
        settings.calcOpen = false;
        render(CalcPanel);
        expect(screen.queryByRole('complementary', { name: 'Calculator' })).toBeNull();
        expect(screen.getByRole('button', { name: 'Show calculator' })).toBeTruthy();
    });
});
