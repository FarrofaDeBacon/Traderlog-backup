import asyncio
from playwright.async_api import async_playwright
import json

async def run():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context()
        page = await context.new_page()

        print("Setting up localStorage to bypass setup wizard...")
        # Inject state before navigation
        await page.add_init_script("""
            window.localStorage.setItem('onboarding_completed', 'true');
            window.localStorage.setItem('app_settings', JSON.stringify({
                userId: "1",
                language: "ptBR",
                theme: "dark"
            }));
            
            // Mock Tauri invoke to prevent app crash
            window.__TAURI_INTERNALS__ = {
                invoke: function(cmd, args) {
                    console.log("Mocked Tauri invoke:", cmd, args);
                    return Promise.resolve();
                }
            };
        """)

        print("Navigating to /risk-control...")
        await page.goto("http://localhost:4173/risk-control", wait_until="networkidle")
        
        # Wait a bit longer for Svelte to mount
        await page.waitForTimeout(2000)

        print("Waiting for generic DOM loading...")
        
        try:
             # Look for the manage button and click it to open the modal
             await page.click("button:has-text('Gerenciar')", timeout=5000)
             print("Clicked the manage button.")
             await page.waitForTimeout(1000)
        except Exception as e:
             print(f"Could not click manage button: {e}")
             # try to find a button with the wheel icon
             try:
                 # It might be an SVG. Let's look for button.
                 await page.click("button.bg-primary\\/10", timeout=5000)
                 print("Clicked a button that might be the settings.")
                 await page.waitForTimeout(1000)
             except Exception as e2:
                 print(f"Could not click alternative button either: {e2}")

        print("Capturing screenshot...")
        await page.screenshot(path="C:\\Users\\Farrofa DeBacon\\.gemini\\antigravity\\brain\\1d9cc20e-c98c-42d1-ba2f-5c4c8b16079a\\risk_settings_modal_capture.png", full_page=True)

        await browser.close()
        print("Done.")

if __name__ == "__main__":
    asyncio.run(run())
