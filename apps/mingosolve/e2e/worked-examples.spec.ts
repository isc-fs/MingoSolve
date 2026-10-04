// Every past FS-Quiz question filed as a worked example, clicked through the real UI: open the script from the
// palette by its id, press the Q-button, and the official answer must be on screen (answer slab or a listed root).
// This is the path a teammate takes on quiz day; the IPC tests prove the engine, this proves the sheet wiring
// (pre-fill, display units, positional tool arguments, auto-run) for every one of them.
import { expect, test } from './fixtures';

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
