// The rail names pinned and recent scripts by their title, cut with an ellipsis when long, with the full title on hover.
import { render } from '@testing-library/svelte';
import { beforeEach, expect, it } from 'vitest';

import Rail from './Rail.svelte';
import { catalog } from './catalog.svelte';
import { session } from './session.svelte';
import { defaultSettings, settings } from './settings.svelte';

const LONG = 'Dynamic event score (skidpad, accel, autocross, endurance)';

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    catalog.scripts = new Map([
        ['event_score', { id: 'event_score', kind: 'tool', title: LONG, aliases: '', topic: null }],
        ['ts_discharge', { id: 'ts_discharge', kind: 'formula', title: 'Discharging the TS through a resistor', aliases: '', topic: null }],
        ['gamma', { id: 'gamma', kind: 'tool', title: 'Reflection coefficient and VSWR of a load', aliases: '', topic: null }],
    ]);
    session.script = null;
});

it('lists pinned and recent scripts by title, not by a shortened id', () => {
    settings.pinned = ['event_score'];
    settings.recent = ['ts_discharge', 'gamma'];
    const { container } = render(Rail);
    const rows = [...container.querySelectorAll<HTMLButtonElement>('.script')];
    expect(rows.map((r) => r.textContent?.trim())).toEqual([LONG, 'Discharging the TS through a resistor', 'Reflection coefficient and VSWR of a load']);
    expect(rows.map((r) => r.title)).toEqual(rows.map((r) => r.textContent?.trim()));
    expect(container.textContent).not.toMatch(/Ts discharge|Event score|\bGamma\b/);
});

it('falls back to a readable name when the catalogue does not know the id yet', () => {
    settings.pinned = ['ts_discharge_old'];
    const { container } = render(Rail);
    expect(container.querySelector('.script')?.textContent?.trim()).toBe('Ts discharge old');
});
