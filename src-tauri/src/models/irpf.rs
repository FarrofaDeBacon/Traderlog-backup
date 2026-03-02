use crate::models::{deserialize_id, deserialize_id_opt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxAppraisal {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(rename = "period_month", alias = "month", alias = "appraisal_month")]
    pub period_month: u8,
    #[serde(rename = "period_year", alias = "year", alias = "appraisal_year")]
    pub period_year: u16,
    pub trade_type: String, // "DayTrade" or "SwingTrade" (Category)
    #[serde(deserialize_with = "deserialize_id")]
    pub tax_rule_id: String, // Link to the specific rule used
    pub revenue_code: String, // "6015" or "3317"
    pub gross_profit: f64,
    pub loss: f64,
    pub net_profit: f64,
    pub compensated_loss: f64,
    pub calculation_basis: f64,
    pub tax_rate: f64,
    pub tax_due: f64,
    pub withheld_tax: f64,
    #[serde(default)]
    pub withholding_credit_used: f64,
    #[serde(default)]
    pub withholding_credit_remaining: f64,
    pub tax_payable: f64,
    #[serde(default)]
    pub tax_accumulated: f64,
    #[serde(default)]
    pub total_payable: f64,
    pub is_exempt: bool,
    pub calculation_date: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub trade_ids: Vec<String>,
    #[serde(default)]
    pub is_complementary: bool,
}

fn default_status() -> String {
    "Pending".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxLoss {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    pub trade_type: String,
    pub amount: f64,
    pub origin_date: String,
    pub balance: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxDarf {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(deserialize_with = "deserialize_id")]
    pub appraisal_id: String,
    pub revenue_code: String,
    pub period: String,
    pub principal_value: f64,
    pub fine: f64,
    pub interest: f64,
    pub total_value: f64,
    pub due_date: String,
    pub payment_date: Option<String>,
    pub status: String,
    pub darf_number: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub account_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub transaction_id: Option<String>,
    #[serde(default)]
    pub is_complementary: bool,
}
