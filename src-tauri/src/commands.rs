use crate::db::DbState;
use crate::models::{
    UserProfile, Account, Currency, Market, AssetType, Asset, EmotionalState, Strategy, ApiConfig,
    Trade, CashTransaction, JournalEntry, FeeProfile, RiskProfile, Modality, Tag, Indicator,
    Timeframe, ChartType
};
use tauri::State;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;


pub mod irpf;

/// Helper: CREATE if new, UPDATE if exists. Same pattern as seeds (proven to persist in SurrealKV).
async fn upsert_record(db: &Surreal<Db>, table: &str, id: &str, data: serde_json::Value) -> Result<(), String> {
    // Utilize native Rust SurrealDB SDK which correctly serializes Things and saves to disk securely
    println!("[DB] Attempting SDK Upsert for {}:{}", table, id);
    // Convert serde_json::Value to surrealdb::sql::Value if needed, or rely on .content()
    // The previous fail was due to .content() attempting to parse the result back into an incompatible type.
    // If we request a generic Option<surrealdb::sql::Value> it fails on the input JSON being an object instead of enum variant Thing.
    // Let's use raw query for 'upsert' to bypass the rigid .content() deserializer constraints.

    let query_str = format!("UPSERT {}:`{}` CONTENT $data;", table, id);
    let parsed_val = surrealdb::sql::json(&data.to_string()).map_err(|e| e.to_string())?;

    let _ = db.query(&query_str)
        .bind(("data", parsed_val))
        .await
        .map_err(|e| {
            println!("[DB] SDK Upsert ERROR for {}:{}. Error: {}", table, id, e);
            e.to_string()
        })?;
        
    println!("[DB] SDK Upsert SUCCESS for {}:{}", table, id);
    Ok(())
}

/// Helper: DELETE a record by table:id (using raw query for reliability)
async fn delete_record(db: &Surreal<Db>, table: &str, id: &str) -> Result<(), String> {
    let query = format!("DELETE {}:{}", table, id);
    db.query(&query).await.map_err(|e| e.to_string())?;
    Ok(())
}

// --- User Profile & Settings ---

#[tauri::command]
pub async fn get_user_profile(db: State<'_, DbState>) -> Result<Option<UserProfile>, String> {
    println!("[COMMAND] get_user_profile called");
    // Explicitly cast ID to string to ensure compatibility with UserProfile struct
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM user_profile:main")
        .await
        .map_err(|e| e.to_string())?;
    
    let raw_json: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;
    println!("[DEBUG] Raw Profile JSON from DB: {:?}", raw_json);
    
    let mut profiles: Vec<UserProfile> = serde_json::from_value(serde_json::Value::Array(raw_json)).map_err(|e| e.to_string())?;
    let profile = profiles.pop();
    println!("[COMMAND] get_user_profile returning: {:?}", profile.is_some());
    Ok(profile)
}

#[tauri::command]
pub async fn save_user_profile(db: State<'_, DbState>, profile: UserProfile) -> Result<(), String> {
    println!("[COMMAND] save_user_profile called: {:?}", profile.name);
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
pub async fn get_api_configs(db: State<'_, DbState>) -> Result<Vec<ApiConfig>, String> {
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM api_config").await.map_err(|e| e.to_string())?;
    let configs: Vec<ApiConfig> = result.take(0).map_err(|e| e.to_string())?;
    Ok(configs)
}

#[tauri::command]
pub async fn save_api_config(db: State<'_, DbState>, config: ApiConfig) -> Result<(), String> {
    let id = config.id.clone();
    let mut json = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "api_config", clean_id, json).await
}

// --- Accounts ---

#[tauri::command]
pub async fn get_accounts(db: State<'_, DbState>) -> Result<Vec<Account>, String> {
    println!("[COMMAND] get_accounts called");

    // DEBUG: Check count
    let count_res = db.0.query("SELECT count() FROM account GROUP ALL").await;
    println!("[DEBUG] Count Result: {:?}", count_res);

    // DEBUG: Check Info
    let info_res = db.0.query("INFO FOR DB").await;
    println!("[DEBUG] Info Result: {:?}", info_res);

    // DEBUG: Raw JSON
    let mut raw_res = db.0.query("SELECT * FROM account").await.map_err(|e| e.to_string())?;
    let raw_json: Vec<serde_json::Value> = raw_res.take(0).unwrap_or_default();
    println!("[DEBUG] Raw Account JSON: {:?}", raw_json);

    // DEBUG: Direct Fetch
    let mut direct_res = db.0.query("SELECT * FROM account:demo_forex").await.map_err(|e| e.to_string())?;
    let direct_json: Option<serde_json::Value> = direct_res.take(0).unwrap_or(None);
    println!("[DEBUG] Direct Fetch demo_forex: {:?}", direct_json);

    // Explicitly cast ID to string - Original Logic
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM account").await.map_err(|e| e.to_string())?;
    let accounts: Vec<Account> = result.take(0).map_err(|e| e.to_string())?;
    println!("[COMMAND] get_accounts returning {} accounts", accounts.len());
    Ok(accounts)
}


#[tauri::command]
pub async fn save_account(db: State<'_, DbState>, account: Account) -> Result<(), String> {
    println!("[COMMAND] save_account: {:?}", account.id);
    let id = account.id.clone();
    let mut json = serde_json::to_value(&account).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "account", clean_id, json).await
}

#[tauri::command]
pub async fn delete_account(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "account", clean_id).await
}

// --- Currencies ---

#[tauri::command]
pub async fn get_currencies(db: State<'_, DbState>) -> Result<Vec<Currency>, String> {
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM currency").await.map_err(|e| e.to_string())?;
    let currencies: Vec<Currency> = result.take(0).map_err(|e| e.to_string())?;
    Ok(currencies)
}

#[tauri::command]
pub async fn save_currency(db: State<'_, DbState>, currency: Currency) -> Result<(), String> {
    let id = currency.id.clone();
    let mut json = serde_json::to_value(&currency).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM market").await.map_err(|e| e.to_string())?;
    let markets: Vec<Market> = result.take(0).map_err(|e| e.to_string())?;
    Ok(markets)
}

#[tauri::command]
pub async fn save_market(db: State<'_, DbState>, market: Market) -> Result<(), String> {
    let id = market.id.clone();
    let mut json = serde_json::to_value(&market).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "market", clean_id, json).await
}

#[tauri::command]
pub async fn delete_market(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "market", clean_id).await
}

// --- Asset Types ---

#[tauri::command]
pub async fn get_asset_types(db: State<'_, DbState>) -> Result<Vec<AssetType>, String> {
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM asset_type").await.map_err(|e| e.to_string())?;
    let types: Vec<AssetType> = result.take(0).map_err(|e| e.to_string())?;
    Ok(types)
}

#[tauri::command]
pub async fn save_asset_type(db: State<'_, DbState>, asset_type: AssetType) -> Result<(), String> {
    let id = asset_type.id.clone();
    let mut json = serde_json::to_value(&asset_type).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM asset").await.map_err(|e| e.to_string())?;
    let assets: Vec<Asset> = result.take(0).map_err(|e| e.to_string())?;
    Ok(assets)
}

#[tauri::command]
pub async fn save_asset(db: State<'_, DbState>, asset: Asset) -> Result<(), String> {
    let id = asset.id.clone();
    let mut json = serde_json::to_value(&asset).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "asset", clean_id, json).await
}

#[tauri::command]
pub async fn delete_asset(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "asset", clean_id).await
}

// --- Emotional States ---

#[tauri::command]
pub async fn get_emotional_states(db: State<'_, DbState>) -> Result<Vec<EmotionalState>, String> {
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM emotional_state").await.map_err(|e| e.to_string())?;
    let states: Vec<EmotionalState> = result.take(0).map_err(|e| e.to_string())?;
    Ok(states)
}

#[tauri::command]
pub async fn save_emotional_state(db: State<'_, DbState>, state: EmotionalState) -> Result<(), String> {
    let id = state.id.clone();
    let mut json = serde_json::to_value(&state).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM strategy").await.map_err(|e| e.to_string())?;
    let strategies: Vec<Strategy> = result.take(0).map_err(|e| e.to_string())?;
    Ok(strategies)
}

#[tauri::command]
pub async fn save_strategy(db: State<'_, DbState>, strategy: Strategy) -> Result<(), String> {
    let id = strategy.id.clone();
    let mut json = serde_json::to_value(&strategy).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "strategy", clean_id, json).await
}

#[tauri::command]
pub async fn delete_strategy(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "strategy", clean_id).await
}

// --- Trades ---

#[tauri::command]
pub async fn get_trades(db: State<'_, DbState>) -> Result<Vec<Trade>, String> {
    println!("[COMMAND] get_trades called");
    
    let sql = "SELECT *, type::string(id) as id FROM trade ORDER BY date DESC";
    let mut response = db.0.query(sql).await.map_err(|e| e.to_string())?;
    
    let trades: Vec<Trade> = response.take(0).map_err(|e| {
        eprintln!("[COMMAND] get_trades error: {}", e);
        e.to_string()
    })?;
    
    println!("[COMMAND] get_trades returning {} trades", trades.len());
    Ok(trades)
}



#[tauri::command]
pub async fn save_trade(db: State<'_, DbState>, trade: Trade) -> Result<(), String> {
    println!("[COMMAND] save_trade START for ID: {}", trade.id);
    let clean_id = trade.id.split(':').last().unwrap_or(&trade.id);
    let mut data = serde_json::to_value(&trade).map_err(|e| e.to_string())?;
    if let Some(obj) = data.as_object_mut() { obj.remove("id"); }
    
    // First, standard upsert
    upsert_record(&db.0, "trade", clean_id, data).await?;
    
    // Then explicit updates for relational fields
    let sql = format!("
        UPDATE trade:{} SET 
            account_id = type::thing(account_id),
            asset_type_id = type::thing(asset_type_id),
            strategy_id = type::thing(strategy_id),
            entry_emotional_state_id = IF entry_emotional_state_id != NONE AND entry_emotional_state_id != '' THEN type::thing(entry_emotional_state_id) ELSE NONE END,
            exit_emotional_state_id = IF exit_emotional_state_id != NONE AND exit_emotional_state_id != '' THEN type::thing(exit_emotional_state_id) ELSE NONE END,
            modality_id = IF modality_id != NONE AND modality_id != '' THEN type::thing(modality_id) ELSE NONE END,
            asset_id = IF asset_id != NONE AND asset_id != '' THEN type::thing(asset_id) ELSE NONE END
        ;
    ", clean_id);
    db.0.query(&sql).await.map_err(|e| e.to_string())?;
    
    println!("[COMMAND] save_trade SUCCESS for trade:{}", clean_id);
    Ok(())
}

#[tauri::command]
pub async fn delete_trade(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "trade", clean_id).await
}

// --- Cash Transactions ---

#[tauri::command]
pub async fn get_cash_transactions(db: State<'_, DbState>) -> Result<Vec<CashTransaction>, String> {
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM cash_transaction").await.map_err(|e| e.to_string())?;
    let transactions: Vec<CashTransaction> = result.take(0).map_err(|e| e.to_string())?;
    Ok(transactions)
}

#[tauri::command]
pub async fn save_cash_transaction(db: State<'_, DbState>, transaction: CashTransaction) -> Result<(), String> {
    let id = transaction.id.clone();
    let mut json = serde_json::to_value(&transaction).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "cash_transaction", clean_id, json).await
}

#[tauri::command]
pub async fn delete_cash_transaction(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id).to_string();
    
    // 1. Check if linked to system logic
    let mut res = db.0.query("SELECT system_linked FROM type::thing('cash_transaction', $id)")
        .bind(("id", clean_id.clone()))
        .await
        .map_err(|e| e.to_string())?;
    
    let linked: Option<bool> = res.take(0).map_err(|e| e.to_string())?;
    if linked == Some(true) {
        return Err("Esta transação está vinculada ao sistema e não pode ser excluída manualmente.".into());
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
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    let clean_id = id.split(':').last().unwrap_or(&id);
    upsert_record(&db.0, "journal_entry", clean_id, json).await
}

#[tauri::command]
pub async fn delete_journal_entry(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    delete_record(&db.0, "journal_entry", clean_id).await
}

// --- Fee Profiles ---

#[tauri::command]
pub async fn get_fees(db: State<'_, DbState>) -> Result<Vec<FeeProfile>, String> {
    println!("[COMMAND] get_fees called");
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM fee_profile").await.map_err(|e| e.to_string())?;
    let fees: Vec<FeeProfile> = result.take(0).map_err(|e| e.to_string())?;
    println!("[COMMAND] get_fees returning {} items", fees.len());
    Ok(fees)
}

// --- Diagnostic Commands ---

#[tauri::command]
pub async fn diagnostic_dump_trades(db: State<'_, DbState>) -> Result<(), String> {
    println!("[DIAGNOSTIC] Dumping all trades in database...");
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM trade").await.map_err(|e| e.to_string())?;
    let trades: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;
    println!("[DIAGNOSTIC] Total trades found: {}", trades.len());
    for t in trades {
        println!("[DIAGNOSTIC] TRADE: {}", serde_json::to_string(&t).unwrap_or_default());
    }
    Ok(())
}

#[tauri::command]
pub async fn diagnostic_dump_users(db: State<'_, DbState>) -> Result<(), String> {
    println!("[DIAGNOSTIC] Dumping all user profiles in database...");
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM user_profile").await.map_err(|e| e.to_string())?;
    let users: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;
    println!("[DIAGNOSTIC] Total users found: {}", users.len());
    for u in users {
        println!("[DIAGNOSTIC] USER: {}", serde_json::to_string(&u).unwrap_or_default());
    }
    Ok(())
}

#[tauri::command]
pub async fn save_fee(db: State<'_, DbState>, fee: FeeProfile) -> Result<(), String> {
    let id = fee.id.clone();
    let mut json = serde_json::to_value(&fee).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM risk_profile").await.map_err(|e| e.to_string())?;
    let profiles: Vec<RiskProfile> = result.take(0).map_err(|e| e.to_string())?;
    println!("[COMMAND] get_risk_profiles returning {} items", profiles.len());
    Ok(profiles)
}

#[tauri::command]
pub async fn save_risk_profile(db: State<'_, DbState>, profile: RiskProfile) -> Result<(), String> {
    let id = profile.id.clone();
    let mut json = serde_json::to_value(&profile).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM modality").await.map_err(|e| e.to_string())?;
    let modalities: Vec<Modality> = result.take(0).map_err(|e| e.to_string())?;
    println!("[COMMAND] get_modalities returning {} items", modalities.len());
    Ok(modalities)
}

#[tauri::command]
pub async fn save_modality(db: State<'_, DbState>, modality: Modality) -> Result<(), String> {
    let id = modality.id.clone();
    let mut json = serde_json::to_value(&modality).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM tag").await.map_err(|e| e.to_string())?;
    let tags: Vec<Tag> = result.take(0).map_err(|e| e.to_string())?;
    Ok(tags)
}

#[tauri::command]
pub async fn save_tag(db: State<'_, DbState>, tag: Tag) -> Result<(), String> {
    let id = tag.id.clone();
    let mut json = serde_json::to_value(&tag).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM indicator").await.map_err(|e| e.to_string())?;
    let indicators: Vec<Indicator> = result.take(0).map_err(|e| e.to_string())?;
    Ok(indicators)
}

#[tauri::command]
pub async fn save_indicator(db: State<'_, DbState>, indicator: Indicator) -> Result<(), String> {
    let id = indicator.id.clone();
    let mut json = serde_json::to_value(&indicator).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM timeframe").await.map_err(|e| e.to_string())?;
    let timeframes: Vec<Timeframe> = result.take(0).map_err(|e| e.to_string())?;
    Ok(timeframes)
}

#[tauri::command]
pub async fn save_timeframe(db: State<'_, DbState>, timeframe: Timeframe) -> Result<(), String> {
    let id = timeframe.id.clone();
    let mut json = serde_json::to_value(&timeframe).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM chart_type").await.map_err(|e| e.to_string())?;
    let types: Vec<ChartType> = result.take(0).map_err(|e| e.to_string())?;
    Ok(types)
}

#[tauri::command]
pub async fn save_chart_type(db: State<'_, DbState>, chart_type: ChartType) -> Result<(), String> {
    let id = chart_type.id.clone();
    let mut json = serde_json::to_value(&chart_type).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
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
        "trade", "cash_transaction", "journal_entry", "account", 
        "strategy", "indicator", "modality", "tag", "fee_profile", "emotional_state",
        "risk_profile", "timeframe", "chart_type", "asset", "asset_type", "market", "currency",
        "tax_rule", "tax_profile", "tax_profile_entry", "tax_mapping", "tax_appraisal", "tax_darf", "tax_loss"
    ];
    
    for table in tables {
        let _ = db.0.query(format!("DELETE {}", table)).await;
    }

    // 2. Reset Onboarding in Profile
    let _ = db.0.query("UPDATE user_profile:main SET onboarding_completed = false, trial_start_date = NONE").await;

    println!("[ADMIN] ✅ Factory reset concluído. Redirecionando para Onboarding na próxima abertura.");
    Ok(())
}

#[tauri::command]
pub async fn ensure_defaults(db: State<'_, DbState>) -> Result<(), String> {
    crate::seed::run_all_seeds(&db.0).await
}

#[tauri::command]
pub async fn check_database_status(db: tauri::State<'_, DbState>) -> Result<String, String> {
    println!("[DIAGNOSTIC] Checking database status...");
    
    // Helper to get count safely
    async fn count_table(db: &surrealdb::Surreal<surrealdb::engine::local::Db>, table: &str) -> i64 {
        let sql = format!("SELECT count() FROM {} GROUP ALL", table);
        let res = db.query(&sql).await;
        
        match res {
            Ok(mut response) => {
                let result: Option<serde_json::Value> = response.take(0).unwrap_or(None);
                result.and_then(|v| v.get("count").and_then(|c| c.as_i64())).unwrap_or(0)
            },
            Err(_) => 0
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
        "account:demo_b3_acoes" => crate::seed::demo_trades_seed::seed_demo_b3_acoes_trades(&db.0).await,
        "account:demo_b3_futuros" => crate::seed::demo_trades_seed::seed_demo_b3_futuros_trades(&db.0).await,
        "account:demo_nasdaq" => crate::seed::demo_trades_seed::seed_demo_nasdaq_trades(&db.0).await,
        "account:demo_crypto" => crate::seed::demo_trades_seed::seed_demo_crypto_trades(&db.0).await,
        _ => Err(format!("Conta desconhecida: {}", account_id)),
    }
}

#[tauri::command]
pub async fn delete_demo_account_data(db: State<'_, DbState>, account_id: String) -> Result<(), String> {
    crate::seed::demo_accounts_seed::delete_demo_account_data(&db.0, &account_id).await
}

#[tauri::command]
pub async fn delete_all_demo_trades(db: State<'_, DbState>) -> Result<(), String> {
    crate::seed::demo_accounts_seed::delete_all_demo_trades(&db.0).await
}

#[tauri::command]
pub async fn seed_demo_data(db: State<'_, DbState>, modules: Vec<String>) -> Result<(), String> {
    println!("[DEMO] Gerando dados de demonstração (Filtrados)...");
    let filter = if modules.is_empty() { None } else { Some(modules) };
    
    // Seed accounts with filter
    crate::seed::demo_accounts_seed::seed_accounts(&db.0, filter.clone()).await?;
    
    // Seed trades with filter
    crate::seed::demo_trades_seed::seed_all_demo_trades(&db.0, filter).await
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
