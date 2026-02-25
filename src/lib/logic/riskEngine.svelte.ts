import { tradesStore } from "$lib/stores/trades.svelte";
import { settingsStore } from "$lib/stores/settings.svelte";
import type { RiskProfile, Trade, EmotionalState } from "$lib/types";
import { getLocalDatePart } from "$lib/utils";

class RiskEngine {
    // Dynamic calculations that depend on the provided profile
    getEmotionalWeight(profile: RiskProfile | null): number {
        if (!profile) return 0;

        const countNeeded = profile.psychological_lookback_count || 10;
        const strategy = profile.psychological_search_strategy || 'Strict';

        let candidates = tradesStore.trades
            .filter(t => t.exit_date || t.date)
            .sort((a, b) => new Date(b.exit_date || b.date).getTime() - new Date(a.exit_date || a.date).getTime());

        let samples: Trade[] = [];
        if (strategy === 'Strict') {
            // Only trades with emotional data
            samples = candidates.filter(t => t.entry_emotional_state_id).slice(0, countNeeded);
        } else {
            // Last N trades regardless
            samples = candidates.slice(0, countNeeded);
        }

        if (samples.length === 0) return 0;

        let total = 0;
        let validSamples = 0;

        samples.forEach(t => {
            if (t.entry_emotional_state_id) {
                const state = settingsStore.emotionalStates.find(s => s.id === t.entry_emotional_state_id);
                if (state) {
                    const impactMultiplier = state.impact === 'Positive' ? 1 : state.impact === 'Negative' ? -1 : 0;
                    total += (state.weight || 5) * impactMultiplier;
                    validSamples++;
                }
            }
        });

        return validSamples > 0 ? total / validSamples : 0;
    }

    checkOutlierDetected(profile: RiskProfile | null): boolean {
        if (!profile || !profile.outlier_regression_enabled) return false;

        const countNeeded = profile.outlier_lookback_count || 20;
        const closedTrades = tradesStore.trades.filter(t => t.exit_price !== null);
        if (closedTrades.length < 5) return false;

        const winners = closedTrades.filter(t => t.result > 0);
        if (winners.length < 3) return false;

        const avgWin = winners.reduce((sum, t) => sum + t.result, 0) / winners.length;

        // Check last N trades for any loss > 2x average win
        const recent = closedTrades
            .sort((a, b) => new Date(b.exit_date || b.date).getTime() - new Date(a.exit_date || a.date).getTime())
            .slice(0, countNeeded);

        return recent.some(t => t.result < 0 && Math.abs(t.result) > (avgWin * 2));
    }

    // Sniper Mode: Check trade count for today
    getTradesCountToday(): number {
        const today = getLocalDatePart(new Date().toISOString());
        return tradesStore.trades.filter(t => getLocalDatePart(t.date) === today).length;
    }

    /**
     * Get suggested lot adjustment based on current risk profile and emotional state
     */
    getSuggestedLotMultiplier(profile: RiskProfile | null): number {
        if (!profile || !profile.psychological_coupling_enabled) return 1.0;

        const weight = this.getEmotionalWeight(profile);
        const threshold = profile.psychological_threshold ?? -2;
        const reduction = profile.lot_reduction_multiplier ?? 0.5;

        if (weight < threshold) {
            return reduction;
        }

        return 1.0;
    }

    /**
     * Returns true if sniper mode selectivity is violated
     */
    isSniperSelectivityViolated(profile: RiskProfile | null): boolean {
        if (!profile || !profile.sniper_mode_enabled) return false;
        return this.getTradesCountToday() >= profile.sniper_mode_selectivity;
    }

    /**
     * Get a list of proactive warnings for the user
     */
    getProactiveWarnings(profile: RiskProfile | null): Array<{ key: string; params?: any }> {
        const warnings: Array<{ key: string; params?: any }> = [];
        if (!profile) return warnings;

        const weight = this.getEmotionalWeight(profile);
        const threshold = profile.psychological_threshold ?? -2;

        if (profile.psychological_coupling_enabled && weight < threshold) {
            const reductionPercent = Math.round((1 - (profile.lot_reduction_multiplier ?? 0.5)) * 100);
            warnings.push({
                key: "trades.wizard.risk.emotional_warning",
                params: {
                    weight: weight.toFixed(1),
                    threshold: threshold,
                    percent: reductionPercent,
                },
            });
        }

        if (profile.outlier_regression_enabled && this.checkOutlierDetected(profile)) {
            warnings.push({
                key: "trades.wizard.risk.outlier_warning",
            });
        }

        if (profile.sniper_mode_enabled && this.isSniperSelectivityViolated(profile)) {
            warnings.push({
                key: "trades.wizard.risk.sniper_warning",
                params: {
                    current: this.getTradesCountToday(),
                    limit: profile.sniper_mode_selectivity,
                },
            });
        }

        return warnings;
    }
}

export const riskEngine = new RiskEngine();
