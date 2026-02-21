import { invoke } from "@tauri-apps/api/core";
import { browser } from "$app/environment";
import type { TaxPayment, Trade, TaxRule, TaxMapping } from "$lib/types";
import { settingsStore } from "./settings.svelte";

type MonthlyTaxReport = {
    month: string; // YYYY-MM
    mappings_snapshot: TaxMapping[];
    details: {
        rule_name: string;
        gross_profit: number; // Lucro Bruto (sem descontar prejuízos passados)
        expenses: number; // Taxas
        net_profit: number; // Resultado Líquido do Mês
        compensated_loss: number; // Prejuízo compensado neste mês
        taxable_base: number; // Base de cálculo após compensação
        tax_due: number; // Imposto devido (antes de deduzir IRRF)
        withholding_tax: number; // IRRF do mês
        net_tax_due: number; // Valor final a pagar (Tax Due - IRRF)
    }[];
    total_tax_due: number;
    total_withholding: number;
    total_net_tax_due: number;
    accumulated_loss: number; // Prejuízo acumulado que sobrou para o próximo mês
};

class FiscalStore {
    payments: TaxPayment[] = $state([]);
    isLoading = $state(false);

    constructor() {
        if (browser) {
            this.loadPayments();
        }
    }

    async loadPayments() {
        this.isLoading = true;
        try {
            const result = await invoke<TaxPayment[]>("get_tax_payments");
            if (result) this.payments = result;
        } catch (e) {
            console.error("Failed to load tax payments from backend", e);
        } finally {
            this.isLoading = false;
        }
    }

    async addPayment(payment: TaxPayment) {
        // Optimistic Update
        const idx = this.payments.findIndex(p => p.period === payment.period);
        if (idx >= 0) {
            this.payments[idx] = payment;
        } else {
            this.payments.push(payment);
        }

        try {
            await invoke("save_tax_payment", { payment });
        } catch (e) {
            console.error("Failed to save tax payment", e);
            // Revert or notify error? For now just log.
        }
    }

    getPayment(period: string): TaxPayment | undefined {
        return this.payments.find(p => p.period === period);
    }

    /**
     * Calculates the tax report for a specific month.
     * Needs access to ALL trades up to that month to calculate accumulated losses correctly.
     */
    calculateMonthReport(
        targetMonth: string, // YYYY-MM
        allTrades: Trade[],
        rules: TaxRule[],
        mappings: TaxMapping[]
    ): MonthlyTaxReport {
        // 1. Sort trades by date
        const sortedTrades = [...allTrades].sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());

        // Helper to get month string
        const getMonth = (dateStr: string) => dateStr.substring(0, 7);

        // Identify distinct months in range, sorted
        const distinctMonths = Array.from(new Set(sortedTrades.map(t => getMonth(t.date)))).sort();

        // Find index of target month. If not in list (e.g. no trades), we still might have carryover.
        // We need to iterate up to targetMonth.
        // Create a range of months from first trade until targetMonth.

        if (distinctMonths.length === 0) return this._emptyReport(targetMonth);

        const firstMonth = distinctMonths[0];
        const monthRange: string[] = [];

        // Generate month range
        let currentIter = firstMonth;
        while (currentIter <= targetMonth) {
            monthRange.push(currentIter);
            // Next month
            const [y, m] = currentIter.split('-').map(Number);
            const date = new Date(y, m, 1); // Month is 0-indexed in JS Date? Wait. 
            // Better parsing:
            const nextDate = new Date(y, m, 1); // This actually goes to next month because m is 1-12 in string but 0-11 in Date if we used (y, m-1). 
            // But 'm' here is 1-based from split. Date(y, m) is Next Month (since m is equivalent to mIndex+1).
            // Example: "2024-01" -> y=2024, m=1. Date(2024, 1) is Feb 1st. Correct.
            const nextY = nextDate.getFullYear();
            const nextM = nextDate.getMonth() + 1;
            currentIter = `${nextY}-${String(nextM).padStart(2, '0')}`;
        }

        // Loss Buckets: ModalityID + RuleID -> Accumulated Loss
        const lossBuckets: Record<string, number> = {};

        let finalReport: MonthlyTaxReport = this._emptyReport(targetMonth);

        // Iterate Month by Month
        for (const m of monthRange) {
            const isTarget = m === targetMonth;
            const mTrades = sortedTrades.filter(t => getMonth(t.date) === m);

            // Bucket trades for this month by (Modality+Rule) 
            // Key: "ModalityID" (Simplification: We assume loss compensation is per Modality)
            // But strictness depends on Rules. We will map Rule -> Modality.

            // For calculating results for this month:
            const monthlyBuckets: Record<string, {
                rule: TaxRule,
                modalityId: string,
                gross_profit: number,
                sales: number,
                trades: Trade[]
            }> = {};

            mTrades.forEach(t => {
                const mapping = mappings.find(map => map.asset_type_id === t.asset_type_id && map.modality_id === t.modality_id);
                if (!mapping) return;
                const rule = rules.find(r => r.id === mapping.tax_rule_id);
                if (!rule) return;

                // Group Key: Modality ID is the safest container for "Type of Operation"
                const key = t.modality_id;

                if (!monthlyBuckets[key]) {
                    monthlyBuckets[key] = {
                        rule, // Store first found rule for metadata
                        modalityId: t.modality_id,
                        gross_profit: 0,
                        sales: 0,
                        trades: []
                    };
                }

                // Net Result of Trade (Profit - Fees)
                const netTrade = t.result - t.fee_total;
                monthlyBuckets[key].gross_profit += netTrade;
                monthlyBuckets[key].trades.push(t);

                // Sales Volume
                const exitPrice = t.exit_price || 0;
                // If SaleAmount basis or profitable, add to sales? 
                // Exemption usually checks TOTAL sales of the modality (Swing Trade).
                monthlyBuckets[key].sales += (exitPrice * t.quantity);
            });

            // Process Monthly Results & Update Losses
            const monthDetails: MonthlyTaxReport['details'] = [];

            for (const key in monthlyBuckets) {
                const bucket = monthlyBuckets[key];
                const rule = bucket.rule;

                const gross = bucket.gross_profit;
                const currentLoss = lossBuckets[key] || 0;

                let taxable = 0;
                let compensated = 0;
                let taxDue = 0;
                let isExempt = false;

                // 1. Check Exemption
                if (gross > 0 && rule.exemption_threshold > 0 && bucket.sales < rule.exemption_threshold) {
                    isExempt = true;
                    // No tax, no compensation used.
                } else if (gross > 0) {
                    // Profitable and Taxable
                    compensated = Math.min(gross, currentLoss);
                    taxable = gross - compensated;
                    taxDue = taxable * (rule.tax_rate / 100);

                    // Reduce accumulated loss
                    lossBuckets[key] = currentLoss - compensated;
                } else if (gross < 0) {
                    // Loss!
                    if (rule.cumulative_losses) {
                        lossBuckets[key] = currentLoss + Math.abs(gross);
                    }
                }

                // Calculate Withholding (IRRF)
                let withholding = 0;
                bucket.trades.forEach(t => {
                    const basis = rule.basis === "NetProfit" ? (t.result - t.fee_total) : ((t.exit_price || 0) * t.quantity);

                    if (rule.basis === "SaleAmount") {
                        withholding += basis * (rule.withholding_rate / 100);
                    } else {
                        // Day Trade IRRF is 1% on POSITIVE Basis only
                        if (basis > 0) withholding += basis * (rule.withholding_rate / 100);
                    }
                });

                if (isTarget) {
                    monthDetails.push({
                        rule_name: `${rule.name}`,
                        gross_profit: gross,
                        expenses: 0,
                        net_profit: gross,
                        compensated_loss: compensated,
                        taxable_base: taxable,
                        tax_due: taxDue,
                        withholding_tax: withholding,
                        net_tax_due: Math.max(0, taxDue - withholding)
                    });
                }
            }

            // If this is the target month, build the report
            if (isTarget) {
                const totalTax = monthDetails.reduce((sum, d) => sum + d.tax_due, 0);
                const totalWithholding = monthDetails.reduce((sum, d) => sum + d.withholding_tax, 0);
                const totalNet = monthDetails.reduce((sum, d) => sum + d.net_tax_due, 0);
                // Total accumulated loss across all buckets
                const totalLoss = Object.values(lossBuckets).reduce((sum, v) => sum + v, 0);

                finalReport = {
                    month: targetMonth,
                    mappings_snapshot: mappings,
                    details: monthDetails,
                    total_tax_due: totalTax,
                    total_withholding: totalWithholding,
                    total_net_tax_due: totalNet,
                    accumulated_loss: totalLoss
                };
            }
        }

        return finalReport;
    }

    private _emptyReport(month: string): MonthlyTaxReport {
        return {
            month,
            mappings_snapshot: [],
            details: [],
            total_tax_due: 0,
            total_withholding: 0,
            total_net_tax_due: 0,
            accumulated_loss: 0
        };
    }
}

export const fiscalStore = new FiscalStore();
