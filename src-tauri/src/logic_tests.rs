use crate::logic::{calculate_appraisal, RuleBucket};
use crate::models::TaxRule;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_rule(trade_type: &str, rate: f64, exemption: f64) -> TaxRule {
        TaxRule {
            id: Some("rule1".to_string()),
            name: "Mock Rule".to_string(),
            tax_rate: rate,
            withholding_rate: 1.0,
            exemption_threshold: exemption,
            basis: "NetProfit".to_string(),
            cumulative_losses: true,
            trade_type: trade_type.to_string(),
            withholding_basis: "Profit".to_string(),
            revenue_code: Some("6015".to_string()),
        }
    }

    #[test]
    fn test_day_trade_no_losses() {
        let rule = create_mock_rule("DayTrade", 20.0, 0.0);
        let bucket = RuleBucket {
            rule: rule.clone(),
            gross_profit: 1000.0,
            gross_loss: 200.0,
            sales_total: 5000.0,
            trade_ids: vec!["t1".to_string()],
        };

        let appraisal = calculate_appraisal(&bucket, 3, 2024, 0.0, 0.0, 0.0, 0.0);

        assert_eq!(appraisal.net_profit, 800.0);
        assert_eq!(appraisal.tax_due, 160.0); // 20% of 800
        assert_eq!(appraisal.withheld_tax, 8.0); // 1% of 800 (withholding_basis: Profit)
        assert_eq!(appraisal.tax_payable, 152.0); // 160 - 8
        assert_eq!(appraisal.total_payable, 152.0);
    }

    #[test]
    fn test_swing_trade_with_exemption() {
        let rule = create_mock_rule("SwingTrade", 15.0, 20000.0);
        rule.clone().withholding_basis = "SalesVolume".to_string(); // Adjusted manually if needed, but default in mock is Profit.

        let bucket = RuleBucket {
            rule: rule.clone(),
            gross_profit: 1000.0,
            gross_loss: 0.0,
            sales_total: 15000.0, // Below 20k threshold
            trade_ids: vec!["t2".to_string()],
        };

        let appraisal = calculate_appraisal(&bucket, 3, 2024, 0.0, 0.0, 0.0, 0.0);

        assert!(appraisal.is_exempt);
        assert_eq!(appraisal.tax_due, 0.0);
        assert_eq!(appraisal.tax_payable, 0.0);
    }

    #[test]
    fn test_loss_compensation() {
        let rule = create_mock_rule("DayTrade", 20.0, 0.0);
        let bucket = RuleBucket {
            rule: rule.clone(),
            gross_profit: 1000.0,
            gross_loss: 0.0,
            sales_total: 5000.0,
            trade_ids: vec!["t3".to_string()],
        };

        // compensation with 600 available loss
        let appraisal = calculate_appraisal(&bucket, 3, 2024, 600.0, 0.0, 0.0, 0.0);

        assert_eq!(appraisal.compensated_loss, 600.0);
        assert_eq!(appraisal.calculation_basis, 400.0);
        assert_eq!(appraisal.tax_due, 80.0); // 20% of 400
    }

    #[test]
    fn test_irrf_credit_carryover() {
        let rule = create_mock_rule("DayTrade", 20.0, 0.0);
        let bucket = RuleBucket {
            rule: rule.clone(),
            gross_profit: 500.0,
            gross_loss: 0.0,
            sales_total: 1000.0,
            trade_ids: vec!["t4".to_string()],
        };

        // tax_due = 100.0. Current IRRF = 5.0. Prev credit = 110.0.
        let appraisal = calculate_appraisal(&bucket, 3, 2024, 0.0, 110.0, 0.0, 0.0);

        assert_eq!(appraisal.tax_due, 100.0);
        assert_eq!(appraisal.withheld_tax, 5.0);
        assert_eq!(appraisal.withholding_credit_used, 100.0);
        assert_eq!(appraisal.tax_payable, 0.0);
        assert_eq!(appraisal.withholding_credit_remaining, 15.0); // (5 + 110) - 100
    }

    #[test]
    fn test_swing_trade_exact_limit() {
        let rule = create_mock_rule("SwingTrade", 15.0, 20000.0);
        let bucket = RuleBucket {
            rule,
            gross_profit: 1000.0,
            gross_loss: 0.0,
            sales_total: 20000.0, // EXACT threshold
            trade_ids: vec!["t5".to_string()],
        };

        let appraisal = calculate_appraisal(&bucket, 3, 2024, 0.0, 0.0, 0.0, 0.0);
        // On B3, exactly 20k IS exempt (up to 20k)
        assert!(appraisal.is_exempt);
        assert_eq!(appraisal.tax_due, 0.0);
    }

    #[test]
    fn test_loss_carryover_overflow() {
        let rule = create_mock_rule("DayTrade", 20.0, 0.0);
        let bucket = RuleBucket {
            rule,
            gross_profit: 500.0,
            gross_loss: 0.0,
            sales_total: 1000.0,
            trade_ids: vec!["t6".to_string()],
        };

        // compensation with 2000 available loss (more than profit)
        let appraisal = calculate_appraisal(&bucket, 3, 2024, 2000.0, 0.0, 0.0, 0.0);

        assert_eq!(appraisal.compensated_loss, 500.0);
        assert_eq!(appraisal.calculation_basis, 0.0);
        assert_eq!(appraisal.tax_due, 0.0);
    }

    #[test]
    fn test_complementary_calculation() {
        let rule = create_mock_rule("DayTrade", 20.0, 0.0);
        let bucket = RuleBucket {
            rule,
            gross_profit: 2000.0, // Increased profit
            gross_loss: 0.0,
            sales_total: 5000.0,
            trade_ids: vec!["t7".to_string()],
        };

        // Original tax was 160.0 (from 800.0 profit at 20%).
        // Now profit is 2000.0 -> total tax due = 400.0.
        // If we already paid 160.0, the delta should be 240.0.
        let appraisal = calculate_appraisal(&bucket, 3, 2024, 0.0, 0.0, 0.0, 160.0);

        assert_eq!(appraisal.tax_due, 400.0);
        assert_eq!(appraisal.total_payable, 220.0); // 400 (due) - 20 (IRRF) - 160 (already paid)
        assert!(appraisal.is_complementary);
    }
}
