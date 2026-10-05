// Automated accessibility audit (axe-core) of the main screens in both themes, glass and solid: no serious or
// critical violations, colour contrast included. Readability is a hard requirement for this app.
import AxeBuilder from '@axe-core/playwright';
import type { Page } from '@playwright/test';

import { answer, choice, expect, field, openFromPalette, test } from './fixtures';

async function audit(page: Page, label: string): Promise<void> {
    const r = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
    const bad = r.violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
    const summary = bad.map((v) => `${v.id} (${v.impact}): ${v.nodes.slice(0, 3).map((n) => n.target.join(' ')).join(' | ')}`);
    expect(summary, `${label}`).toEqual([]);
}

for (const theme of ['Night glass', 'Paper glass'] as const) {
    test(`${theme}: home, a solved script, Topics, Chain, Rules, Help, Session log and Settings pass axe`, async ({ app }) => {
        await app.getByRole('button', { name: 'Settings' }).click();
        await app.getByRole('radio', { name: new RegExp(theme) }).click();
        await audit(app, `${theme} settings`);
        await app.getByRole('button', { name: 'Solve' }).click();
        await audit(app, `${theme} home`);
        await openFromPalette(app, 'battery load', /Battery/);
        await app.getByRole('button', { name: /^Q34 ·/ }).click();
        await expect(field(app, 'N_s')).toHaveValue('103');
        await audit(app, `${theme} solved sheet`);
        await app.getByRole('button', { name: 'Topics' }).click();
        await app.getByRole('textbox', { name: 'Search the library' }).fill('spring');
        await audit(app, `${theme} topics, searching`);
        await app.keyboard.press('ControlOrMeta+k');
        await app.getByRole('combobox', { name: 'Search scripts' }).fill('tsal');
        await audit(app, `${theme} palette open`);
        await app.keyboard.press('Escape');
        await app.getByRole('button', { name: 'Chain' }).click();
        await audit(app, `${theme} chain`);
        await app.getByRole('button', { name: 'Rules' }).click();
        await expect(app.getByText('Load the rulebook once')).toBeVisible();
        await audit(app, `${theme} rules, no rulebook loaded`);
        await app.getByRole('button', { name: 'Help', exact: true }).click();
        await expect(app.getByRole('heading', { level: 1, name: 'Help' })).toBeVisible();
        await audit(app, `${theme} help`);
        await app.getByRole('button', { name: 'Session log', exact: true }).click();
        await audit(app, `${theme} session log, empty`);
        await app.getByRole('button', { name: 'Start a mock quiz' }).click();
        await app.getByRole('button', { name: 'Solve' }).click();
        await openFromPalette(app, 'battery load', /Battery/);
        await app.getByRole('button', { name: /^Q34 ·/ }).click();
        await expect(app.locator('.answer .copy')).toBeVisible();
        await app.locator('.answer .copy').click();
        await app.getByRole('button', { name: 'Session log', exact: true }).click();
        await expect(app.locator('tbody tr')).toHaveCount(1);
        await audit(app, `${theme} session log, with an entry`);
    });
}

test('the palette is a labelled dialog that keyboard users can drive', async ({ app }) => {
    await app.keyboard.press('ControlOrMeta+k');
    const dialog = app.getByRole('dialog', { name: 'Find a script' });
    await expect(dialog).toBeVisible();
    await expect(app.getByRole('combobox', { name: 'Search scripts' })).toBeFocused();
    await app.keyboard.type('spring');
    await app.keyboard.press('ArrowDown');
    const selected = dialog.getByRole('option', { selected: true });
    await expect(selected).toContainText(/Helical spring|Wheel rate|Quarter car/);
    // screen readers follow the highlighted row through the combobox, not through focus
    const active = await app.getByRole('combobox', { name: 'Search scripts' }).getAttribute('aria-activedescendant');
    await expect(selected).toHaveAttribute('id', active!);
    await app.keyboard.press('Enter');
    await expect(dialog).toBeHidden();
    await expect(app.locator('.sheet h2')).toContainText(/spring|Wheel rate|Quarter/i);
    await app.keyboard.press('ControlOrMeta+k');
    await app.keyboard.press('Escape');
    await expect(dialog).toBeHidden();
});

// ---- states the first audit does not reach: results, errors, option check, chain, rules, palette; glass and solid ----

const BRIDGE = 'http://127.0.0.1:8799';
const CORS = { 'access-control-allow-origin': '*' };

/** The Rules screens need a loaded rulebook, which is a PDF the user owns; the bridge answers for one instead. */
async function fakeRulebook(page: Page): Promise<void> {
    await page.route(`${BRIDGE}/invoke/rulebook_status`, (r) =>
        r.fulfill({ json: [{ year: '2027', source: 'FS_Rules_2027_v1.0.pdf', loaded_at: 1_790_000_000, pages: 115, entries: 1533 }], headers: CORS }),
    );
    await page.route(`${BRIDGE}/invoke/search_rules`, (r) =>
        r.fulfill({
            json: [{ year: '2027', id: 'T 2.9.2', title: 'Tread depth', page: 42, snippet: 'The tread depth must be at least 1.5 mm.', matches: [[4, 9]] }],
            headers: CORS,
        }),
    );
    await page.reload();
    await expect(page.locator('.topic-row').first()).toBeVisible({ timeout: 30_000 });
}

const looks = [
    { name: 'Night glass', radio: /Night glass/, solid: false },
    { name: 'Paper glass', radio: /Paper glass/, solid: false },
    { name: 'Night, solid surfaces', radio: /Night glass/, solid: true },
    { name: 'Paper, solid surfaces', radio: /Paper glass/, solid: true },
] as const;

for (const look of looks) {
    test(`${look.name}: result, error, options check, chain, Rules and the palette's Rules group pass axe`, async ({ app }) => {
        await fakeRulebook(app);
        await app.getByRole('button', { name: 'Settings' }).click();
        await app.getByRole('radio', { name: look.radio }).click();
        const solid = app.getByRole('checkbox', { name: /Solid surfaces/ });
        if (look.solid) await solid.check();
        else await solid.uncheck();
        await app.getByRole('button', { name: 'Solve' }).click();

        // a tool sheet with a result (Q378: event score under the legacy rules)
        await app.getByRole('button', { name: 'Topics' }).click();
        await app.getByRole('textbox', { name: 'Search the library' }).fill('event score');
        await app.locator('.detail').getByRole('button', { name: /^Dynamic event score/ }).click();
        await choice(app, 'event').selectOption('skidpad');
        await field(app, 't_team').fill('5.6');
        await field(app, 't_min').fill('5.1');
        await choice(app, 'rules').selectOption('legacy');
        await app.getByRole('button', { name: /^Run/ }).click();
        await expect(answer(app)).toHaveText('41.117');
        await audit(app, `${look.name} tool result`);
        await app.locator('.answer .copy').focus();
        await audit(app, `${look.name} tool result, Copy focused`);

        // an error state: a wrong unit
        await openFromPalette(app, 'average speed', /Average speed/);
        await field(app, 's').fill('3 kg');
        await expect(app.locator('.note-bad')).toContainText('needs a length');
        await audit(app, `${look.name} error`);

        // the options check with a best match, and the answer slab
        await openFromPalette(app, 'battery load', /Battery/);
        await app.getByRole('button', { name: /^Q34 ·/ }).click();
        await expect(answer(app)).toHaveText(/^77\.887\d* A$/);
        await app.getByRole('button', { name: 'Check against options' }).click();
        await app.getByPlaceholder(/a\) 73\.4 A/).fill('a) 73.4 A\nb) 77.9 A\nc) 80.4 A');
        await expect(app.locator('.opts li.best')).toContainText('b) 77.9 A');
        await audit(app, `${look.name} options check`);

        // chain results (Q515)
        await app.getByRole('button', { name: 'Chain' }).click();
        await app.getByRole('combobox', { name: 'Target' }).fill('v');
        await app.getByRole('textbox', { name: 'Show in' }).fill('km/h');
        await app.locator('textarea').fill('h_cg = 0.205 m\nR_c = 14.5 m\nt_tr = 1.24 m');
        await app.getByRole('button', { name: 'Chain', exact: true }).last().click();
        await expect(app.locator('.answer')).toContainText(/74\.66\d* km\/h/);
        await audit(app, `${look.name} chain result`);

        // Rules view with a rulebook, and the palette showing the Rules group
        await app.getByRole('button', { name: 'Rules' }).click();
        await expect(app.getByRole('heading', { level: 1, name: 'Rules' })).toBeVisible();
        await audit(app, `${look.name} rules`);
        await app.keyboard.press('ControlOrMeta+k');
        await app.getByRole('combobox', { name: 'Search scripts' }).fill('tread');
        await expect(app.getByRole('dialog', { name: 'Find a script' }).getByRole('group', { name: 'Rules' })).toBeVisible();
        await audit(app, `${look.name} palette with the Rules group`);
    });
}

test('a solved field shows its value as visible text with the unit, next to the field', async ({ app }) => {
    await openFromPalette(app, 'battery load', /Battery/);
    await app.getByRole('button', { name: /^Q34 ·/ }).click();
    await expect(answer(app)).toHaveText(/^77\.887\d* A$/);
    const solved = app.locator('[data-solved]');
    await expect(solved.first()).toBeVisible();
    await expect(solved.first()).toContainText(/= [\d.]+ \S+/);
});

test('Tab stays inside the open palette and Escape returns focus to the button that opened it', async ({ app }) => {
    // opened from the keyboard: WebKit (and so macOS WKWebView) never focuses a button on click, so there is no
    // focused opener to return to after a mouse click; keyboard users are the ones who need focus returned
    const opener = app.getByRole('button', { name: /Find a script/ }).first();
    await opener.focus();
    await app.keyboard.press('Enter');
    const dialog = app.getByRole('dialog', { name: 'Find a script' });
    const box = app.getByRole('combobox', { name: 'Search scripts' });
    await expect(box).toBeFocused();
    const insideDialog = () => app.evaluate(() => document.activeElement?.closest('[role="dialog"]') !== null);

    // a short query has one stop: Tab and Shift+Tab never leave the box
    await box.fill('tsal');
    for (let i = 0; i < 3; i++) {
        await app.keyboard.press('Tab');
        expect(await insideDialog()).toBe(true);
    }
    await app.keyboard.press('Shift+Tab');
    expect(await insideDialog()).toBe(true);

    // six words switch to problem mode (announced), which adds a second stop: Tab from the last returns to the first
    await box.fill('a car of 250 kg brakes hard');
    await expect(app.getByRole('status').filter({ hasText: 'Searching as a problem' })).toHaveCount(1);
    await app.keyboard.press('Tab');
    await expect(dialog.getByRole('button', { name: /Find the scripts for this problem/ })).toBeFocused();
    await app.keyboard.press('Tab');
    await expect(box).toBeFocused();
    await app.keyboard.press('Shift+Tab');
    await expect(dialog.getByRole('button', { name: /Find the scripts for this problem/ })).toBeFocused();

    await app.keyboard.press('Escape');
    await expect(dialog).toBeHidden();
    await expect(opener).toBeFocused();
});

test('the theme radio group moves with the arrow keys and is a single Tab stop', async ({ app }) => {
    await app.getByRole('button', { name: 'Settings' }).click();
    const theme = app.getByRole('radiogroup', { name: 'Theme' });
    await theme.getByRole('radio', { name: /Follow the OS/ }).click();
    await app.keyboard.press('ArrowRight');
    await expect(theme.getByRole('radio', { name: /Night glass/ })).toBeFocused();
    await expect(theme.getByRole('radio', { name: /Night glass/ })).toHaveAttribute('aria-checked', 'true');
    await expect(app.locator('html')).toHaveAttribute('data-theme', 'dark');
    await app.keyboard.press('ArrowRight');
    await expect(theme.getByRole('radio', { name: /Paper glass/ })).toBeFocused();
    await expect(app.locator('html')).toHaveAttribute('data-theme', 'light');
    await expect(theme.getByRole('radio', { name: /Night glass/ })).toHaveAttribute('tabindex', '-1');
});

// Cmd/Ctrl +/- (zoomHotkeysEnabled in tauri.conf.json) is a native WebView feature; Playwright's web mode cannot
// press it, so it is checked by hand in the real app. The Text size setting is what the browser can test.
test('the Text size setting scales the text of the whole app, and survives a restart', async ({ app }) => {
    const sizeOf = (sel: string) => app.evaluate((s) => parseFloat(getComputedStyle(document.querySelector(s)!).fontSize), sel);
    const base = await sizeOf('body');
    const baseRow = await sizeOf('.topic-row .name');
    await app.getByRole('button', { name: 'Settings' }).click();
    await app.getByRole('radio', { name: '130 %' }).click();
    expect(await sizeOf('body')).toBeCloseTo(base * 1.3, 1);
    await app.getByRole('radio', { name: '90 %' }).click();
    expect(await sizeOf('body')).toBeCloseTo(base * 0.9, 1);
    await app.getByRole('radio', { name: '130 %' }).click();
    await app.getByRole('button', { name: 'Topics' }).click();
    // library rows use a type-scale token rather than the body size, so they must scale too
    expect(await sizeOf('.topic-row .name')).toBeCloseTo(baseRow * 1.3, 1);
    // the setting is saved by a debounced autosave: wait for it to reach the store before restarting
    await expect
        .poll(() => app.evaluate(() => JSON.parse(localStorage.getItem('e2e-store:settings.json') ?? '{}').all?.textSize))
        .toBe(130);
    await app.reload();
    await expect(app.locator('.topic-row').first()).toBeVisible({ timeout: 30_000 });
    expect(await sizeOf('body')).toBeCloseTo(base * 1.3, 1);
});

test('at 1000 px the calculator is folded away and the rail shows icons only; at 1400 px both are open', async ({ app }) => {
    const rail = app.locator('.rail');
    const calc = app.getByRole('complementary', { name: 'Calculator' });

    await app.setViewportSize({ width: 1400, height: 820 });
    await expect(calc).toBeVisible();
    expect((await rail.boundingBox())!.width).toBeGreaterThan(200);

    await app.setViewportSize({ width: 1000, height: 820 });
    await expect(calc).toHaveCount(0);
    await expect(app.getByRole('button', { name: 'Show calculator' })).toBeVisible();
    expect((await rail.boundingBox())!.width).toBeLessThan(80);
    // still operable and named
    await expect(app.getByRole('button', { name: /^Topics/ })).toBeVisible();
    await expect(app.getByRole('button', { name: /^Topics/ })).toHaveAttribute('title', /Topics/);
    expect(await app.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);

    // opening it on purpose while narrow works, and the wide window goes back to the saved choice
    await app.getByRole('button', { name: 'Show calculator' }).click();
    await expect(calc).toBeVisible();
    await app.setViewportSize({ width: 1400, height: 820 });
    await expect(calc).toBeVisible();
});

test('a calculator the user closed stays closed when the window becomes wide again', async ({ app }) => {
    await app.getByRole('button', { name: 'Hide calculator' }).click();
    await app.setViewportSize({ width: 1000, height: 820 });
    await app.setViewportSize({ width: 1500, height: 820 });
    await expect(app.getByRole('complementary', { name: 'Calculator' })).toHaveCount(0);
    await expect(app.getByRole('button', { name: 'Show calculator' })).toBeVisible();
});

test('at 2560 px the content is centred and the Copy button sits next to the value', async ({ app }) => {
    await app.setViewportSize({ width: 2560, height: 1200 });
    await openFromPalette(app, 'battery load', /Battery/);
    await app.getByRole('button', { name: /^Q34 ·/ }).click();
    await expect(answer(app)).toHaveText(/^77\.887\d* A$/);
    const value = (await answer(app).boundingBox())!;
    const copy = (await app.locator('.answer .copy').boundingBox())!;
    expect(copy.x - (value.x + value.width)).toBeLessThan(300);

    const sheet = (await app.locator('.sheet').boundingBox())!;
    const main = (await app.locator('main').boundingBox())!;
    expect(sheet.width).toBeLessThanOrEqual(1300);
    const left = sheet.x - main.x;
    const right = main.x + main.width - (sheet.x + sheet.width);
    expect(Math.abs(left - right)).toBeLessThanOrEqual(24);

    await app.getByRole('button', { name: 'Chain' }).click();
    const chain = (await app.locator('main section').first().boundingBox())!;
    const chainLeft = chain.x - main.x;
    const chainRight = main.x + main.width - (chain.x + chain.width);
    expect(Math.abs(chainLeft - chainRight)).toBeLessThanOrEqual(24);
});
