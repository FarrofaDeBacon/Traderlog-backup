use serde::{Deserialize, Serialize};
use serde_json;
use surrealdb::sql::Thing;

use serde::de::{self, Visitor};
use std::fmt;

// Robust helper to handle IDs from both JSON (WebSocket/HTTP) and Binary (SurrealKV)
struct SurrealIdInner(String);

impl<'de> Deserialize<'de> for SurrealIdInner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InnerVisitor;
        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = SurrealIdInner;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid ID value (string, number, object, enum)")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SurrealIdInner(v.to_string()))
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SurrealIdInner(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SurrealIdInner(v.to_string()))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SurrealIdInner(v.to_string()))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SurrealIdInner(v.to_string()))
            }
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SurrealIdInner(v.to_string()))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                // Manually parse map to avoid codec-incompatible serde_json::Value
                let mut string_val = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "String" || key == "Uuid" {
                        string_val = Some(map.next_value::<String>()?);
                    } else {
                        let _ = map.next_value::<de::IgnoredAny>()?;
                    }
                }
                Ok(SurrealIdInner(
                    string_val.unwrap_or_else(|| "id".to_string()),
                ))
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                let (variant, variant_access) = data.variant::<String>()?;
                use serde::de::VariantAccess;
                let content: String = match variant.as_str() {
                    "String" | "Uuid" => variant_access.newtype_variant()?,
                    "Number" => {
                        let n: i64 = variant_access.newtype_variant()?;
                        n.to_string()
                    }
                    _ => {
                        let _ = variant_access.newtype_variant::<de::IgnoredAny>()?;
                        variant
                    }
                };
                Ok(SurrealIdInner(content))
            }
        }
        deserializer.deserialize_any(InnerVisitor)
    }
}

struct IdVisitor;

impl<'de> Visitor<'de> for IdVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string, a map with tb/id, or a Thing enum")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_string())
    }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        let mut tb = None;
        let mut id = None;

        while let Some(key) = map.next_key::<String>()? {
            if key == "tb" {
                tb = Some(map.next_value::<String>()?);
            } else if key == "id" {
                let val: SurrealIdInner = map.next_value()?;
                id = Some(val.0);
            } else {
                let _ = map.next_value::<de::IgnoredAny>()?;
            }
        }

        let id_str = id.unwrap_or_default();
        if let Some(t) = tb {
            Ok(format!("{}:{}", t, id_str))
        } else {
            Ok(id_str)
        }
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        let (variant, variant_access) = data.variant::<String>()?;
        use serde::de::VariantAccess;
        match variant.as_str() {
            "String" | "Uuid" => variant_access.newtype_variant(),
            _ => {
                let val = variant_access
                    .newtype_variant::<String>()
                    .unwrap_or_else(|_| variant);
                Ok(val)
            }
        }
    }
}

pub fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_any(IdVisitor)
}

pub fn deserialize_id_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptionIdVisitor;
    impl<'de> Visitor<'de> for OptionIdVisitor {
        type Value = Option<String>;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional string, map, or Thing")
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserialize_id(deserializer).map(Some)
        }
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_string()))
            }
        }
        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
    }
    deserializer.deserialize_option(OptionIdVisitor)
}

pub fn deserialize_as_json<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct JsonValueVisitor;

    // Wrapper to allow recursion using our custom visitor instead of default serde_json::Value impl
    struct JsonWrapper(serde_json::Value);
    impl<'de> Deserialize<'de> for JsonWrapper {
        fn deserialize<Dr>(deserializer: Dr) -> Result<Self, Dr::Error>
        where
            Dr: serde::Deserializer<'de>,
        {
            deserialize_as_json(deserializer).map(JsonWrapper)
        }
    }

    impl<'de> Visitor<'de> for JsonValueVisitor {
        type Value = serde_json::Value;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("any JSON value")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
            Ok(serde_json::Value::Bool(v))
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(serde_json::json!(v))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(serde_json::json!(v))
        }
        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
            Ok(serde_json::json!(v))
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(serde_json::Value::String(v.to_string()))
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(serde_json::Value::String(v))
        }
        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(serde_json::Value::Null)
        }
        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(serde_json::Value::Null)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(wrapper) = seq.next_element::<JsonWrapper>()? {
                vec.push(wrapper.0);
            }
            Ok(serde_json::Value::Array(vec))
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let mut obj = serde_json::Map::new();
            while let Some(key) = map.next_key::<String>()? {
                let wrapper = map.next_value::<JsonWrapper>()?;
                obj.insert(key, wrapper.0);
            }
            Ok(serde_json::Value::Object(obj))
        }

        fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
        where
            A: de::EnumAccess<'de>,
        {
            let (variant, variant_access) = data.variant::<String>()?;
            use serde::de::VariantAccess;
            match variant.as_str() {
                "String" | "Uuid" => {
                    let s: String = variant_access.newtype_variant()?;
                    Ok(serde_json::Value::String(s))
                }
                "Number" | "Int" | "Float" => {
                    // Try as f64 first
                    if let Ok(n) = variant_access.newtype_variant::<f64>() {
                        Ok(serde_json::json!(n))
                    } else {
                        Ok(serde_json::Value::Null)
                    }
                }
                "None" | "Null" => {
                    let _ = variant_access.unit_variant();
                    Ok(serde_json::Value::Null)
                }
                "Array" => {
                    let wrapper: JsonWrapper = variant_access.newtype_variant()?;
                    Ok(wrapper.0)
                }
                "Object" => {
                    let wrapper: JsonWrapper = variant_access.newtype_variant()?;
                    Ok(wrapper.0)
                }
                _ => {
                    // Fallback: try newtype, if fails return string representation
                    match variant_access.newtype_variant::<JsonWrapper>() {
                        Ok(wrapper) => Ok(wrapper.0),
                        Err(_) => Ok(serde_json::Value::String(variant)),
                    }
                }
            }
        }
    }
    deserializer.deserialize_any(JsonValueVisitor)
}

pub fn deserialize_vec_id<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct VecIdVisitor;
    impl<'de> Visitor<'de> for VecIdVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of strings, maps, or Things")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(wrapper) = seq.next_element::<SurrealIdInner>()? {
                vec.push(wrapper.0);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_any(VecIdVisitor)
}

pub mod irpf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub cpf: String,
    pub theme: String,    // "dark" | "light"
    pub language: String, // "pt-BR" | "en"
    #[serde(default = "default_timezone")]
    pub timezone: String,
    pub main_currency: String,
    pub avatar: Option<String>,
    #[serde(default)]
    pub convert_all_to_main: bool,
    #[serde(default)]
    pub onboarding_completed: bool,
    pub currency_api_url: Option<String>,
    pub birth_date: Option<String>,
    pub trial_start_date: Option<String>, // ISO Date
    pub license_key: Option<String>,      // Signed license string or file path
    #[serde(default = "default_utc_offset")]
    pub utc_offset: i32, // Offset in minutes (e.g., -180 for Brasilia)
}

fn default_utc_offset() -> i32 {
    -180
}

fn default_timezone() -> String {
    "America/Sao_Paulo".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub provider: String,
    pub api_key: String,
    pub enabled: bool,
    pub daily_limit: i64,
    pub requests_today: i64,
    pub extra_config: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub nickname: String,
    pub account_type: String, // "Real" | "Demo" | "Prop"
    pub broker: String,
    pub account_number: String,
    pub currency: String,
    pub balance: f64,
    pub custom_logo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Currency {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub symbol: String,
    pub name: String,
    pub exchange_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradingSession {
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Market {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub trading_days: Vec<i32>,
    pub trading_sessions: Vec<TradingSession>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetType {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub code: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub market_id: String,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub default_fee_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_profile_id: Option<String>, // New field
    pub unit_label: String,
    pub result_type: String, // "points" | "currency"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub symbol: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub asset_type_id: String,
    pub point_value: f64,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub default_fee_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_profile_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmotionalState {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub impact: String, // "Positive" | "Negative" | "Neutral"
    pub description: String,
    pub potential_impact: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    5.0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategyImage {
    pub path: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Strategy {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(deserialize_with = "crate::models::deserialize_vec_id")]
    pub market_ids: Vec<String>,
    #[serde(deserialize_with = "crate::models::deserialize_vec_id")]
    pub timeframes: Vec<String>,
    #[serde(deserialize_with = "crate::models::deserialize_vec_id")]
    pub asset_types: Vec<String>,
    #[serde(deserialize_with = "crate::models::deserialize_vec_id")]
    pub indicators: Vec<String>,
    #[serde(deserialize_with = "crate::models::deserialize_vec_id")]
    pub specific_assets: Vec<String>,
    pub entry_criteria: String,
    pub exit_criteria: String,
    pub management_criteria: String,
    pub has_partial: bool,
    pub partial_description: String,
    pub images: Vec<StrategyImage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashTransaction {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub date: String,
    pub amount: f64,
    pub r#type: String, // "Deposit" | "Withdraw" | "Adjustment"
    pub description: String,
    #[serde(deserialize_with = "crate::models::deserialize_id")]
    pub account_id: String,
    pub trade_ids: Option<Vec<String>>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub system_linked: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JournalEntry {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub date: String,
    pub content: String,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub emotional_state_id: Option<String>,
    #[serde(default)]
    pub intensity: i32,
    #[serde(default)]
    pub followed_plan: bool,
    #[serde(default)]
    pub risk_accepted: bool,
    #[serde(default)]
    pub market_context: String,
    #[serde(default)]
    pub daily_score: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub asset_symbol: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub asset_type_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub strategy_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub account_id: String,
    #[serde(default)]
    pub result: f64,
    #[serde(default)]
    pub quantity: f64,
    #[serde(default)]
    pub direction: String, // "Buy" | "Sell"
    #[serde(default)]
    pub entry_price: f64,
    #[serde(default)]
    pub exit_price: Option<f64>,
    #[serde(default)]
    pub exit_date: Option<String>,
    #[serde(default)]
    pub fee_total: f64,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub timeframe: String,
    #[serde(default)]
    pub volatility: String,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub entry_emotional_state_id: Option<String>,
    #[serde(default)]
    pub entry_emotional_state_name: Option<String>,
    #[serde(default)]
    pub exit_reason: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub exit_emotional_state_id: Option<String>,
    #[serde(default)]
    pub exit_emotional_state_name: Option<String>,
    #[serde(default)]
    pub entry_rationale: String,
    #[serde(default)]
    pub confirmation_signals: String,
    #[serde(default)]
    pub market_context: String,
    #[serde(default)]
    pub relevant_news: String,
    #[serde(default)]
    pub followed_plan: bool,
    #[serde(default)]
    pub what_worked: String,
    #[serde(default)]
    pub mistakes_improvements: String,
    #[serde(default)]
    pub lessons_learned: String,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default, deserialize_with = "crate::models::deserialize_as_json")]
    pub partial_exits: serde_json::Value,
    #[serde(default, deserialize_with = "crate::models::deserialize_id_opt")]
    pub asset_id: Option<String>,
    #[serde(default, deserialize_with = "crate::models::deserialize_id_opt")]
    pub modality_id: Option<String>,
    #[serde(default)]
    pub stop_loss: Option<f64>,
    #[serde(default)]
    pub take_profit: Option<f64>,
    #[serde(default)]
    pub intensity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeeCustomItem {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub value: f64,
    pub r#type: String,   // "fixed" | "percentage"
    pub category: String, // "cost" | "tax"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeeProfile {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub broker: String,
    pub fixed_fee: f64,
    pub percentage_fee: f64,
    pub exchange_fee: f64,
    pub iss: f64,
    pub currency_spread: f64,
    pub withholding_tax: f64,
    pub income_tax_rate: f64,
    pub custom_items: Vec<FeeCustomItem>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_rule_id: Option<String>,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrowthPhaseProgressionRule {
    pub condition: String, // "profit_target" | "days_positive" | "consistency_days"
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrowthPhaseRegressionRule {
    pub condition: String, // "drawdown_limit" | "max_daily_loss_streak"
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrowthPhase {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_lots: f64,
    pub max_daily_loss: f64,
    pub progression_rules: Vec<GrowthPhaseProgressionRule>,
    pub regression_rules: Vec<GrowthPhaseRegressionRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiskProfile {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub max_daily_loss: f64,
    pub daily_target: f64,
    pub max_risk_per_trade_percent: f64,
    pub max_trades_per_day: i32,
    pub min_risk_reward: f64,
    pub lock_on_loss: bool,
    pub account_type_applicability: String, // "All" | "Prop" | "Real" | "Demo"
    pub growth_plan_enabled: bool,
    pub current_phase_index: i32,
    pub growth_phases: Vec<GrowthPhase>,
    #[serde(default)]
    pub psychological_coupling_enabled: bool,
    #[serde(default)]
    pub outlier_regression_enabled: bool,
    #[serde(default)]
    pub sniper_mode_enabled: bool,
    #[serde(default)]
    pub sniper_mode_selectivity: i32,
    #[serde(default = "default_psyc_lookback")]
    pub psychological_lookback_count: i32,
    #[serde(default = "default_outlier_lookback")]
    pub outlier_lookback_count: i32,
    #[serde(default = "default_psyc_threshold")]
    pub psychological_threshold: i32,
    #[serde(default = "default_lot_reduction")]
    pub lot_reduction_multiplier: f64,
    #[serde(default = "default_psyc_strategy")]
    pub psychological_search_strategy: String, // "Strict" | "Sequence"
}

fn default_psyc_lookback() -> i32 {
    10
}
fn default_outlier_lookback() -> i32 {
    20
}
fn default_psyc_threshold() -> i32 {
    -2
}
fn default_lot_reduction() -> f64 {
    0.5
}
fn default_psyc_strategy() -> String {
    "Strict".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Modality {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndicatorParameter {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Indicator {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub category: String,  // "Trend" | "Oscillator" | "Volume" | "Other"
    pub plot_type: String, // "Overlay" | "SubWindow"
    pub default_color: String,
    pub usage_description: String,
    pub parameters: Vec<IndicatorParameter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timeframe {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartType {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub base_type: String, // "TimeBased" | "Renko" | "Range"
    pub parameter: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxRule {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub tax_rate: f64,
    pub withholding_rate: f64,
    pub exemption_threshold: f64,
    pub basis: String, // "NetProfit" | "GrossProfit"
    pub cumulative_losses: bool,
    #[serde(default = "default_trade_type")]
    pub trade_type: String, // "DayTrade" | "SwingTrade"
    #[serde(default = "default_withholding_basis")]
    pub withholding_basis: String, // "Profit" | "SalesVolume"
    #[serde(default)]
    pub revenue_code: String, // e.g., "6015", "3317"
}

fn default_trade_type() -> String {
    "SwingTrade".to_string()
}
fn default_withholding_basis() -> String {
    "Profit".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxMapping {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub asset_type_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub modality_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub tax_rule_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxProfile {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxProfileEntry {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub tax_profile_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub modality_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub tax_rule_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxPayment {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub period: String, // "YYYY-MM"
    pub tax_due: f64,
    pub fine: f64,
    pub interest: f64,
    pub total_paid: f64,
    pub payment_date: Option<String>,
    pub status: String, // "Pending" | "Paid"
    pub notes: String,
}
