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
        // CRITICAL for detached windows: should start empty to avoid theme flash/override
        expect(settingsStore.userProfile.theme).toBe('');
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

    describe('duplicateRiskProfile', () => {
        it('should deep clone a risk profile, including linked asset profiles, with new IDs', async () => {
            const originalProfileId = crypto.randomUUID();
            const originalAssetProfileId = crypto.randomUUID();

            settingsStore.riskProfiles = [{
                id: originalProfileId,
                name: 'Original Profile',
                active: true,
                max_daily_loss: 1000,
                daily_target: 2000,
                max_risk_per_trade_percent: 1,
                max_trades_per_day: 5,
                min_risk_reward: 2,
                lock_on_loss: true,
                target_type: 'Financial',
                capital_source: 'Fixed',
                fixed_capital: 10000,
                linked_account_id: 'acc1',
                growth_plan_enabled: true,
                current_phase_index: 0,
                growth_phases: [{ id: 'gp1', level: 1, name: 'Phase 1', lot_size: 1, conditions_to_advance: [], conditions_to_demote: [] }],
                psychological_coupling_enabled: false,
                outlier_regression_enabled: false,
                sniper_mode_enabled: false,
                sniper_mode_selectivity: 3,
                psychological_lookback_count: 10,
                outlier_lookback_count: 20,
                psychological_threshold: -2,
                lot_reduction_multiplier: 0.5,
                psychological_search_strategy: 'Strict',
                account_type_applicability: 'All',
                account_ids: [],
                linked_asset_risk_profile_ids: [originalAssetProfileId]
            }];

            settingsStore.assetRiskProfiles = [{
                id: originalAssetProfileId,
                asset_id: 'asset1',
                name: 'Original Asset Profile',
                default_stop_points: 100,
                min_contracts: 1,
                max_contracts: 10,
                growth_override_enabled: true,
                growth_phases_override: [{ id: 'ago1', level: 1, name: 'Asset Phase 1', lot_size: 2, conditions_to_advance: [], conditions_to_demote: [] }]
            }];

            const uuidSpy = vi.spyOn(crypto, 'randomUUID');

            const newId = await settingsStore.duplicateRiskProfile(originalProfileId);

            expect(newId).toBeTruthy();
            expect(newId).not.toBe(originalProfileId);

            const clonedProfile = settingsStore.riskProfiles.find(p => p.id === newId);
            expect(clonedProfile).toBeDefined();
            expect(clonedProfile!.name).toBe('Original Profile (Cópia)');
            expect(clonedProfile!.active).toBe(false); 
            expect(clonedProfile!.max_daily_loss).toBe(1000);
            
            expect(clonedProfile!.growth_phases?.length).toBe(1);
            expect(clonedProfile!.growth_phases![0].id).not.toBe('gp1');
            expect(clonedProfile!.growth_phases![0].name).toBe('Phase 1');

            expect(clonedProfile!.linked_asset_risk_profile_ids?.length).toBe(1);
            const newAssetProfileId = clonedProfile!.linked_asset_risk_profile_ids![0];
            expect(newAssetProfileId).not.toBe(originalAssetProfileId);

            const clonedAssetProfile = settingsStore.assetRiskProfiles.find(a => a.id === newAssetProfileId);
            expect(clonedAssetProfile).toBeDefined();
            expect(clonedAssetProfile!.asset_id).toBe('asset1');
            
            expect(clonedAssetProfile!.growth_phases_override?.length).toBe(1);
            expect(clonedAssetProfile!.growth_phases_override![0].id).not.toBe('ago1');

            uuidSpy.mockRestore();
        });
    });
});
