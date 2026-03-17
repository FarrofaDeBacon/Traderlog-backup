import type { Trade, RiskProfile, GrowthPhase as AppGrowthPhase } from '$lib/types';
import type { 
    RiskProfileConfig, 
    TradeRiskSnapshot, 
    GrowthPhase as DomainGrowthPhase,
    EmotionalStateTag
} from './types';

/**
 * Converte um RiskProfile do aplicativo (armazenado no banco/store) 
 * para o formato puro RiskProfileConfig usado pelo domínio.
 */
export function adaptSettingsProfileToDomain(profile: RiskProfile): RiskProfileConfig {
    return {
        id: profile.id,
        name: profile.name,
        isActive: profile.active,
        capitalSourceType: profile.capital_source === 'Fixed' ? 'fixed' : 'subaccount',
        fixedCapital: profile.fixed_capital,
        subaccountId: profile.linked_account_id || undefined,
        
        dailyTargetType: profile.target_type === 'Financial' ? 'currency' : 'points',
        dailyTargetValue: profile.daily_target,
        
        maxDailyLossType: profile.target_type === 'Financial' ? 'currency' : 'points', // Assumindo mesmo tipo do target por padrão na v1
        maxDailyLossValue: profile.max_daily_loss,
        
        maxRiskPerTradePercent: profile.max_risk_per_trade_percent,
        minimumRiskReward: profile.min_risk_reward,
        maxTradesPerDay: profile.max_trades_per_day,
        
        lockOnLoss: profile.lock_on_loss,
        
        sniperModeEnabled: profile.sniper_mode_enabled,
        sniperMaxTradesPerDay: profile.sniper_mode_selectivity,
        
        emotionalCouplingEnabled: profile.psychological_coupling_enabled,
        emotionalLookbackTrades: profile.psychological_lookback_count,
        emotionalPenaltyThreshold: profile.psychological_threshold,
        
        growthPlanEnabled: profile.growth_plan_enabled
    };
}

/**
 * Converte um Trade do aplicativo (do tradesStore) para a interface
 * simplificada e pura TradeRiskSnapshot usada pelo domínio.
 */
export function adaptTradeToDomain(trade: Trade): TradeRiskSnapshot {
    const isWin = trade.result > 0;
    const isLoss = trade.result < 0;
    
    // Normalizando o estado emocional
    // Pode vir como nome ou ID dependendo do histórico. Mapeamos de forma solta.
    let emotionalState: EmotionalStateTag | undefined;
    if (trade.entry_emotional_state_name) {
        emotionalState = trade.entry_emotional_state_name.toLowerCase() as EmotionalStateTag;
    }

    // Cálculo básico de Result R, caso aplicável. 
    // Se não tiver SL/TP mapeados precisamente, assumiremos 0 ou um estimado.
    // Para v1 do adaptador onde o foco é pnl/pnlPoints, deixaremos fallback.
    const resultR = trade.stop_loss && trade.entry_price && trade.exit_price 
        ? (trade.exit_price - trade.entry_price) / Math.abs(trade.entry_price - trade.stop_loss)
        : (isWin ? 1 : isLoss ? -1 : 0);
        
    // pnlPoints não existe nativamente no Trade base (é armazenado PNL em account currency).
    // Num fluxo real, seria extraido baseando-se no tick size. Por hora, passamos 0 ou delta.
    const pnlPoints = trade.exit_price && trade.entry_price 
        ? Math.abs(trade.exit_price - trade.entry_price) * (trade.direction === 'Buy' ? 1 : -1)
        : 0;

    return {
        tradeId: trade.id,
        date: trade.exit_date || trade.date,
        pnl: trade.result,
        pnlPoints: isNaN(pnlPoints) ? 0 : pnlPoints,
        resultR: isNaN(resultR) || !isFinite(resultR) ? (isWin ? 1 : isLoss ? -1 : 0) : resultR,
        isWin,
        isLoss,
        contracts: trade.quantity,
        violatedPlan: !trade.followed_plan,
        emotionalState
    };
}

/**
 * Converte uma Fase de Crescimento (GrowthPhase) do modelo do App
 * para o modelo interno simplificado do Growth Engine.
 */
export function adaptGrowthPhaseToDomain(phase: AppGrowthPhase): DomainGrowthPhase | undefined {
    if (!phase) return undefined;

    // Extrai lógicas dinâmicas de conditions_to_advance/demote
    // Exemplo: { metric: 'WinRate', operator: '>=', value: 40 }
    const getCondition = (arr: any[], metricName: string, fallback: number) => {
        const cond = arr.find(c => c.metric === metricName);
        return cond ? Number(cond.value) : fallback;
    };

    return {
        id: phase.id || phase.name,
        name: phase.name,
        maxContracts: phase.lot_size,
        minTrades: getCondition(phase.conditions_to_advance, 'MinTrades', 0),
        minWinRate: getCondition(phase.conditions_to_advance, 'WinRate', 0),
        minProfitFactor: getCondition(phase.conditions_to_advance, 'ProfitFactor', 0),
        minExpectancyR: getCondition(phase.conditions_to_advance, 'Expectancy', 0),
        minNetPnL: getCondition(phase.conditions_to_advance, 'NetProfit', 0),
        maxDrawdownPercent: getCondition(phase.conditions_to_demote, 'Drawdown', 100),
        allowPromotion: phase.conditions_to_advance.length > 0,
        allowRegression: phase.conditions_to_demote.length > 0
    };
}
