// Every past FS-Quiz question filed as a worked example, clicked through the real UI: open the script from the
// palette by its id, press the Q-button, and the official answer must be on screen (answer slab or a listed root).
// This is the path a teammate takes on quiz day; the IPC tests prove the engine, this proves the sheet wiring
// (pre-fill, display units, positional tool arguments, auto-run) for every one of them.
import type { Locator, Page } from '@playwright/test';
import { expect, test } from './fixtures';

/** The "Past questions" fold is closed until opened and remembers how it was left: open it only if it is closed. */
async function openPastQuestions(scope: Locator | Page): Promise<void> {
    const head = scope.getByRole('button', { name: 'Past questions' });
    if ((await head.getAttribute('aria-expanded')) !== 'true') await head.click();
    await expect(head).toHaveAttribute('aria-expanded', 'true');
}

const BRIDGE = 'http://127.0.0.1:8799';

interface Example {
    id: number;
    what: string;
    cmd: string;
    answer: number;
}

test.describe.configure({ mode: 'serial' });

test('every worked example shows its official answer', async ({ app, request }, info) => {
    test.skip(info.project.name !== 'chromium', 'one browser is enough for the data sweep');
    test.setTimeout(600_000);
    const call = async <T>(cmd: string, args: object = {}): Promise<T> =>
        (await request.post(`${BRIDGE}/invoke/${cmd}`, { data: args })).json() as Promise<T>;
    const topics = await call<{ scripts: { id: string; title: string }[] }[]>('list_topics');
    const failures: string[] = [];
    let checked = 0;
    for (const s of topics.flatMap((t) => t.scripts)) {
        const examples = await call<Example[]>('script_examples', { script: s.id });
        if (examples.length === 0) continue;
        await app.keyboard.press('ControlOrMeta+k');
        await app.getByRole('combobox', { name: 'Search scripts' }).fill(s.id);
        await app.getByRole('option').filter({ hasText: s.title }).first().click();
        await expect(app.locator('.sheet h2')).toHaveText(s.title);
        for (const ex of examples) {
            await app.getByRole('button', { name: new RegExp(`^Q${ex.id} ·`) }).first().click();
            const shown = async (): Promise<number[]> => {
                const texts = await app.locator('.answer .a-value, .results .res .mono').allTextContents();
                return texts.flatMap((t) => (t.match(/-?\d+(\.\d+)?(e[-+]?\d+)?/gi) ?? []).map(Number));
            };
            const hit = (ns: number[]) => ns.some((n) => Math.abs(n - ex.answer) <= 0.005 * Math.abs(ex.answer) + 1e-9);
            try {
                await expect.poll(async () => hit(await shown()), { timeout: 4000 }).toBe(true);
            } catch {
                failures.push(`Q${ex.id} ${s.id}: official ${ex.answer}, screen shows ${JSON.stringify(await shown())}`);
            }
            checked += 1;
        }
    }
    expect(checked).toBeGreaterThan(100);
    expect(failures, failures.join('\n')).toEqual([]);
});

// The past questions solved with `chain` and `calc` belong to no single script: they live in the Chain view and the
// calculator, one click each, and must show the official answer too.
const numbersIn = (texts: string[]): number[] =>
    texts.flatMap((t) => (t.match(/-?\d+(\.\d+)?(e[-+]?\d+)?/gi) ?? []).map(Number));
const near = (ns: number[], answer: number): boolean =>
    ns.some((n) => Math.abs(n - answer) <= 0.005 * Math.abs(answer) + 1e-9);

test('every chain past question shows its official answer in the Chain view', async ({ app, request }, info) => {
    test.skip(info.project.name !== 'chromium', 'one browser is enough for the data sweep');
    const res = await request.post(`${BRIDGE}/invoke/chain_examples`, { data: {} });
    const examples = (await res.json()) as Example[];
    expect(examples.length).toBeGreaterThan(5);
    await app.getByRole('button', { name: 'Chain' }).click();
    const section = app.getByRole('region', { name: 'Past questions' });
    await openPastQuestions(section);
    const failures: string[] = [];
    for (const ex of examples) {
        await section.getByRole('button', { name: new RegExp(`^Q${ex.id}\\b`) }).click();
        const shown = async () => numbersIn(await app.locator('.answer .a-value').allTextContents());
        try {
            await expect.poll(async () => near(await shown(), ex.answer), { timeout: 4000 }).toBe(true);
        } catch {
            failures.push(`Q${ex.id} chain: official ${ex.answer}, screen shows ${JSON.stringify(await shown())}`);
        }
    }
    expect(failures, failures.join('\n')).toEqual([]);
});

test('every calculator past question shows its official answer in the calculator', async ({ app, request }, info) => {
    test.skip(info.project.name !== 'chromium', 'one browser is enough for the data sweep');
    const res = await request.post(`${BRIDGE}/invoke/calc_examples`, { data: {} });
    const examples = (await res.json()) as Example[];
    expect(examples.length).toBeGreaterThan(5);
    const panel = app.getByRole('complementary', { name: 'Calculator' });
    await openPastQuestions(panel);
    const failures: string[] = [];
    for (const ex of examples) {
        await panel.getByRole('button', { name: new RegExp(`^Q${ex.id}\\b`) }).click();
        const shown = async () => numbersIn(await panel.locator('ul li').first().locator('.result').allTextContents());
        try {
            await expect.poll(async () => near(await shown(), ex.answer), { timeout: 4000 }).toBe(true);
        } catch {
            failures.push(`Q${ex.id} calc: official ${ex.answer}, screen shows ${JSON.stringify(await shown())}`);
        }
    }
    expect(failures, failures.join('\n')).toEqual([]);
});
