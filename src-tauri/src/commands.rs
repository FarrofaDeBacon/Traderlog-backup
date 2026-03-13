use crate::db::DbState;
use crate::models::{
    Account, ApiConfig, Asset, AssetType, CashTransaction, ChartType, Currency, EmotionalState,
    FeeProfile, Indicator, JournalEntry, Market, Modality, RiskProfile, Strategy, Tag, Timeframe,
    Trade, UserProfile,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tauri::State;

use hmac::{Hmac, Mac};
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;
use base64::Engine as _;

pub mod irpf;
pub mod profit_import;

const LICENSE_SECRET_KEY: &[u8] = b"TRADERLOGPRO_SECRET_KEY_2026";

/// Helper: CREATE if new, UPDATE if exists.
/// CRITICAL: Uses ⟨UUID⟩ angle-bracket notation in the SurrealQL Record ID.
/// UUIDs contain hyphens which break the SurrealQL parser when unescaped.
/// Correct syntax: `UPSERT trade:⟨uuid-with-hyphens⟩ CONTENT $data`
/// (backticks are for table/field name escaping, NOT Record ID escaping)
async fn upsert_record(
    db: &Surreal<Db>,
    table: &str,
    id: &str,
    data: serde_json::Value,
) -> Result<(), String> {
    println!("[DB] Standard Upsert for {}:{}", table, id);

    // Use ⟨⟩ angle brackets to escape UUID hyphens in SurrealQL Record IDs
    // Result: UPSERT trade:⟨ccd50e28-cb72-...⟩ CONTENT $data
    let full_id = format!("{}:⟨{}⟩", table, id);
    let query = format!("UPSERT {} CONTENT $data RETURN NONE", full_id);

    db.query(&query).bind(("data", data)).await.map_err(|e| {
        println!("[DB] SDK Upsert ERROR for {}. Error: {}", full_id, e);
        e.to_string()
    })?;

    Ok(())
}

/// Helper: DELETE a record by table:id using ⟨UUID⟩ angle-bracket notation.
async fn delete_record(db: &Surreal<Db>, table: &str, id: &str) -> Result<(), String> {
    let query = format!("DELETE {}:⟨{}⟩ RETURN NONE", table, id);
    db.query(&query).await.map_err(|e| e.to_string())?;
    Ok(())
}

// --- User Profile & Settings ---

#[tauri::command]
pub async fn get_user_profile(db: State<'_, DbState>) -> Result<Option<UserProfile>, String> {
    println!("[COMMAND] get_user_profile called");
    // Explicitly cast ID to string to ensure compatibility with UserProfile struct
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM user_profile:main")
            .await
            .map_err(|e| e.to_string())?;

    let mut profiles: Vec<UserProfile> = result.take(0).map_err(|e| {
        println!("[ERROR] get_user_profile deserialization failure: {}", e);
        e.to_string()
    })?;

    let profile = profiles.pop();
    println!(
        "[COMMAND] get_user_profile returning: {:?}",
        profile.is_some()
    );
    Ok(profile)
}

#[tauri::command]
pub async fn save_user_profile(
    db: State<'_, DbState>,
    mut profile: UserProfile,
) -> Result<(), String> {
    println!("[COMMAND] save_user_profile called: {:?}", profile.name);

    // Hash new password if provided and not already hashed (bcrypt hashes start with $2)
    if let Some(pwd) = &profile.password_hash {
        if !pwd.is_empty() && !pwd.starts_with("$2") {
            println!("[DEBUG] Hashing new password for user...");
            match hash(pwd, DEFAULT_COST) {
                Ok(hashed) => profile.password_hash = Some(hashed),
                Err(e) => return Err(format!("Falha ao processar senha: {}", e)),
            }
        }
    }

    // Hash new recovery key if provided and not already hashed
    if let Some(rk) = &profile.recovery_hash {
        if !rk.is_empty() && !rk.starts_with("$2") {
            println!("[DEBUG] Hashing new recovery key for user...");
            match hash(rk, DEFAULT_COST) {
                Ok(hashed) => profile.recovery_hash = Some(hashed),
                Err(e) => return Err(format!("Falha ao processar chave de recuperação: {}", e)),
            }
        }
    }

    let mut profile_json = serde_json::to_value(&profile).map_err(|e| e.to_string())?;
    println!("[DEBUG] Incoming Profile JSON to Save: {:?}", profile_json);

    if let Some(obj) = profile_json.as_object_mut() {
        obj.remove("id");
    }

    // Using UPSERT to ensure it creates if missing, and raw query to avoid high-level API result serialization issues
    db.0.query("UPSERT user_profile:main CONTENT $profile RETURN NONE")
        .bind(("profile", profile_json))
        .await
        .map_err(|e| e.to_string())?;

    println!("[COMMAND] save_user_profile SUCCESS");
    Ok(())
}

#[tauri::command]
pub async fn verify_password(db: State<'_, DbState>, password: String) -> Result<bool, String> {
    println!("[COMMAND] verify_password called");

    // Fetch current user profile to get the stored hash
    let mut result =
        db.0.query("SELECT password_hash FROM user_profile:main")
            .await
            .map_err(|e| e.to_string())?;

    let profiles: Vec<crate::models::SurrealJson> = result.take(0).map_err(|e| e.to_string())?;

    if let Some(profile_data) = profiles.first() {
        if let Some(hash_val) = profile_data.0.get("password_hash") {
            if let Some(stored_hash) = hash_val.as_str() {
                if stored_hash.is_empty() {
                    println!("[DEBUG] No password set for user, allowing access.");
                    return Ok(true); // No password set
                }

                // Perform bcrypt verification
                let is_valid = verify(&password, stored_hash).unwrap_or(false);
                println!("[DEBUG] Password verification match: {}", is_valid);
                return Ok(is_valid);
            }
        }
    }

    // If no profile exists or no password field exists, consider it valid (first run / no password set)
    println!("[DEBUG] No profile or password hash found, allowing access by default.");
    Ok(true)
}

#[tauri::command]
pub async fn verify_recovery_key(db: State<'_, DbState>, key: String) -> Result<bool, String> {
    println!("[COMMAND] verify_recovery_key called");

    let mut result =
        db.0.query("SELECT * FROM user_profile:main")
            .await
            .map_err(|e| e.to_string())?;
    let profiles: Vec<crate::models::SurrealJson> = result.take(0).map_err(|e| e.to_string())?;

    if let Some(profile_data) = profiles.first() {
        if let Some(hash_val) = profile_data.0.get("recovery_hash") {
            if let Some(stored_hash) = hash_val.as_str() {
                if stored_hash.is_empty() {
                    return Ok(false);
                }
                let is_valid = verify(&key, stored_hash).unwrap_or(false);
                return Ok(is_valid);
            }
        }
    }

    Ok(false)
}

#[tauri::command]
pub async fn reset_password(
    db: State<'_, DbState>,
    recovery_key: String,
    new_password: String,
) -> Result<bool, String> {
    println!("[COMMAND] reset_password called");

    // 1. Inline recovery key verification (avoid db.clone() which causes serialization issues)
    let mut result =
        db.0.query("SELECT recovery_hash FROM user_profile:main")
            .await
            .map_err(|e| e.to_string())?;

    let profiles: Vec<crate::models::SurrealJson> = result.take(0).map_err(|e| e.to_string())?;

    let is_key_valid = if let Some(profile_data) = profiles.first() {
        if let Some(hash_val) = profile_data.0.get("recovery_hash") {
            if let Some(stored_hash) = hash_val.as_str() {
                if stored_hash.is_empty() {
                    false
                } else {
                    verify(&recovery_key, stored_hash).unwrap_or(false)
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if !is_key_valid {
        println!("[COMMAND] reset_password: invalid recovery key");
        return Ok(false);
    }

    // 2. Hash new password
    let hashed_password = hash(new_password, DEFAULT_COST).map_err(|e| e.to_string())?;

    // 3. Update only the password_hash field
    db.0.query("UPDATE user_profile:main SET password_hash = $pwd")
        .bind(("pwd", hashed_password))
        .await
        .map_err(|e| e.to_string())?;

    println!("[COMMAND] reset_password SUCCESS");
    Ok(true)
}

#[tauri::command]
pub async fn get_api_configs(db: State<'_, DbState>) -> Result<Vec<ApiConfig>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM api_config")
            .await
            .map_err(|e| e.to_string())?;
    let configs: Vec<ApiConfig> = result.take(0).map_err(|e| e.to_string())?;
    Ok(configs)
}

#[tauri::command]
pub async fn save_api_config(db: State<'_, DbState>, config: ApiConfig) -> Result<(), String> {
    let id = config.id.clone();
    let mut json = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "api_config", clean_id, json).await
}

// --- Accounts ---

#[tauri::command]
pub async fn get_accounts(db: State<'_, DbState>) -> Result<Vec<Account>, String> {
    println!("[COMMAND] get_accounts called");

    let mut result =
        db.0.query("SELECT 
                type::string(id) as id, 
                nickname, 
                account_type, 
                broker, 
                account_number, 
                type::string(currency_id) as currency_id, 
                balance, 
                custom_logo, 
                currency_id.code as currency 
            FROM account")
            .await
            .map_err(|e| e.to_string())?;
    let accounts: Vec<Account> = result.take(0).map_err(|e| {
        println!("[ERROR] get_accounts deserialization failure: {}", e);
        e.to_string()
    })?;

    println!(
        "[COMMAND] get_accounts returning {} accounts",
        accounts.len()
    );
    Ok(accounts)
}

#[tauri::command]
pub async fn save_account(db: State<'_, DbState>, mut account: Account) -> Result<(), String> {
    println!("[COMMAND] save_account called: {:?}", account);

    let id = if let Some(ref id_str) = account.id {
        id_str.clone()
    } else {
        format!("account:{}", uuid::Uuid::new_v4())
    };

    // Normalize currency_id: if it's a 3-letter code, prefix with 'currency:'
    if let Some(ref curr) = account.currency_id {
        if curr.len() == 3 && !curr.contains(':') {
            account.currency_id = Some(format!("currency:{}", curr));
        }
    }

    let mut json = serde_json::to_value(&account).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
        obj.remove("currency"); // Don't persist virtual field
    }

    let clean_id = id.split(':').last().unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "");
        
    upsert_record(&db.0, "account", &clean_id, json).await
}

#[tauri::command]
pub async fn delete_account(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(id.as_str())
        .replace("⟨", "")
        .replace("⟩", "");
    delete_record(&db.0, "account", &clean_id).await
}

// --- Currencies ---

#[tauri::command]
pub async fn get_currencies(db: State<'_, DbState>) -> Result<Vec<Currency>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM currency")
            .await
            .map_err(|e| e.to_string())?;
    let currencies: Vec<Currency> = result.take(0).map_err(|e| {
        println!("[ERROR] get_currencies deserialization failure: {}", e);
        e.to_string()
    })?;
    Ok(currencies)
}

#[tauri::command]
pub async fn save_currency(db: State<'_, DbState>, currency: Currency) -> Result<(), String> {
    let id = currency.id.clone();
    let mut json = serde_json::to_value(&currency).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "currency", clean_id, json).await
}

#[tauri::command]
pub async fn delete_currency(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "currency", clean_id).await
}

// --- Markets ---

#[tauri::command]
pub async fn get_markets(db: State<'_, DbState>) -> Result<Vec<Market>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM market")
            .await
            .map_err(|e| e.to_string())?;
    let markets: Vec<Market> = result.take(0).map_err(|e| {
        println!("[ERROR] get_markets deserialization failure: {}", e);
        e.to_string()
    })?;
    Ok(markets)
}

#[tauri::command]
pub async fn save_market(db: State<'_, DbState>, market: Market) -> Result<(), String> {
    let id = market.id.clone();
    let mut json = serde_json::to_value(&market).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let id_str = id.unwrap_or_default();
    let clean_id = id_str
        .split(':')
        .last()
        .unwrap_or(&id_str)
        .replace("⟨", "")
        .replace("⟩", "");
    upsert_record(&db.0, "market", &clean_id, json).await
}

#[tauri::command]
pub async fn delete_market(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(id.as_str())
        .replace("⟨", "")
        .replace("⟩", "");
    delete_record(&db.0, "market", &clean_id).await
}

// --- Asset Types ---

#[tauri::command]
pub async fn get_asset_types(db: State<'_, DbState>) -> Result<Vec<AssetType>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM asset_type")
            .await
            .map_err(|e| e.to_string())?;
    let types: Vec<AssetType> = result.take(0).map_err(|e| {
        println!("[ERROR] get_asset_types deserialization failure: {}", e);
        e.to_string()
    })?;
    Ok(types)
}

#[tauri::command]
pub async fn save_asset_type(db: State<'_, DbState>, asset_type: AssetType) -> Result<(), String> {
    let mut json = serde_json::to_value(&asset_type).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let id_str = asset_type.id.clone().unwrap_or_default();
    let clean_id = id_str.split(':').last().unwrap_or(&id_str);
    upsert_record(&db.0, "asset_type", clean_id, json).await
}

#[tauri::command]
pub async fn delete_asset_type(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "asset_type", clean_id).await
}

// --- Assets ---

#[tauri::command]
pub async fn get_assets(db: State<'_, DbState>) -> Result<Vec<Asset>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM asset")
            .await
            .map_err(|e| e.to_string())?;
    let assets: Vec<Asset> = result.take(0).map_err(|e| {
        println!("[ERROR] get_assets deserialization failure: {}", e);
        e.to_string()
    })?;
    Ok(assets)
}

#[tauri::command]
pub async fn save_asset(db: State<'_, DbState>, asset: Asset) -> Result<(), String> {
    let id = asset.id.clone();
    let mut json = serde_json::to_value(&asset).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let id_str = id.unwrap_or_default();
    let clean_id = id_str
        .split(':')
        .last()
        .unwrap_or(&id_str)
        .replace("⟨", "")
        .replace("⟩", "")
        .replace("`", "")
        .trim()
        .to_string();

    // Standard upsert
    upsert_record(&db.0, "asset", &clean_id, json).await?;

    // Relational field conversion (ensures they are Things, not Strings)
    let full_id = format!("asset:⟨{}⟩", clean_id);
    let sql = format!("
        UPDATE {} SET 
            asset_type_id = type::thing(asset_type_id),
            default_fee_id = (IF default_fee_id THEN type::thing(default_fee_id) ELSE null END),
            tax_profile_id = (IF tax_profile_id THEN type::thing(tax_profile_id) ELSE null END),
            root_id = (IF root_id THEN type::thing(root_id) ELSE null END)
        WHERE id = {};
    ", full_id, full_id);

    db.0.query(&sql).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_asset(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(id.as_str())
        .replace("⟨", "")
        .replace("⟩", "");
    delete_record(&db.0, "asset", &clean_id).await
}

// --- Emotional States ---

#[tauri::command]
pub async fn get_emotional_states(db: State<'_, DbState>) -> Result<Vec<EmotionalState>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM emotional_state")
            .await
            .map_err(|e| e.to_string())?;
    let states: Vec<EmotionalState> = result.take(0).map_err(|e| e.to_string())?;
    Ok(states)
}

#[tauri::command]
pub async fn save_emotional_state(
    db: State<'_, DbState>,
    state: EmotionalState,
) -> Result<(), String> {
    let id = state.id.clone();
    let mut json = serde_json::to_value(&state).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let id_str = id.clone().unwrap_or_default();
    let clean_id = id_str.split(':').last().unwrap_or(&id_str);
    upsert_record(&db.0, "emotional_state", clean_id, json).await
}

#[tauri::command]
pub async fn delete_emotional_state(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "emotional_state", clean_id).await
}

// --- Strategies ---

#[tauri::command]
pub async fn get_strategies(db: State<'_, DbState>) -> Result<Vec<Strategy>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM strategy")
            .await
            .map_err(|e| e.to_string())?;
    let strategies: Vec<Strategy> = result.take(0).map_err(|e| e.to_string())?;
    Ok(strategies)
}

#[tauri::command]
pub async fn save_strategy(db: State<'_, DbState>, strategy: Strategy) -> Result<(), String> {
    let id = strategy.id.clone();
    let mut json = serde_json::to_value(&strategy).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(id.as_str())
        .replace("⟨", "")
        .replace("⟩", "");
    upsert_record(&db.0, "strategy", &clean_id, json).await
}

#[tauri::command]
pub async fn delete_strategy(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(id.as_str())
        .replace("⟨", "")
        .replace("⟩", "");
    delete_record(&db.0, "strategy", &clean_id).await
}

// --- Trades ---

#[tauri::command]
pub async fn get_trades(db: State<'_, DbState>) -> Result<Vec<Trade>, String> {
    println!("[COMMAND] get_trades called");

    // Explicitly cast ALL ID fields to string for frontend/JSON portability.
    // This avoids the "{ String: '...' }" or Thing object issues once and for all.
    let sql = "SELECT *, 
            type::string(id) as id, 
            type::string(account_id) as account_id,
            type::string(asset_type_id) as asset_type_id,
            type::string(strategy_id) as strategy_id,
            (IF entry_emotional_state_id THEN type::string(entry_emotional_state_id) ELSE null END) as entry_emotional_state_id,
            (IF exit_emotional_state_id THEN type::string(exit_emotional_state_id) ELSE null END) as exit_emotional_state_id,
            (IF modality_id THEN type::string(modality_id) ELSE null END) as modality_id,
            (IF asset_id THEN type::string(asset_id) ELSE null END) as asset_id
        FROM trade ORDER BY date DESC";

    let mut response = db.0.query(sql).await.map_err(|e| e.to_string())?;

    let trades: Vec<Trade> = response.take(0).map_err(|e| {
        eprintln!("[COMMAND] get_trades deserialization error: {}", e);
        e.to_string()
    })?;

    println!("[COMMAND] get_trades returning {} trades", trades.len());

    Ok(trades)
}

#[tauri::command]
pub async fn save_trade(db: State<'_, DbState>, trade: Trade) -> Result<(), String> {
    // Robust ID extraction: handles table prefix, angle brackets, and backticks
    let raw_id = &trade.id;
    let clean_id = raw_id
        .replace("trade:", "")
        .replace("⟨", "")
        .replace("⟩", "")
        .replace("`", "")
        .trim()
        .to_string();

    println!(
        "[COMMAND] save_trade START: clean_id='{}' (raw_id='{}') asset='{}' result='{}'",
        clean_id, raw_id, trade.asset_symbol, trade.result
    );

    // Diagnostic check for existence BEFORE upserting
    let check_sql = format!("SELECT id FROM trade:⟨{}⟩ LIMIT 1", clean_id);
    if let Ok(mut res) = db.0.query(&check_sql).await {
        if let Ok(existing) = res.take::<Vec<crate::models::SurrealJson>>(0) {
            if !existing.is_empty() {
                println!("[DEBUG] save_trade: Record trade:⟨{}⟩ already exists. Performing UPDATE.", clean_id);
            } else {
                println!("[DEBUG] save_trade: Record trade:⟨{}⟩ NOT found. Performing INSERT.", clean_id);
                // Log all IDs in the table to find potential mismatches
                if let Ok(mut all_res) = db.0.query("SELECT type::string(id) as id FROM trade LIMIT 5").await {
                   if let Ok(ids) = all_res.take::<Vec<crate::models::SurrealJson>>(0) {
                        println!("[DEBUG] save_trade: Sample existing IDs in 'trade' table: {:?}", ids);
                   }
                }
            }
        }
    }

    let mut data = serde_json::to_value(&trade).map_err(|e| e.to_string())?;
    // Content should NOT include the ID as it is the primary key and immutable in SurrealDB
    if let Some(obj) = data.as_object_mut() {
        obj.remove("id");
    }

    // First, standard upsert using string-interpolated Record ID
    upsert_record(&db.0, "trade", &clean_id, data).await?;

    // Then explicit UPDATE for relational fields using ⟨UUID⟩ Record ID
    let full_trade_id = format!("trade:⟨{}⟩", clean_id);
    let sql = format!("
        UPDATE {} SET 
            account_id = (IF account_id THEN type::thing(account_id) ELSE null END),
            asset_type_id = (IF asset_type_id THEN type::thing(asset_type_id) ELSE null END),
            asset_id = (IF asset_id THEN type::thing(asset_id) ELSE null END),
            strategy_id = (IF strategy_id THEN type::thing(strategy_id) ELSE null END),
            modality_id = (IF modality_id THEN type::thing(modality_id) ELSE null END),
            entry_emotional_state_id = (IF entry_emotional_state_id THEN type::thing(entry_emotional_state_id) ELSE null END),
            exit_emotional_state_id = (IF exit_emotional_state_id THEN type::thing(exit_emotional_state_id) ELSE null END)
        WHERE id = {};
    ", full_trade_id, full_trade_id);

    db.0.query(&sql).await.map_err(|e| e.to_string())?;

    println!("[COMMAND] save_trade SUCCESS for trade:{}", clean_id);

    // --- AUTO-SYNC DAILY CLOSURE (EXTRATO) ---
    let date_only = if let Some(ref ed) = trade.exit_date {
        ed.split('T').next().unwrap_or(ed).split(' ').next().unwrap_or(ed).to_string()
    } else {
        trade.date.split('T').next().unwrap_or(&trade.date).split(' ').next().unwrap_or(&trade.date).to_string()
    };

    let acc_clean_current = trade
        .account_id
        .as_deref()
        .unwrap_or("")
        .split(':')
        .last()
        .unwrap_or(trade.account_id.as_deref().unwrap_or(""))
        .replace("⟨", "")
        .replace("⟩", "")
        .replace("`", "")
        .trim()
        .to_string();

    // 1. Find ALL closures that currently contain this trade ID
    // and ALSO find the "target" closure for the trade's current date/account
    let find_sync_sql = format!(
        "SELECT *, type::string(id) as id, type::string(account_id) as account_id FROM cash_transaction WHERE system_linked = true AND category = 'Trading'"
    );

    if let Ok(mut find_res) = db.0.query(&find_sync_sql).await {
        if let Ok(all_system_closures) = find_res.take::<Vec<crate::models::CashTransaction>>(0) {
            
            // A. Identify closures to update
            let mut closures_to_process = Vec::new();
            let mut target_closure_found = false;

            for c in all_system_closures {
            let c_acc_clean = c.account_id.as_deref().unwrap_or("")
                    .split(':').last().unwrap_or(c.account_id.as_deref().unwrap_or(""))
                    .replace("⟨", "").replace("⟩", "").replace("`", "").trim().to_string();
                let c_date_only = c.date.split('T').next().unwrap_or(&c.date).split(' ').next().unwrap_or(&c.date);
                
                let is_target = c_acc_clean == acc_clean_current && c_date_only == date_only;
                let mut contains_trade = false;

                if let Some(ref tids) = c.trade_ids {
                    contains_trade = tids.iter().any(|tid| {
                        let tid_clean = tid.as_str().split(':').last().unwrap_or(tid.as_str())
                            .replace("⟨", "").replace("⟩", "").replace("`", "");
                        tid_clean == clean_id
                    });
                }

                if is_target {
                    target_closure_found = true;
                    closures_to_process.push((c, true)); // (Closure, is_target)
                } else if contains_trade {
                    closures_to_process.push((c, false)); // It's an orphan! (Moved trade)
                }
            }

            // B. If target closure wasn't found in system closures (maybe newly created or trade just moved), 
            // we should try specifically finding it even if it doesn't have the trade yet.
            if !target_closure_found {
                 let find_target_sql = format!(
                    "SELECT *, type::string(id) as id, type::string(account_id) as account_id FROM cash_transaction WHERE system_linked = true AND category = 'Trading' AND string::startsWith(type::string(date), '{}')",
                    date_only
                );
                if let Ok(mut target_res) = db.0.query(&find_target_sql).await {
                    if let Ok(target_closures) = target_res.take::<Vec<crate::models::CashTransaction>>(0) {
                        for c in target_closures {
                            let c_acc_clean = c.account_id.as_deref().unwrap_or("")
                                .split(':').last().unwrap_or(c.account_id.as_deref().unwrap_or(""))
                                .replace("⟨", "").replace("⟩", "").replace("`", "").trim().to_string();
                            if c_acc_clean == acc_clean_current {
                                closures_to_process.push((c, true));
                                break; 
                            }
                        }
                    }
                }
            }

            // C. Process updates
            for (closure, is_target) in closures_to_process {
                let ct_clean = closure.id.split(':').last().unwrap_or(&closure.id)
                    .replace("⟨", "").replace("⟩", "").replace("`", "").to_string();
                let ct_full = format!("cash_transaction:⟨{}⟩", ct_clean);
                
                let cur_acc_clean = closure.account_id.as_deref().unwrap_or("")
                    .split(':').last().unwrap_or(closure.account_id.as_deref().unwrap_or(""))
                    .replace("⟨", "").replace("⟩", "").replace("`", "").trim().to_string();
                let cur_acc_full = format!("account:⟨{}⟩", cur_acc_clean);

                let mut trade_ids = closure.trade_ids.unwrap_or_default();
                let mut changed = false;

                if is_target {
                    // Ensure linked
                    let already_linked = trade_ids.iter().any(|tid| {
                        let tid_clean = tid.as_str().split(':').last().unwrap_or(tid.as_str())
                            .replace("⟨", "").replace("⟩", "").replace("`", "");
                        tid_clean == clean_id
                    });
                    if !already_linked {
                        trade_ids.push(full_trade_id.clone());
                        changed = true;
                    }
                } else {
                    // Remove from old closure
                    let original_len = trade_ids.len();
                    trade_ids.retain(|tid| {
                        let tid_clean = tid.as_str()
                            .split(':').last().unwrap_or(tid.as_str())
                            .replace("⟨", "").replace("⟩", "").replace("`", "");
                        tid_clean != clean_id
                    });
                    if trade_ids.len() != original_len {
                        changed = true;
                    }
                }

                if changed || is_target {
                    // Recalculate Total
                    let mut new_total = 0.0_f64;
                    for tid in &trade_ids {
                        let tid_clean_loop = tid.as_str().split(':').last().unwrap_or(tid.as_str())
                            .replace("⟨", "").replace("⟩", "").replace("`", "").trim().to_string();
                        
                        if tid_clean_loop == clean_id {
                            new_total += trade.result;
                        } else {
                            let sql_res = format!("SELECT result FROM trade:⟨{}⟩ LIMIT 1", tid_clean_loop);
                            if let Ok(mut res_query) = db.0.query(&sql_res).await {
                                if let Ok(results) = res_query.take::<Vec<crate::models::SurrealJson>>(0) {
                                    if let Some(first) = results.first() {
                                        new_total += first.0.get("result").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    }
                                }
                            }
                        }
                    }

                    let amount_diff = new_total - closure.amount;
                    if amount_diff != 0.0 || changed {
                        println!("[COMMAND] save_trade sync: {} is_target={} amount {} -> {} (diff: {})", ct_full, is_target, closure.amount, new_total, amount_diff);
                        
                        let tx_type = if new_total >= 0.0 { "Deposit" } else { "Withdraw" };
                        let _ = db.0.query(format!("UPDATE {} SET amount = $amount, trade_ids = $t_ids, type = $tx_type", ct_full))
                            .bind(("amount", new_total))
                            .bind(("t_ids", trade_ids))
                            .bind(("tx_type", tx_type.to_string()))
                            .await;

                        if amount_diff != 0.0 {
                            let _ = db.0.query(format!("UPDATE {} SET balance += $diff", cur_acc_full))
                                .bind(("diff", amount_diff)).await;
                        }
                    }
                }
            }
        }
    }

    println!("[COMMAND] save_trade SUCCESS for trade:{}", clean_id);
    Ok(())
}

async fn internal_delete_trade(db: &Surreal<Db>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "")
        .replace("`", "")
        .trim()
        .to_string();
    let full_id = format!("trade:⟨{}⟩", clean_id);

    // 1. Get the trade result BEFORE deleting (must use ⟨UUID⟩ direct interpolation)
    let sql_trade = format!("SELECT result FROM {} LIMIT 1", full_id);
    println!(
        "[COMMAND] internal_delete_trade: id='{}' clean_id='{}' full_id='{}'",
        id, clean_id, full_id
    );

    let mut trade_result = 0.0_f64;
    if let Ok(mut trade_res) = db.query(&sql_trade).await {
        if let Ok(results) = trade_res.take::<Vec<crate::models::SurrealJson>>(0) {
            println!(
                "[COMMAND] internal_delete_trade: trade result query returned {} rows",
                results.len()
            );
            if let Some(first) = results.first() {
                trade_result = first.0.get("result").and_then(|v| v.as_f64()).unwrap_or(0.0);
            }
        }
    }
    println!("[COMMAND] internal_delete_trade: trade_result={}", trade_result);

    // 2. Find associated daily closures (cash_transactions with this trade in trade_ids)
    let sql_ct =
        "SELECT *, type::string(id) as id, type::string(account_id) as account_id FROM cash_transaction WHERE system_linked = true AND category = 'Trading'";
    let mut cashflow_to_update: Vec<crate::models::CashTransaction> = Vec::new();
    if let Ok(mut ct_res) = db.query(sql_ct).await {
        if let Ok(all_cts) = ct_res.take::<Vec<crate::models::CashTransaction>>(0) {
            for ct in all_cts {
                if let Some(ref tids) = ct.trade_ids {
                    let linked = tids.iter().any(|tid| {
                        let tid_clean = tid.as_str()
                            .split(':')
                            .last()
                            .unwrap_or(tid.as_str())
                            .replace("⟨", "")
                            .replace("⟩", "")
                            .replace("`", "");
                        tid_clean == clean_id
                    });
                    if linked {
                        cashflow_to_update.push(ct);
                    }
                }
            }
        }
    }
    println!(
        "[COMMAND] internal_delete_trade: found {} linked cash_transactions",
        cashflow_to_update.len()
    );

    for mut ct in cashflow_to_update {
        let ct_clean_id = ct
            .id
            .split(':')
            .last()
            .unwrap_or(&ct.id)
            .replace("⟨", "")
            .replace("⟩", "")
            .replace("`", "")
            .to_string();
        let ct_full_id = format!("cash_transaction:⟨{}⟩", ct_clean_id);

        if let Some(mut t_ids) = ct.trade_ids.take() {
            // Remove this trade from the list
            t_ids.retain(|tid| {
                let tid_clean = tid.as_str()
                    .split(':')
                    .last()
                    .unwrap_or(tid.as_str())
                    .replace("⟨", "")
                    .replace("⟩", "")
                    .replace("`", "");
                tid_clean != clean_id
            });

            // Update account balance: subtract the deleted trade's result
            let acc_clean = ct
                .account_id
                .as_deref()
                .unwrap_or("")
                .split(':')
                .last()
                .unwrap_or(ct.account_id.as_deref().unwrap_or(""))
                .replace("⟨", "")
                .replace("⟩", "")
                .replace("`", "")
                .to_string();
            let acc_full = format!("account:⟨{}⟩", acc_clean);
            let sql_acc = format!("UPDATE {} SET balance -= $trade_result", acc_full);
            let _ =
                db.query(&sql_acc)
                    .bind(("trade_result", trade_result))
                    .await;

            if t_ids.is_empty() {
                // Last trade in the closure → delete the closure
                println!(
                    "[COMMAND] internal_delete_trade: deleting empty closure {}",
                    ct_full_id
                );
                let _ = delete_record(db, "cash_transaction", &ct_clean_id).await;
            } else {
                // Multiple trades → recalculate total based on REMAINING trades
                let mut new_total = 0.0_f64;
                for tid in &t_ids {
                    let tid_clean_rem = tid
                        .split(':')
                        .last()
                        .unwrap_or(tid)
                        .replace("⟨", "")
                        .replace("⟩", "")
                        .replace("`", "")
                        .trim()
                        .to_string();
                    
                    let sql_res = format!("SELECT result FROM trade:⟨{}⟩ LIMIT 1", tid_clean_rem);
                    if let Ok(mut res_query) = db.query(&sql_res).await {
                        if let Ok(results) = res_query.take::<Vec<crate::models::SurrealJson>>(0) {
                            if let Some(first) = results.first() {
                                new_total +=
                                    first.0.get("result").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            }
                        }
                    }
                }

                let tx_type = if new_total >= 0.0 {
                    "Deposit"
                } else {
                    "Withdraw"
                };
                println!(
                    "[COMMAND] internal_delete_trade: updating closure {} new_amount={}",
                    ct_full_id, new_total
                );
                let sql_update = format!(
                    "UPDATE {} SET amount = $amount, trade_ids = $t_ids, type = $tx_type",
                    ct_full_id
                );
                let _ =
                    db.query(&sql_update)
                        .bind(("amount", new_total))
                        .bind(("t_ids", t_ids))
                        .bind(("tx_type", tx_type.to_string()))
                        .await;
            }
        }
    }

    // 3. Delete the trade itself
    println!("[COMMAND] internal_delete_trade: deleting trade {}", full_id);
    delete_record(db, "trade", &clean_id).await
}

#[tauri::command]
pub async fn delete_trade(db: State<'_, DbState>, id: String) -> Result<(), String> {
    internal_delete_trade(&db.0, id).await
}

#[tauri::command]
pub async fn delete_trades_by_ids(db: State<'_, DbState>, ids: Vec<String>) -> Result<(), String> {
    println!("[COMMAND] delete_trades_by_ids: count={}", ids.len());
    for id in ids {
        internal_delete_trade(&db.0, id).await?;
    }
    Ok(())
}


// --- Cash Transactions ---

#[tauri::command]
pub async fn get_cash_transactions(db: State<'_, DbState>) -> Result<Vec<CashTransaction>, String> {
    // Order by date DESC (primary) and ID DESC (stable creation proxy in SurrealDB)
    let sql = "SELECT *, type::string(id) as id, type::string(account_id) as account_id FROM cash_transaction ORDER BY date DESC, id DESC";
    let mut result = db.0.query(sql).await.map_err(|e| e.to_string())?;
    let transactions: Vec<CashTransaction> = result.take(0).map_err(|e| {
        println!(
            "[ERROR] get_cash_transactions deserialization failure: {}",
            e
        );
        e.to_string()
    })?;

    // Diagnostic: Print the last 10 transactions to check if the refund is present
    println!(
        "[DEBUG] get_cash_transactions returning {} items. Last 10:",
        transactions.len()
    );
    for tx in transactions.iter().take(10) {
        println!(
            "  - id: {}, date: {}, type: {:?}, desc: {}, amt: {}, acc: {:?}, cat: {:?}",
            tx.id, tx.date, tx.r#type, tx.description, tx.amount, tx.account_id, tx.category
        );
    }

    Ok(transactions)
}

#[tauri::command]
pub async fn save_cash_transaction(
    db: State<'_, DbState>,
    transaction: CashTransaction,
) -> Result<(), String> {
    println!(
        "[COMMAND] save_cash_transaction START for ID: {}",
        transaction.id
    );
    let id = transaction.id.clone();
    let mut json = serde_json::to_value(&transaction).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "");
    let res = upsert_record(&db.0, "cash_transaction", &clean_id, json).await;
    match &res {
        Ok(_) => println!(
            "[COMMAND] save_cash_transaction SUCCESS for id: {}",
            clean_id
        ),
        Err(e) => eprintln!(
            "[COMMAND] save_cash_transaction ERROR for id {}: {}",
            clean_id, e
        ),
    }
    res
}

#[tauri::command]
pub async fn diagnostic_closure_dump(db: State<'_, DbState>) -> Result<(), String> {
    println!("--- [DIAGNOSTIC DUMP: CLOSURE] ---");

    // 1. Inspect Trades
    println!("\n[TRADES]");
    let query = "SELECT type::string(id) as id, type::string(date) as date, (IF exit_date != NONE THEN type::string(exit_date) ELSE null END) as exit_date, type::float(result) as result, type::float(exit_price) as exit_price, (IF account_id != NONE THEN type::string(account_id) ELSE null END) as account_id FROM trade ORDER BY date DESC LIMIT 1";
    let mut res = db.0.query(query).await.map_err(|e| e.to_string())?;
    let all_fields_trades: Vec<crate::models::SurrealJson> = res.take(0).map_err(|e| e.to_string())?;
    if let Some(t) = all_fields_trades.first() {
        println!(
            "SAMPLE TRADE DATA: {}",
            serde_json::to_string_pretty(t).unwrap_or_default()
        );
    }

    // 2. Inspect Transactions
    println!("\n[CASH TRANSACTIONS]");
    let query_tx = "SELECT type::string(id) as id, date, amount, description, account_id, system_linked FROM cash_transaction ORDER BY date DESC LIMIT 20";
    let mut res_tx = db.0.query(query_tx).await.map_err(|e| e.to_string())?;
    let txs: Vec<crate::models::SurrealJson> = res_tx.take(0).map_err(|e| e.to_string())?;
    for tx in txs {
        println!(
            "Tx: id={}, date={:?}, amt={:?}, desc={:?}, acc={:?}, linked={:?}",
            tx.0["id"],
            tx.0["date"],
            tx.0["amount"],
            tx.0["description"],
            tx.0["account_id"],
            tx.0["system_linked"]
        );
    }

    // 3. Inspect Accounts
    println!("\n[ACCOUNTS]");
    let query_acc = "SELECT type::string(id) as id, nickname, balance FROM account";
    let mut res_acc = db.0.query(query_acc).await.map_err(|e| e.to_string())?;
    let accs: Vec<crate::models::SurrealJson> = res_acc.take(0).map_err(|e| e.to_string())?;
    for a in accs {
        println!(
            "Account: id={}, name={:?}, bal={:?}",
            a.0["id"], a.0["nickname"], a.0["balance"]
        );
    }

    println!("\n--- [DUMP END] ---");
    Ok(())
}

#[tauri::command]
pub async fn delete_cash_transaction(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "");

    // 1. Check if linked to system logic
    let mut res =
        db.0.query("SELECT VALUE system_linked FROM type::thing('cash_transaction', $id)")
            .bind(("id", clean_id.clone()))
            .await
            .map_err(|e| e.to_string())?;

    let linked: Option<bool> = res.take(0).map_err(|e| e.to_string())?;
    if linked == Some(true) && !clean_id.starts_with("daily_closure_") {
        return Err(
            "Esta transação está vinculada ao sistema e não pode ser excluída manualmente.".into(),
        );
    }

    delete_record(&db.0, "cash_transaction", &clean_id).await
}

// --- Journal Entries ---

#[tauri::command]
pub async fn get_journal_entries(db: State<'_, DbState>) -> Result<Vec<JournalEntry>, String> {
    let sql = "SELECT 
        type::string(id) as id,
        date,
        (content OR '') as content,
        (IF emotional_state_id THEN type::string(emotional_state_id) ELSE null END) as emotional_state_id,
        (intensity OR 0) as intensity,
        (followed_plan OR false) as followed_plan,
        (risk_accepted OR false) as risk_accepted,
        (market_context OR '') as market_context,
        (daily_score OR 0) as daily_score
        FROM journal_entry ORDER BY date DESC";

    let mut result = db.0.query(sql).await.map_err(|e| e.to_string())?;
    let entries: Vec<JournalEntry> = result.take(0).map_err(|e| e.to_string())?;
    Ok(entries)
}

#[tauri::command]
pub async fn save_journal_entry(db: State<'_, DbState>, entry: JournalEntry) -> Result<(), String> {
    let id = entry.id.clone();
    let mut json = serde_json::to_value(&entry).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(id.as_str())
        .replace("⟨", "")
        .replace("⟩", "");
    upsert_record(&db.0, "journal_entry", &clean_id, json).await
}

#[tauri::command]
pub async fn delete_journal_entry(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "");
    delete_record(&db.0, "journal_entry", &clean_id).await
}

// --- Fee Profiles ---

#[tauri::command]
pub async fn get_fees(db: State<'_, DbState>) -> Result<Vec<FeeProfile>, String> {
    println!("[COMMAND] get_fees called");
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM fee_profile")
            .await
            .map_err(|e| e.to_string())?;
    let fees: Vec<FeeProfile> = result.take(0).map_err(|e| {
        println!("[ERROR] get_fees deserialization failure: {}", e);
        e.to_string()
    })?;
    println!("[COMMAND] get_fees returning {} items", fees.len());
    Ok(fees)
}

// --- Diagnostic Commands ---

#[tauri::command]
pub async fn diagnostic_dump_trades(db: State<'_, DbState>) -> Result<(), String> {
    println!("[DIAGNOSTIC] Dumping all trades in database...");
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM trade")
            .await
            .map_err(|e| e.to_string())?;
    let trades: Vec<crate::models::SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    println!("[DIAGNOSTIC] Total trades found: {}", trades.len());
    for t in trades {
        println!(
            "[DIAGNOSTIC] TRADE: {}",
            serde_json::to_string(&t).unwrap_or_default()
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn diagnostic_dump_users(db: State<'_, DbState>) -> Result<(), String> {
    println!("[DIAGNOSTIC] Dumping all user profiles in database...");
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM user_profile")
            .await
            .map_err(|e| e.to_string())?;
    let users: Vec<crate::models::SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    println!("[DIAGNOSTIC] Total users found: {}", users.len());
    for u in users {
        println!(
            "[DIAGNOSTIC] USER: {}",
            serde_json::to_string(&u).unwrap_or_default()
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn save_fee(db: State<'_, DbState>, fee: FeeProfile) -> Result<(), String> {
    let id = fee.id.clone();
    let mut json = serde_json::to_value(&fee).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(id.as_str());
    upsert_record(&db.0, "fee_profile", clean_id, json).await
}

#[tauri::command]
pub async fn delete_fee(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "fee_profile", clean_id).await
}

// --- Risk Profiles ---

#[tauri::command]
pub async fn get_risk_profiles(db: State<'_, DbState>) -> Result<Vec<RiskProfile>, String> {
    println!("[COMMAND] get_risk_profiles called");
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM risk_profile")
            .await
            .map_err(|e| e.to_string())?;
    let profiles: Vec<RiskProfile> = result.take(0).map_err(|e| e.to_string())?;
    println!(
        "[COMMAND] get_risk_profiles returning {} items",
        profiles.len()
    );
    Ok(profiles)
}

#[tauri::command]
pub async fn save_risk_profile(db: State<'_, DbState>, profile: RiskProfile) -> Result<(), String> {
    let id = profile.id.clone();
    let mut json = serde_json::to_value(&profile).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(id.as_str());
    upsert_record(&db.0, "risk_profile", clean_id, json).await
}

#[tauri::command]
pub async fn delete_risk_profile(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "risk_profile", clean_id).await
}

// --- Modalities ---

#[tauri::command]
pub async fn get_modalities(db: State<'_, DbState>) -> Result<Vec<Modality>, String> {
    println!("[COMMAND] get_modalities called");
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM modality")
            .await
            .map_err(|e| e.to_string())?;
    let modalities: Vec<Modality> = result.take(0).map_err(|e| e.to_string())?;
    println!(
        "[COMMAND] get_modalities returning {} items",
        modalities.len()
    );
    Ok(modalities)
}

#[tauri::command]
pub async fn save_modality(db: State<'_, DbState>, modality: Modality) -> Result<(), String> {
    let id = modality.id.clone();
    let mut json = serde_json::to_value(&modality).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let id_str = id.unwrap_or_default();
    let clean_id = id_str.split(':').last().unwrap_or(&id_str);
    upsert_record(&db.0, "modality", clean_id, json).await
}

#[tauri::command]
pub async fn delete_modality(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "modality", clean_id).await
}


// --- Tags ---

#[tauri::command]
pub async fn get_tags(db: State<'_, DbState>) -> Result<Vec<Tag>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM tag")
            .await
            .map_err(|e| e.to_string())?;
    let tags: Vec<Tag> = result.take(0).map_err(|e| e.to_string())?;
    Ok(tags)
}

#[tauri::command]
pub async fn save_tag(db: State<'_, DbState>, tag: Tag) -> Result<(), String> {
    let id = tag.id.clone();
    let mut json = serde_json::to_value(&tag).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(id.as_str());
    upsert_record(&db.0, "tag", clean_id, json).await
}

#[tauri::command]
pub async fn delete_tag(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "tag", clean_id).await
}

// --- Indicators ---

#[tauri::command]
pub async fn get_indicators(db: State<'_, DbState>) -> Result<Vec<Indicator>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM indicator")
            .await
            .map_err(|e| e.to_string())?;
    let indicators: Vec<Indicator> = result.take(0).map_err(|e| e.to_string())?;
    Ok(indicators)
}

#[tauri::command]
pub async fn save_indicator(db: State<'_, DbState>, indicator: Indicator) -> Result<(), String> {
    let id = indicator.id.clone();
    let mut json = serde_json::to_value(&indicator).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(id.as_str());
    upsert_record(&db.0, "indicator", clean_id, json).await
}

#[tauri::command]
pub async fn delete_indicator(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "indicator", clean_id).await
}

// --- Timeframes ---

#[tauri::command]
pub async fn get_timeframes(db: State<'_, DbState>) -> Result<Vec<Timeframe>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM timeframe")
            .await
            .map_err(|e| e.to_string())?;
    let timeframes: Vec<Timeframe> = result.take(0).map_err(|e| e.to_string())?;
    Ok(timeframes)
}

#[tauri::command]
pub async fn save_timeframe(db: State<'_, DbState>, timeframe: Timeframe) -> Result<(), String> {
    let id = timeframe.id.clone();
    let mut json = serde_json::to_value(&timeframe).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(id.as_str());
    upsert_record(&db.0, "timeframe", clean_id, json).await
}

#[tauri::command]
pub async fn delete_timeframe(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "timeframe", clean_id).await
}

// --- Chart Types ---

#[tauri::command]
pub async fn get_chart_types(db: State<'_, DbState>) -> Result<Vec<ChartType>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM chart_type")
            .await
            .map_err(|e| e.to_string())?;
    let types: Vec<ChartType> = result.take(0).map_err(|e| e.to_string())?;
    Ok(types)
}

#[tauri::command]
pub async fn save_chart_type(db: State<'_, DbState>, chart_type: ChartType) -> Result<(), String> {
    let id = chart_type.id.clone();
    let mut json = serde_json::to_value(&chart_type).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(id.as_str());
    upsert_record(&db.0, "chart_type", clean_id, json).await
}

#[tauri::command]
pub async fn delete_chart_type(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "chart_type", clean_id).await
}

#[tauri::command]
pub async fn force_reseed(db: State<'_, DbState>) -> Result<(), String> {
    crate::seed::force_reseed_all(&db.0).await
}

#[tauri::command]
pub async fn factory_reset(db: State<'_, DbState>) -> Result<(), String> {
    println!("\n[ADMIN] 🔥 FACTORY RESET: Zerando banco de dados e onboarding...\n");
    // 1. Delete all operational data
    let tables = vec![
        "trade",
        "cash_transaction",
        "journal_entry",
        "account",
        "strategy",
        "indicator",
        "modality",
        "tag",
        "fee_profile",
        "emotional_state",
        "risk_profile",
        "timeframe",
        "chart_type",
        "asset",
        "asset_type",
        "market",
        "currency",
        "tax_rule",
        "tax_profile",
        "tax_profile_entry",
        "tax_mapping",
        "tax_appraisal",
        "tax_darf",
        "tax_loss",
    ];

    for table in tables {
        let _ = db.0.query(format!("DELETE {} RETURN NONE", table)).await;
    }

    // 2. Reset Onboarding in Profile
    let _ = db
        .0
        .query("UPDATE user_profile:main SET onboarding_completed = false, trial_start_date = NONE RETURN NONE")
        .await;

    println!(
        "[ADMIN] ✅ Factory reset concluído. Redirecionando para Onboarding na próxima abertura."
    );
    Ok(())
}

#[tauri::command]
pub async fn ensure_defaults(db: State<'_, DbState>) -> Result<(), String> {
    crate::seed::run_base_seeds(&db.0).await?;
    
    // Check if any account exists
    let mut response = db.0.query("SELECT count() FROM account GROUP ALL")
        .await
        .map_err(|e| e.to_string())?;
    
    let count: Option<i64> = response.take(0).map_err(|e| e.to_string())
        .and_then(|v: Vec<serde_json::Value>| {
            v.first()
                .and_then(|val| val.get("count"))
                .and_then(|c| c.as_i64())
                .ok_or_else(|| "Failed to parse count".to_string())
        })
        .ok();

    if count.unwrap_or(0) == 0 {
        println!("[SEED] No accounts found, seeding base real account...");
        crate::seed::demo_accounts_seed::seed_accounts(&db.0, Some(vec!["account:real".to_string()]))
            .await?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn finish_custom_onboarding(
    db: State<'_, DbState>,
    currencies: Vec<String>,
    markets: Vec<String>,
    asset_types: Vec<String>,
) -> Result<(), String> {
    println!("[ONBOARDING] Executando rotina restrita de seeds customizadas...");
    crate::seed::run_custom_seeds(&db.0, currencies, markets, asset_types).await?;
    
    // Ensure at least one account exists
    let mut response = db.0.query("SELECT count() FROM account GROUP ALL")
        .await
        .map_err(|e| e.to_string())?;
        
    let count: Option<i64> = response.take(0).map_err(|e| e.to_string())
        .and_then(|v: Vec<serde_json::Value>| {
            v.first()
                .and_then(|val| val.get("count"))
                .and_then(|c| c.as_i64())
                .ok_or_else(|| "Failed to parse count".to_string())
        })
        .ok();

    if count.unwrap_or(0) == 0 {
        println!("[ONBOARDING] No accounts selected or found, seeding base real account...");
        crate::seed::demo_accounts_seed::seed_accounts(&db.0, Some(vec!["account:real".to_string()]))
            .await?;
    }

    // CRITICAL: Mark onboarding as completed in the database
    db.0.query("UPDATE user_profile:main SET onboarding_completed = true RETURN NONE")
        .await
        .map_err(|e| e.to_string())?;

    println!("[ONBOARDING] Sucesso: Custom seeds aplicadas e onboarding marcado como OK.");
    Ok(())
}

#[tauri::command]
pub async fn check_database_status(db: tauri::State<'_, DbState>) -> Result<String, String> {
    println!("[DIAGNOSTIC] Checking database status...");

    // Helper to get count safely
    async fn count_table(
        db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
        table: &str,
    ) -> i64 {
        let sql = format!("SELECT count() FROM {} GROUP ALL", table);
        let res = db.query(&sql).await;

        match res {
            Ok(mut response) => {
                let result: Option<crate::models::SurrealJson> = response.take(0).unwrap_or(None);
                result
                    .and_then(|v| v.0.get("count").and_then(|c| c.as_i64()))
                    .unwrap_or(0)
            }
            Err(_) => 0,
        }
    }

    let currencies = count_table(&db.0, "currency").await;
    let strategies = count_table(&db.0, "strategy").await;
    let accounts = count_table(&db.0, "account").await;
    let trades = count_table(&db.0, "trade").await;
    let assets = count_table(&db.0, "asset").await;
    let emotional_states = count_table(&db.0, "emotional_state").await;

    let msg = format!(
        "📊 Database Status:\n\
        ----------------------\n\
        - Currencies: {}\n\
        - Strategies: {}\n\
        - Accounts: {}\n\
        - Trades: {}\n\
        - Assets: {}\n\
        - Emotional States: {}\n",
        currencies, strategies, accounts, trades, assets, emotional_states
    );

    println!("{}", msg);
    Ok(msg)
}

// --- Demo Data Management ---

#[tauri::command]
pub async fn seed_demo_account(db: State<'_, DbState>, account_id: String) -> Result<(), String> {
    println!("[DEMO] Gerando dados para {}...", account_id);

    // First ensure the account exists
    crate::seed::demo_accounts_seed::seed_accounts(&db.0, None).await?;

    // Then seed trades for the specific account
    match account_id.as_str() {
        "account:demo_forex" => crate::seed::demo_trades_seed::seed_demo_forex_trades(&db.0).await,
        "account:demo_b3_acoes" => {
            crate::seed::demo_trades_seed::seed_demo_b3_acoes_trades(&db.0).await
        }
        "account:demo_b3_futuros" => {
            crate::seed::demo_trades_seed::seed_demo_b3_futuros_trades(&db.0).await
        }
        "account:demo_nasdaq" => {
            crate::seed::demo_trades_seed::seed_demo_nasdaq_trades(&db.0).await
        }
        "account:demo_crypto" => {
            crate::seed::demo_trades_seed::seed_demo_crypto_trades(&db.0).await
        }
        _ => Err(format!("Conta desconhecida: {}", account_id)),
    }
}

#[tauri::command]
pub async fn delete_demo_account_data(
    db: State<'_, DbState>,
    account_id: String,
) -> Result<(), String> {
    crate::seed::demo_accounts_seed::delete_demo_account_data(&db.0, &account_id).await
}

#[tauri::command]
pub async fn delete_all_demo_trades(db: State<'_, DbState>) -> Result<(), String> {
    crate::seed::demo_accounts_seed::delete_all_demo_trades(&db.0).await
}

#[tauri::command]
pub async fn seed_demo_data(db: State<'_, DbState>, modules: Vec<String>) -> Result<(), String> {
    println!("[DEMO] Gerando dados de demonstração (Filtrados)...");
    let filter = if modules.is_empty() {
        None
    } else {
        Some(modules)
    };

    // Seed accounts with filter
    crate::seed::demo_accounts_seed::seed_accounts(&db.0, filter.clone()).await?;

    // Seed trades with filter
    crate::seed::demo_trades_seed::seed_all_demo_trades(&db.0, filter).await?;

    // Seed demo tax records
    crate::seed::tax_seed::seed_initial_tax_records(&db.0).await
}

#[tauri::command]
pub async fn seed_stress_data(db: State<'_, DbState>, count: usize) -> Result<(), String> {
    crate::seed::stress_seed::seed_stress_records(&db.0, count).await
}

#[tauri::command]
pub async fn complete_onboarding(db: State<'_, DbState>) -> Result<(), String> {
    println!("[COMMAND] Finalizando onboarding no banco de dados...");

    // Using raw query to avoid high-level API result serialization issues
    db.0.query("UPDATE user_profile:main SET onboarding_completed = true RETURN NONE")
        .await
        .map_err(|e| e.to_string())?;

    println!("[COMMAND] Onboarding marcado como concluído.");
    Ok(())
}


#[tauri::command]
pub async fn get_onboarding_meta() -> Result<crate::models::SurrealJson, String> {
    let defaults = crate::seed::defaults::get_onboarding_defaults();
    let mut modules = serde_json::json!([]);

    for m in defaults {
        let mut items = serde_json::json!([]);
        for i in m.items {
            items.as_array_mut().unwrap().push(serde_json::json!({
                "id": format!("{}:{}", m.id, i.id),
                "label": i.label
            }));
        }
        modules.as_array_mut().unwrap().push(serde_json::json!({
            "id": m.id,
            "label": m.label,
            "items": items
        }));
    }

    Ok(crate::models::SurrealJson(modules))
}

#[tauri::command]
pub async fn seed_emotional_states(db: tauri::State<'_, crate::db::DbState>) -> Result<(), String> {
    crate::seed::emotional_states_seed::seed_emotional_states(&db.0, None).await
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LicenseResult {
    pub valid: bool,
    pub plan: String,
    pub expiration: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn validate_license_cmd(key: String, expected_cid: String) -> LicenseResult {
    println!("[LICENSE] Validating key for CID: {}", expected_cid);

    // FORMAT: TLP-v1-<B64_PAYLOAD>-<HEX_SIGNATURE>
    let parts: Vec<&str> = key.split('-').collect();
    if parts.len() != 4 || parts[0] != "TLP" || parts[1] != "v1" {
        return LicenseResult {
            valid: false,
            plan: "Trial".into(),
            expiration: None,
            error: Some("Formato de chave inválido.".into()),
        };
    }

    let payload_b64 = parts[2];
    let signature_hex = parts[3];
    let data_to_verify = format!("{}-{}-{}", parts[0], parts[1], parts[2]);

    // 1. Verify Signature
    let mut mac =
        HmacSha256::new_from_slice(LICENSE_SECRET_KEY).expect("HMAC can take key of any size");
    mac.update(data_to_verify.as_bytes());

    let signature_bytes = match hex::decode(signature_hex) {
        Ok(b) => b,
        Err(_) => {
            return LicenseResult {
                valid: false,
                plan: "Trial".into(),
                expiration: None,
                error: Some("Assinatura inválida (Hex decode failed).".into()),
            }
        }
    };

    if mac.verify_slice(&signature_bytes).is_err() {
        return LicenseResult {
            valid: false,
            plan: "Trial".into(),
            expiration: None,
            error: Some("Assinatura da chave inválida (Chave adulterada).".into()),
        };
    }

    // 2. Decode Payload
    let payload_bytes = match base64::engine::general_purpose::STANDARD.decode(payload_b64) {
        Ok(b) => b,
        Err(_) => {
            return LicenseResult {
                valid: false,
                plan: "Trial".into(),
                expiration: None,
                error: Some("Falha ao decodificar payload (Base64).".into()),
            }
        }
    };

    let payload_str = String::from_utf8_lossy(&payload_bytes);
    let payload: serde_json::Value = match serde_json::from_str(&payload_str) {
        Ok(v) => v,
        Err(_) => {
            return LicenseResult {
                valid: false,
                plan: "Trial".into(),
                expiration: None,
                error: Some("Payload corrompido (JSON).".into()),
            }
        }
    };

    // 3. Check CID (Identity + Hardware Lock)
    if let Some(payload_cid) = payload["cid"].as_str() {
        if payload_cid != expected_cid {
            println!(
                "[LICENSE] CID MISMATCH: expected {}, got {}",
                expected_cid, payload_cid
            );
            return LicenseResult {
                valid: false,
                plan: payload["plan"].as_str().unwrap_or("Pro").into(),
                expiration: payload["exp"].as_str().map(|s| s.into()),
                error: Some("Licença vinculada a outra identidade (ID incorreto).".into()),
            };
        }
    }

    // 4. Return Success
    LicenseResult {
        valid: true,
        plan: payload["plan"].as_str().unwrap_or("Pro").into(),
        expiration: payload["exp"].as_str().map(|s| s.into()),
        error: None,
    }
}

#[tauri::command]
pub fn get_machine_id_cmd() -> String {
    crate::hardware::get_machine_id()
}

// --- Backup & Restore ---

/// Export all user data to a JSON file. Returns the path of the saved file.
#[tauri::command]
pub async fn backup_database(db: State<'_, DbState>, path: String) -> Result<String, String> {
    println!("[COMMAND] backup_database to: {}", path);

    // Ensure correct context
    db.0.use_ns("traderlog")
        .use_db("main")
        .await
        .map_err(|e| format!("Database context error: {}", e))?;

    // Tables to export (in order)
    let tables = vec![
        "user_profile",
        "account",
        "currency",
        "market",
        "asset_type",
        "asset",
        "strategy",
        "emotional_state",
        "modality",
        "tag",
        "timeframe",
        "chart_type",
        "indicator",
        "fee_profile",
        "risk_profile",
        "api_config",
        "trade",
        "cash_transaction",
        "journal_entry",
        "tax_rule",
        "tax_profile",
        "tax_profile_entry",
        "tax_mapping",
        "tax_appraisal",
        "tax_darf",
        "tax_loss",
    ];

    let mut backup = serde_json::json!({
        "version": 1,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "tables": {}
    });

    for table in &tables {
        let sql = format!("SELECT *, type::string(id) as id FROM {}", table);
        match db.0.query(&sql).await {
            Ok(mut res) => match res.take::<Vec<crate::models::SurrealJson>>(0) {
                Ok(rows) => {
                    let row_count = rows.len();
                    backup["tables"][table] = serde_json::json!(rows);
                    println!("[BACKUP] Table {}: {} rows confirmed", table, row_count);
                    if row_count > 0 {
                        if let Some(first) = rows.first() {
                             println!("[BACKUP]   Sample ID for {}: {:?}", table, first.0.get("id"));
                        }
                    }
                }
                Err(e) => {
                    println!("[BACKUP] Warning: failed to read {}: {}", table, e);
                    backup["tables"][table] = serde_json::json!([]);
                }
            },
            Err(e) => {
                println!("[BACKUP] Warning: query failed for {}: {}", table, e);
                backup["tables"][table] = serde_json::json!([]);
            }
        }
    }

    let json_str = serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, &json_str)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    println!("[COMMAND] backup_database SUCCESS: {}", path);
    Ok(path)
}

/// Import user data from a JSON backup file. Restores all tables via UPSERT.
#[tauri::command]
pub async fn restore_database(db: State<'_, DbState>, path: String) -> Result<usize, String> {
    println!("[COMMAND] restore_database from: {}", path);

    // Ensure correct context
    db.0.use_ns("traderlog")
        .use_db("main")
        .await
        .map_err(|e| format!("Database context error: {}", e))?;

    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let backup: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Invalid backup file: {}", e))?;

    let tables = backup["tables"]
        .as_object()
        .ok_or_else(|| "Invalid backup format: missing 'tables'".to_string())?;

    let mut total_restored = 0usize;

    for (table, rows) in tables {
        let rows_arr = match rows.as_array() {
            Some(r) => r,
            None => continue,
        };

        for row in rows_arr {
            // Extract the SurrealDB ID
            let raw_id = row
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Row in '{}' is missing 'id' field", table))?;

            // Extract clean id (after colon if table:id format)
            // Handle both "table:id" and "table:⟨id⟩"
            let mut id_part = raw_id.split(':').last().unwrap_or(raw_id).to_string();
            
            // If it already has ⟨ ⟩, remove them so upsert_record doesn't double wrap
            id_part = id_part.replace("⟨", "").replace("⟩", "");

            // Remove 'id' from data, since we pass it via type::thing
            let mut data = row.clone();
            if let Some(obj) = data.as_object_mut() {
                obj.remove("id");
            }

            println!("[RESTORE] Restoring {}:{}", table, id_part);

            if let Err(e) = upsert_record(&db.0, table, &id_part, data).await {
                println!(
                    "[RESTORE] Warning: failed to restore {}:{} - {}",
                    table, id_part, e
                );
            } else {
                total_restored += 1;
            }
        }
    }

    println!(
        "[COMMAND] restore_database SUCCESS: {} records restored",
        total_restored
    );
    Ok(total_restored)
}

#[tauri::command]
pub async fn open_detached_trade_window(app_handle: tauri::AppHandle, theme: Option<String>) -> Result<(), String> {
    println!("[COMMAND] Spawning detached trade window with theme: {:?}", theme);
    
    // Check if window already exists to avoid duplicates
    if let Some(window) = tauri::Manager::get_webview_window(&app_handle, "detached-trade") {
        let _ = window.set_focus();
        return Ok(());
    }

    let url_str = match &theme {
        Some(t) => format!("detached-trade?theme={}", t),
        None => "detached-trade".to_string(),
    };

    let _window = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "detached-trade",
        tauri::WebviewUrl::App(url_str.into()),
    )
    .title("TraderLog Pro - Novo Trade (Independente)")
    .inner_size(850.0, 750.0)
    .resizable(true)
    .always_on_top(false) // Better UX: don't force on top by default, but allow user to manage
    .center()
    .build()
    .map_err(|e| {
        println!("[ERROR] Failed to build detached window: {}", e);
        e.to_string()
    })?;

    println!("[COMMAND] Detached trade window spawned.");
    Ok(())
}
