import type { RiskProfile, Asset, AssetRiskProfile, GrowthPhase } from '$lib/types';
import type { PositionSizingInput } from './position-sizing-engine';

/**
 * Adaptador sujo (Lê os dados brutos e complexos do App / DB) -> Posição Limpa (Engine Puro)
 * 
 * @param profile O Perfil de Risco atual do usuário.
 * @param asset O ativo que ele selecionou para operar.
 * @param assetRiskProfile O perfil de ativo vinculado selecionado para este sizing.
 * @param currentPhase Se o Growth Plan estiver ativo, é a fase na qual ele está atualmente.
 * @param dynamicBalanceFallback Se a fonte de capital for LinkedAccount, deve ser injetado o Account Balance via store aqui fora do engine.
 * @returns {PositionSizingInput|null} Retorna null se for impossível processar por falta violenta de dados de config.
 */
export function adaptPositionSizingInput(
    profile: RiskProfile,
    asset: Asset,
    assetRiskProfile: AssetRiskProfile,
    currentPhase?: GrowthPhase,
    dynamicBalanceFallback?: number
): PositionSizingInput | null {
    
    // 1. Resolver Capital Base
    let capital = 0;
    if (profile.capital_source === 'Fixed') {
        capital = profile.fixed_capital ?? 0;
    } else if (profile.capital_source === 'LinkedAccount') {
        if (dynamicBalanceFallback === undefined) {
             // Conta lincada, mas não achamos o saldo.
             return null;
        }
        capital = dynamicBalanceFallback;
    }

    // Se o setup explícito do RiskProfile capou o RiskPerTrade
    const riskPercent = profile.max_risk_per_trade_percent ?? 0;

    // 2. Extrair Regras do Ativo (Fonte de Verdade Estrutural)
    const pointValue = asset.point_value;

    // 2.5 Regras Operacionais do Trader (Asset Risk Profile)
    // Extrai stop, min e max DO PERFIL ESPECÍFICO DE ATIVO, não mais do perfil geral de risco
    const stopPoints = assetRiskProfile.default_stop_points ?? 0; // Default zero vai falhar no engine controladamente.
    const minContracts = assetRiskProfile.min_contracts;
    const maxContracts = assetRiskProfile.max_contracts;

    // 3. Extrair Regras da Fase Ativa de Crescimento
    const maxContractsPhase = (profile.growth_plan_enabled && currentPhase) 
        ? currentPhase.lot_size 
        : undefined;

    return {
        capital,
        riskPerTradePercent: riskPercent,
        pointValue,
        stopPoints,
        minContracts,
        maxContracts,
        maxContractsPhase
    };
}
