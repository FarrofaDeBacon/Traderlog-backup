import { chromium } from 'playwright';

(async () => {
    const browser = await chromium.launch();
    const context = await browser.newContext({
        viewport: { width: 1920, height: 1080 },
        locale: 'pt-BR'
    });
    
    await context.addInitScript(() => {
        try {
            localStorage.setItem('traderlogpro_settings', JSON.stringify({
                state: { userProfile: { language: 'pt' } }
            }));
        } catch (e) {}
    });

    const page = await context.newPage();

    console.log("Navigating to Fiscal Rules...");
    await page.goto('http://127.0.0.1:3005/settings/fiscal/rules', { waitUntil: 'networkidle' });
    await page.waitForTimeout(3000); 
    console.log("Capturing Fiscal Rules modal...");
    await page.screenshot({ path: 'docs/assets/settings/fiscal/rules/modal_add.png' });

    console.log("Navigating to Risk Profile...");
    await page.goto('http://127.0.0.1:3005/settings/risk', { waitUntil: 'networkidle' });
    await page.waitForTimeout(3000); 
    console.log("Capturing Risk Profile modal...");
    await page.screenshot({ path: 'docs/assets/settings/risk/modal_add.png' });

    await browser.close();
    console.log("Captures complete.");
})();
