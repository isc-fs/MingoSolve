// Shared helpers: a page whose clipboard writes are captured, and small actions the journeys reuse.
import { test as base, expect, type Page } from '@playwright/test';

export const test = base.extend<{ app: Page; tour: boolean; update: boolean }>({
    /** Opt in with `test.use({ tour: true })`: every other spec starts as someone who already did the tour. */
    tour: [false, { option: true }],
    /** `test.use({ update: true })`: the fake updater offers v9.9.9 (src/e2e-ipc.ts). */
    update: [false, { option: true }],
    app: async ({ page, tour, update }, use) => {
        if (update) await page.addInitScript(() => localStorage.setItem('e2e-update', 'available'));
        await page.addInitScript((seedTourDone) => {
            // only when nothing is stored yet, so a reload keeps what the app saved (src/e2e-ipc.ts keeps the store here)
            const key = 'e2e-store:settings.json';
            if (seedTourDone && localStorage.getItem(key) === null) localStorage.setItem(key, JSON.stringify({ all: { tourDone: true } }));
        }, !tour);
        await page.addInitScript(() => {
            const w = window as unknown as { __copied: string[] };
            w.__copied = [];
            Object.defineProperty(navigator, 'clipboard', {
                value: { writeText: async (t: string) => void w.__copied.push(t) },
                configurable: true,
            });
        });
        await page.goto('/');
        // ready = the script catalogue has loaded (the library's topic list is drawn from it); the tour pastes a
        // sample problem, which hides the library, so its card is the signal there
        // (cold start: several browsers boot in parallel against one test bridge)
        const ready = tour ? page.locator('[data-tour-card]') : page.locator('.topic-row').first();
        await expect(ready).toBeVisible({ timeout: 30_000 });
        await use(page);
    },
});

export { expect };

export async function copied(page: Page): Promise<string[]> {
    return page.evaluate(() => (window as unknown as { __copied: string[] }).__copied);
}

/** Paste text with no field focused, as when the user hits Cmd/Ctrl+V on the window. */
export async function pasteAnywhere(page: Page, text: string): Promise<void> {
    await page.evaluate((t) => {
        (document.activeElement as HTMLElement | null)?.blur();
        const data = new DataTransfer();
        data.setData('text', t);
        window.dispatchEvent(new ClipboardEvent('paste', { clipboardData: data, bubbles: true }));
    }, text);
}

export async function openFromPalette(page: Page, query: string, title: RegExp): Promise<void> {
    await page.keyboard.press('ControlOrMeta+k');
    const box = page.getByRole('combobox', { name: 'Search scripts' });
    await expect(box).toBeFocused();
    await box.fill(query);
    await page.getByRole('dialog', { name: 'Find a script' }).getByRole('option').filter({ hasText: title }).first().click();
}

/** The text input of a field (a select for tool choices is `choice`, a tool's on/off box `switchBox`). */
export function field(page: Page, variable: string) {
    return page.locator(`label.field[data-var="${variable}"]`).locator('input:not([type="checkbox"])');
}

export function choice(page: Page, parameter: string) {
    return page.locator(`label.field[data-var="${parameter}"]`).locator('select');
}

export function switchBox(page: Page, parameter: string) {
    return page.locator(`label.field[data-var="${parameter}"]`).locator('input[type="checkbox"]');
}

export function answer(page: Page) {
    return page.locator('.answer .a-value');
}
