// Finding a script fast, against the real catalogue: the words a teammate actually types, every script by its own
// title, and every script within two clicks of the home screen.
import { expect, test } from './fixtures';
import type { Page } from '@playwright/test';

const BRIDGE = 'http://127.0.0.1:8799';

async function paletteTop(app: Page, query: string, n: number): Promise<string[]> {
    await app.keyboard.press('ControlOrMeta+k');
    const box = app.getByRole('combobox', { name: 'Search scripts' });
    await expect(box).toBeFocused();
    await box.fill(query);
    const titles = (await app.getByRole('dialog', { name: 'Find a script' }).getByRole('option').allTextContents()).slice(0, n);
    await app.keyboard.press('Escape');
    return titles;
}

// Queries that returned nothing or buried the answer before the ranked search (council review, 2026-10-04).
const QUERIES: [string, RegExp, number][] = [
    ['spring rate', /Wheel rate|Helical spring/, 3],
    ['tsal', /TS rule values/, 1],
    ['insulation', /TS rule values/, 3],
    ['gear ratio top speed', /Gear ratio/, 2],
    ['gear ratio top speed', /Power-limited top speed/, 3],
    ['k_s', /Wheel rate|Helical spring/, 2],
    ['aceleration', /Constant acceleration/, 5],
    ['susp', /Wheel rate|Helical spring|Quarter car/, 3],
    ['weight transfer braking', /Longitudinal load transfer/, 2],
    ['thermistor', /NTC/, 1],
];

for (const [query, expected, within] of QUERIES) {
    test(`"${query}" finds ${expected.source} in the top ${within}`, async ({ app }) => {
        const top = await paletteTop(app, query, within);
        expect(top.some((t) => expected.test(t)), `top ${within} for "${query}": ${top.join(' | ')}`).toBe(true);
    });
}

test('typing any script title puts that script first', async ({ app, request }, info) => {
    test.skip(info.project.name !== 'chromium', 'one browser is enough for the data sweep');
    test.setTimeout(180_000);
    const topics = (await (await request.post(`${BRIDGE}/invoke/list_topics`, { data: {} })).json()) as { scripts: { title: string }[] }[];
    const misses: string[] = [];
    const box = app.getByRole('textbox', { name: 'Search the library' });
    for (const s of topics.flatMap((t) => t.scripts)) {
        await box.fill(s.title);
        await expect(app.locator('.detail h2')).toHaveText(`Results for “${s.title.trim()}”`);
        const first = app.locator('.detail .script .title').first();
        const got = (await first.textContent())?.replace(/\s*tool\s*$/, '').trim();
        if (got !== s.title.trim()) misses.push(`"${s.title}" → first is "${got}"`);
    }
    expect(misses, misses.join('\n')).toEqual([]);
});

test('every script is two clicks from home: its topic, then the script', async ({ app, request }, info) => {
    test.skip(info.project.name !== 'chromium', 'one browser is enough for the data sweep');
    const topics = (await (await request.post(`${BRIDGE}/invoke/list_topics`, { data: {} })).json()) as {
        name: string;
        scripts: { title: string }[];
    }[];
    for (const t of topics) {
        await app.getByRole('option', { name: new RegExp(`^${t.name}`) }).click();
        for (const s of t.scripts) {
            await expect(app.locator('.detail .script', { hasText: s.title }).first()).toBeVisible();
        }
    }
    await app.locator('.detail .script', { hasText: topics.at(-1)!.scripts[0].title }).first().click();
    await expect(app.locator('.sheet h2')).toHaveText(topics.at(-1)!.scripts[0].title);
});

test('keyboard only: arrows through topics, into the list, Enter opens; typing searches', async ({ app }) => {
    const motion = app.getByRole('option', { name: /^Motion and energy/ });
    await motion.click();
    await app.keyboard.press('ArrowDown');
    await expect(app.getByRole('option', { name: /^Vehicle dynamics/ })).toBeFocused();
    await expect(app.locator('.detail h2')).toHaveText('Vehicle dynamics');
    await app.keyboard.press('ArrowRight');
    await expect(app.locator('.detail .script').first()).toBeFocused();
    await app.keyboard.press('ArrowDown');
    await app.keyboard.press('Enter');
    await expect(app.locator('.sheet h2')).toHaveText('Longitudinal load transfer');
    await app.getByRole('button', { name: 'Topics' }).click();
    await app.keyboard.type('tsal');
    await expect(app.getByRole('textbox', { name: 'Search the library' })).toHaveValue('tsal');
    await app.keyboard.press('Enter');
    await expect(app.locator('.sheet h2')).toHaveText(/TS rule values/);
});

test('the palette with nothing typed offers pinned scripts first', async ({ app }) => {
    const top = await paletteTop(app, '', 3);
    expect(top[0]).toMatch(/Battery/);
});
