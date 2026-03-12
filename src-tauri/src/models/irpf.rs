use crate::models::{deserialize_id_opt, deserialize_vec_id, dto, ToDto};
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
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_rule_id: Option<String>, // Link to the specific rule used
    #[serde(default)]
    pub revenue_code: Option<String>, // "6015" or "3317"
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
    #[serde(default, deserialize_with = "deserialize_vec_id")]
    pub trade_ids: Vec<String>,
    #[serde(default)]
    pub is_complementary: bool,
}

impl ToDto for TaxAppraisal {
    type Dto = dto::TaxAppraisalDto;
    fn to_dto(&self) -> Self::Dto {
        dto::TaxAppraisalDto {
            id: self.id.clone(),
            period_month: self.period_month,
            period_year: self.period_year,
            trade_type: self.trade_type.clone(),
            tax_rule_id: self.tax_rule_id.clone(),
            revenue_code: self.revenue_code.clone(),
            gross_profit: self.gross_profit,
            loss: self.loss,
            net_profit: self.net_profit,
            compensated_loss: self.compensated_loss,
            calculation_basis: self.calculation_basis,
            tax_rate: self.tax_rate,
            tax_due: self.tax_due,
            withheld_tax: self.withheld_tax,
            withholding_credit_used: self.withholding_credit_used,
            withholding_credit_remaining: self.withholding_credit_remaining,
            tax_payable: self.tax_payable,
            tax_accumulated: self.tax_accumulated,
            total_payable: self.total_payable,
            is_exempt: self.is_exempt,
            calculation_date: self.calculation_date.clone(),
            status: self.status.clone(),
            trade_ids: self.trade_ids.clone(),
            is_complementary: self.is_complementary,
        }
    }
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

impl ToDto for TaxLoss {
    type Dto = dto::TaxLossDto;
    fn to_dto(&self) -> Self::Dto {
        dto::TaxLossDto {
            id: self.id.clone(),
            trade_type: self.trade_type.clone(),
            amount: self.amount,
            origin_date: self.origin_date.clone(),
            balance: self.balance,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxDarf {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(deserialize_with = "deserialize_id_opt")]
    pub appraisal_id: Option<String>,
    #[serde(default)]
    pub revenue_code: Option<String>,
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

impl ToDto for TaxDarf {
    type Dto = dto::TaxDarfDto;
    fn to_dto(&self) -> Self::Dto {
        dto::TaxDarfDto {
            id: self.id.clone(),
            appraisal_id: self.appraisal_id.clone(),
            revenue_code: self.revenue_code.clone(),
            period: self.period.clone(),
            principal_value: self.principal_value,
            fine: self.fine,
            interest: self.interest,
            total_value: self.total_value,
            due_date: self.due_date.clone(),
            payment_date: self.payment_date.clone(),
            status: self.status.clone(),
            darf_number: self.darf_number.clone(),
            account_id: self.account_id.clone(),
            transaction_id: self.transaction_id.clone(),
            is_complementary: self.is_complementary,
        }
    }
}
