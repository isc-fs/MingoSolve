// Automated accessibility audit (axe-core) of the main screens in both themes, glass and solid: no serious or
// critical violations, colour contrast included. Readability is a hard requirement for this app.
import AxeBuilder from '@axe-core/playwright';
import type { Page } from '@playwright/test';

import { expect, field, openFromPalette, test } from './fixtures';

async function audit(page: Page, label: string): Promise<void> {
    const r = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
    const bad = r.violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
    const summary = bad.map((v) => `${v.id} (${v.impact}): ${v.nodes.slice(0, 3).map((n) => n.target.join(' ')).join(' | ')}`);
    expect(summary, `${label}`).toEqual([]);
}

for (const theme of ['Night glass', 'Paper glass'] as const) {
    test(`${theme}: home, a solved script, Topics, Chain, Help, Session log and Settings pass axe`, async ({ app }) => {
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
