import { describe, it, expect, vi, beforeEach } from 'vitest';
import { riskStore } from './riskStore.svelte';
import { settingsStore } from './settings.svelte';

// Mock the Tauri APIs that might get called on settingsStore initialization
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
    message: vi.fn(),
    confirm: vi.fn(),
}));

// We only need to override the getters on settingsStore to mock the inputs
describe('RiskStore Position Sizing Integration', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Reset state
        riskStore.activeAssetId = null;
        settingsStore.riskProfiles = [];
        settingsStore.assets = [];
        settingsStore.accounts = [];
    });

    it('returns null if there is no active profile', () => {
        settingsStore.riskProfiles = []; 
        expect(riskStore.positionSizingInput).toBeNull();
        expect(riskStore.positionSizingResult).toBeNull();
    });

    it('returns null if there is no active asset selected', () => {
        settingsStore.riskProfiles = [{ 
            id: '1', 
            name: 'Test', 
            active: true, 
            max_risk_per_trade_percent: 1,
            capital_source: 'Fixed',
            fixed_capital: 1000,
            growth_plan_enabled: false,
            current_phase_index: 0,
            growth_phases: []
        } as any];

        riskStore.activeAssetId = null;

        expect(riskStore.positionSizingInput).toBeNull();
    });

    it('assembles the input correctly based on selected asset and profile', () => {
        settingsStore.riskProfiles = [{ 
            id: '1', 
            name: 'Test', 
            active: true, 
            max_risk_per_trade_percent: 1.5,
            capital_source: 'Fixed',
            fixed_capital: 5000,
            growth_plan_enabled: false,
            current_phase_index: 0,
            growth_phases: [],
            linked_asset_risk_profile_ids: ['profile-wdo']
        } as any];

        settingsStore.assets = [{
            id: 'WDO',
            symbol: 'WDO',
            name: 'Mini Dólar',
            point_value: 10.0
        } as any];

        settingsStore.assetRiskProfiles = [{
            id: 'profile-wdo',
            asset_id: 'WDO',
            name: 'WDO Base',
            default_stop_points: 5,
            min_contracts: 1,
            max_contracts: 10
        } as any];

        riskStore.activeAssetId = 'WDO';

        const input = riskStore.positionSizingInput;
        expect(input).not.toBeNull();
        if(!input) return;

        expect(input.capital).toBe(5000);
        expect(input.riskPerTradePercent).toBe(1.5);
        expect(input.pointValue).toBe(10.0);
        expect(input.stopPoints).toBe(5);
        expect(input.minContracts).toBe(1);
        expect(input.maxContracts).toBe(10);
    });

    it('computes result securely from the pure engine returning invalid state if stop is zero', () => {
        settingsStore.riskProfiles = [{ 
            active: true, 
            max_risk_per_trade_percent: 1,
            capital_source: 'Fixed',
            fixed_capital: 1000,
            linked_asset_risk_profile_ids: ['profile-win']
        } as any];

        settingsStore.assets = [{
            id: 'WIN',
            point_value: 0.20
        } as any];

        settingsStore.assetRiskProfiles = [{
            id: 'profile-win',
            asset_id: 'WIN',
            name: 'WIN No Stop',
            default_stop_points: 0, // NO STOP GIVEN!
            min_contracts: 1,
            max_contracts: 10
        } as any];

        riskStore.activeAssetId = 'WIN';

        const result = riskStore.positionSizingResult;
        expect(result).not.toBeNull();
        
        expect(result?.isValid).toBe(false);
        expect(result?.allowedContracts).toBe(0);
        expect(result?.reasons).toContain("Distância do Stop (stopPoints) deve ser maior que zero.");
    });

    describe('resolvedGrowthContext', () => {
        it('returns null if there is no active growth plan', () => {
            settingsStore.riskProfiles = [{ 
                id: '1', active: true, growth_plan_enabled: false,
                linked_asset_risk_profile_ids: ['profile-wdo']
            } as any];
            settingsStore.assets = [{ id: 'WDO' } as any];
            settingsStore.assetRiskProfiles = [{ id: 'profile-wdo', asset_id: 'WDO' } as any];
            riskStore.activeAssetId = 'WDO';
            
            expect(riskStore.resolvedGrowthContext).toBeNull();
        });

        it('returns null if there is no active asset profile linked to the asset', () => {
            settingsStore.riskProfiles = [{ 
                id: '1', active: true, growth_plan_enabled: true, growth_phases: [{}], current_phase_index: 0,
                linked_asset_risk_profile_ids: [] // Missing link
            } as any];
            settingsStore.assets = [{ id: 'WDO' } as any];
            settingsStore.assetRiskProfiles = [{ id: 'profile-wdo', asset_id: 'WDO' } as any];
            riskStore.activeAssetId = 'WDO';
            
            expect(riskStore.resolvedGrowthContext).toBeNull();
        });

        it('returns global source when asset profile has no override', () => {
            const globalPhaseMock = { level: 1, lot_size: 5 };
            const profileMock = { 
                id: '1', active: true, growth_plan_enabled: true, 
                growth_phases: [globalPhaseMock], current_phase_index: 0,
                linked_asset_risk_profile_ids: ['profile-wdo']
            };
            const assetMock = { id: 'WDO' };
            const assetProfileMock = { 
                id: 'profile-wdo', asset_id: 'WDO', 
                growth_override_enabled: false 
            };

            settingsStore.riskProfiles = [profileMock as any];
            settingsStore.assets = [assetMock as any];
            settingsStore.assetRiskProfiles = [assetProfileMock as any];
            riskStore.activeAssetId = 'WDO';
            
            const context = riskStore.resolvedGrowthContext;
            
            expect(context).not.toBeNull();
            expect(context?.growthSourceType).toBe('global');
            expect(context?.growthPhase.level).toBe(1);
        });

        it('returns assetProfile source overriding global when enabled and valid', () => {
            const globalPhaseMock = { level: 1, lot_size: 5 };
            const overridePhaseMock = { level: 2, lot_size: 15 }; // Override phase
            
            const profileMock = { 
                id: '1', active: true, growth_plan_enabled: true, 
                growth_phases: [globalPhaseMock], current_phase_index: 0,
                linked_asset_risk_profile_ids: ['profile-wdo']
            };
            const assetMock = { id: 'WDO' };
            
            const assetProfileMock = { 
                id: 'profile-wdo', asset_id: 'WDO', 
                growth_override_enabled: true,
                growth_phases_override: [overridePhaseMock],
                current_phase_index: 0
            };

            settingsStore.riskProfiles = [profileMock as any];
            settingsStore.assets = [assetMock as any];
            settingsStore.assetRiskProfiles = [assetProfileMock as any];
            riskStore.activeAssetId = 'WDO';
            
            const context = riskStore.resolvedGrowthContext;
            
            expect(context).not.toBeNull();
            expect(context?.growthSourceType).toBe('assetProfile');
            expect(context?.growthPhase.level).toBe(2); // Should pick the override
        });
    });
});
