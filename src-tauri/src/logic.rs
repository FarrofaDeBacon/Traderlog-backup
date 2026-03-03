use crate::models::irpf::TaxAppraisal;
use crate::models::TaxRule;

#[derive(Debug, Clone)]
pub struct RuleBucket {
    pub rule: TaxRule,
    pub modality_id: String,
    pub gross_profit: f64,
    pub gross_loss: f64,
    pub sales_total: f64,
    pub trade_ids: Vec<String>,
}

pub fn calculate_appraisal(
    bucket: &RuleBucket,
    month: u8,
    year: u16,
    available_loss: f64,
    previous_credit: f64,
    tax_accumulated: f64,
) -> TaxAppraisal {
    let rule = &bucket.rule;
    let gross_profit = bucket.gross_profit;
    let gross_loss = bucket.gross_loss;
    let bucket_trades = &bucket.trade_ids;

    // 1. Net Result
    let net_profit = gross_profit - gross_loss;

    // 2. Exemption Logic
    let threshold = rule.exemption_threshold;
    let is_exempt = threshold > 0.0 && bucket.sales_total <= threshold && net_profit > 0.0;

    // 3. Compensation Logic
    let mut compensated_loss = 0.0;
    let mut calculation_basis;

    let basis_profit = if rule.basis == "GrossProfit" {
        gross_profit
    } else {
        net_profit
    };

    if is_exempt || !rule.cumulative_losses {
        calculation_basis = if is_exempt { 0.0 } else { basis_profit };
    } else {
        if basis_profit > 0.0 {
            if available_loss > 0.0 {
                if available_loss >= basis_profit {
                    compensated_loss = basis_profit;
                    calculation_basis = 0.0;
                } else {
                    compensated_loss = available_loss;
                    calculation_basis = basis_profit - available_loss;
                }
            } else {
                calculation_basis = basis_profit;
            }
        } else {
            calculation_basis = 0.0;
        }
    }

    // 4. Tax Calculation
    let rate_decimal = rule.tax_rate / 100.0;
    let tax_due = calculation_basis * rate_decimal;

    // Withholding (Dedo-duro)
    let withheld_rate_decimal = rule.withholding_rate / 100.0;
    let total_irrf = if rule.withholding_basis == "Profit" {
        if net_profit > 0.0 {
            net_profit * withheld_rate_decimal
        } else {
            0.0
        }
    } else {
        if bucket.sales_total > 0.0 {
            bucket.sales_total * withheld_rate_decimal
        } else {
            0.0
        }
    };

    let total_deduction_available = total_irrf + previous_credit;
    let mut withholding_credit_used = 0.0;
    let mut tax_after_irrf = 0.0;
    let mut withholding_credit_remaining = 0.0;

    if tax_due > 0.0 {
        if total_deduction_available >= tax_due {
            withholding_credit_used = tax_due;
            tax_after_irrf = 0.0;
            withholding_credit_remaining = total_deduction_available - tax_due;
        } else {
            withholding_credit_used = total_deduction_available;
            tax_after_irrf = tax_due - total_deduction_available;
            withholding_credit_remaining = 0.0;
        }
    } else {
        withholding_credit_used = 0.0;
        tax_after_irrf = 0.0;
        withholding_credit_remaining = total_deduction_available;
    }

    let mut tax_payable = tax_after_irrf;
    if tax_payable < 0.0 {
        tax_payable = 0.0;
    }

    let total_payable = tax_payable + tax_accumulated;

    TaxAppraisal {
        id: None,
        period_month: month,
        period_year: year,
        trade_type: rule.trade_type.clone(),
        tax_rule_id: rule.id.clone(),
        revenue_code: rule.revenue_code.clone(),
        gross_profit,
        loss: gross_loss,
        net_profit,
        compensated_loss,
        calculation_basis,
        tax_rate: rule.tax_rate,
        tax_due,
        withheld_tax: total_irrf,
        withholding_credit_used,
        withholding_credit_remaining,
        tax_payable,
        tax_accumulated,
        total_payable,
        is_exempt,
        calculation_date: "".to_string(),
        status: "Pending".to_string(),
        trade_ids: bucket_trades.clone(),
        is_complementary: false,
    }
}
