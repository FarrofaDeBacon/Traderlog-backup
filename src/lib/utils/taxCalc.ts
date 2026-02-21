import type { Trade, TaxRule, TaxMapping } from "$lib/types";

export type TaxResult = {
    trade_id: string;
    tax_due: number; // Imposto a pagar (sem descontar IRRF)
    withholding_tax: number; // IRRF (Dedo-duro) calculado
    basis: number; // Base de cálculo (Lucro Líquido ou Valor de Venda)
    rate: number; // Alíquota aplicada
    is_exempt: boolean; // Se foi isento (ex: Swing Trade < 20k)
    rule_applied: string; // Nome da regra aplicada
};

/**
 * Calculates the estimated tax for a single trade based on applied rules.
 * NOTE: This is a specialized function for individual trade estimation.
 * Final tax calculation (DARF) requires monthly aggregation and is handled by FiscalDashboard.
 */
export function calculateTradeTax(
    trade: Trade,
    rule: TaxRule | undefined
): TaxResult {
    const result: TaxResult = {
        trade_id: trade.id,
        tax_due: 0,
        withholding_tax: 0,
        basis: 0,
        rate: 0,
        is_exempt: false,
        rule_applied: rule ? rule.name : "Sem Regra",
    };

    if (!rule) return result;


    // 0. Pre-calculate Net Result (since trade.result is Gross)
    const netResult = trade.result - trade.fee_total;

    // 1. Determine Basis
    // NetProfit: Basis is the Net Result (Profit - Fees)
    // SaleAmount: Basis is the Total Sale Amount (Quantity * Exit Price)
    if (rule.basis === "NetProfit") {
        result.basis = netResult;
    } else if (rule.basis === "SaleAmount") {
        // Warning: This logic assumes 'trade.exit_price' and 'trade.quantity' are correct.
        // For partial exists, this might need adjustment if trade struct changes.
        const exitPrice = trade.exit_price || 0;
        result.basis = exitPrice * trade.quantity;
    }

    // 2. Calculate Withholding Tax (IRRF / Dedo-duro)
    // IRRF is ALWAYS positive, even on loss, if based on SaleAmount (Swing Trade).
    // If based on NetProfit (Day Trade), it is only positive if there is profit.
    if (rule.basis === "SaleAmount") {
        // Swing Trade Ações: 0.005% sobre valor da venda
        result.withholding_tax = result.basis * (rule.withholding_rate / 100);
    } else {
        // Day Trade: 1% sobre o Lucro Líquido POSITIVO (SOMENTE SE TIVER LUCRO)
        if (result.basis > 0) {
            result.withholding_tax = result.basis * (rule.withholding_rate / 100);
        }
    }

    // 3. Calculate Tax Due (Imposto a Pagar)
    // Only applied if result is positive.
    // NOTE: Exemption logic (20k limit) cannot be fully applied here because it depends on MONTHLY volume.
    // We will flag it as 'potentially exempt' if rule has threshold, but final check is in Dashboard.
    // For single trade estimation, we assume it MIGHT be taxed if profitable.

    if (netResult > 0) {
        result.tax_due = netResult * (rule.tax_rate / 100);
        result.rate = rule.tax_rate;
    }

    return result;
}

/**
 * Utility to find the correct Tax Rule for a trade based on mappings.
 */
export function findTaxRuleForTrade(
    trade: Trade,
    mappings: TaxMapping[],
    rules: TaxRule[]
): TaxRule | undefined {
    // 1. Try to find explicit mapping for (AssetType + Modality)
    const mapping = mappings.find(
        m => m.asset_type_id === trade.asset_type_id && m.modality_id === trade.modality_id
    );

    if (mapping) {
        return rules.find(r => r.id === mapping.tax_rule_id);
    }

    // 2. Fallback? Currently no global fallback.
    return undefined;
}
