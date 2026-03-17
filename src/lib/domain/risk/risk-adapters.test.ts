import { describe, it, expect } from 'vitest';
import { 
    adaptSettingsProfileToDomain, 
    adaptTradeToDomain, 
    adaptGrowthPhaseToDomain 
} from './risk-adapters';
import type { RiskProfile, Trade, GrowthPhase as AppGrowthPhase } from '$lib/types';

describe('Risk Adapters', () => {
    it('deve adaptar um RiskProfile do App para RiskProfileConfig do Dominio', () => {
        const appProfile: RiskProfile = {
            id: 'prof1',
            name: 'Teste',
            max_daily_loss: 50,
            daily_target: 100,
            max_risk_per_trade_percent: 2,
            max_trades_per_day: 5,
            min_risk_reward: 1.5,
            lock_on_loss: true,
            account_type_applicability: 'All',
            account_ids: [],
            target_type: 'Financial',
            capital_source: 'Fixed',
            fixed_capital: 1000,
            linked_account_id: null,
            growth_plan_enabled: true,
            current_phase_index: 0,
            growth_phases: [],
            psychological_coupling_enabled: true,
            outlier_regression_enabled: false,
            sniper_mode_enabled: true,
            sniper_mode_selectivity: 2,
            psychological_lookback_count: 3,
            outlier_lookback_count: 5,
            psychological_threshold: 50,
            lot_reduction_multiplier: 0.5,
            psychological_search_strategy: 'Strict',
            active: true
        };

        const domainProfile = adaptSettingsProfileToDomain(appProfile);

        expect(domainProfile.id).toBe('prof1');
        expect(domainProfile.isActive).toBe(true);
        expect(domainProfile.capitalSourceType).toBe('fixed');
        expect(domainProfile.dailyTargetType).toBe('currency');
        expect(domainProfile.dailyTargetValue).toBe(100);
        expect(domainProfile.maxDailyLossValue).toBe(50);
        expect(domainProfile.growthPlanEnabled).toBe(true);
        expect(domainProfile.sniperModeEnabled).toBe(true);
        expect(domainProfile.sniperMaxTradesPerDay).toBe(2);
    });

    it('deve adaptar um Trade fechado para TradeRiskSnapshot mantendo estado emocional nulo se vazio', () => {
        const trade: Trade = {
            id: 't1',
            date: '2023-10-01',
            asset_symbol: 'WIN',
            asset_type_id: '1',
            strategy_id: '1',
            account_id: '1',
            result: -50,
            quantity: 1,
            direction: 'Buy',
            entry_price: 110000,
            exit_price: 109500,
            exit_date: '2023-10-01',
            fee_total: 2,
            notes: '',
            timeframe: 'M5',
            volatility: 'Low',
            entry_emotional_state_id: null,
            entry_emotional_state_name: 'Tilt',
            exit_reason: null,
            exit_emotional_state_id: null,
            exit_emotional_state_name: null,
            entry_rationale: '',
            confirmation_signals: '',
            market_context: '',
            relevant_news: '',
            psychology_analysis_during: '',
            followed_plan: false,
            what_worked: '',
            mistakes_improvements: '',
            lessons_learned: '',
            images: [],
            partial_exits: [],
            modality_id: '1',
            stop_loss: 109000,
            take_profit: 112000,
            intensity: 5
        };

        const snapshot = adaptTradeToDomain(trade);

        expect(snapshot.tradeId).toBe('t1');
        expect(snapshot.pnl).toBe(-50);
        expect(snapshot.isLoss).toBe(true);
        expect(snapshot.violatedPlan).toBe(true);
        // Tilt em lowercase do adaptador
        expect(snapshot.emotionalState).toBe('tilt'); 
        
        // entry = 110000, exit = 109500, stop = 109000 -> Risk foi 1000. Loss de 500 = -0.5R.
        expect(snapshot.resultR).toBe(-0.5); 
    });

    it('deve adaptar GrowthPhase', () => {
        const phase: AppGrowthPhase = {
            id: 'phase1',
            level: 1,
            name: 'Fase Inicial',
            lot_size: 1,
            conditions_to_advance: [
                { metric: 'WinRate', operator: '>=', value: 40 },
                { metric: 'MinTrades', operator: '>=', value: 5 }
            ],
            conditions_to_demote: [
                { metric: 'Drawdown', operator: '>=', value: 5 }
            ]
        };

        const domainPhase = adaptGrowthPhaseToDomain(phase);
        expect(domainPhase?.minWinRate).toBe(40);
        expect(domainPhase?.minTrades).toBe(5);
        expect(domainPhase?.maxDrawdownPercent).toBe(5);
        expect(domainPhase?.allowPromotion).toBe(true); // Because length > 0
    });
});
