import { test, expect } from '@playwright/test';

test('capture risk modal', async ({ page }) => {
  // Inject localStorage to bypass setup
  await page.addInitScript(() => {
    window.localStorage.setItem('onboarding_completed', 'true');
    window.localStorage.setItem('app_settings', JSON.stringify({
      userId: "1",
      language: "ptBR",
      theme: "dark"
    }));
    window.__TAURI_INTERNALS__ = {
      invoke: function(cmd, args) {
        console.log("Mocked Tauri invoke:", cmd, args);
        return Promise.resolve();
      }
    };
  });

  await page.goto('http://localhost:4173/risk-control');
  
  // Wait for the manage button and click it
  const manageButton = page.locator('button:has-text("Gerenciar")').first();
  await manageButton.waitFor({ state: 'visible', timeout: 5000 }).catch(() => null);
  
  if (await manageButton.isVisible()) {
      await manageButton.click();
  } else {
      // Try alternate selector if text isn't exact
      const altButton = page.locator('button.bg-primary\\/10').first();
      await altButton.click();
  }

  // Wait for modal to open
  await page.waitForTimeout(1000);

  // Take screenshot of the first tab
  await page.locator('[role="dialog"]').screenshot({ path: 'C:/Users/Farrofa DeBacon/.gemini/antigravity/brain/1d9cc20e-c98c-42d1-ba2f-5c4c8b16079a/risk_modal_tab_general.png' });

  // Click Risk Engine tab and capture
  await page.click('button[value="risk-engine"]');
  await page.waitForTimeout(500);
  await page.locator('[role="dialog"]').screenshot({ path: 'C:/Users/Farrofa DeBacon/.gemini/antigravity/brain/1d9cc20e-c98c-42d1-ba2f-5c4c8b16079a/risk_modal_tab_engine.png' });

  // Click Growth Plan tab and capture
  await page.click('button[value="growth"]');
  await page.waitForTimeout(500);
  await page.locator('[role="dialog"]').screenshot({ path: 'C:/Users/Farrofa DeBacon/.gemini/antigravity/brain/1d9cc20e-c98c-42d1-ba2f-5c4c8b16079a/risk_modal_tab_growth.png' });
});
