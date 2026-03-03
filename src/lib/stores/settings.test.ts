import { describe, it, expect, vi, beforeEach } from 'vitest';
import { settingsStore } from './settings.svelte';

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
    message: vi.fn(),
    confirm: vi.fn(),
}));

// Mock License Utility
vi.mock('$lib/utils/license', () => ({
    validateLicenseKey: vi.fn().mockResolvedValue({ valid: true, plan: "Pro", expiration: null }),
    computeCustomerId: vi.fn().mockResolvedValue("MOCK_CID"),
}));

describe('SettingsStore Unit Tests', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Reset state for each test if possible, or handle dirty state
    });

    it('should initialize with default userProfile values', () => {
        // userProfile is initialized with BRL and pt-BR
        expect(settingsStore.userProfile.language).toBe('pt-BR');
        expect(settingsStore.userProfile.main_currency).toBe('BRL');
    });

    it('should calculate trial days correctly', () => {
        const now = new Date();
        const trialStart = new Date(now);
        trialStart.setDate(now.getDate() - 2); // 2 days ago

        settingsStore.userProfile.trial_start_date = trialStart.toISOString();

        // Code has: Math.max(0, 7 - diffDays)
        // 7 - 2 = 5
        expect(settingsStore.trialDaysLeft).toBe(5);
    });

    it('should handle expired trial correctly', () => {
        const now = new Date();
        const trialStart = new Date(now);
        trialStart.setDate(now.getDate() - 10); // 10 days ago (limit is 7)

        settingsStore.userProfile.trial_start_date = trialStart.toISOString();
        expect(settingsStore.trialDaysLeft).toBe(0);
        expect(settingsStore.licenseStatus).toBe('expired');
    });

    it('should calculate license days remaining correctly', () => {
        const now = new Date();
        const expiration = new Date(now);
        expiration.setDate(now.getDate() + 15); // Expiring in 15 days

        settingsStore.licenseDetails = {
            valid: true,
            plan: "Pro",
            expiration: expiration.toISOString(),
            createdAt: now.toISOString()
        };

        expect(settingsStore.licenseDaysRemaining).toBe(15);
        expect(settingsStore.licenseStatus).toBe('active');
    });

    it('should handle lifetime licenses correctly', () => {
        settingsStore.licenseDetails = {
            valid: true,
            plan: "Lifetime",
            expiration: null,
            createdAt: new Date().toISOString()
        };

        expect(settingsStore.licensePlanName).toBe('Lifetime');
        expect(settingsStore.licenseDaysRemaining).toBeNull();
        expect(settingsStore.licenseTotalDays).toBeNull();
    });

    it('should auto-detect asset types and point values in ensureAssetExists', () => {
        // Mock dependencies
        settingsStore.assetTypes = [
            { id: 'type_index', name: 'Contrato Futuro', code: 'INDEX' },
            { id: 'type_stock', name: 'Ações', code: 'STK' }
        ] as any;
        settingsStore.assets = [];

        // Mock crypto.randomUUID
        const uuidSpy = vi.spyOn(crypto, 'randomUUID').mockReturnValue('mock-uuid' as any);

        // Test Mini Index (WIN)
        settingsStore.ensureAssetExists('WINJ24');
        let asset = settingsStore.assets.find(a => a.symbol === 'WINJ24');
        expect(asset).toBeDefined();
        expect(asset?.asset_type_id).toBe('type_index');
        expect(asset?.point_value).toBe(0.20);

        // Test Mini Dollar (WDO)
        settingsStore.ensureAssetExists('WDOK24');
        asset = settingsStore.assets.find(a => a.symbol === 'WDOK24');
        expect(asset?.point_value).toBe(10.0);

        // Test Stocks (PETR4)
        settingsStore.ensureAssetExists('PETR4');
        asset = settingsStore.assets.find(a => a.symbol === 'PETR4');
        expect(asset?.asset_type_id).toBe('type_stock');
        expect(asset?.point_value).toBe(1.0);

        uuidSpy.mockRestore();
    });
});
