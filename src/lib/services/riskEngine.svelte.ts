import { settingsStore } from "$lib/stores/settings.svelte";

export interface RiskSettings {
    maxRiskPerTrade: number; // 1% or 2%
    maxRiskPerDay: number;   // 3% or 6%
    equity: number;
}

class RiskEngine {
    settings = $state<RiskSettings>({
        maxRiskPerTrade: 1,
        maxRiskPerDay: 3,
        equity: 0
    });

    results = $derived.by(() => {
        const eq = this.settings.equity || 10000; // fallback para exemplo
        return {
            maxLossTrade: (eq * this.settings.maxRiskPerTrade) / 100,
            maxLossDay: (eq * this.settings.maxRiskPerDay) / 100,
        };
    });

    calculateSuggestedLot(entry: number, stop: number, pointValue: number): number {
        const riskAmount = this.results.maxLossTrade;
        const pointsDiff = Math.abs(entry - stop);
        if (pointsDiff === 0 || pointValue === 0) return 0;

        // Ex: Gain/Loss em BRL = pontos * lotes * pt_val
        // Lote = Montante_Risco / (pontos * pt_val)
        return Math.floor(riskAmount / (pointsDiff * pointValue));
    }

    checkTradeValidity(potentialLoss: number): { valid: boolean; reason?: string } {
        if (potentialLoss > this.results.maxLossTrade) {
            return { valid: false, reason: "Ultrapassa o risco máximo por trade." };
        }
        return { valid: true };
    }
}

export const riskEngine = new RiskEngine();
