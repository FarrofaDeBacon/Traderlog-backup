use serde::{Deserialize, Serialize};
use serde_json;
use surrealdb::sql::{Thing, Value as SurrealValue};

use serde::de::{self, Visitor};
use std::fmt;

pub trait ToDto {
    type Dto;
    fn to_dto(&self) -> Self::Dto;
}

// Robust helper to handle IDs from both JSON (WebSocket/HTTP) and Binary (SurrealKV)
#[derive(Debug, Clone)]
pub struct SurrealId(pub String);

struct UniversalVisitor;

/// DeserializeSeed that recursively applies UniversalVisitor so enums (SurrealDB Things)
/// inside sequences and maps are handled correctly.
struct UniversalSeed;

impl<'de> de::DeserializeSeed<'de> for UniversalSeed {
    type Value = serde_json::Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<serde_json::Value, D::Error> {
        d.deserialize_any(UniversalVisitor)
    }
}

impl<'de> Visitor<'de> for UniversalVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any value (String, Object, Seq, or Enum)")
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
        let mut arr = Vec::new();
        // Use UniversalSeed so SurrealDB Things (enums) inside arrays are handled recursively
        while let Some(val) = seq.next_element_seed(UniversalSeed)? {
            arr.push(val);
        }
        Ok(serde_json::Value::Array(arr))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut obj = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            // Use UniversalSeed so SurrealDB Things (enums) inside maps are handled recursively
            let val = map.next_value_seed(UniversalSeed)?;
            obj.insert(key, val);
        }
        Ok(serde_json::Value::Object(obj))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        use serde::de::VariantAccess;
        let (tag, variant) = data.variant::<String>()?;
        println!("[DEBUG] UniversalVisitor::visit_enum tag: {}", tag);
        match tag.as_str() {
            "Thing" => {
                let t: Thing = variant.newtype_variant()?;
                let mut obj = serde_json::Map::new();
                obj.insert("tb".to_string(), t.tb.into());
                obj.insert("id".to_string(), t.id.to_string().into());
                Ok(serde_json::Value::Object(obj))
            }
            "Strand" | "String" | "Uuid" | "Number" | "Int" | "Float" | "Decimal" | "Array"
            | "Object" => {
                // These are common data-carrying variants in SurrealDB's Id and Value enums.
                // We use UniversalSeed to recursively handle any nested enums.
                let val: serde_json::Value = variant.newtype_variant_seed(UniversalSeed)?;
                Ok(val)
            }
            "None" | "Null" => {
                let _ = variant.unit_variant()?;
                Ok(serde_json::Value::Null)
            }
            _ => {
                println!("[DEBUG] UniversalVisitor::visit_enum UNKNOWN tag: {}", tag);
                // For other enums, we assume they are simple unit variants (like categories).
                // If they carry data, we might miss it here, but this prevents "moved value" errors.
                match variant.unit_variant() {
                    Ok(_) => Ok(serde_json::Value::String(tag)),
                    Err(e) => {
                        // If it's not a unit variant, try to capture it as a newtype variant (last resort)
                        // Note: we can't retry the variant access easily if it failed once,
                        // but this part is already quite deep in the fallback.
                        eprintln!(
                            "[ERROR] UniversalVisitor::visit_enum fallback failed for tag {}: {}",
                            tag, e
                        );
                        Ok(serde_json::Value::String(format!("{}:[DATA]", tag)))
                    }
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for SurrealId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = deserializer.deserialize_any(UniversalVisitor)?;
        match val {
            serde_json::Value::String(s) => Ok(SurrealId(s)),
            serde_json::Value::Object(ref obj) => {
                if let (Some(tb), Some(id)) = (obj.get("tb"), obj.get("id")) {
                    let tb_str = tb.as_str().unwrap_or("");
                    let id_str = match id {
                        serde_json::Value::String(s) => s.clone(),
                        _ => id.to_string(),
                    };
                    Ok(SurrealId(format!("{}:{}", tb_str, id_str)))
                } else {
                    match val {
                        serde_json::Value::Null => Ok(SurrealId(String::new())),
                        _ => Ok(SurrealId(val.to_string())),
                    }
                }
            }
            serde_json::Value::Null => Ok(SurrealId(String::new())),
            _ => Ok(SurrealId(val.to_string())),
        }
    }
}

pub fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    SurrealId::deserialize(deserializer).map(|s| s.0)
}

pub fn deserialize_id_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = deserializer.deserialize_any(UniversalVisitor)?;
    match val {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(ref s) => {
            if s.is_empty() || s == "null" {
                Ok(None)
            } else {
                Ok(Some(s.clone()))
            }
        }
        serde_json::Value::Object(ref obj) => {
            if let (Some(tb), Some(id)) = (obj.get("tb"), obj.get("id")) {
                let tb_str = tb.as_str().unwrap_or("");
                let id_str = match id {
                    serde_json::Value::String(ref s) => s.clone(),
                    _ => id.to_string(),
                };
                if id_str == "null" {
                    Ok(None)
                } else {
                    Ok(Some(format!("{}:{}", tb_str, id_str)))
                }
            } else {
                let s = val.to_string();
                if s == "null" || s == "\"null\"" { Ok(None) } else { Ok(Some(s)) }
            }
        }
        _ => {
            let s = val.to_string();
            if s == "null" || s == "\"null\"" { Ok(None) } else { Ok(Some(s)) }
        }
    }
}

pub fn deserialize_string_opt<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = deserializer.deserialize_any(UniversalVisitor)?;
    match val {
        serde_json::Value::Null => Ok(String::new()),
        serde_json::Value::String(s) => {
            if s == "null" { Ok(String::new()) } else { Ok(s) }
        }
        _ => {
            let s = val.to_string();
            if s == "null" || s == "\"null\"" { Ok(String::new()) } else { Ok(s) }
        }
    }
}

#[allow(dead_code)]
pub fn surreal_to_json(v: SurrealValue) -> serde_json::Value {
    match v {
        SurrealValue::None | SurrealValue::Null => serde_json::Value::Null,
        SurrealValue::Bool(b) => serde_json::Value::Bool(b),
        SurrealValue::Number(n) => match n {
            surrealdb::sql::Number::Int(i) => serde_json::json!(i),
            surrealdb::sql::Number::Float(f) => serde_json::json!(f),
            surrealdb::sql::Number::Decimal(d) => serde_json::json!(d.to_string()),
            _ => serde_json::Value::Null,
        },
        SurrealValue::Strand(s) => serde_json::Value::String(s.0),
        SurrealValue::Array(a) => {
            serde_json::Value::Array(a.0.into_iter().map(surreal_to_json).collect())
        }
        SurrealValue::Object(o) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in o.0 {
                obj.insert(k, surreal_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        SurrealValue::Thing(t) => {
            let mut obj = serde_json::Map::new();
            obj.insert("tb".to_string(), t.tb.into());
            obj.insert("id".to_string(), t.id.to_string().into());
            serde_json::Value::Object(obj)
        }
        SurrealValue::Uuid(u) => u.to_string().into(),
        SurrealValue::Datetime(d) => d.to_string().into(),
        _ => serde_json::Value::String(v.to_string()),
    }
}

pub fn deserialize_as_json<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_any(UniversalVisitor)
}

/// A wrapper for serde_json::Value that uses our custom robust deserializer.
/// This is essential when fetching raw data from SurrealDB that contains Record IDs (Things),
/// as the default serde_json::Value deserializer does not support the enum-based representation
/// used by the SurrealDB Rust driver.
#[derive(Debug, Serialize, Clone, Default)]
pub struct SurrealJson(pub serde_json::Value);

impl<'de> Deserialize<'de> for SurrealJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_as_json(deserializer).map(SurrealJson)
    }
}

pub fn deserialize_vec_id<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = deserializer.deserialize_any(UniversalVisitor)?;
    match val {
        serde_json::Value::Array(ref arr) => {
            let mut result = Vec::new();
            for item in arr {
                match item {
                    serde_json::Value::String(s) => result.push(s.clone()),
                    serde_json::Value::Object(ref obj) => {
                        if let (Some(tb), Some(id)) = (obj.get("tb"), obj.get("id")) {
                            let tb_str = tb.as_str().unwrap_or("");
                            let id_str = match id {
                                serde_json::Value::String(s) => s.clone(),
                                _ => id.to_string(),
                            };
                            result.push(format!("{}:{}", tb_str, id_str));
                        } else {
                            result.push(item.to_string());
                        }
                    }
                    _ => result.push(item.to_string()),
                }
            }
            Ok(result)
        }
        serde_json::Value::String(s) => Ok(vec![s]),
        _ => Ok(Vec::new()),
    }
}

pub mod dto;
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
    #[serde(default)]
    pub password_hash: Option<String>, // Local authentication hash
    #[serde(default)]
    pub recovery_hash: Option<String>, // Recovery key hash
    #[serde(default = "default_utc_offset")]
    pub utc_offset: i32, // Offset in minutes (e.g., -180 for Brasilia)
}

impl ToDto for UserProfile {
    type Dto = dto::UserProfileDto;
    fn to_dto(&self) -> Self::Dto {
        dto::UserProfileDto {
            id: self.id.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            cpf: self.cpf.clone(),
            theme: self.theme.clone(),
            language: self.language.clone(),
            timezone: self.timezone.clone(),
            main_currency: self.main_currency.clone(),
            avatar: self.avatar.clone(),
            onboarding_completed: self.onboarding_completed,
            birth_date: self.birth_date.clone(),
        }
    }
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
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub nickname: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub account_type: String, // "Checking", "Savings", "Brokerage", etc.
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub broker: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub account_number: String,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub currency_id: Option<String>,
    #[serde(default)]
    pub currency: Option<String>, // Virtual field for code during reads
    pub balance: f64,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub custom_logo: Option<String>,
}

impl ToDto for Account {
    type Dto = dto::AccountDto;
    fn to_dto(&self) -> Self::Dto {
        dto::AccountDto {
            id: self.id.clone(),
            nickname: self.nickname.clone(),
            account_type: self.account_type.clone(),
            broker: self.broker.clone(),
            account_number: self.account_number.clone(),
            currency: self.currency.clone().unwrap_or_default(),
            balance: self.balance,
            custom_logo: self.custom_logo.clone(),
        }
    }
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
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub code: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub timezone: String,
    #[serde(default)]
    pub trading_sessions: Vec<TradingSession>,
}

impl ToDto for Market {
    type Dto = dto::MarketDto;
    fn to_dto(&self) -> Self::Dto {
        dto::MarketDto {
            id: self.id.clone(),
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetType {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub code: String,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub market_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub default_fee_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_profile_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub unit_label: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub result_type: String, // "points" | "currency"
}

impl ToDto for AssetType {
    type Dto = dto::AssetTypeDto;
    fn to_dto(&self) -> Self::Dto {
        dto::AssetTypeDto {
            id: self.id.clone(),
            name: self.name.clone(),
            code: self.code.clone(),
            market_id: self.market_id.clone(),
            unit_label: self.unit_label.clone(),
            result_type: self.result_type.clone(),
            tax_profile_id: self.tax_profile_id.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub symbol: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub asset_type_id: Option<String>,
    pub point_value: f64,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub default_fee_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_profile_id: Option<String>,
    #[serde(default)]
    pub is_root: bool,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub root_id: Option<String>,
    // Position Sizing Engine Configurations
    #[serde(default)]
    pub contract_size: Option<f64>,
}

impl ToDto for Asset {
    type Dto = dto::AssetDto;
    fn to_dto(&self) -> Self::Dto {
        dto::AssetDto {
            id: self.id.clone(),
            symbol: self.symbol.clone(),
            name: self.name.clone(),
            asset_type_id: self.asset_type_id.clone(),
            point_value: self.point_value,
            tax_profile_id: self.tax_profile_id.clone(),
            is_root: self.is_root,
            root_id: self.root_id.clone(),
            contract_size: self.contract_size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetRiskProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub asset_id: Thing,
    #[serde(default)]
    pub default_stop_points: f64,
    #[serde(default)]
    pub min_contracts: i32,
    #[serde(default)]
    pub max_contracts: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ToDto for AssetRiskProfile {
    type Dto = dto::AssetRiskProfileDto;
    fn to_dto(&self) -> Self::Dto {
        dto::AssetRiskProfileDto {
            id: self.id.clone(),
            name: self.name.clone(),
            asset_id: Some(self.asset_id.to_string()),
            default_stop_points: self.default_stop_points,
            min_contracts: self.min_contracts,
            max_contracts: self.max_contracts,
            notes: self.notes.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmotionalState {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub impact: String, // "Positive" | "Negative" | "Neutral"
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
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
    #[serde(default, deserialize_with = "crate::models::deserialize_id_opt")]
    pub account_id: Option<String>,
    pub trade_ids: Option<Vec<String>>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub system_linked: Option<bool>,
}

impl ToDto for CashTransaction {
    type Dto = dto::CashTransactionDto;
    fn to_dto(&self) -> Self::Dto {
        dto::CashTransactionDto {
            id: self.id.clone(),
            date: self.date.clone(),
            amount: self.amount,
            r#type: self.r#type.clone(),
            description: self.description.clone(),
            account_id: self.account_id.clone(),
        }
    }
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

impl ToDto for JournalEntry {
    type Dto = dto::JournalEntryDto;
    fn to_dto(&self) -> Self::Dto {
        dto::JournalEntryDto {
            id: self.id.clone(),
            date: self.date.clone(),
            content: self.content.clone(),
            emotional_state_id: self.emotional_state_id.clone(),
            daily_score: self.daily_score,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub asset_symbol: String,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub asset_type_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub strategy_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub account_id: Option<String>,
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
    #[serde(default)]
    pub partial_exits: crate::models::SurrealJson,
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

impl ToDto for Trade {
    type Dto = dto::TradeDto;
    fn to_dto(&self) -> Self::Dto {
        dto::TradeDto {
            id: self.id.clone(),
            date: self.date.clone(),
            asset_symbol: self.asset_symbol.clone(),
            asset_type_id: self.asset_type_id.clone(),
            strategy_id: self.strategy_id.clone(),
            account_id: self.account_id.clone(),
            result: self.result,
            quantity: self.quantity,
            direction: self.direction.clone(),
            entry_price: self.entry_price,
            exit_price: self.exit_price,
            exit_date: self.exit_date.clone(),
            fee_total: self.fee_total,
            notes: self.notes.clone(),
            timeframe: self.timeframe.clone(),
            followed_plan: self.followed_plan,
            asset_id: self.asset_id.clone(),
            modality_id: self.modality_id.clone(),
        }
    }
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
    #[serde(default)]
    pub custom_items: Vec<FeeCustomItem>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_rule_id: Option<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiskCondition {
    pub metric: String,
    pub operator: String,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrowthPhase {
    #[serde(default, deserialize_with = "crate::models::deserialize_id_opt")]
    pub id: Option<String>,
    #[serde(default)]
    pub level: i32,
    pub name: String,
    #[serde(default)]
    pub lot_size: i32,
    #[serde(default)]
    pub conditions_to_advance: Vec<RiskCondition>,
    #[serde(default)]
    pub conditions_to_demote: Vec<RiskCondition>,
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
    pub account_type_applicability: String, // "All" | "Prop" | "Real" | "Demo" | "Specific"
    #[serde(default)]
    pub account_ids: Vec<String>,
    
    #[serde(default = "default_target_type")]
    pub target_type: String, // "Financial" | "Points"
    #[serde(default = "default_capital_source")]
    pub capital_source: String, // "Fixed" | "LinkedAccount"
    #[serde(default)]
    pub fixed_capital: f64,
    #[serde(default, deserialize_with = "crate::models::deserialize_id_opt")]
    pub linked_account_id: Option<String>,

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
    #[serde(default)]
    pub psychological_search_strategy: String, // "Strict" | "Sequence"
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub default_stop_points: Option<f64>,
    #[serde(default)]
    pub min_contracts: Option<i32>,
    #[serde(default)]
    pub max_contracts: Option<i32>,
}

fn default_target_type() -> String {
    "Financial".to_string()
}
fn default_capital_source() -> String {
    "Fixed".to_string()
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
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub description: String,
}

impl ToDto for Modality {
    type Dto = dto::ModalityDto;
    fn to_dto(&self) -> Self::Dto {
        dto::ModalityDto {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
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
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub name: String,
    pub tax_rate: f64,
    pub withholding_rate: f64,
    pub exemption_threshold: f64,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub basis: String, // "NetProfit" | "GrossProfit"
    pub cumulative_losses: bool,
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub trade_type: String, // "DayTrade" | "SwingTrade"
    #[serde(default, deserialize_with = "deserialize_string_opt")]
    pub withholding_basis: String, // "Profit" | "SalesVolume"
    #[serde(default)]
    pub revenue_code: Option<String>, // e.g., "6015", "3317"
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ToDto for TaxRule {
    type Dto = dto::TaxRuleDto;
    fn to_dto(&self) -> Self::Dto {
        dto::TaxRuleDto {
            id: self.id.clone(),
            name: self.name.clone(),
            tax_rate: self.tax_rate,
            withholding_rate: self.withholding_rate,
            exemption_threshold: self.exemption_threshold,
            basis: self.basis.clone(),
            cumulative_losses: self.cumulative_losses,
            trade_type: self.trade_type.clone(),
            withholding_basis: self.withholding_basis.clone(),
            revenue_code: self.revenue_code.clone(),
        }
    }
}

fn default_trade_type() -> String {
    "SwingTrade".to_string()
}
fn default_withholding_basis() -> String {
    "Profit".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxMapping {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub asset_type_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub modality_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_rule_id: Option<String>,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ToDto for TaxMapping {
    type Dto = dto::TaxMappingDto;
    fn to_dto(&self) -> Self::Dto {
        dto::TaxMappingDto {
            id: self.id.clone(),
            asset_type_id: self.asset_type_id.clone(),
            modality_id: self.modality_id.clone(),
            tax_rule_id: self.tax_rule_id.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxProfile {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ToDto for TaxProfile {
    type Dto = dto::TaxProfileDto;
    fn to_dto(&self) -> Self::Dto {
        dto::TaxProfileDto {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxProfileEntry {
    #[serde(
        default,
        deserialize_with = "deserialize_id_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_profile_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub modality_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_id_opt")]
    pub tax_rule_id: Option<String>,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ToDto for TaxProfileEntry {
    type Dto = dto::TaxProfileEntryDto;
    fn to_dto(&self) -> Self::Dto {
        dto::TaxProfileEntryDto {
            id: self.id.clone(),
            tax_profile_id: self.tax_profile_id.clone(),
            modality_id: self.modality_id.clone(),
            tax_rule_id: self.tax_rule_id.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
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
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}
