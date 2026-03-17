import type { 
    GrowthPhase, 
    TradeRiskSnapshot, 
    GrowthEvaluationResult,
    GrowthMetrics
} from './types';

/**
 * Calcula a taxa de acerto (Win Rate) percentual.
 */
export function calculateWinRate(trades: TradeRiskSnapshot[]): number {
    if (trades.length === 0) return 0;
    
    const wins = trades.filter((t) => t.isWin).length;
    return (wins / trades.length) * 100;
}

/**
 * Calcula o fator de lucro (Gross Profit / Gross Loss).
 * Retorna Infinity em caso de Gross Loss zerado e lucro positivo.
 */
export function calculateProfitFactor(trades: TradeRiskSnapshot[]): number {
    const grossProfit = trades
        .filter((t) => t.pnl > 0)
        .reduce((sum, t) => sum + t.pnl, 0);
        
    const grossLoss = trades
        .filter((t) => t.pnl < 0)
        .reduce((sum, t) => sum + Math.abs(t.pnl), 0);
    
    if (grossLoss === 0) {
        return grossProfit > 0 ? Infinity : 0; 
    }
    
    return grossProfit / grossLoss;
}

/**
 * Calcula a expectativa matemática R (média dos múltiplos de risco).
 */
export function calculateExpectancyR(trades: TradeRiskSnapshot[]): number {
    if (trades.length === 0) return 0;
    
    const totalR = trades.reduce((sum, t) => sum + (t.resultR || 0), 0);
    return totalR / trades.length;
}

/**
 * Calcula o resultado financeiro líquido contínuo.
 */
export function calculateNetPnL(trades: TradeRiskSnapshot[]): number {
    return trades.reduce((sum, t) => sum + t.pnl, 0);
}

/**
 * Calcula o Drawdown percentual máximo de uma amostra de trades sobre um capital incial.
 */
export function calculateDrawdownPercent(
    trades: TradeRiskSnapshot[], 
    startingCapital: number
): number {
    if (startingCapital <= 0 || trades.length === 0) return 0;

    let peak = startingCapital;
    let currentEquity = startingCapital;
    let maxDrawdown = 0;

    for (const trade of trades) {
        currentEquity += trade.pnl;
        
        if (currentEquity > peak) {
            peak = currentEquity;
        }
        
        const currentDrawdown = peak - currentEquity;
        const currentDrawdownPercent = (currentDrawdown / peak) * 100;
        
        if (currentDrawdownPercent > maxDrawdown) {
            maxDrawdown = currentDrawdownPercent;
        }
    }

    return maxDrawdown;
}

/**
 * Avalia de forma cruzada a performance da janela atual versus as metas da fase do plano.
 * Valida de pode promover ou se deve forçar uma regressão em nome do risco.
 */
export function evaluateGrowthPhase(
    currentPhase: GrowthPhase,
    trades: TradeRiskSnapshot[],
    startingCapital: number
): GrowthEvaluationResult {
    const metrics: GrowthMetrics = {
        tradeCount: trades.length,
        winRate: calculateWinRate(trades),
        profitFactor: calculateProfitFactor(trades),
        expectancyR: calculateExpectancyR(trades),
        drawdownPercent: calculateDrawdownPercent(trades, startingCapital),
        netPnL: calculateNetPnL(trades)
    };

    let canPromote = false;
    let shouldRegress = false;
    const reasons: string[] = [];

    // Verificação de Promoção
    if (currentPhase.allowPromotion) {
        const meetsTradeCount = metrics.tradeCount >= currentPhase.minTrades;
        const meetsWinRate = metrics.winRate >= currentPhase.minWinRate;
        const meetsProfitFactor = metrics.profitFactor >= currentPhase.minProfitFactor;
        const meetsExpectancy = metrics.expectancyR >= currentPhase.minExpectancyR;
        const meetsNetPnL = metrics.netPnL >= currentPhase.minNetPnL;

        if (meetsTradeCount && meetsWinRate && meetsProfitFactor && meetsExpectancy && meetsNetPnL) {
            canPromote = true;
            reasons.push('Todas as métricas de promoção foram atingidas.');
        } else {
            if (!meetsTradeCount) reasons.push(`Mínimo de trades não alcançado: ${metrics.tradeCount}/${currentPhase.minTrades}`);
            if (!meetsWinRate) reasons.push(`Taxa de acerto abaixo da meta: ${metrics.winRate.toFixed(1)}% / ${currentPhase.minWinRate}%`);
            if (!meetsProfitFactor) reasons.push(`Fator de lucro abaixo da meta: ${metrics.profitFactor.toFixed(2)} / ${currentPhase.minProfitFactor}`);
            if (!meetsExpectancy) reasons.push(`Expectativa (R) abaixo da meta: ${metrics.expectancyR.toFixed(2)} / ${currentPhase.minExpectancyR}`);
            if (!meetsNetPnL) reasons.push(`Lucratividade líquida insuficiente: ${metrics.netPnL.toFixed(2)} / ${currentPhase.minNetPnL}`);
        }
    } else {
        reasons.push('Promoção desabilitada para esta fase.');
    }

    // Verificação Crítica de Regressão (Anula eventual promoção)
    if (currentPhase.allowRegression) {
        const maxDrawdownExceeded = metrics.drawdownPercent > currentPhase.maxDrawdownPercent;
        const negativeExpectancy = metrics.expectancyR < 0; 

        if (maxDrawdownExceeded) {
            shouldRegress = true;
            canPromote = false;
            reasons.push(`Aviso Crítico: Drawdown máximo atingido (${metrics.drawdownPercent.toFixed(1)}% > ${currentPhase.maxDrawdownPercent}%)`);
        }
        
        if (negativeExpectancy) {
            shouldRegress = true;
            canPromote = false;
            reasons.push(`Aviso Crítico: Expectativa matemática negativa detectada (${metrics.expectancyR.toFixed(2)}R)`);
        }
    }

    return {
        currentPhaseId: currentPhase.id,
        canPromote,
        shouldRegress,
        reasons,
        metrics
    };
}
