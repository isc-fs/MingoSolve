// Shared helpers: a page whose clipboard writes are captured, and small actions the journeys reuse.
import { test as base, expect, type Page } from '@playwright/test';

export const test = base.extend<{ app: Page }>({
    app: async ({ page }, use) => {
        await page.addInitScript(() => {
            const w = window as unknown as { __copied: string[] };
            w.__copied = [];
            Object.defineProperty(navigator, 'clipboard', {
                value: { writeText: async (t: string) => void w.__copied.push(t) },
                configurable: true,
            });
        });
        await page.goto('/');
        // ready = the script catalogue has loaded (the topic grid is drawn from it)
        // (cold start: several browsers boot in parallel against one test bridge)
        await expect(page.locator('.topic').first()).toBeVisible({ timeout: 30_000 });
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
    const box = page.getByRole('textbox', { name: 'Search scripts' });
    await expect(box).toBeFocused();
    await box.fill(query);
    await page.getByRole('option').filter({ hasText: title }).first().click();
}

export function field(page: Page, variable: string) {
    return page.locator('label.field').filter({ has: page.locator('.var', { hasText: new RegExp(`^${variable}$`) }) }).locator('input');
}

export function answer(page: Page) {
    return page.locator('.answer .a-value');
}
