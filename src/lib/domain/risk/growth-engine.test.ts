import { describe, it, expect } from 'vitest';
import { 
    calculateWinRate, 
    calculateProfitFactor, 
    calculateExpectancyR,
    calculateDrawdownPercent,
    evaluateGrowthPhase
} from './growth-engine';
import type { GrowthPhase, TradeRiskSnapshot } from './types';

describe('Domain: Growth Engine - Metricas Isoladas', () => {
    const today = new Date().toISOString();

    const createTestTrade = (isWin: boolean, pnl: number, resultR: number): TradeRiskSnapshot => {
        return {
            tradeId: Math.random().toString(),
            date: today,
            pnl,
            pnlPoints: pnl * 2,
            resultR,
            isWin,
            isLoss: !isWin,
            contracts: 1
        };
    };

    it('deve calcular win rate em percentual corretamente', () => {
        const trades = [
            createTestTrade(true, 100, 1),
            createTestTrade(false, -50, -1),
            createTestTrade(true, 100, 1),
            createTestTrade(false, -50, -1),
        ]; // 50%
        
        expect(calculateWinRate(trades)).toBe(50);
        expect(calculateWinRate([])).toBe(0);
    });

    it('deve calcular profit factor corretamente', () => {
        const trades = [
            createTestTrade(true, 150, 1),
            createTestTrade(false, -50, -1),
        ]; // 150 / 50 = 3
        
        expect(calculateProfitFactor(trades)).toBe(3);

        const onlyWins = [createTestTrade(true, 100, 1)];
        expect(calculateProfitFactor(onlyWins)).toBe(Infinity);
    });

    it('deve calcular expectancyR precisamente', () => {
        const trades = [
            createTestTrade(true, 100, 2),
            createTestTrade(false, -50, -1),
        ]; //  2 - 1 = 1 / 2 = 0.5
        
        expect(calculateExpectancyR(trades)).toBe(0.5);
    });

    it('deve calcular drawdown percent corretamente com sequência de pico e queda', () => {
        const capital = 1000;
        const trades = [
            createTestTrade(true, 200, 1),   // peak = 1200
            createTestTrade(false, -600, -1), // drop to 600. Drawdown = 600 (50% of 1200)
            createTestTrade(true, 100, 1)    // recup to 700. Drawdown is still 600 from 1200 (50%)
        ]; 
        
        expect(calculateDrawdownPercent(trades, capital)).toBe(50);
    });
});

describe('Domain: Growth Engine - evaluateGrowthPhase', () => {
    const today = new Date().toISOString();
    
    const createTestTrade = (isWin: boolean, pnl: number, resultR: number): TradeRiskSnapshot => {
        return {
            tradeId: Math.random().toString(),
            date: today,
            pnl,
            pnlPoints: pnl * 2,
            resultR,
            isWin,
            isLoss: !isWin,
            contracts: 1
        };
    };

    const targetPhase: GrowthPhase = {
        id: '1',
        name: 'Fase 1',
        maxContracts: 2,
        minTrades: 5,
        minWinRate: 40,
        minProfitFactor: 1.5,
        minExpectancyR: 0.5,
        maxDrawdownPercent: 10,
        minNetPnL: 500,
        allowPromotion: true,
        allowRegression: true
    };

    it('deve autorizar canPromote = true quando todos os critérios superam a baseline', () => {
        const trades = [
            createTestTrade(true, 200, 1),
            createTestTrade(true, 200, 1),
            createTestTrade(true, 200, 1),
            createTestTrade(false, -50, 0),
            createTestTrade(false, -50, 0),
        ]; 
        
        const startingCapital = 10000;
        const result = evaluateGrowthPhase(targetPhase, trades, startingCapital);
        
        expect(result.canPromote).toBe(true);
        expect(result.shouldRegress).toBe(false);
        expect(result.metrics.netPnL).toBe(500);
        expect(result.metrics.expectancyR).toBe(0.6);
        expect(result.metrics.tradeCount).toBe(5);
    });

    it('deve detectar shouldRegress = true e quebrar promotion quando drawdown estoura', () => {
        const trades = [
            createTestTrade(false, -1500, -2) // 15% drawdown num balance de 10k
        ];
        
        const startingCapital = 10000;
        const result = evaluateGrowthPhase(targetPhase, trades, startingCapital);
        
        expect(result.shouldRegress).toBe(true);
        expect(result.canPromote).toBe(false);
        
        const hasCriticalReason = result.reasons.some(r => r.includes('Aviso Crítico: Drawdown máximo atingido'));
        expect(hasCriticalReason).toBe(true);
    });

    it('deve detectar shouldRegress = true e quebrar promotion quando expectancyR fica negativa independentemente da quantidade de trades', () => {
        const trades = [
            createTestTrade(false, -50, -1) // 1 único trade perdedor deixando Expectancy = -1
        ];
        
        const startingCapital = 10000;
        const result = evaluateGrowthPhase(targetPhase, trades, startingCapital);
        
        expect(result.shouldRegress).toBe(true);
        expect(result.canPromote).toBe(false);
        
        const hasExpectancyReason = result.reasons.some(r => r.includes('Aviso Crítico: Expectativa matemática negativa detectada'));
        expect(hasExpectancyReason).toBe(true);
    });

    it('não deve promover quando faltar apenas 1 critério, mesmo com todos os outros perfeitos', () => {
        const trades = [
            createTestTrade(true, 200, 1),
            createTestTrade(true, 200, 1),
            createTestTrade(true, 200, 1),
            createTestTrade(true, 200, 1), 
            // Only 4 trades instead of the required 5. All other metrics (WR 100%, PF inf, net 800) are perfect.
        ];
        
        const startingCapital = 10000;
        const result = evaluateGrowthPhase(targetPhase, trades, startingCapital);
        
        expect(result.canPromote).toBe(false);
        expect(result.shouldRegress).toBe(false);
        const reason = result.reasons.find(r => r.includes('Mínimo de trades não alcançado'));
        expect(reason).toBeDefined();
    });
});
