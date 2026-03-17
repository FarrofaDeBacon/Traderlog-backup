import { describe, it, expect } from 'vitest';
import { calculateDailyRiskStatus, calculateCurrentCapital, calculateAllowedRiskPerTrade } from './risk-engine';
import type { RiskProfileConfig, TradeRiskSnapshot } from './types';

describe('Domain: Risk Engine', () => {
    const today = new Date().toISOString().substring(0, 10);
    const yesterday = new Date(Date.now() - 86400000).toISOString().substring(0, 10);

    const baseProfile: RiskProfileConfig = {
        id: '1',
        name: 'Test Profile',
        isActive: true,
        capitalSourceType: 'fixed',
        fixedCapital: 1000,
        dailyTargetType: 'currency',
        dailyTargetValue: 100,
        maxDailyLossType: 'currency',
        maxDailyLossValue: 50,
        lockOnLoss: true
    };

    it('deve retornar NO_TRADES quando não há trades no dia', () => {
        const status1 = calculateDailyRiskStatus(baseProfile, []);
        expect(status1.statusLabel).toBe('NO_TRADES');

        const pastTrade: TradeRiskSnapshot = {
            tradeId: 't1',
            date: yesterday,
            pnl: 50,
            pnlPoints: 100,
            resultR: 1,
            isWin: true,
            isLoss: false,
            contracts: 1
        };

        const status2 = calculateDailyRiskStatus(baseProfile, [pastTrade]);
        expect(status2.statusLabel).toBe('NO_TRADES');
    });

    it('deve detectar TARGET_HIT corretamente ao atingir limite financeiro', () => {
        const trade: TradeRiskSnapshot = {
            tradeId: 't1',
            date: today,
            pnl: 100, 
            pnlPoints: 200,
            resultR: 2,
            isWin: true,
            isLoss: false,
            contracts: 1
        };

        const status = calculateDailyRiskStatus(baseProfile, [trade]);
        
        expect(status.statusLabel).toBe('TARGET_HIT');
        expect(status.dailyTargetHit).toBe(true);
    });

    it('deve detectar LOSS_LIMIT_HIT corretamente e não travar se lockOnLoss for false', () => {
        const profileNoLock = { ...baseProfile, lockOnLoss: false };
        const trade: TradeRiskSnapshot = {
            tradeId: 't1',
            date: today,
            pnl: -60, 
            pnlPoints: -100,
            resultR: -1,
            isWin: false,
            isLoss: true,
            contracts: 1
        };

        const status = calculateDailyRiskStatus(profileNoLock, [trade]);
        
        expect(status.statusLabel).toBe('LOSS_LIMIT_HIT');
        expect(status.dailyLossHit).toBe(true);
        expect(status.isLocked).toBe(false);
    });

    it('deve ativar LOCKED quando lockOnLoss estiver habilitado e perda limite for atingida', () => {
        const trade: TradeRiskSnapshot = {
            tradeId: 't1',
            date: today,
            pnl: -50, 
            pnlPoints: -100,
            resultR: -1,
            isWin: false,
            isLoss: true,
            contracts: 1
        };

        const status = calculateDailyRiskStatus(baseProfile, [trade]);
        
        expect(status.statusLabel).toBe('LOCKED');
        expect(status.dailyLossHit).toBe(true);
        expect(status.isLocked).toBe(true);
    });

    it('deve calcular corretamente metas e loss em pontos', () => {
        const pointsProfile = { 
            ...baseProfile, 
            dailyTargetType: 'points' as const, 
            dailyTargetValue: 300,
            maxDailyLossType: 'points' as const,
            maxDailyLossValue: 150
        };

        // Testando Win em Pontos
        const winTrade: TradeRiskSnapshot = {
            tradeId: 't1', date: today, pnl: 50, pnlPoints: 300, resultR: 1, isWin: true, isLoss: false, contracts: 1
        };
        const winStatus = calculateDailyRiskStatus(pointsProfile, [winTrade]);
        expect(winStatus.statusLabel).toBe('TARGET_HIT');
        expect(winStatus.remainingTargetToHit).toBe(0);

        // Testando Loss em Pontos
        const lossTrade: TradeRiskSnapshot = {
            tradeId: 't2', date: today, pnl: -10, pnlPoints: -150, resultR: -1, isWin: false, isLoss: true, contracts: 1
        };
        const lossStatus = calculateDailyRiskStatus(pointsProfile, [lossTrade]);
        expect(lossStatus.statusLabel).toBe('LOCKED'); // Hit exactly the 150 points loss and lockOnLoss is true
        expect(lossStatus.remainingLossAllowance).toBe(0);
    });

    it('deve calcular current capital corretamente (fixed e subaccount)', () => {
        const fixedProf = { ...baseProfile, capitalSourceType: 'fixed' as const, fixedCapital: 5000 };
        expect(calculateCurrentCapital(fixedProf)).toBe(5000);

        const subProf = { ...baseProfile, capitalSourceType: 'subaccount' as const };
        expect(calculateCurrentCapital(subProf, 2500)).toBe(2500);
        // Fallback para 0 se invalido
        expect(calculateCurrentCapital(subProf, undefined)).toBe(0);
    });

    it('deve calcular allowed risk limit corretamente, lidando com capitais ausentes', () => {
        expect(calculateAllowedRiskPerTrade(1000, 2)).toBe(20); // 2% de 1000
        expect(calculateAllowedRiskPerTrade(1000, undefined)).toBe(0); // Sem rule
        expect(calculateAllowedRiskPerTrade(0, 5)).toBe(0); // Capital zero
        expect(calculateAllowedRiskPerTrade(-500, 5)).toBe(0); // Capital negativo
    });
});
