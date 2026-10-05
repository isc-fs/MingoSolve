// The script search's ranking rules on a small hand-made library. What it finds in the real catalogue (the words
// teammates type, every title) is checked end to end in e2e/finding.spec.ts.
import { beforeEach, describe, expect, it } from 'vitest';

import { catalog, searchScripts, type Script } from './catalog.svelte';

function formula(id: string, title: string, vars: [string, string][], tags: string[] = [], aliases = ''): Script {
    return {
        id,
        kind: 'formula',
        title,
        aliases,
        topic: null,
        formula: { key: id, title, eqs: [], tex: [], tags, notes: '', vars: vars.map(([name, desc]) => ({ name, desc, unit: '', signed: false, default: null, tex: name, unit_shown: '', dims: '' })) },
    };
}

const ids = (q: string, prefer: string[] = []) => searchScripts(q, 30, prefer).map((s) => s.id);

beforeEach(() => {
    const lib = [
        formula('front_lift', 'Front wheels lift under acceleration', [['ax', 'longitudinal acceleration'], ['h_cg', 'CG height']], ['traction']),
        formula('traction_limit', 'Friction-limited acceleration', [['ax', 'longitudinal acceleration'], ['mu', 'friction coefficient']], ['grip']),
        formula('wheel_rate', 'Wheel rate, ride rate, frequency', [['k_s', 'spring rate'], ['MR', 'motion ratio']], [], 'spring rate damping'),
        formula('coil_spring', 'Helical spring rate', [['k_s', 'spring rate'], ['d_wire', 'wire diameter']]),
        formula('gear_drive', 'Gear ratio: speeds, torques, road speed', [['i_g', 'gear ratio'], ['v', 'road speed']], [], 'top speed final drive'),
        formula('top_speed_power', 'Power-limited top speed', [['P', 'power'], ['v', 'top speed']], [], 'max speed drag gear'),
        formula('uniform_accel', 'Constant acceleration', [['a', 'acceleration'], ['t', 'time']]),
    ];
    catalog.scripts = new Map(lib.map((s) => [s.id, s]));
});

describe('ranking', () => {
    it('a title typed with its punctuation ranks that script first (hyphens split like the index)', () => {
        expect(ids('Friction-limited acceleration')[0]).toBe('traction_limit');
    });

    it('a variable symbol finds the formulas that use it, ahead of description-only matches', () => {
        expect(ids('k_s').slice(0, 2).sort()).toEqual(['coil_spring', 'wheel_rate']);
    });

    it('an alias the title does not contain still finds the script', () => {
        expect(ids('damping')[0]).toBe('wheel_rate');
    });

    it('the start of a word is enough while typing', () => {
        expect(ids('helic')[0]).toBe('coil_spring');
    });

    it('one typo is forgiven in longer words', () => {
        expect(ids('aceleration')).toContain('uniform_accel');
    });

    it('scripts matching every word come first, those missing one follow; an extra word does not empty the list', () => {
        const r = ids('gear ratio top speed');
        expect(r[0]).toBe('gear_drive');
        expect(r).toContain('top_speed_power');
        expect(ids('helical spring xyzzy')[0]).toBe('coil_spring');
    });

    it('scripts missing two or more words of a long query are left out', () => {
        expect(ids('gear ratio top speed')).not.toContain('front_lift');
    });

    it('with nothing typed, preferred scripts (pinned, then recent) lead', () => {
        expect(ids('', ['top_speed_power', 'coil_spring']).slice(0, 2)).toEqual(['top_speed_power', 'coil_spring']);
    });
});
