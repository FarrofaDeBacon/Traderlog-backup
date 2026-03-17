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
            growth_phases: []
        } as any];

        settingsStore.assets = [{
            id: 'WDO',
            symbol: 'WDO',
            name: 'Mini Dólar',
            point_value: 10.0,
            default_stop_points: 5,
            min_contracts: 1,
            max_contracts: 10,
            is_root: false,
            asset_type_id: 'x'
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
            fixed_capital: 1000
        } as any];

        settingsStore.assets = [{
            id: 'WIN',
            point_value: 0.20,
            default_stop_points: 0 // NO STOP GIVEN!
        } as any];

        riskStore.activeAssetId = 'WIN';

        const result = riskStore.positionSizingResult;
        expect(result).not.toBeNull();
        
        expect(result?.isValid).toBe(false);
        expect(result?.allowedContracts).toBe(0);
        expect(result?.reasons).toContain("Distância do Stop (stopPoints) deve ser maior que zero.");
    });
});
