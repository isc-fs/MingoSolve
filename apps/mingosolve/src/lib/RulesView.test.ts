// The Rules view and the palette's Rules group against a fake engine: the empty state, the load flow through the
// file picker, results with highlighted words, removal, and the palette keeping its combobox behaviour.
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';

import Palette from './Palette.svelte';
import RulesView from './RulesView.svelte';
import { catalog, type Script } from './catalog.svelte';
import { rulebooks } from './rulebook.svelte';
import { session } from './session.svelte';
import { fakeEngine } from '../test/ipc';
import type { RuleHit, RulebookStatus } from './types';

const status = (year: string): RulebookStatus => ({ year, source: `FS_Rules_${year}_v1.0.pdf`, loaded_at: 1_790_000_000, pages: 115, entries: 1533 });

// the shape of a real hit: "[start, end)" UTF-16 spans of the matched words inside the snippet
const trackHit: RuleHit = {
    year: '2027',
    id: 'D 6.1.1',
    title: 'Autocross Track Layout',
    page: 108,
    snippet: 'The minimum track width is 3 m, with cones every 5 m.',
    matches: [
        [12, 17],
        [18, 23],
    ],
};

beforeEach(() => {
    // jsdom has no layout, so no scrollIntoView (the palette calls it to keep the active row visible)
    Element.prototype.scrollIntoView = () => {};
    rulebooks.loaded = [];
    session.rulesQuery = '';
    session.activeView = 'solve';
    session.paletteOpen = false;
    catalog.scripts = new Map();
});

describe('Rules view', () => {
    it('without a rulebook it explains where to get the PDF and offers no search', async () => {
        const engine = fakeEngine({ rulebook_status: () => [] });
        render(RulesView);
        await screen.findByText('Load the rulebook once');
        expect(document.body.textContent).toContain('formulastudent.de');
        expect(screen.queryByRole('searchbox', { name: 'Search the rules' })).toBeNull();
        expect(screen.getByRole('button', { name: 'Load the 2027 PDF' })).toBeTruthy();
        expect(engine.calls.some((c) => c.cmd === 'search_rules')).toBe(false);
    });

    it('loading a PDF sends the picked file and year, then switches to search', async () => {
        let saved: RulebookStatus[] = [];
        const engine = fakeEngine({
            rulebook_status: () => saved,
            'plugin:dialog|open': () => '/home/me/Downloads/FS_Rules_2027_v1.0.pdf',
            load_rulebook: () => {
                saved = [status('2027')];
                return saved[0];
            },
        });
        render(RulesView);
        await userEvent.click(await screen.findByRole('button', { name: 'Load the 2027 PDF' }));
        await screen.findByRole('searchbox', { name: 'Search the rules' });
        expect(engine.calls.find((c) => c.cmd === 'load_rulebook')?.args).toEqual({ path: '/home/me/Downloads/FS_Rules_2027_v1.0.pdf', year: '2027' });
        expect(screen.queryByText('Load the rulebook once')).toBeNull();
        expect(document.body.textContent).toContain('1533 rules, 115 pages');
        expect(screen.getByRole('button', { name: 'Replace the 2027 PDF' })).toBeTruthy();
    });

    it('cancelling the file picker loads nothing', async () => {
        const engine = fakeEngine({ rulebook_status: () => [], 'plugin:dialog|open': () => null });
        render(RulesView);
        await userEvent.click(await screen.findByRole('button', { name: 'Load the 2027 PDF' }));
        await waitFor(() => expect(engine.calls.some((c) => c.cmd === 'plugin:dialog|open')).toBe(true));
        expect(engine.calls.some((c) => c.cmd === 'load_rulebook')).toBe(false);
        expect(screen.queryByRole('alert')).toBeNull();
    });

    it('a PDF the engine rejects is reported and nothing becomes loaded', async () => {
        fakeEngine({
            rulebook_status: () => [],
            'plugin:dialog|open': () => '/tmp/notes.pdf',
            load_rulebook: () => {
                throw 'found only 0 rules in this PDF: is it the official FS-Rules document?';
            },
        });
        render(RulesView);
        await userEvent.click(await screen.findByRole('button', { name: 'Load the 2027 PDF' }));
        expect((await screen.findByRole('alert')).textContent).toContain('found only 0 rules');
        expect(screen.queryByRole('searchbox')).toBeNull();
    });

    it('typing searches the loaded year and shows id, title, page and the matched words highlighted', async () => {
        const engine = fakeEngine({ rulebook_status: () => [status('2027')], search_rules: () => [trackHit] });
        render(RulesView);
        await userEvent.type(await screen.findByRole('searchbox', { name: 'Search the rules' }), 'track width');
        const list = await screen.findByRole('list', { name: 'Rule results' });
        expect(list.textContent).toContain('D 6.1.1');
        expect(list.textContent).toContain('Autocross Track Layout');
        expect(list.textContent).toContain('page 108');
        expect([...list.querySelectorAll('mark')].map((m) => m.textContent)).toEqual(['track', 'width']);
        expect(list.querySelector('.snippet')?.textContent).toBe(trackHit.snippet);
        const last = engine.calls.filter((c) => c.cmd === 'search_rules').at(-1)!;
        expect(last.args).toMatchObject({ query: 'track width', year: '2027' });
    });

    it('says so when nothing matches, and deleting the rulebook returns to the empty state', async () => {
        let saved = [status('2027')];
        const engine = fakeEngine({
            rulebook_status: () => saved,
            search_rules: () => [],
            remove_rulebook: () => {
                saved = [];
                return null;
            },
        });
        render(RulesView);
        await userEvent.type(await screen.findByRole('searchbox'), 'zebra');
        await screen.findByText(/No rule matches/);
        await userEvent.click(screen.getByRole('button', { name: 'Remove the 2027 rulebook' }));
        await screen.findByText('Load the rulebook once');
        expect(engine.calls.find((c) => c.cmd === 'remove_rulebook')?.args).toEqual({ year: '2027' });
    });
});

describe('palette', () => {
    const script: Script = {
        id: 'track_script',
        kind: 'formula',
        title: 'Track width script',
        aliases: '',
        topic: null,
        formula: { key: 'track_script', title: 'Track width script', eqs: [], tex: [], tags: [], notes: '', vars: [] },
    };

    async function openPalette(): Promise<HTMLElement> {
        catalog.scripts = new Map([[script.id, script]]);
        render(Palette);
        session.paletteOpen = true;
        const box = await screen.findByRole('combobox', { name: 'Search scripts' });
        await userEvent.type(box, 'track');
        return box;
    }

    it('lists matching rules in their own group under the scripts, and the combobox walks through both', async () => {
        rulebooks.loaded = [status('2027')];
        fakeEngine({ search_rules: () => [trackHit] });
        const box = await openPalette();
        const rules = await screen.findByRole('group', { name: 'Rules' });
        const options = screen.getAllByRole('option');
        expect(options.map((o) => o.textContent)).toEqual([expect.stringContaining('Track width script'), expect.stringContaining('D 6.1.1')]);
        expect(rules.contains(options[1])).toBe(true);
        expect(box.getAttribute('aria-activedescendant')).toBe(options[0].id);
        await userEvent.keyboard('{ArrowDown}');
        expect(box.getAttribute('aria-activedescendant')).toBe(options[1].id);
        expect(options[1].getAttribute('aria-selected')).toBe('true');
        expect(box.getAttribute('aria-expanded')).toBe('true');
    });

    it('choosing a rule opens the Rules view with the typed query', async () => {
        rulebooks.loaded = [status('2027')];
        fakeEngine({ search_rules: () => [trackHit] });
        await openPalette();
        await screen.findByRole('group', { name: 'Rules' });
        await userEvent.keyboard('{ArrowDown}{Enter}');
        expect(session.activeView).toBe('rules');
        expect(session.rulesQuery).toBe('track');
        expect(session.paletteOpen).toBe(false);
    });

    it('without a rulebook there is no Rules group and the engine is not asked for rules', async () => {
        const engine = fakeEngine({});
        await openPalette();
        await screen.findByRole('option', { name: /Track width script/ });
        expect(screen.queryByRole('group', { name: 'Rules' })).toBeNull();
        await new Promise((r) => setTimeout(r, 250));
        expect(engine.calls.some((c) => c.cmd === 'search_rules')).toBe(false);
    });
});
