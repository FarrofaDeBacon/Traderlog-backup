import type { 
    RiskProfileConfig, 
    TradeRiskSnapshot, 
    DisciplineEvaluationResult 
} from './types';

// Array de estados emocionais que indicam risco ou perda de controle
const NEGATIVE_EMOTIONS = [
    'tilt', 
    'revenge', 
    'greed', 
    'fear', 
    'fomo', 
    'angry', 
    'frustrated'
];

/**
 * Isola estritamente as transações relativas ao dia vigente.
 */
function getTodayTrades(trades: TradeRiskSnapshot[]): TradeRiskSnapshot[] {
    const todayStr = new Date().toISOString().substring(0, 10);
    
    return trades.filter((t) => {
        const tDate = typeof t.date === 'string' ? t.date : t.date.toISOString();
        return tDate.substring(0, 10) === todayStr;
    });
}

/**
 * Analisa as últimas trades identificando perdas sequenciais vinculadas a um emocional nocivo.
 */
function analyzeEmotionalRisk(trades: TradeRiskSnapshot[], lookback: number): boolean {
    if (trades.length < 2) return false;
    
    const recentTrades = trades.slice(-lookback);
    let consecutiveLossesWithNegativeEmotion = 0;

    for (const trade of recentTrades) {
        const state = trade.emotionalState ? trade.emotionalState.toLowerCase() : '';
        const hasNegativeEmotion = NEGATIVE_EMOTIONS.some((emotion) => state.includes(emotion));

        if (trade.isLoss && hasNegativeEmotion) {
            consecutiveLossesWithNegativeEmotion++;
            if (consecutiveLossesWithNegativeEmotion >= 2) {
                return true; 
            }
        } else {
            // Conta quebra de sequência em caso de Gain ou estado neutro/positivo
            consecutiveLossesWithNegativeEmotion = 0;
        }
    }

    return false;
}

/**
 * Calcula o score de disciplina e gera infrações com base nas atitudes técnicas operadas.
 */
export function evaluateDiscipline(
    profile: RiskProfileConfig,
    trades: TradeRiskSnapshot[]
): DisciplineEvaluationResult {
    let score = 100;
    const warnings: string[] = [];
    const violations: string[] = [];
    let overtradingDetected = false;
    let emotionalRiskDetected = false;

    const todayTrades = getTodayTrades(trades);
    const todayTradeCount = todayTrades.length;

    // 1. Checagem de Overtrading Geral (Hard Violation)
    const hasMaxTradesLimit = profile.maxTradesPerDay !== undefined && profile.maxTradesPerDay > 0;
    
    if (hasMaxTradesLimit && todayTradeCount > profile.maxTradesPerDay!) {
        overtradingDetected = true;
        score -= 20;
        violations.push(`Overtrading detectado: Operações do dia (${todayTradeCount}) excederam o limite global (${profile.maxTradesPerDay}).`);
    }

    // 2. Checagem de Sniper Mode (Soft Warning)
    const hasSniperMode = profile.sniperModeEnabled && profile.sniperMaxTradesPerDay !== undefined && profile.sniperMaxTradesPerDay > 0;
    
    if (hasSniperMode && todayTradeCount > profile.sniperMaxTradesPerDay!) {
        warnings.push(`Aviso Sniper Mode: Operações do dia (${todayTradeCount}) excederam seu limite hiper-focado (${profile.sniperMaxTradesPerDay}).`);
        score -= 5;
    }

    // 3. Checagem de Violar o Plano de Trade
    const violatedTrades = trades.filter((t) => t.violatedPlan).length;
    
    if (violatedTrades > 0) {
        score -= (10 * violatedTrades);
        violations.push(`Foram registradas ${violatedTrades} operação(ões) que violaram o plano de trade original.`);
    }

    // 4. Checagem de Acoplamento Emocional (Tilt / Revenge)
    if (profile.emotionalCouplingEnabled) {
        const lookback = profile.emotionalLookbackTrades ?? 5; 
        emotionalRiskDetected = analyzeEmotionalRisk(trades, lookback);

        if (emotionalRiskDetected) {
            const penalty = profile.emotionalPenaltyThreshold ?? 30;
            score -= penalty;
            warnings.push('Risco emocional alto: Detectamos perdas consecutivas associadas a estados mentais nocivos (Tilt/Revenge).');
        }
    }

    // Garante score mínimo zero
    score = Math.max(0, score);

    return {
        score,
        warnings,
        violations,
        overtradingDetected,
        emotionalRiskDetected
    };
}
