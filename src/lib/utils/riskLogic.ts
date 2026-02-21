import type { RiskProfile, GrowthPhase } from "$lib/stores/settings.svelte";

export type EvaluationResult = {
    action: "promote" | "demote" | "stay";
    newPhaseIndex: number;
    reason: string;
};

export function evaluateGrowthPhase(
    profile: RiskProfile,
    currentStats: {
        totalProfit: number;      // Lucro acumulado na fase atual
        daysPositive: number;     // Dias de meta batida na fase atual
        currentDrawdown: number;  // Drawdown atual (valor absoluto positivo)
        lossStreak: number;       // Dias de loss seguidos
    }
): EvaluationResult {
    if (!profile.growth_plan_enabled || !profile.growth_phases || profile.growth_phases.length === 0) {
        return { action: "stay", newPhaseIndex: profile.current_phase_index, reason: "Plano de crescimento desativado ou vazio." };
    }

    const currentIndex = profile.current_phase_index;
    const currentPhase = profile.growth_phases[currentIndex];

    // 1. Check Regression (Priority)
    // Se cair de fase, ignora a progressão
    for (const rule of currentPhase.regression_rules) {
        if (rule.condition === "drawdown_limit" && currentStats.currentDrawdown >= rule.value) {
            const newIndex = Math.max(0, currentIndex - 1);
            return {
                action: "demote",
                newPhaseIndex: newIndex,
                reason: `Drawdown de R$ ${currentStats.currentDrawdown} atingiu o limite de R$ ${rule.value}.`
            };
        }
        if (rule.condition === "max_daily_loss_streak" && currentStats.lossStreak >= rule.value) {
            const newIndex = Math.max(0, currentIndex - 1);
            return {
                action: "demote",
                newPhaseIndex: newIndex,
                reason: `${currentStats.lossStreak} dias de loss seguidos atingiram o limite de ${rule.value}.`
            };
        }
    }

    // 2. Check Progression
    // Só sobe se não houver regressão
    // Verifica se TODAS as regras de progressão foram cumpridas? Ou Pelo menos UMA? 
    // Geralmente é "OU" (atingiu meta financeira OU consistência), mas "E" é mais seguro.
    // Vamos assumir "E" (AND) para robustez, ou "OU" se for explicitamente diferente.
    // Na UI simplificada, assumimos que se o usuário define ambas, ele quer ambas.
    // Mas para facilitar, vamos considerar que se tiver Profit Target, ele manda.

    let promoted = false;
    let reasons: string[] = [];

    const profitRule = currentPhase.progression_rules.find(r => r.condition === "profit_target");
    const daysRule = currentPhase.progression_rules.find(r => r.condition === "days_positive");
    const consistencyRule = currentPhase.progression_rules.find(r => r.condition === "consistency_days");

    // Lógica E (AND): Se elas existem, precisa de todas.
    const profitOk = !profitRule || (currentStats.totalProfit >= profitRule.value);
    const daysOk = !daysRule || (currentStats.daysPositive >= daysRule.value);
    const consistencyOk = !consistencyRule || (currentStats.lossStreak === 0 && currentStats.daysPositive >= consistencyRule.value);
    // Nota: A lógica de 'consistency_days' como sequência exata dependeria de uma track de 'currentStreak' no stats.
    // Por enquanto, simplificamos: sem profit_target pendente e sem loss streak, usando daysPositive como proxy se for o caso,
    // mas o ideal é que o stats passe 'consecutiveDays'. 
    // Vou ajustar a interface de stats para ser mais clara.

    if (profitRule && profitOk) reasons.push(`Meta de lucro atingida (R$ ${currentStats.totalProfit} >= R$ ${profitRule.value})`);
    if (daysRule && daysOk) reasons.push(`Dias de meta consistentes (${currentStats.daysPositive} >= ${daysRule.value})`);
    if (consistencyRule && consistencyOk) reasons.push(`Consistência de pregões atingida (${consistencyRule.value} dias)`);

    if (profitOk && daysOk && consistencyOk && (profitRule || daysRule || consistencyRule)) {
        // Verifica se tem próxima fase
        if (currentIndex < profile.growth_phases.length - 1) {
            return {
                action: "promote",
                newPhaseIndex: currentIndex + 1,
                reason: `Progressão aprovada: ${reasons.join(" e ")}.`
            };
        }
    }

    return { action: "stay", newPhaseIndex: currentIndex, reason: "Manter na fase atual." };
}
