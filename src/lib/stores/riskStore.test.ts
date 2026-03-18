import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
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

    describe('Risk Flow Integration Tests (ETAPA 10)', () => {
        it('Scenario A: Asset selected, NO AssetRiskProfile linked', () => {
            settingsStore.riskProfiles = [{ 
                id: 'global-1', active: true, growth_plan_enabled: true, growth_phases: [{}], current_phase_index: 0,
                // Missing link to 'profile-wdo'
                linked_asset_risk_profile_ids: [] 
            } as any];
            settingsStore.assets = [{ id: 'WDO', symbol: 'WDO', point_value: 10 } as any];
            settingsStore.assetRiskProfiles = [{ id: 'profile-wdo', asset_id: 'WDO' } as any];
            
            riskStore.activeAssetId = 'WDO';
            
            expect(riskStore.resolvedAssetRiskProfile).toBeNull();
            expect(riskStore.resolvedGrowthContext).toBeNull();
            expect(riskStore.positionSizingResult).toBeNull();
        });

        it('Scenario B: Asset selected, AssetRiskProfile linked, Global Growth', () => {
            settingsStore.riskProfiles = [{ 
                id: 'global-1', active: true, growth_plan_enabled: true, 
                growth_phases: [{ level: 1, lot_size: 5 }], current_phase_index: 0,
                linked_asset_risk_profile_ids: ['profile-wdo']
            } as any];
            settingsStore.assets = [{ id: 'WDO', symbol: 'WDO', point_value: 10 } as any];
            settingsStore.assetRiskProfiles = [{ 
                id: 'profile-wdo', asset_id: 'WDO', 
                growth_override_enabled: false 
            } as any];
            
            riskStore.activeAssetId = 'WDO';
            
            expect(riskStore.resolvedGrowthContext?.growthSourceType).toBe('global');
            expect(riskStore.resolvedGrowthContext?.growthPhase.level).toBe(1);
        });

        it('Scenario C: Asset selected, AssetRiskProfile linked, Override Growth', () => {
            settingsStore.riskProfiles = [{ 
                id: 'global-1', active: true, growth_plan_enabled: true, 
                growth_phases: [{ level: 1, lot_size: 5 }], current_phase_index: 0,
                linked_asset_risk_profile_ids: ['profile-wdo']
            } as any];
            settingsStore.assets = [{ id: 'WDO', symbol: 'WDO', point_value: 10 } as any];
            settingsStore.assetRiskProfiles = [{ 
                id: 'profile-wdo', asset_id: 'WDO', 
                growth_override_enabled: true,
                growth_phases_override: [{ level: 1, lot_size: 15 }], // Override
                current_phase_index: 0
            } as any];
            
            riskStore.activeAssetId = 'WDO';
            
            expect(riskStore.resolvedGrowthContext?.growthSourceType).toBe('assetProfile');
            expect(riskStore.resolvedGrowthContext?.growthPhase.lot_size).toBe(15);
        });

        it('Scenario D: Asset selected, valid config, output isValid = true with allowedContracts', () => {
             settingsStore.riskProfiles = [{ 
                id: 'global-1', active: true, max_risk_per_trade_percent: 2.0, capital_source: 'Fixed', fixed_capital: 10000,
                linked_asset_risk_profile_ids: ['profile-wdo']
            } as any];
            settingsStore.assets = [{ id: 'WDO', symbol: 'WDO', point_value: 10.0 } as any];
            settingsStore.assetRiskProfiles = [{ 
                id: 'profile-wdo', asset_id: 'WDO', default_stop_points: 10, min_contracts: 1, max_contracts: 50
            } as any];
            
            riskStore.activeAssetId = 'WDO';
            
            const positionSizing = riskStore.positionSizingResult;
            expect(positionSizing).not.toBeNull();
            expect(positionSizing?.isValid).toBe(true);
            
            // Calc: 10000 * 2% = $200
            // Risk per contract: 10 points * $10 = $100
            // Allowed: $200 / $100 = 2 contracts
            expect(positionSizing?.allowedContracts).toBe(2);
        });

        it('Scenario E: Asset selected, missing stop, output isValid = false with reasons', () => {
             settingsStore.riskProfiles = [{ 
                id: 'global-1', active: true, max_risk_per_trade_percent: 1.0, capital_source: 'Fixed', fixed_capital: 1000,
                linked_asset_risk_profile_ids: ['profile-win']
            } as any];
            settingsStore.assets = [{ id: 'WIN', symbol: 'WIN', point_value: 0.20 } as any];
            settingsStore.assetRiskProfiles = [{ id: 'profile-win', asset_id: 'WIN', default_stop_points: 0, min_contracts: 1, max_contracts: 10 } as any];
            
            riskStore.activeAssetId = 'WIN';
            
            const positionSizing = riskStore.positionSizingResult;
            expect(positionSizing).not.toBeNull();
            expect(positionSizing?.isValid).toBe(false);
            expect(positionSizing?.allowedContracts).toBe(0);
            expect(positionSizing?.reasons.length).toBeGreaterThan(0);
        });

        it('Scenario F: Asset change triggers context switch', () => {
            settingsStore.riskProfiles = [{ 
                id: 'global-1', active: true, growth_plan_enabled: true, growth_phases: [{ level: 1, lot_size: 5 }], current_phase_index: 0,
                linked_asset_risk_profile_ids: ['profile-wdo', 'profile-win']
            } as any];
            
            settingsStore.assets = [
                { id: 'WDO', symbol: 'WDO', point_value: 10 } as any,
                { id: 'WIN', symbol: 'WIN', point_value: 0.20 } as any
            ];
            
            settingsStore.assetRiskProfiles = [
                { id: 'profile-wdo', asset_id: 'WDO', growth_override_enabled: false } as any, // Uses global
                { id: 'profile-win', asset_id: 'WIN', growth_override_enabled: true, growth_phases_override: [{ level: 1, lot_size: 99 }], current_phase_index: 0 } as any // Uses override
            ];
            
            // Select WDO
            riskStore.activeAssetId = 'WDO';
            expect(riskStore.resolvedAssetRiskProfile?.asset_id).toBe('WDO');
            expect(riskStore.resolvedGrowthContext?.growthSourceType).toBe('global');
            
            // Switch to WIN
            riskStore.activeAssetId = 'WIN';
            expect(riskStore.resolvedAssetRiskProfile?.asset_id).toBe('WIN');
            expect(riskStore.resolvedGrowthContext?.growthSourceType).toBe('assetProfile');
            expect(riskStore.resolvedGrowthContext?.growthPhase.lot_size).toBe(99);
        });
    });
});

describe('RiskStore Persistence (activeAssetId)', () => {
    let mockStore: Record<string, string> = {};

    beforeEach(() => {
        mockStore = {};
        vi.stubGlobal('localStorage', {
            getItem: (key: string) => mockStore[key] || null,
            setItem: (key: string, value: string) => { mockStore[key] = value.toString(); },
            removeItem: (key: string) => { delete mockStore[key]; },
            clear: () => { mockStore = {}; },
        });
        settingsStore.assets = [];
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it('restores activeAssetId from localStorage if present upon initialization', async () => {
        mockStore['risk_activeAssetId'] = 'WDO';
        
        const { RiskStore } = await import('./riskStore.svelte');
        const testStore = new RiskStore();

        expect(testStore.activeAssetId).toBe('WDO');
    });

    it('cleans activeAssetId if the linked asset no longer exists in settingsStore', async () => {
        mockStore['risk_activeAssetId'] = 'DEAD_ASSET';
        
        settingsStore.assets = [
            { id: 'WDO', symbol: 'WDO', point_value: 10 } as any
        ];
        
        const { RiskStore } = await import('./riskStore.svelte');
        const testStore = new RiskStore();
        
        // Wait a tick for $effect.root
        await new Promise(r => setTimeout(r, 0));
        
        expect(testStore.activeAssetId).toBeNull();
        expect(mockStore['risk_activeAssetId']).toBeUndefined(); // or not existing
    });
});
