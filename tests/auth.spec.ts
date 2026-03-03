import { test, expect } from '@playwright/test';
import { mockTauri } from './utils/tauriMock';

test.describe('Authentication & Onboarding Flow', () => {
    test('should load the app and land on welcome wizard if fresh', async ({ page }) => {
        // Pipe browser console to terminal
        page.on('console', msg => console.log(`[BROWSER] ${msg.text()}`));

        // Reset state for onboarding
        await mockTauri(page, { onboardingCompleted: false });

        // Go to the home page
        await page.goto('/');

        // Check for title (Wait for SvelteKit to load)
        // The current title is "Tauri + SvelteKit + Typescript App"
        await expect(page).toHaveTitle(/Tauri|TraderLogPro/);

        // Initial check for loading state
        await page.waitForLoadState('networkidle');

        // Should see OnboardingWizard
        // Use a more generic locator that matches the translated or English text
        const onboardingMatch = page.getByText(/Setup Wizard/i).or(page.getByText(/Bem-vindo/i)).or(page.getByText(/TraderLog Pro/i));

        try {
            await expect(onboardingMatch.first()).toBeVisible({ timeout: 15000 });
            console.log('App started and shows OnboardingWizard correctly');
        } catch (e) {
            console.error('Failed to find OnboardingWizard. Page content snapshot:');
            console.log(await page.content());
            throw e;
        }
    });

    test('should navigate to settings and back when onboarding is done', async ({ page }) => {
        page.on('console', msg => console.log(`[BROWSER] ${msg.text()}`));

        await mockTauri(page, { onboardingCompleted: true });

        await page.goto('/');
        await page.waitForLoadState('networkidle');

        // Find settings link in sidebar - using Role is more robust
        const settingsLink = page.getByRole('link', { name: /Configurações|Settings/i });

        try {
            await expect(settingsLink.first()).toBeVisible({ timeout: 15000 });
            await settingsLink.first().click();

            // Verify URL
            await expect(page).toHaveURL(/\/settings/);

            // Go back to dashboard (home)
            const dashboardLink = page.getByRole('link', { name: /Dashboard/i });
            await expect(dashboardLink.first()).toBeVisible();
            await dashboardLink.first().click();
            await expect(page).toHaveURL(/\//);
        } catch (e) {
            console.error('Failed to navigate. Nav HTML:');
            console.log(await page.locator('nav').innerHTML().catch(() => 'Nav not found'));
            throw e;
        }
    });

    test('should complete the full onboarding flow', async ({ page }) => {
        page.on('console', msg => console.log(`[BROWSER] ${msg.text()}`));
        await mockTauri(page, { onboardingCompleted: false });
        await page.goto('/');

        // Step 1: Welcome & Language
        await page.getByRole('button', { name: /Próximo|Next/i }).click();

        // Step 2: Profile
        await page.fill('input#name', 'Playwright Tester');
        await page.fill('input#email', 'playwright@test.com');
        await page.fill('input#password', 'password123');
        await page.fill('input#confirmPassword', 'password123');
        await page.getByRole('button', { name: /Próximo|Next/i }).click();

        // Step 3: Recovery Key
        await page.getByRole('button', { name: /Próximo|Next/i }).click();

        // Step 4: License
        await page.getByRole('button', { name: /Pular|Skip|Próximo/i }).click();

        // Step 5: Finish
        const finishBtn = page.getByRole('button', { name: /Começar Agora|Start Now/i });
        await expect(finishBtn).toBeVisible();
        await finishBtn.click();

        // Wait for Dashboard (sidebar)
        const dashboardLink = page.getByRole('link', { name: /Dashboard/i });
        await expect(dashboardLink.first()).toBeVisible({ timeout: 20000 });
        await expect(page).toHaveURL(/\//);
    });
});
