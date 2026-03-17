import { describe, it, expect } from 'vitest';
import { evaluateDiscipline } from './discipline-engine';
import type { RiskProfileConfig, TradeRiskSnapshot } from './types';

describe('Domain: Discipline Engine', () => {
    const today = new Date().toISOString().substring(0, 10);
    const yesterday = new Date(Date.now() - 86400000).toISOString().substring(0, 10);

    const baseProfile: RiskProfileConfig = {
        id: '1',
        name: 'Profile',
        isActive: true,
        capitalSourceType: 'fixed',
        dailyTargetType: 'currency',
        dailyTargetValue: 100,
        maxDailyLossType: 'currency',
        maxDailyLossValue: 50,
        maxTradesPerDay: 3,
        emotionalCouplingEnabled: true,
        emotionalLookbackTrades: 5,
        emotionalPenaltyThreshold: 30
    };

    const createTestTrade = (
        date: string, 
        violated: boolean = false, 
        isLoss: boolean = false, 
        emotionalState: string = ''
    ): TradeRiskSnapshot => {
        return {
            tradeId: Math.random().toString(),
            date,
            pnl: isLoss ? -10 : 10,
            pnlPoints: isLoss ? -20 : 20,
            resultR: isLoss ? -1 : 1,
            isWin: !isLoss,
            isLoss,
            contracts: 1,
            violatedPlan: violated,
            emotionalState
        };
    };

    it('deve punir e flagar overtrading quando exposta além do limite', () => {
        const trades = [
            createTestTrade(today),
            createTestTrade(today),
            createTestTrade(today),
            createTestTrade(today), 
        ]; // 4 trades num cap local de 3

        const result = evaluateDiscipline(baseProfile, trades);
        
        expect(result.overtradingDetected).toBe(true);
        expect(result.score).toBeLessThan(100);
        expect(result.violations.some(v => v.includes('Overtrading'))).toBe(true);
    });

    it('deve reduzir 10 pontos de score para cada trade de fato violada do setup original', () => {
        const trades = [
            createTestTrade(yesterday, true), 
            createTestTrade(yesterday, true)
        ]; // 2 falhas antigas

        const result = evaluateDiscipline(baseProfile, trades);
        
        expect(result.score).toBe(80); // 100 - (2 * 10)
        expect(result.violations.some(v => v.includes('violaram o plano'))).toBe(true);
    });

    it('deve acusar risco nocivo emotionalRiskDetected quando aplicável via tracking do tracker', () => {
        const trades = [
            createTestTrade(today, false, true, 'tilt'),
            createTestTrade(today, false, true, 'revenge')
        ]; // 2 perdas seguidas engatilhadas de estados tiltantes

        const result = evaluateDiscipline(baseProfile, trades);
        
        expect(result.emotionalRiskDetected).toBe(true);
        expect(result.score).toBe(70); // 100 - 30
        
        const hasTiltWarning = result.warnings.some(w => w.includes('Risco emocional alto'));
        expect(hasTiltWarning).toBe(true);
    });

    it('não deve deixar o score cair abaixo de zero', () => {
        const trades = Array(15).fill(createTestTrade(yesterday, true)); // 15 violated trades = -150 penalty
        
        const result = evaluateDiscipline(baseProfile, trades);
        
        // 100 init - 150 = -50, mas tem que limitar em 0
        expect(result.score).toBe(0); 
    });

    it('deve gerar warning e retirar apenas 5 pontos no sniper mode, sem quebrar regras sistêmicas', () => {
        const sniperProfile = { ...baseProfile, sniperModeEnabled: true, sniperMaxTradesPerDay: 1 };
        
        const trades = [
            createTestTrade(today),
            createTestTrade(today) 
        ]; // 2 trades num dia (sniper limit is 1, max overall limit is 3)
        
        const result = evaluateDiscipline(sniperProfile, trades);
        
        expect(result.score).toBe(95); // 100 - 5
        expect(result.overtradingDetected).toBe(false); // only sniper warning
        const hasSniperWarning = result.warnings.some(w => w.includes('Aviso Sniper Mode'));
        expect(hasSniperWarning).toBe(true);
    });

    it('não deve detectar emotionalRisk quando emotionalCouplingEnabled for false', () => {
        const noEmotionProfile = { ...baseProfile, emotionalCouplingEnabled: false };
        const trades = [
            createTestTrade(today, false, true, 'tilt'),
            createTestTrade(today, false, true, 'revenge')
        ]; // 2 perdas seguidas com emocionais mas engine estaria desativada

        const result = evaluateDiscipline(noEmotionProfile, trades);
        
        expect(result.emotionalRiskDetected).toBe(false);
        expect(result.score).toBe(100); 
        expect(result.warnings.length).toBe(0);
    });
});
