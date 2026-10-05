// The guided tour against the real app and engine: a keyboard walkthrough where every step's highlighted element is
// on screen, inside the spotlight and not covered by the card; skipping; replaying from Help; a narrow window.
import type { Page } from '@playwright/test';

import { expect, test } from './fixtures';

test.use({ tour: true });

const STEPS = [
    { title: 'Paste the problem', targets: ['[data-tour="problem"]'] },
    { title: 'Matches and values found', targets: ['[data-tour="chips"]', '[data-tour="matches"]'] },
    { title: 'The answer', targets: ['[data-tour="answer"]'] },
    { title: 'Check against options', targets: ['[data-tour="check"]'] },
    { title: 'Finding scripts', targets: ['[data-tour="search"]'] },
    { title: 'Chain several formulas', targets: ['[data-tour="nav-chain"]'] },
    { title: 'Help, rehearsals and this tour', targets: ['[data-tour="nav-log"]', '[data-tour="nav-help"]'] },
];

const dialog = (page: Page) => page.getByRole('dialog', { name: /./ });

/** Every target is visible, 16 px clear of the window edge, inside the spotlight rectangle, and the card does not touch it. */
async function expectSpotlightOn(page: Page, targets: string[]): Promise<void> {
    for (const t of targets) await expect(page.locator(t).first()).toBeVisible();
    await expect
        .poll(
            () =>
                page.evaluate((sels) => {
                    const hole = document.querySelector('.tour .hole')?.getBoundingClientRect();
                    const card = document.querySelector('[data-tour-card]')!.getBoundingClientRect();
                    if (hole === undefined) return 'no spotlight';
                    for (const sel of sels) {
                        const r = document.querySelector(sel)!.getBoundingClientRect();
                        const inside = r.left >= hole.left - 1 && r.top >= hole.top - 1 && r.right <= hole.right + 1 && r.bottom <= hole.bottom + 1;
                        if (!inside) return `${sel} outside the spotlight`;
                        const m = 16;
                        if (r.left < m || r.top < m || r.right > window.innerWidth - m || r.bottom > window.innerHeight - m) {
                            return `${sel} within ${m}px of the window edge (${Math.round(r.top)}-${Math.round(r.bottom)} of ${window.innerHeight})`;
                        }
                        const apart = card.right <= r.left || card.left >= r.right || card.bottom <= r.top || card.top >= r.bottom;
                        if (!apart) return `card covers ${sel}`;
                    }
                    if (card.left < 0 || card.top < 0 || card.right > window.innerWidth || card.bottom > window.innerHeight) return 'card outside the window';
                    return 'ok';
                }, targets),
            { timeout: 10_000 },
        )
        .toBe('ok');
}

async function walk(page: Page): Promise<void> {
    for (const [i, step] of STEPS.entries()) {
        await expect(page.getByText(`Step ${i + 1} of ${STEPS.length}`, { exact: true })).toBeVisible();
        await expect(dialog(page)).toHaveAccessibleName(step.title);
        await expectSpotlightOn(page, step.targets);
        if (i < STEPS.length - 1) await page.keyboard.press('ArrowRight');
    }
}

async function savedTourDone(page: Page): Promise<boolean> {
    return page.evaluate(() => JSON.parse(localStorage.getItem('e2e-store:settings.json') ?? '{}').all?.tourDone === true);
}

test('first launch: the keyboard walks the whole tour on the real screen, then it is gone for good', async ({ app }) => {
    await expect(app.locator('#problem')).toHaveValue(/skidpad/);
    await walk(app);
    await expect(app.locator('.answer .a-value')).toHaveText(/^11\.759\d* m\/s$/);
    await app.keyboard.press('Enter');
    await expect(dialog(app)).toBeHidden();
    // the sample problem and the script the tour opened are put away
    await expect(app.locator('#problem')).toHaveValue('');
    await expect(app.locator('.answer')).toHaveCount(0);
    await expect.poll(() => savedTourDone(app)).toBe(true);
    await app.reload();
    await expect(app.locator('.topic-row').first()).toBeVisible({ timeout: 30_000 });
    await expect(dialog(app)).toHaveCount(0);
});

test('Skip tour from the first step ends it, clears the sample and remembers', async ({ app }) => {
    await app.getByRole('button', { name: 'Skip tour' }).click();
    await expect(dialog(app)).toBeHidden();
    await expect(app.locator('#problem')).toHaveValue('');
    await expect.poll(() => savedTourDone(app)).toBe(true);
    await app.reload();
    await expect(app.locator('.topic-row').first()).toBeVisible({ timeout: 30_000 });
    await expect(dialog(app)).toHaveCount(0);
});

test('replay from Help, Escape skips and focus goes back to the button', async ({ app }) => {
    await app.keyboard.press('Escape');
    await expect(dialog(app)).toBeHidden();
    await app.getByRole('button', { name: 'Help', exact: true }).click();
    const replay = app.getByRole('button', { name: 'Take the tour' });
    await replay.focus();
    await app.keyboard.press('Enter');
    await expect(dialog(app)).toBeVisible();
    await expect(app.getByText('Step 1 of 7', { exact: true })).toBeVisible();
    await expectSpotlightOn(app, STEPS[0].targets);
    await app.keyboard.press('Escape');
    await expect(dialog(app)).toBeHidden();
    await expect(app.getByRole('heading', { level: 1, name: 'Help' })).toBeVisible();
    await expect(replay).toBeFocused();
});

test('a narrow window collapses the rail and every step still shows its target without being covered', async ({ app }) => {
    await app.setViewportSize({ width: 860, height: 700 });
    await expect(app.locator('.rail.compact')).toBeVisible();
    await walk(app);
});
