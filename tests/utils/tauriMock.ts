import type { Page } from '@playwright/test';

/**
 * Mocks Tauri 2.0 internals for Playwright tests in a standard browser.
 */
export async function mockTauri(page: Page, options: { onboardingCompleted?: boolean } = {}) {
    const { onboardingCompleted = true } = options;

    await page.addInitScript((args) => {
        // Force logged in state to bypass LoginView
        window.localStorage.setItem('isLoggedIn', 'true');
        window.localStorage.setItem('locale', 'pt-BR');

        const mockState = {
            onboardingCompleted: args.onboardingCompleted
        };

        const invokeHandler = async (cmd: string, invokeArgs: any) => {
            console.log('[MOCK INVOKE]', cmd, invokeArgs);

            const mocks: Record<string, any> = {
                'get_user_profile': {
                    id: 'main',
                    name: 'Test Trader',
                    email: 'test@example.com',
                    onboarding_completed: mockState.onboardingCompleted,
                    theme: 'dark',
                    main_currency: 'BRL',
                    trial_start_date: new Date().toISOString(),
                    password_hash: "hashed_pass"
                },
                'get_machine_id_cmd': 'MOCK-HWID-1234',
                'get_accounts': [{ id: 'acc1', nickname: 'Mock Acc', currency: 'BRL', balance: 1000 }],
                'get_currencies': [{ code: 'BRL', symbol: 'R$', exchange_rate: 1.0 }],
                'get_trades': [],
                'ensure_defaults': null,
                'save_user_profile': null,
                'seed_demo_data': null,
                'complete_onboarding': null,
                'diagnostic_dump_trades': null,
                'diagnostic_dump_users': null
            };

            if (cmd in mocks) return mocks[cmd];

            // Default returns for generic gets to avoid breaking dataLoaded Promise.all
            if (cmd.startsWith('get_')) return [];

            console.warn(`[MOCK INVOKE] UNHANDLED COMMAND: ${cmd}`);
            return null;
        };

        // Polyfill all known Tauri 2.0 / 1.0 IPC entry points
        (window as any).__TAURI_INTERNALS__ = {
            invoke: invokeHandler
        };

        (window as any).__TAURI__ = {
            core: {
                invoke: invokeHandler
            }
        };

        // Tauri 2.0 uses window.__TAURI_IPC__ for its core messaging
        (window as any).__TAURI_IPC__ = async (message: any) => {
            const { cmd, callback, error, ...payload } = message;
            try {
                const result = await invokeHandler(cmd, payload);
                console.log(`[MOCK IPC] ${cmd} -> SUCCESS`, result);
                (window as any)[`_${callback}`](result);
            } catch (e) {
                console.error(`[MOCK IPC] ${cmd} -> ERROR`, e);
                (window as any)[`_${error}`](e);
            }
        };

        console.log('[MOCK] Tauri internals initialized');
    }, { onboardingCompleted });
}
