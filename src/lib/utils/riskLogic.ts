import type { RiskProfile, GrowthPhase, RiskCondition, Trade } from "$lib/types";

export type EvaluationResult = {
    action: "promote" | "demote" | "stay";
    newPhaseIndex: number;
    reasons: string[];
};

export type TradeStatsSummary = {
    totalProfit: number;
    daysPositive: number;
    winRate: number;
    consistencyDays: number;
    currentDrawdown: number;
    dailyLossLimit: number;
    maxDailyLossStreak: number;
};

export type DailyEquityPoint = { day: string; value: number };

export type RiskStatsResult = TradeStatsSummary & {
    dailyEquityCurve: DailyEquityPoint[];
};

/**
 * Computes real risk statistics from closed trades for a given risk profile.
 * Filters by linked account when capital_source is 'LinkedAccount'.
 * Looks back 30 calendar days for stats and 14 days for the equity curve.
 */
export function computeRiskStats(
    trades: Trade[],
    profile: RiskProfile
): RiskStatsResult {
    const now = new Date();
    const msPerDay = 86_400_000;

    // Determine which account IDs to filter by
    const linkedAccountId = profile.capital_source === "LinkedAccount" ? profile.linked_account_id : null;

    // Get closed trades from the last 30 days
    const cutoff30 = new Date(now.getTime() - 30 * msPerDay);
    const relevantTrades = trades.filter(t => {
        if (t.exit_price === null || t.exit_price === undefined) return false;
        const dateStr = t.exit_date || t.date;
        if (!dateStr) return false;
        const tradeDate = new Date(dateStr);
        if (tradeDate < cutoff30) return false;
        if (linkedAccountId) {
            // Normalize IDs for comparison (handle 'table:id' format)
            const normalizeId = (id: string) => id?.toString().split(":").pop() ?? id;
            return normalizeId(t.account_id) === normalizeId(linkedAccountId);
        }
        return true;
    });

    // Group trades by calendar day
    const byDay = new Map<string, number[]>();
    for (const t of relevantTrades) {
        const day = (t.exit_date || t.date).substring(0, 10);
        if (!byDay.has(day)) byDay.set(day, []);
        byDay.get(day)!.push(Number(t.result) || 0);
    }

    // Win rate
    const total = relevantTrades.length;
    const wins = relevantTrades.filter(t => (Number(t.result) || 0) > 0).length;
    const winRate = total > 0 ? (wins / total) * 100 : 0;

    // Total profit (sum all results)
    const totalProfit = relevantTrades.reduce((s, t) => s + (Number(t.result) || 0), 0);

    // Days positive (days where net result > 0)
    const sortedDays = Array.from(byDay.entries()).sort(([a], [b]) => a.localeCompare(b));
    const daysPositive = sortedDays.filter(([, results]) => results.reduce((s, v) => s + v, 0) > 0).length;

    // Consistency days: consecutive positive days ending today (backwards)
    let consistencyDays = 0;
    for (let i = sortedDays.length - 1; i >= 0; i--) {
        const dayNet = sortedDays[i][1].reduce((s, v) => s + v, 0);
        if (dayNet > 0) consistencyDays++;
        else break;
    }

    // Current drawdown: worst single-day loss (absolute value) among last 30 days
    // Used as proxy for "how much of the daily loss limit has been consumed today"
    const today = now.toISOString().substring(0, 10);
    const todayResults = byDay.get(today) ?? [];
    const todayNet = todayResults.reduce((s, v) => s + v, 0);
    // currentDrawdown is today's loss (positive = losing, negative = winning)
    const currentDrawdown = todayNet < 0 ? Math.abs(todayNet) : 0;

    // Max daily loss streak: longest sequence of consecutive days with negative net
    let maxStreak = 0;
    let curStreak = 0;
    for (const [, results] of sortedDays) {
        const net = results.reduce((s, v) => s + v, 0);
        if (net < 0) { curStreak++; maxStreak = Math.max(maxStreak, curStreak); }
        else curStreak = 0;
    }

    // Daily equity curve for the last 14 days (cumulative from start of period)
    const cutoff14 = new Date(now.getTime() - 13 * msPerDay);
    const dailyEquityCurve: DailyEquityPoint[] = [];
    let cumulative = 0;
    for (let d = 0; d < 14; d++) {
        const dayDate = new Date(cutoff14.getTime() + d * msPerDay);
        const dayStr = dayDate.toISOString().substring(0, 10);
        const dayNet = (byDay.get(dayStr) ?? []).reduce((s, v) => s + v, 0);
        cumulative += dayNet;
        dailyEquityCurve.push({ day: `Dia ${d + 1}`, value: parseFloat(cumulative.toFixed(2)) });
    }

    return {
        totalProfit,
        daysPositive,
        winRate,
        consistencyDays,
        currentDrawdown,
        dailyLossLimit: profile.max_daily_loss,
        maxDailyLossStreak: maxStreak,
        dailyEquityCurve,
    };
}

export function evaluateCondition(stats: TradeStatsSummary, rule: RiskCondition): { passed: boolean, actual: number } {
    let statValue = 0;
    switch (rule.metric) {
        case "profit_target": statValue = stats.totalProfit; break;
        case "days_positive": statValue = stats.daysPositive; break;
        case "win_rate": statValue = stats.winRate; break;
        case "consistency_days": statValue = stats.consistencyDays; break;
        case "drawdown_limit": statValue = stats.currentDrawdown; break;
        case "daily_loss_limit": statValue = stats.dailyLossLimit; break;
        case "max_daily_loss_streak": statValue = stats.maxDailyLossStreak; break;
    }

    let passed = false;
    switch (rule.operator) {
        case ">=": passed = statValue >= rule.value; break;
        case ">": passed = statValue > rule.value; break;
        case "<=": passed = statValue <= rule.value; break;
        case "<": passed = statValue < rule.value; break;
        case "==": passed = statValue === rule.value; break;
    }
    return { passed, actual: statValue };
}

export function evaluateGrowthPhase(
    profile: RiskProfile,
    currentStats: TradeStatsSummary
): EvaluationResult {
    if (!profile.growth_plan_enabled || !profile.growth_phases || profile.growth_phases.length === 0) {
        return { action: "stay", newPhaseIndex: profile.current_phase_index, reasons: ["Plano de crescimento desativado ou vazio."] };
    }

    const currentIndex = profile.current_phase_index;
    const currentPhase = profile.growth_phases[currentIndex];

    // 1. Check Demotion (Highest Priority)
    let demoteReasons: string[] = [];
    if (currentPhase.conditions_to_demote && currentPhase.conditions_to_demote.length > 0) {
        for (const rule of currentPhase.conditions_to_demote) {
            const result = evaluateCondition(currentStats, rule);
            if (result.passed) {
                demoteReasons.push(`${rule.metric} (${result.actual}) atingiu/quebrou a regra ${rule.operator} ${rule.value}`);
            }
        }
    }

    if (demoteReasons.length > 0) {
        const newIndex = Math.max(0, currentIndex - 1);
        return {
            action: "demote",
            newPhaseIndex: newIndex,
            reasons: demoteReasons
        };
    }

    // 2. Check Promotion
    let advanceReasons: string[] = [];
    let allAdvancePassed = true;

    if (currentPhase.conditions_to_advance && currentPhase.conditions_to_advance.length > 0) {
        for (const rule of currentPhase.conditions_to_advance) {
            const result = evaluateCondition(currentStats, rule);
            if (result.passed) {
                advanceReasons.push(`${rule.metric} cumprido (${result.actual} ${rule.operator} ${rule.value})`);
            } else {
                allAdvancePassed = false;
            }
        }

        if (allAdvancePassed && currentIndex < profile.growth_phases.length - 1) {
            return {
                action: "promote",
                newPhaseIndex: currentIndex + 1,
                reasons: advanceReasons
            };
        }
    }

    return { action: "stay", newPhaseIndex: currentIndex, reasons: [] };
}
