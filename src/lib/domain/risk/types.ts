/**
 * Configuração principal do Perfil de Risco do Trader.
 * Define os limites, metas e modos operacionais.
 */

export type RiskStatusLabel = 'NO_TRADES' | 'RUNNING' | 'TARGET_HIT' | 'LOSS_LIMIT_HIT' | 'LOCKED';

export type EmotionalStateTag = 'tilt' | 'revenge' | 'greed' | 'fear' | 'fomo' | 'angry' | 'frustrated' | (string & {});

export interface RiskProfileConfig {
    id: string;
    name: string;
    isActive: boolean;
    
    // Capital
    capitalSourceType: 'fixed' | 'subaccount';
    fixedCapital?: number;
    subaccountId?: string;
    
    // Metas Diárias
    dailyTargetType: 'currency' | 'points';
    dailyTargetValue: number;
    
    // Limites de Perda
    maxDailyLossType: 'currency' | 'points';
    maxDailyLossValue: number;
    maxRiskPerTradePercent?: number;
    minimumRiskReward?: number;
    
    // Limites Operacionais
    maxTradesPerDay?: number;
    lockOnLoss?: boolean;
    
    // Modos Especiais
    sniperModeEnabled?: boolean;
    sniperMaxTradesPerDay?: number;
    
    emotionalCouplingEnabled?: boolean;
    emotionalLookbackTrades?: number;
    emotionalPenaltyThreshold?: number;
    
    growthPlanEnabled?: boolean;
}

/**
 * Definição de uma Fase do Plano de Crescimento.
 * Contém os critérios mínimos para promoção ou regressão.
 */
export interface GrowthPhase {
    id: string;
    name: string;
    
    // Limites da Fase
    maxContracts: number;
    
    // Critérios de Promoção Mínimos
    minTrades: number;
    minWinRate: number;
    minProfitFactor: number;
    minExpectancyR: number;
    minNetPnL: number;
    
    // Critérios de Regressão
    maxDrawdownPercent: number;
    
    // Permissões
    allowPromotion: boolean;
    allowRegression: boolean;
}

/**
 * Representação de uma Trade formatada para análise de Risco.
 */
export interface TradeRiskSnapshot {
    tradeId: string;
    date: Date | string;
    
    // Resultados Financeiros
    pnl: number;
    pnlPoints: number;
    resultR: number;
    
    // Resultados Binários
    isWin: boolean;
    isLoss: boolean;
    
    // Dados Operacionais
    contracts: number;
    
    // Análises Comportamentais
    emotionalState?: EmotionalStateTag;
    violatedPlan?: boolean;
}

/**
 * Resumo diário do status de risco do Trader.
 */
export interface DailyRiskStatus {
    date: string;
    
    // Resultados Acumulados
    dailyPnL: number;
    dailyPnLPoints: number;
    
    // Status de Limites
    dailyTargetHit: boolean;
    dailyLossHit: boolean;
    
    // Distâncias para os Limites
    remainingLossAllowance: number;
    remainingTargetToHit: number;
    
    // Travamentos de Segurança
    isLocked: boolean;
    statusLabel: RiskStatusLabel;
}

/**
 * Consolidação das métricas necessárias para avaliação de Crescimento.
 */
export interface GrowthMetrics {
    tradeCount: number;
    winRate: number;
    profitFactor: number;
    expectancyR: number;
    drawdownPercent: number;
    netPnL: number;
}

/**
 * Resultado da avaliação de uma Fase de Crescimento.
 */
export interface GrowthEvaluationResult {
    currentPhaseId: string;
    canPromote: boolean;
    shouldRegress: boolean;
    reasons: string[];
    metrics: GrowthMetrics;
}

/**
 * Resultado da avaliação de Disciplina e Controle Emocional.
 */
export interface DisciplineEvaluationResult {
    score: number;
    warnings: string[];
    violations: string[];
    overtradingDetected: boolean;
    emotionalRiskDetected: boolean;
}

/**
 * Avaliação genérica de condição.
 */
export interface ConditionEvaluation {
    passed: boolean;
    actual: number;
}
