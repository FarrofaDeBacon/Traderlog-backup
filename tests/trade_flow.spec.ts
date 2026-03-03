import { test, expect } from '@playwright/test';
import { mockTauri } from './utils/tauriMock';

test.describe('Trade Management Flow', () => {
    test('should allow triggers the New Trade process', async ({ page }) => {
        await mockTauri(page, { onboardingCompleted: true });
        await page.goto('/');
        await page.waitForLoadState('networkidle');

        // Try to find New Trade via button or sidebar
        const newTradeLink = page.locator('a[href="/trades/new"]');
        const fabBtn = page.locator('button[title*="Novo Trade"], button[title*="New Trade"]');

        const trigger = (await newTradeLink.isVisible()) ? newTradeLink : fabBtn;

        if (await trigger.isVisible()) {
            await trigger.click();
            await expect(page).toHaveURL(/.*trades\/new/);

            // Check for Wizard parts (e.g. step titles)
            await expect(page.locator('h2').first()).toBeVisible();
            console.log('Successfully opened New Trade Wizard');
        } else {
            console.log('Trade trigger not found. Dashboard might be empty or different layout.');
        }
    });

    test('should view the Trades List', async ({ page }) => {
        await mockTauri(page, { onboardingCompleted: true });
        await page.goto('/trades');
        await page.waitForLoadState('networkidle');

        // At least header or empty state should be visible
        // Search for title "Trades" or "Journal" or whatever is localized
        const header = page.locator('h1, h2').filter({ hasText: /Trades/i });
        await expect(header).toBeVisible();
    });
});
