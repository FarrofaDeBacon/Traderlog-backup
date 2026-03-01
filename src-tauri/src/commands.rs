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

pub mod irpf;

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
    let query = format!("UPSERT {} CONTENT $data", full_id);

    db.query(&query).bind(("data", data)).await.map_err(|e| {
        println!("[DB] SDK Upsert ERROR for {}. Error: {}", full_id, e);
        e.to_string()
    })?;

    Ok(())
}

/// Helper: DELETE a record by table:id using ⟨UUID⟩ angle-bracket notation.
async fn delete_record(db: &Surreal<Db>, table: &str, id: &str) -> Result<(), String> {
    let query = format!("DELETE {}:⟨{}⟩", table, id);
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
    db.0.query("UPSERT user_profile:main CONTENT $profile")
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

    let profiles: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;

    if let Some(profile_data) = profiles.first() {
        if let Some(hash_val) = profile_data.get("password_hash") {
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
    let profiles: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;

    if let Some(profile_data) = profiles.first() {
        if let Some(hash_val) = profile_data.get("recovery_hash") {
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

    let profiles: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;

    let is_key_valid = if let Some(profile_data) = profiles.first() {
        if let Some(hash_val) = profile_data.get("recovery_hash") {
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

    // Explicitly cast ID to string - Original Logic
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM account")
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
pub async fn save_account(db: State<'_, DbState>, account: Account) -> Result<(), String> {
    println!("[COMMAND] save_account: {:?}", account.id);
    let id = account.id.clone();
    let mut json = serde_json::to_value(&account).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "");
    upsert_record(&db.0, "account", &clean_id, json).await
}

#[tauri::command]
pub async fn delete_account(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
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
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "");
    upsert_record(&db.0, "market", &clean_id, json).await
}

#[tauri::command]
pub async fn delete_market(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
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
    let id = asset_type.id.clone();
    let mut json = serde_json::to_value(&asset_type).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }
    let clean_id = id.split(':').last().unwrap_or(&id);
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
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "");
    upsert_record(&db.0, "asset", &clean_id, json).await
}

#[tauri::command]
pub async fn delete_asset(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
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
    let clean_id = id.split(':').last().unwrap_or(&id);
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
        .unwrap_or(&id)
        .replace("⟨", "")
        .replace("⟩", "");
    upsert_record(&db.0, "strategy", &clean_id, json).await
}

#[tauri::command]
pub async fn delete_strategy(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id
        .split(':')
        .last()
        .unwrap_or(&id)
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
    println!(
        "[COMMAND] save_trade (v2-logging) START for ID: {}",
        trade.id
    );
    // Strip prefix and angle/back-tick wrappers to get the raw UUID
    let clean_id = trade
        .id
        .split(':')
        .last()
        .unwrap_or(&trade.id)
        .replace("⟨", "")
        .replace("⟩", "")
        .replace("`", "");
    println!(
        "[DEBUG] save_trade clean_id: '{}' (from '{}') on account_id: '{}'",
        clean_id, trade.id, trade.account_id
    );

    let mut data = serde_json::to_value(&trade).map_err(|e| e.to_string())?;
    if let Some(obj) = data.as_object_mut() {
        obj.remove("id");
    }

    // First, standard upsert using string-interpolated Record ID (avoids wrapping issues)
    upsert_record(&db.0, "trade", &clean_id, data).await?;

    // Then explicit UPDATE for relational fields using ⟨UUID⟩ angle-bracket Record ID
    let full_trade_id = format!("trade:⟨{}⟩", clean_id);
    let sql = format!("
        UPDATE {} SET 
            account_id = type::thing(account_id),
            asset_type_id = type::thing(asset_type_id),
            strategy_id = type::thing(strategy_id),
            entry_emotional_state_id = IF entry_emotional_state_id != NONE AND entry_emotional_state_id != '' THEN type::thing(entry_emotional_state_id) ELSE NONE END,
            exit_emotional_state_id = IF exit_emotional_state_id != NONE AND exit_emotional_state_id != '' THEN type::thing(exit_emotional_state_id) ELSE NONE END,
            modality_id = IF modality_id != NONE AND modality_id != '' THEN type::thing(modality_id) ELSE NONE END,
            asset_id = IF asset_id != NONE AND asset_id != '' THEN type::thing(asset_id) ELSE NONE END
        ;
    ", full_trade_id);
    db.0.query(&sql).await.map_err(|e| e.to_string())?;

    // --- AUTO-SYNC DAILY CLOSURE (EXTRATO) ---
    // If there is an existing cash_transaction for this account and date, auto-update it.
    let date_only = trade.date.split('T').next().unwrap_or(&trade.date).split(' ').next().unwrap_or(&trade.date);
    let acc_clean = trade
        .account_id
        .split(':')
        .last()
        .unwrap_or(&trade.account_id)
        .replace("⟨", "")
        .replace("⟩", "")
        .replace("`", "")
        .trim()
        .to_string();
    let acc_full = format!("account:⟨{}⟩", acc_clean);

    // Find closure for this date and account safely
    let find_ct_sql = format!(
        "SELECT *, type::string(id) as id, type::string(account_id) as account_id FROM cash_transaction WHERE system_linked = true AND string::startsWith(type::string(date), '{}')",
        date_only
    );

    if let Ok(mut find_res) = db.0.query(&find_ct_sql).await {
        if let Ok(closures) = find_res.take::<Vec<crate::models::CashTransaction>>(0) {
            println!("[COMMAND] save_trade auto-sync: found {} closures for date {} for matching", closures.len(), date_only);
            let mut matched_closure = None;
            for c in closures {
                let c_acc_clean = c.account_id.split(':').last().unwrap_or(&c.account_id)
                    .replace("⟨", "").replace("⟩", "").replace("`", "").trim().to_string();
                
                if c_acc_clean == acc_clean {
                    matched_closure = Some(c);
                    break;
                }
            }

            if let Some(mut closure) = matched_closure {
                let ct_clean = closure
                    .id
                    .split(':')
                    .last()
                    .unwrap_or(&closure.id)
                    .replace("⟨", "")
                    .replace("⟩", "")
                    .replace("`", "")
                    .to_string();
                let ct_full = format!("cash_transaction:⟨{}⟩", ct_clean);

                println!(
                    "[COMMAND] save_trade auto-sync: found exact match closure {} for date {}",
                    ct_full, date_only
                );

                // Ensure this trade is in the list
                let mut current_trade_ids = closure.trade_ids.unwrap_or_default();
                let already_linked = current_trade_ids.iter().any(|tid| {
                    let tid_clean = tid
                        .split(':')
                        .last()
                        .unwrap_or(tid)
                        .replace("⟨", "")
                        .replace("⟩", "")
                        .replace("`", "");
                    tid_clean == clean_id
                });

                if !already_linked {
                    current_trade_ids.push(full_trade_id.clone());
                }

                // Recalculate the sum of all trades in this closure
                let mut new_total = 0.0_f64;
                for tid in &current_trade_ids {
                    let tid_clean = tid
                        .split(':')
                        .last()
                        .unwrap_or(tid)
                        .replace("⟨", "")
                        .replace("⟩", "")
                        .replace("`", "")
                        .trim()
                        .to_string();
                    let sql_res = format!("SELECT result FROM trade:⟨{}⟩ LIMIT 1", tid_clean);
                    if let Ok(mut res_query) = db.0.query(&sql_res).await {
                        if let Ok(results) = res_query.take::<Vec<serde_json::Value>>(0) {
                            if let Some(first) = results.first() {
                                new_total +=
                                    first.get("result").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            }
                        }
                    }
                }

                let tx_type = if new_total >= 0.0 {
                    "Deposit"
                } else {
                    "Withdraw"
                };

                // Calculate difference to update account balance
                let amount_diff = new_total - closure.amount;

                println!(
                    "[COMMAND] save_trade auto-sync: updating {} amount to {} (diff: {})",
                    ct_full, new_total, amount_diff
                );

                let update_ct_sql = format!(
                    "UPDATE {} SET amount = $amount, trade_ids = $t_ids, type = $tx_type",
                    ct_full
                );
                let _ =
                    db.0.query(&update_ct_sql)
                        .bind(("amount", new_total))
                        .bind(("t_ids", current_trade_ids))
                        .bind(("tx_type", tx_type.to_string()))
                        .await;

                if amount_diff != 0.0 {
                    let sql_acc = format!("UPDATE {} SET balance += $diff", acc_full);
                    let _ = db.0.query(&sql_acc).bind(("diff", amount_diff)).await;
                }
            }
        }
    }

    println!("[COMMAND] save_trade SUCCESS for trade:{}", clean_id);
    Ok(())
}

#[tauri::command]
pub async fn delete_trade(db: State<'_, DbState>, id: String) -> Result<(), String> {
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
        "[COMMAND] delete_trade: id='{}' clean_id='{}' full_id='{}'",
        id, clean_id, full_id
    );

    let mut trade_result = 0.0_f64;
    if let Ok(mut trade_res) = db.0.query(&sql_trade).await {
        if let Ok(results) = trade_res.take::<Vec<serde_json::Value>>(0) {
            println!(
                "[COMMAND] delete_trade: trade result query returned {} rows: {:?}",
                results.len(),
                results
            );
            if let Some(first) = results.first() {
                trade_result = first.get("result").and_then(|v| v.as_f64()).unwrap_or(0.0);
            }
        }
    }
    println!("[COMMAND] delete_trade: trade_result={}", trade_result);

    // 2. Find associated daily closures (cash_transactions with this trade in trade_ids)
    // Search with multiple ID formats to handle legacy data
    let sql_ct =
        "SELECT *, type::string(id) as id FROM cash_transaction WHERE system_linked = true";
    let mut cashflow_to_update: Vec<crate::models::CashTransaction> = Vec::new();
    if let Ok(mut ct_res) = db.0.query(sql_ct).await {
        if let Ok(all_cts) = ct_res.take::<Vec<crate::models::CashTransaction>>(0) {
            for ct in all_cts {
                if let Some(ref tids) = ct.trade_ids {
                    let linked = tids.iter().any(|tid| {
                        let tid_clean = tid
                            .split(':')
                            .last()
                            .unwrap_or(tid)
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
        "[COMMAND] delete_trade: found {} linked cash_transactions",
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
                let tid_clean = tid
                    .split(':')
                    .last()
                    .unwrap_or(tid)
                    .replace("⟨", "")
                    .replace("⟩", "")
                    .replace("`", "");
                tid_clean != clean_id
            });

            if t_ids.is_empty() {
                // Last trade in the closure → delete the closure
                println!(
                    "[COMMAND] delete_trade: deleting empty closure {}",
                    ct_full_id
                );
                let _ = delete_record(&db.0, "cash_transaction", &ct_clean_id).await;
            } else {
                // Multiple trades → recalculate and update
                ct.amount -= trade_result;
                let tx_type = if ct.amount >= 0.0 {
                    "Deposit"
                } else {
                    "Withdraw"
                };
                println!(
                    "[COMMAND] delete_trade: updating closure {} new_amount={}",
                    ct_full_id, ct.amount
                );
                let sql_update = format!(
                    "UPDATE {} SET amount = $amount, trade_ids = $t_ids, type = $tx_type",
                    ct_full_id
                );
                let _ =
                    db.0.query(&sql_update)
                        .bind(("amount", ct.amount))
                        .bind(("t_ids", t_ids))
                        .bind(("tx_type", tx_type.to_string()))
                        .await;
            }

            // Update account balance: subtract the deleted trade's result
            let acc_clean = ct
                .account_id
                .split(':')
                .last()
                .unwrap_or(&ct.account_id)
                .replace("⟨", "")
                .replace("⟩", "")
                .replace("`", "")
                .to_string();
            let acc_full = format!("account:⟨{}⟩", acc_clean);
            let sql_acc = format!("UPDATE {} SET balance -= $trade_result", acc_full);
            let _ =
                db.0.query(&sql_acc)
                    .bind(("trade_result", trade_result))
                    .await;
        }
    }

    // 3. Delete the trade itself
    println!("[COMMAND] delete_trade: deleting trade {}", full_id);
    delete_record(&db.0, "trade", &clean_id).await
}

// --- Cash Transactions ---

#[tauri::command]
pub async fn get_cash_transactions(db: State<'_, DbState>) -> Result<Vec<CashTransaction>, String> {
    // Order by date DESC (primary) and ID DESC (stable creation proxy in SurrealDB)
    let sql = "SELECT *, type::string(id) as id FROM cash_transaction ORDER BY date DESC, id DESC";
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
            "  - id: {}, date: {}, type: {:?}, desc: {}, amt: {}, acc: {}, cat: {:?}",
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
    let query = "SELECT type::string(id) as id, type::string(date) as date, (IF exit_date != NONE THEN type::string(exit_date) ELSE null END) as exit_date, type::float(result) as result, type::float(exit_price) as exit_price, type::string(account_id) as account_id FROM trade ORDER BY date DESC LIMIT 1";
    let mut res = db.0.query(query).await.map_err(|e| e.to_string())?;
    let all_fields_trades: Vec<serde_json::Value> = res.take(0).map_err(|e| e.to_string())?;
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
    let txs: Vec<serde_json::Value> = res_tx.take(0).map_err(|e| e.to_string())?;
    for tx in txs {
        println!(
            "Tx: id={}, date={:?}, amt={:?}, desc={:?}, acc={:?}, linked={:?}",
            tx["id"],
            tx["date"],
            tx["amount"],
            tx["description"],
            tx["account_id"],
            tx["system_linked"]
        );
    }

    // 3. Inspect Accounts
    println!("\n[ACCOUNTS]");
    let query_acc = "SELECT type::string(id) as id, nickname, balance FROM account";
    let mut res_acc = db.0.query(query_acc).await.map_err(|e| e.to_string())?;
    let accs: Vec<serde_json::Value> = res_acc.take(0).map_err(|e| e.to_string())?;
    for a in accs {
        println!(
            "Account: id={}, name={:?}, bal={:?}",
            a["id"], a["nickname"], a["balance"]
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
        .unwrap_or(&id)
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
    let trades: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;
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
    let users: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;
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
    let clean_id = id.split(':').last().unwrap_or(&id);
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
    let clean_id = id.split(':').last().unwrap_or(&id);
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
    let clean_id = id.split(':').last().unwrap_or(&id);
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
    let clean_id = id.split(':').last().unwrap_or(&id);
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
    let clean_id = id.split(':').last().unwrap_or(&id);
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
    let clean_id = id.split(':').last().unwrap_or(&id);
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
    let clean_id = id.split(':').last().unwrap_or(&id);
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
        let _ = db.0.query(format!("DELETE {}", table)).await;
    }

    // 2. Reset Onboarding in Profile
    let _ = db
        .0
        .query("UPDATE user_profile:main SET onboarding_completed = false, trial_start_date = NONE")
        .await;

    println!(
        "[ADMIN] ✅ Factory reset concluído. Redirecionando para Onboarding na próxima abertura."
    );
    Ok(())
}

#[tauri::command]
pub async fn ensure_defaults(db: State<'_, DbState>) -> Result<(), String> {
    crate::seed::run_base_seeds(&db.0).await?;
    // Ensure at least an empty Real account exists for new users (no trades)
    crate::seed::demo_accounts_seed::seed_accounts(&db.0, Some(vec!["account:real".to_string()]))
        .await?;
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
                let result: Option<serde_json::Value> = response.take(0).unwrap_or(None);
                result
                    .and_then(|v| v.get("count").and_then(|c| c.as_i64()))
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
pub async fn complete_onboarding(db: State<'_, DbState>) -> Result<(), String> {
    println!("[COMMAND] Finalizando onboarding no banco de dados...");

    // Using raw query to avoid high-level API result serialization issues
    db.0.query("UPDATE user_profile:main SET onboarding_completed = true")
        .await
        .map_err(|e| e.to_string())?;

    println!("[COMMAND] Onboarding marcado como concluído.");
    Ok(())
}

#[tauri::command]
pub async fn seed_custom_data(db: State<'_, DbState>, modules: Vec<String>) -> Result<(), String> {
    crate::seed::run_selective_seeds(&db.0, modules).await
}

#[tauri::command]
pub async fn get_onboarding_meta() -> Result<serde_json::Value, String> {
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

    Ok(modules)
}

#[tauri::command]
pub async fn seed_emotional_states(db: tauri::State<'_, crate::db::DbState>) -> Result<(), String> {
    crate::seed::emotional_states_seed::seed_emotional_states(&db.0, None).await
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
        let sql = format!("SELECT * FROM {}", table);
        match db.0.query(&sql).await {
            Ok(mut res) => match res.take::<Vec<serde_json::Value>>(0) {
                Ok(rows) => {
                    backup["tables"][table] = serde_json::json!(rows);
                    println!("[BACKUP] Table {}: {} rows", table, rows.len());
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
            let clean_id = raw_id.split(':').last().unwrap_or(raw_id);

            // Remove 'id' from data, since we pass it via type::thing
            let mut data = row.clone();
            if let Some(obj) = data.as_object_mut() {
                obj.remove("id");
            }

            if let Err(e) = upsert_record(&db.0, table, clean_id, data).await {
                println!(
                    "[RESTORE] Warning: failed to restore {}:{} - {}",
                    table, clean_id, e
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
