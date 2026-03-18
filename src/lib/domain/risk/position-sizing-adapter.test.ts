import { describe, it, expect } from 'vitest';
import { adaptPositionSizingInput } from './position-sizing-adapter';
import type { RiskProfile, Asset, AssetRiskProfile, GrowthPhase } from '$lib/types';

describe('Position Sizing Adapter', () => {
    
    // Mock base configuration objects
    const baseProfile: Partial<RiskProfile> = {
        max_risk_per_trade_percent: 1.5,
        capital_source: 'Fixed',
        fixed_capital: 10000,
        growth_plan_enabled: true,
        current_phase_index: 0
    };

    const baseAsset: Partial<Asset> = {
        id: 'WIN',
        point_value: 0.20
    };

    const baseAssetRiskProfile: Partial<AssetRiskProfile> = {
        id: 'win-profile-1',
        asset_id: 'WIN',
        default_stop_points: 150,
        min_contracts: 1,
        max_contracts: 100
    };

    const basePhase: Partial<GrowthPhase> = {
        lot_size: 5
    };

    it('extracts input correctly when all configs exist', () => {
        const input = adaptPositionSizingInput(
            baseProfile as RiskProfile, 
            baseAsset as Asset,
            baseAssetRiskProfile as AssetRiskProfile,
            basePhase as GrowthPhase,
            undefined // Account balance overrides
        );

        expect(input).toBeDefined();
        if(!input) return;

        expect(input.capital).toBe(10000);
        expect(input.riskPerTradePercent).toBe(1.5);
        expect(input.stopPoints).toBe(150); // From AssetRiskProfile
        expect(input.pointValue).toBe(0.20); // From Asset
        expect(input.minContracts).toBe(1); // From AssetRiskProfile
        expect(input.maxContracts).toBe(100); // From AssetRiskProfile
        expect(input.maxContractsPhase).toBe(5); // From Phase
    });

    it('returns null if profile capital_source is missing override when needed', () => {
        // Linked Account but we didn't pass the linked balance
        const linkProfile = { ...baseProfile, capital_source: 'LinkedAccount', linked_account_id: 'x' };
        
        const input = adaptPositionSizingInput(
            linkProfile as RiskProfile, 
            baseAsset as Asset,
            baseAssetRiskProfile as AssetRiskProfile,
            undefined,
            undefined // Missing parameter from store layer
        );

        expect(input).toBeNull();
    });

    it('accepts balance overrides if capital_source is LinkedAccount', () => {
        const linkProfile = { ...baseProfile, capital_source: 'LinkedAccount', linked_account_id: 'x' };
        
        const input = adaptPositionSizingInput(
            linkProfile as RiskProfile, 
            baseAsset as Asset,
            baseAssetRiskProfile as AssetRiskProfile,
            undefined,
            25000 // Valid external balance
        );

        expect(input).toBeDefined();
        if(!input) return;
        
        expect(input.capital).toBe(25000);
    });

    it('falls back to optional undefined constraints if AssetRiskProfile does not provide them', () => {
        const emptyAsset: Partial<Asset> = {
            id: 'WIN',
            point_value: 1.0
        };

        const emptyAssetProfile: Partial<AssetRiskProfile> = { 
            id: 'win-profile-2',
            asset_id: 'WIN',
            default_stop_points: undefined,
            min_contracts: undefined,
            max_contracts: undefined
        };

        const input = adaptPositionSizingInput(
            baseProfile as RiskProfile, 
            emptyAsset as Asset,
            emptyAssetProfile as AssetRiskProfile,
            undefined, 
            undefined 
        );

        expect(input).toBeDefined();
        if(!input) return;

        expect(input.stopPoints).toBe(0); // If undefined, should pass 0 to let engine reject it!
        expect(input.minContracts).toBeUndefined();
        expect(input.maxContracts).toBeUndefined();
        expect(input.maxContractsPhase).toBeUndefined();
    });
});
