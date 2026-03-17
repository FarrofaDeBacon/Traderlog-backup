use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxAppraisalDto {
    pub id: Option<String>,
    pub period_month: u8,
    pub period_year: u16,
    pub trade_type: String,
    pub tax_rule_id: Option<String>,
    pub revenue_code: Option<String>,
    pub gross_profit: f64,
    pub loss: f64,
    pub net_profit: f64,
    pub compensated_loss: f64,
    pub calculation_basis: f64,
    pub tax_rate: f64,
    pub tax_due: f64,
    pub withheld_tax: f64,
    pub withholding_credit_used: f64,
    pub withholding_credit_remaining: f64,
    pub tax_payable: f64,
    pub tax_accumulated: f64,
    pub total_payable: f64,
    pub is_exempt: bool,
    pub calculation_date: String,
    pub status: String,
    pub trade_ids: Vec<String>,
    pub is_complementary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxLossDto {
    pub id: Option<String>,
    pub trade_type: String,
    pub amount: f64,
    pub origin_date: String,
    pub balance: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxDarfDto {
    pub id: Option<String>,
    pub appraisal_id: Option<String>,
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
    pub account_id: Option<String>,
    pub transaction_id: Option<String>,
    pub is_complementary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfileDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub cpf: String,
    pub theme: String,
    pub language: String,
    pub timezone: String,
    pub main_currency: String,
    pub avatar: Option<String>,
    pub onboarding_completed: bool,
    pub birth_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountDto {
    pub id: Option<String>,
    pub nickname: String,
    pub account_type: String,
    pub broker: String,
    pub account_number: String,
    pub currency: String,
    pub balance: f64,
    pub custom_logo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetDto {
    pub id: Option<String>,
    pub symbol: String,
    pub name: String,
    pub asset_type_id: Option<String>,
    pub point_value: f64,
    pub tax_profile_id: Option<String>,
    pub is_root: bool,
    pub root_id: Option<String>,
    pub contract_size: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetRiskProfileDto {
    pub id: Option<String>,
    pub name: String,
    pub asset_id: Option<String>,
    pub default_stop_points: f64,
    pub min_contracts: i32,
    pub max_contracts: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetTypeDto {
    pub id: Option<String>,
    pub name: String,
    pub code: String,
    pub market_id: Option<String>,
    pub unit_label: String,
    pub result_type: String,
    pub tax_profile_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketDto {
    pub id: Option<String>,
    pub code: String,
    pub name: String,
    pub timezone: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModalityDto {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeDto {
    pub id: String,
    pub date: String,
    pub asset_symbol: String,
    pub asset_type_id: Option<String>,
    pub strategy_id: Option<String>,
    pub account_id: Option<String>,
    pub result: f64,
    pub quantity: f64,
    pub direction: String,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub exit_date: Option<String>,
    pub fee_total: f64,
    pub notes: String,
    pub timeframe: String,
    pub followed_plan: bool,
    pub asset_id: Option<String>,
    pub modality_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JournalEntryDto {
    pub id: String,
    pub date: String,
    pub content: String,
    pub emotional_state_id: Option<String>,
    pub daily_score: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashTransactionDto {
    pub id: String,
    pub date: String,
    pub amount: f64,
    pub r#type: String,
    pub description: String,
    pub account_id: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxRuleDto {
    pub id: Option<String>,
    pub name: String,
    pub tax_rate: f64,
    pub withholding_rate: f64,
    pub exemption_threshold: f64,
    pub basis: String,
    pub cumulative_losses: bool,
    pub trade_type: String,
    pub withholding_basis: String,
    pub revenue_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxMappingDto {
    pub id: Option<String>,
    pub asset_type_id: Option<String>,
    pub modality_id: Option<String>,
    pub tax_rule_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxProfileDto {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxProfileEntryDto {
    pub id: Option<String>,
    pub tax_profile_id: Option<String>,
    pub modality_id: Option<String>,
    pub tax_rule_id: Option<String>,
}
