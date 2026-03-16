import { test, expect } from '@playwright/test';
import path from 'path';
import fs from 'fs';

const BASE_URL = 'http://localhost:3005';
const ASSETS_DIR = path.join(process.cwd(), 'docs', 'assets');

test.describe('Capture pt-BR Screenshots', () => {

    test.beforeAll(() => {
        if (!fs.existsSync(ASSETS_DIR)) {
            fs.mkdirSync(ASSETS_DIR, { recursive: true });
        }
    });

    test.use({
        colorScheme: 'dark',
        viewport: { width: 1920, height: 1080 },
        locale: 'pt-BR',
        timezoneId: 'America/Sao_Paulo'
    });

    async function preparePage(page: any) {
        await page.evaluate(() => {
            localStorage.setItem('theme', 'dark');
            localStorage.setItem('locale', 'pt-BR');
            localStorage.setItem('sidebar_state', JSON.stringify({ isCollapsed: false }));
        });
    }

    test('Capture page_dashboard.png', async ({ page }) => {
        console.log(`Navigating to ${BASE_URL}/...`);
        await page.goto(`${BASE_URL}/`, { waitUntil: 'domcontentloaded' });
        await preparePage(page);
        await page.reload({ waitUntil: 'networkidle' });

        await page.waitForTimeout(5000); // Wait for animations and charts
        try {
            await page.waitForSelector('canvas', { state: 'attached', timeout: 5000 });
        } catch (e) {
            console.log("No canvas found or timed out, proceeding anyway.");
        }
        await page.waitForTimeout(2000); // Extra time for canvas rendering

        const dest = path.join(ASSETS_DIR, 'page_dashboard.png');
        await page.screenshot({ path: dest, fullPage: true });
        console.log(`Saved screenshot: page_dashboard.png`);
    });
});
