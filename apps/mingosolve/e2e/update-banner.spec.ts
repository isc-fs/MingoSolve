// The update banner (fake updater, see e2e-ipc.ts) must not sit in the macOS overlay title-bar zone. WebKit's user
// agent is macOS, so the app applies data-platform="mac" there; Chromium's is Linux.
import { test as base, expect } from '@playwright/test';

const test = base.extend({
    page: async ({ page }, use) => {
        await page.addInitScript(() => localStorage.setItem('e2e-update', 'available'));
        await page.goto('/');
        await expect(page.locator('.topic-row').first()).toBeVisible({ timeout: 30_000 });
        await use(page);
    },
});

test('the banner is below the title-bar zone on macOS and flush with the main column elsewhere', async ({ page }) => {
    const banner = page.getByRole('status').filter({ hasText: 'v9.9.9' });
    await expect(banner).toBeVisible();
    const platform = await page.evaluate(() => document.documentElement.dataset.platform);
    const b = (await banner.boundingBox())!;
    const m = (await page.locator('main').boundingBox())!;
    expect(b.x).toBeGreaterThanOrEqual(m.x);
    expect(b.x + b.width).toBeLessThanOrEqual(m.x + m.width);
    if (platform === 'mac') {
        // traffic lights at x=20,y=26 + the 28px drag strip: nothing of the banner may touch x 0-90, y 0-44
        expect(b.y).toBeGreaterThanOrEqual(44);
        const hitsWindowButtons = b.x < 90 && b.y < 44;
        expect(hitsWindowButtons).toBe(false);
    } else {
        expect(b.y - m.y).toBeLessThanOrEqual(16);
    }
    await expect(banner.getByRole('button', { name: 'Install & restart' })).toBeVisible();
    await expect(banner.getByRole('link', { name: "What's new" })).toBeVisible();
    // the view below is not covered
    const view = (await page.locator('main .view').first().boundingBox())!;
    expect(view.y).toBeGreaterThanOrEqual(b.y + b.height - 1);
});

test('Install shows download progress', async ({ page }) => {
    const banner = page.getByRole('status').filter({ hasText: 'v9.9.9' });
    await banner.getByRole('button', { name: 'Install & restart' }).click();
    await expect(banner).toContainText('Downloading 40%');
});

test('Later dismisses the banner', async ({ page }) => {
    await page.getByRole('button', { name: 'Later' }).click();
    await expect(page.getByRole('status').filter({ hasText: 'v9.9.9' })).toBeHidden();
});

test('a failed download shows the error with Retry and Dismiss', async ({ page }) => {
    await page.evaluate(() => localStorage.setItem('e2e-update-fail', '1'));
    const banner = page.getByRole('status').filter({ hasText: 'v9.9.9' });
    await banner.getByRole('button', { name: 'Install & restart' }).click();
    await expect(banner).toContainText('failed');
    await expect(banner.getByRole('button', { name: 'Retry' })).toBeVisible();
    await banner.getByRole('button', { name: 'Dismiss' }).click();
    await expect(page.getByRole('status').filter({ hasText: 'v9.9.9' })).toBeHidden();
});
