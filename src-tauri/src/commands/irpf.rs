use crate::db::DbState;
use crate::models::irpf::{TaxAppraisal, TaxLoss, TaxDarf};
use crate::models::Trade;
use chrono::{Datelike, NaiveDate};
use serde_json;
use tauri::State;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IrpfTrade {
    #[serde(deserialize_with = "crate::models::deserialize_id")]
    pub id: String,
    pub date: String,
    pub exit_date: Option<String>,
    pub result: Option<f64>,
    pub fee_total: Option<f64>,
    pub quantity: Option<f64>,
    pub exit_price: Option<f64>,
    #[serde(default, deserialize_with = "crate::models::deserialize_id_opt")]
    pub asset_id: Option<String>,
    #[serde(default, deserialize_with = "crate::models::deserialize_id_opt")]
    pub asset_type_id: Option<String>,
    #[serde(default, deserialize_with = "crate::models::deserialize_id_opt")]
    pub modality_id: Option<String>,
}

use crate::models::{TaxRule, TaxProfile, TaxProfileEntry, Asset, AssetType, TaxMapping, Modality};

/// Calculates monthly tax using Dynamic Rules Engine (Profile-based)
#[tauri::command]
pub async fn calculate_monthly_tax(db: State<'_, DbState>, month: u8, year: u16) -> Result<Vec<TaxAppraisal>, String> {
    println!("DEBUG: calculate_monthly_tax (Profile-Based) called for {}/{}", month, year);
    
    // Trades (Fetch all and filter in memory to avoid format quirks in SurrealDB)
    let query_trades = "SELECT 
            type::string(id) as id,
            type::string(date) as date, 
            (IF exit_date != NONE AND exit_date != '' THEN type::string(exit_date) ELSE null END) as exit_date, 
            result, 
            fee_total, 
            quantity, 
            exit_price,
            (IF asset_id THEN type::string(asset_id) ELSE null END) as asset_id,
            (IF asset_type_id THEN type::string(asset_type_id) ELSE null END) as asset_type_id,
            (IF modality_id THEN type::string(modality_id) ELSE null END) as modality_id
         FROM trade WHERE 
         exit_date IS NOT NONE AND
         exit_date != '' AND
         result IS NOT NONE";
         
    let mut trade_res = db.0.query(query_trades).await.map_err(|e| e.to_string())?;
    let all_trades: Vec<IrpfTrade> = trade_res.take(0).map_err(|e| e.to_string())?;
    
    // In-memory format agnostic filter
    let prefix = format!("{}-{:02}-", year, month);
    let trades: Vec<IrpfTrade> = all_trades.into_iter().filter(|t| {
        if let Some(ed) = &t.exit_date {
            ed.starts_with(&prefix)
        } else {
            false
        }
    }).collect();

    println!("[IRPF] Total de trades encontrados no banco após filtro memória: {}", trades.len());

    // Assets & Asset Types
    let mut asset_res = db.0.query("SELECT id, symbol, name, asset_type_id, point_value, default_fee_id, type::string(id) as id, (IF asset_type_id THEN type::string(asset_type_id) ELSE null END) as asset_type_id FROM asset").await.map_err(|e| e.to_string())?;
    let assets: Vec<Asset> = asset_res.take(0).map_err(|e| e.to_string())?;

    let mut at_res = db.0.query("SELECT id, name, code, market_id, default_fee_id, tax_profile_id, unit_label, result_type, type::string(id) as id, (IF tax_profile_id THEN type::string(tax_profile_id) ELSE null END) as tax_profile_id FROM asset_type").await.map_err(|e| e.to_string())?;
    let asset_types: Vec<AssetType> = at_res.take(0).map_err(|e| e.to_string())?;

    // Modalities
    let mut mod_res = db.0.query("SELECT *, type::string(id) as id FROM modality").await.map_err(|e| e.to_string())?;
    let modalities: Vec<Modality> = mod_res.take(0).map_err(|e| e.to_string())?;

    // Tax Profiles - Just used for resolution mapping
    let mut entry_res = db.0.query("SELECT id, tax_profile_id, modality_id, tax_rule_id, type::string(id) as id, (IF tax_profile_id THEN type::string(tax_profile_id) ELSE null END) as tax_profile_id, (IF modality_id THEN type::string(modality_id) ELSE null END) as modality_id, (IF tax_rule_id THEN type::string(tax_rule_id) ELSE null END) as tax_rule_id FROM tax_profile_entry").await.map_err(|e| e.to_string())?;
    let profile_entries: Vec<TaxProfileEntry> = entry_res.take(0).map_err(|e| e.to_string())?;
    
    // Tax Rules
    let mut rule_res = db.0.query("SELECT *, type::string(id) as id FROM tax_rule").await.map_err(|e| e.to_string())?;
    let rules: Vec<TaxRule> = rule_res.take(0).map_err(|e| e.to_string())?;

    // Accumulated Losses
    let mut loss_res = db.0.query("SELECT *, type::string(id) as id FROM tax_loss WHERE balance > 0").await.map_err(|e| e.to_string())?;
    let losses: Vec<TaxLoss> = loss_res.take(0).map_err(|e| e.to_string())?;

    // Existing Appraisals (to preserve IDs)
    let exist_query = "SELECT *, type::string(id) as id FROM tax_appraisal WHERE period_month = $month AND period_year = $year";
    let mut exist_res = db.0.query(exist_query).bind(("month", month)).bind(("year", year)).await.map_err(|e| e.to_string())?;
    let existing_appraisals: Vec<TaxAppraisal> = exist_res.take(0).map_err(|e| e.to_string())?;

    println!("[IRPF] DEBUG: Loaded {} profile_entries", profile_entries.len());
    for (i, e) in profile_entries.iter().enumerate() {
        println!("[IRPF] DEBUG: Entry {}: profile_id={}, modality_id={}, rule_id={}", i, e.tax_profile_id, e.modality_id, e.tax_rule_id);
    }

    // 2. Processing Buckets
    struct RuleBucket {
        rule: TaxRule,
        gross_profit: f64,
        gross_loss: f64,
        sales_total: f64,
    }
    
    // Helper: Find Tax Profile ID
    let resolve_tax_profile = |t: &IrpfTrade| -> Option<String> {
        // 1. Try via Asset ID
        if let Some(ref aid) = t.asset_id {
             if let Some(asset) = assets.iter().find(|a| a.id == *aid || a.id.ends_with(aid) || aid.ends_with(&a.id)) {
                let at_clean = asset.asset_type_id.split(':').last().unwrap_or(&asset.asset_type_id);
                if let Some(at) = asset_types.iter().find(|t| {
                    let curr_t_id = t.id.split(':').last().unwrap_or(&t.id);
                    curr_t_id == at_clean
                }) {
                    println!("[IRPF] Mapped via Asset {} -> Type {} -> Profile {:?}", asset.symbol, at.name, at.tax_profile_id);
                    return at.tax_profile_id.clone();
                }
             }
        }
        
        // 2. Try via Asset Type ID directly from Trade
        if let Some(ref atid) = t.asset_type_id {
            let at_clean = atid.split(':').last().unwrap_or(atid);
            if let Some(at) = asset_types.iter().find(|t| {
                let curr_t_id = t.id.split(':').last().unwrap_or(&t.id);
                curr_t_id == at_clean
            }) {
                 println!("[IRPF] Mapped via Trade AssetType {} -> Profile {:?}", at.name, at.tax_profile_id);
                 return at.tax_profile_id.clone();
            }
        }
        
        println!("[IRPF] Could not resolve Tax Profile for trade {}", t.id);
        None
    };

    // Helper: Find modality name for ID
    let get_modality_name = |mid: &str| -> Option<String> {
        let clean_mid = mid.split(':').last().unwrap_or(mid);
        modalities.iter().find(|m| {
            let curr_m_id = m.id.split(':').last().unwrap_or(&m.id);
            curr_m_id == clean_mid
        }).map(|m| m.name.clone())
    };

    // Helper: Find Rule for (Profile + Modality)
    let find_rule = |profile_id: &str, is_day_trade: bool| -> Option<TaxRule> {
        let modality_name = if is_day_trade { "DayTrade" } else { "SwingTrade" };
        
        let clean_pid = profile_id.split(':').last().unwrap_or(profile_id).trim_matches('`');

        let entry = profile_entries.iter().find(|e| {
            let e_pid = e.tax_profile_id.split(':').last().unwrap_or(&e.tax_profile_id).trim_matches('`');
            let e_mid = e.modality_id.split(':').last().unwrap_or(&e.modality_id).trim_matches('`');
            
            // Resolve modality name from ID
            let e_mname = get_modality_name(e_mid).unwrap_or_else(|| "".to_string());
            
            let match_pid = e_pid == clean_pid;
            
            // Robust space-insensitive match
            let mname_clean = e_mname.to_lowercase().replace(" ", "");
            let target_clean = modality_name.to_lowercase().replace(" ", "");
            let match_mid = mname_clean.contains(&target_clean);
            
            // println!("[IRPF] DEBUG: Comparing (pid: {} == {}, mname: {} >= {}) -> {}, {}", e_pid, clean_pid, e_mname, modality_name, match_pid, match_mid);
            
            match_pid && match_mid
        });

        if let Some(e) = entry {
            let r_id = e.tax_rule_id.split(':').last().unwrap_or(&e.tax_rule_id).trim_matches('`');
            let rule = rules.iter().find(|r| {
                let curr_r_id = r.id.split(':').last().unwrap_or(&r.id).trim_matches('`');
                curr_r_id == r_id
            }).cloned();
            
            if rule.is_none() {
                println!("[IRPF] Found entry for Profile {}, but Rule ID {} not found in total {} rules.", clean_pid, e.tax_rule_id, rules.len());
            } else {
                println!("[IRPF] Successfully matched Rule {} for Profile {} and Modality {}", r_id, clean_pid, modality_name);
            }
            rule
        } else {
            println!("[IRPF] No Tax Profile Entry found for Profile {} and Modality {}", clean_pid, modality_name);
            None
        }
    };

    // Group Trades into Buckets keyed by Tax Rule ID
    use std::collections::HashMap;
    let mut buckets: HashMap<String, RuleBucket> = HashMap::new();

    println!("[IRPF] Total de trades encontrados no banco: {}", trades.len());
    for trade in &trades {
        println!("[IRPF] Processando Trade: id={}, date={}, asset_type_id={:?}", trade.id, trade.date, trade.asset_type_id);
        
        // Determine Day Trade: prefer modality_id from trade, fallback to date comparison
        let is_day_trade = if let Some(ref mid) = trade.modality_id {
            let clean_mid = mid.split(':').last().unwrap_or(mid).trim_matches('`');
            let mname = get_modality_name(clean_mid).unwrap_or_default();
            let is_dt = mname.to_lowercase().contains("day");
            println!("[IRPF] Trade {} modality_id={} name='{}' -> looks like DayTrade? {}", trade.id, mid, mname, is_dt);
            is_dt
        } else {
            let entry_date = NaiveDate::parse_from_str(&trade.date[0..10], "%Y-%m-%d").unwrap_or_default();
            let exit_date_str = trade.exit_date.as_ref().unwrap();
            let exit_date = NaiveDate::parse_from_str(&exit_date_str[0..10], "%Y-%m-%d").unwrap_or_default();
            let is_dt = entry_date == exit_date;
            println!("[IRPF] Trade {} no modality_id, fallback dating match? {}", trade.id, is_dt);
            is_dt
        };

        let result_val = trade.result.unwrap_or(0.0) - trade.fee_total.unwrap_or(0.0);
        
        // RESOLUTION LOGIC: Asset -> AssetType fallback
        let mut final_profile_id = None;

        // 1. Try Asset Specific Profile
        if let Some(aid) = &trade.asset_id {
            if let Some(asset) = assets.iter().find(|a| a.id == *aid || a.id.ends_with(aid) || aid.ends_with(&a.id)) {
                let atid = &asset.asset_type_id;
                if let Some(at) = asset_types.iter().find(|t| t.id == *atid || t.id.ends_with(atid) || atid.ends_with(&t.id)) {
                    final_profile_id = at.tax_profile_id.clone();
                    println!("[IRPF] Trade {} -> Tax Profile {} resolvido via Ativo", trade.id, final_profile_id.as_deref().unwrap_or("None"));
                }
            }
        }

        // 2. Fallback to Trade's AssetType directly if still None
        if final_profile_id.is_none() {
            if let Some(atid) = &trade.asset_type_id {
                if let Some(at) = asset_types.iter().find(|t| t.id == *atid || t.id.ends_with(atid) || atid.ends_with(&t.id)) {
                    final_profile_id = at.tax_profile_id.clone();
                    println!("[IRPF] Trade {} -> Tax Profile {} resolvido via Tipo de Ativo", trade.id, final_profile_id.as_deref().unwrap_or("None"));
                }
            }
        }

        if let Some(profile_id) = final_profile_id {
             if let Some(rule) = find_rule(&profile_id, is_day_trade) {
                 println!("[IRPF] Trade {} -> Regra Fiscal {} encontrada", trade.id, rule.id);
                 let entry = buckets.entry(rule.id.clone()).or_insert(RuleBucket {
                     rule: rule.clone(),
                     gross_profit: 0.0,
                     gross_loss: 0.0,
                     sales_total: 0.0,
                 });
                 // ... resto da lógica
                 
                 if result_val > 0.0 {
                     entry.gross_profit += result_val;
                 } else {
                     entry.gross_loss += result_val.abs();
                 }
                 
                 if !is_day_trade { 
                     if let (Some(price), Some(qty)) = (trade.exit_price, trade.quantity) {
                         entry.sales_total += price * qty;
                     }
                 }
             }
        } else {
            println!("[IRPF] Trade {} skipped: No Tax Profile found (Asset: {:?}, Type: {:?})", trade.id, trade.asset_id, trade.asset_type_id);
        }
    }
    
    println!("[IRPF] Distribution finished. Buckets: {}", buckets.len());

    // 3. Calculate Appraisal per Bucket
    let calculation_date = chrono::Utc::now().to_rfc3339();
    let mut appraisals = Vec::new();

    for (_, bucket) in buckets {
        let rule = &bucket.rule;
        
        // 3.1 Net Result
        let net_profit = bucket.gross_profit - bucket.gross_loss;
        
        // 3.2 Exemption Logic
        let threshold = rule.exemption_threshold;
        let is_exempt = threshold > 0.0 && bucket.sales_total < threshold && net_profit > 0.0;
        
        // 3.3 Compensation Logic
        // HEURISTIC: Use Rule Name to detect "Day" vs "Swing" to map to legacy TaxLoss buckets.
        let loss_category = if rule.name.contains("Day") { "DayTrade" } else { "SwingTrade" };
        
        let mut compensated_loss = 0.0;
        let mut calculation_basis;
        
        if is_exempt {
            calculation_basis = 0.0;
        } else {
            if net_profit > 0.0 {
                // Try to compensate
                let current_month_start = format!("{}-{:02}-01", year, month);
                let available_loss: f64 = losses.iter()
                    .filter(|l| l.trade_type == loss_category && l.origin_date < current_month_start)
                    .map(|l| l.balance)
                    .sum();
                
                println!("[IRPF] Balanço: {} lucro, {} prejuízo acumulado disponível (antes de {})", net_profit, available_loss, current_month_start);
                    
                if available_loss > 0.0 {
                    if available_loss >= net_profit {
                        compensated_loss = net_profit;
                        calculation_basis = 0.0;
                    } else {
                        compensated_loss = available_loss;
                        calculation_basis = net_profit - available_loss;
                    }
                } else {
                    calculation_basis = net_profit;
                }
            } else {
                calculation_basis = 0.0;
                // Loss will be accumulated
            }
        }
        
        // 3.4 Tax Calculation
        // tax_rate is stored as percentage (e.g. 15.0 = 15%), convert to decimal
        let rate_decimal = rule.tax_rate / 100.0;
        let tax_due = calculation_basis * rate_decimal;
        // Withholding (Dedo-duro) - withholding_rate is already in decimal (e.g. 0.005 = 0.5%)
        let withheld = if rule.withholding_rate > 0.0 && calculation_basis > 0.0 {
             calculation_basis * rule.withholding_rate 
        } else { 0.0 };
        
        let tax_payable = tax_due - withheld;

        // 3.5 Accumulation Logic (< R$ 10)
        let trade_type_cat = if rule.name.to_lowercase().contains("day") { "DayTrade" } else { "SwingTrade" };
        
        // Find latest pending appraisal for this category that was below 10
        let accum_query = "SELECT * FROM tax_appraisal WHERE trade_type = $type AND status = 'Pending' AND total_payable < 10.0 AND (period_year < $year OR (period_year = $year AND period_month < $month)) ORDER BY period_year DESC, period_month DESC LIMIT 1";
        println!("DEBUG: Running accumulation query: {}", accum_query);
            
        let mut accum_res = db.0.query(accum_query)
            .bind(("type", trade_type_cat))
            .bind(("year", year))
            .bind(("month", month))
            .await.map_err(|e| e.to_string())?;
            
        let prev_accum: Vec<serde_json::Value> = accum_res.take(0).map_err(|e| e.to_string())?;
        let tax_accumulated = if let Some(v) = prev_accum.first() {
            v.get("total_payable").and_then(|t| t.as_f64()).unwrap_or(0.0)
        } else {
            0.0
        };

        let total_payable = tax_payable + tax_accumulated;
        
        // 3.6 Status
        // Legacy `trade_type` field in TaxAppraisal stores Rule Name or "SwingTrade"/"DayTrade" legacy string.
        let existing = existing_appraisals.iter().find(|a| a.trade_type == trade_type_cat); // Use category for matching
        
        let status = if let Some(ex) = existing {
            if ex.status == "Paid" { "Paid".to_string() } else if total_payable > 0.0 { "Pending".to_string() } else { "Ok".to_string() }
        } else {
            if total_payable > 0.0 { "Pending".to_string() } else { "Ok".to_string() }
        };

        appraisals.push(TaxAppraisal {
            id: existing.and_then(|e| e.id.clone()),
            period_month: month,
            period_year: year,
            trade_type: trade_type_cat.to_string(),
            gross_profit: bucket.gross_profit,
            loss: bucket.gross_loss,
            net_profit,
            compensated_loss,
            calculation_basis,
            tax_rate: rate_decimal,
            tax_due,
            withheld_tax: withheld,
            tax_payable,
            tax_accumulated,
            total_payable,
            is_exempt,
            calculation_date: calculation_date.clone(),
            status,
        });
    }

    println!("DEBUG: Generated {} appraisals (Profile)", appraisals.len());
    Ok(appraisals)
}

/// Lists saved appraisals
#[tauri::command]
pub async fn get_appraisals(db: State<'_, DbState>, year: Option<u16>) -> Result<Vec<TaxAppraisal>, String> {
    // PRE-MIGRATION: Robust multi-stage field renaming
    let migrations = vec![
        "UPDATE tax_appraisal SET period_month = `month`, period_year = `year`, `month` = NONE, `year` = NONE WHERE `month` != NONE",
        "UPDATE tax_appraisal SET period_month = appraisal_month, period_year = appraisal_year, appraisal_month = NONE, appraisal_year = NONE WHERE appraisal_month != NONE"
    ];
    for m in migrations {
        let _ = db.0.query(m).await;
    }

    let query = if let Some(y) = year {
        format!("SELECT *, type::string(id) as id FROM tax_appraisal WHERE period_year = {} OR status = 'Pending' ORDER BY period_year DESC, period_month DESC", y)
    } else {
        "SELECT *, type::string(id) as id FROM tax_appraisal ORDER BY period_year DESC, period_month DESC".to_string()
    };
    
    let mut result = db.0.query(query).await.map_err(|e| e.to_string())?;
    let appraisals: Vec<TaxAppraisal> = result.take(0).map_err(|e| e.to_string())?;
    
    Ok(appraisals)
}

/// Saves an appraisal (Upsert logic) and updates Tax Loss balance
#[tauri::command]
pub async fn save_appraisal(db: State<'_, DbState>, data: TaxAppraisal) -> Result<TaxAppraisal, String> {
    
    // --- LOSS COMPENSATION LOGIC ---
    // 1. If we are compensating a loss (profit > 0 and using accumulated loss)
    if data.compensated_loss > 0.0 {
        let mut remaining_compensation = data.compensated_loss;
        
        // Fetch accumulated losses for this trade type, oldest first
        let loss_query = "SELECT * FROM tax_loss WHERE balance > 0 AND trade_type = $type ORDER BY origin_date ASC";
        let mut loss_result = db.0.query(loss_query)
            .bind(("type", data.trade_type.clone()))
            .await
            .map_err(|e| e.to_string())?;
            
        let losses: Vec<TaxLoss> = loss_result.take(0usize).map_err(|e| e.to_string())?;
        
        for mut loss_record in losses {
            if remaining_compensation <= 0.0 {
                break;
            }
            
            let deduction = if loss_record.balance >= remaining_compensation {
                remaining_compensation
            } else {
                loss_record.balance
            };
            
            loss_record.balance -= deduction;
            remaining_compensation -= deduction;
            
            // Update the loss record (strip ID from content to avoid conflict)
            if let Some(loss_id) = &loss_record.id.clone() {
                let parts: Vec<&str> = loss_id.split(':').collect();
                let l_tb = if parts.len() > 1 { parts[0] } else { "tax_loss" };
                let l_id = if parts.len() > 1 { parts[1] } else { loss_id };
                loss_record.id = None;

                let _: Option<TaxLoss> = db.0
                    .update((l_tb, l_id))
                    .content(loss_record)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    
    // 2. If this month resulted in a NET loss, create a new TaxLoss record
    if data.net_profit < 0.0 {
        let loss_amount = data.net_profit.abs();
        let origin_date = format!("{}-{:02}-01", data.period_year, data.period_month); 
        // First day of the month as origin
        
        let check_query = "SELECT *, type::string(id) as id FROM tax_loss WHERE origin_date = $date AND trade_type = $type";
        let mut check_result = db.0.query(check_query)
            .bind(("date", origin_date.clone()))
            .bind(("type", data.trade_type.clone()))
            .await
            .map_err(|e| e.to_string())?;
            
        let existing_loss: Option<TaxLoss> = check_result.take(0).map_err(|e| e.to_string())?;
        
        if let Some(mut loss) = existing_loss {
            // Update existing loss record
            loss.amount = loss_amount; 
            loss.balance = loss_amount; // Reset balance to full new loss for this month
            
            if let Some(lid) = &loss.id.clone() {
                 let parts: Vec<&str> = lid.split(':').collect();
                 let l_tb = if parts.len() > 1 { parts[0] } else { "tax_loss" };
                 let l_id = if parts.len() > 1 { parts[1] } else { lid };
                 loss.id = None;

                 let _: Option<TaxLoss> = db.0
                    .update((l_tb, l_id))
                    .content(loss)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        } else {
            // Create new loss record
            let new_loss = TaxLoss {
                id: None,
                trade_type: data.trade_type.clone(),
                amount: data.loss,
                origin_date,
                balance: data.loss,
            };
            
            let _: Option<TaxLoss> = db.0
                .create("tax_loss")
                .content(new_loss)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    // --- END LOSS COMPENSATION LOGIC ---

    // 1. If ID is present, update existing
    if let Some(id) = &data.id {
        let parts: Vec<&str> = id.split(':').collect();
        let tb = if parts.len() > 1 { parts[0] } else { "tax_appraisal" };
        let tid = if parts.len() > 1 { parts[1] } else { id };

        let mut content = data.clone();
        content.id = None;

        println!("[IRPF] Saving Appraisal UPDATE: {}:{}", tb, tid);

        let updated: Option<TaxAppraisal> = db.0
            .update((tb, tid))
            .content(content)
            .await
            .map_err(|e| {
                println!("[IRPF] Error updating appraisal SDK: {}", e);
                e.to_string()
            })?;
            
        if updated.is_none() {
             println!("[IRPF] Warning: SDK update returned None for {}:{}", tb, tid);
             // Fallback to CREATE if UPDATE failed (Upsert behavior)
             let created: Option<TaxAppraisal> = db.0
                .create(tb)
                .content(data.clone())
                .await
                .map_err(|e| e.to_string())?;
             return created.ok_or_else(|| "Failed to upsert appraisal".to_string());
        }
        return Ok(updated.unwrap());
    }

    // 2. If no ID, check if one exists for this Month/Year/Type to avoid duplicates
    let query = "SELECT * FROM tax_appraisal WHERE period_month = $month AND period_year = $year AND trade_type = $type";
    let mut result = db.0.query(query)
        .bind(("month", data.period_month))
        .bind(("year", data.period_year))
        .bind(("type", data.trade_type.clone()))
        .await
        .map_err(|e| e.to_string())?;
        
    let existing: Option<TaxAppraisal> = result.take(0).map_err(|e| e.to_string())?;

    if let Some(existing_record) = existing {
        if let Some(id) = &existing_record.id {
             let parts: Vec<&str> = id.split(':').collect();
             let tb = if parts.len() > 1 { parts[0] } else { "tax_appraisal" };
             let tid = if parts.len() > 1 { parts[1] } else { id };

             println!("[IRPF] Updating EXISTING record found by month/year/type: {}:{}", tb, tid);

             let mut new_data = data.clone();
             new_data.id = None;

             let updated: Option<TaxAppraisal> = db.0
                .update((tb, tid))
                .content(new_data)
                .await
                .map_err(|e| e.to_string())?;
            return updated.ok_or_else(|| "Failed to update existing record".to_string());
        }
    }

    // 3. Create new if doesn't exist
    println!("[IRPF] Creating NEW appraisal record");
    let mut new_data = data.clone();
    new_data.id = None;
    
    let created: Option<TaxAppraisal> = db.0
        .create("tax_appraisal")
        .content(new_data)
        .await
        .map_err(|e| e.to_string())?;
    
    created.ok_or_else(|| "Failed to create new appraisal".to_string())
}

/// Fetches accumulated losses by type
#[tauri::command]
pub async fn get_accumulated_losses(db: State<'_, DbState>) -> Result<Vec<TaxLoss>, String> {
    let mut result = db.0
        .query("SELECT *, type::string(id) as id FROM tax_loss WHERE balance > 0 ORDER BY origin_date ASC")
        .await
        .map_err(|e| e.to_string())?;
    
    let losses: Vec<TaxLoss> = result.take(0).map_err(|e| e.to_string())?;
    
    Ok(losses)
}

/// Deletes an appraisal and its associated tax loss if applicable
#[tauri::command]
pub async fn delete_appraisal(db: State<'_, DbState>, id: String) -> Result<(), String> {
    
    // 1. Fetch the appraisal to check if it generated a loss
    let appraisal: Option<TaxAppraisal> = db.0.select(("tax_appraisal", &id))
        .await
        .map_err(|e| e.to_string())?;
        
    if let Some(appraisal) = appraisal {
        // 2. If it generated a loss, delete the corresponding TaxLoss record
        if appraisal.net_profit < 0.0 {
            let origin_date = format!("{}-{:02}-01", appraisal.period_year, appraisal.period_month);
            
            // Find the tax_loss record
            let query = "SELECT * FROM tax_loss WHERE origin_date = $date AND trade_type = $type";
            let mut result = db.0.query(query)
                .bind(("date", origin_date))
                .bind(("type", appraisal.trade_type.clone()))
                .await
                .map_err(|e| e.to_string())?;
                
            let loss_record: Option<TaxLoss> = result.take(0).map_err(|e| e.to_string())?;
            
            if let Some(loss) = loss_record {
                if let Some(lid) = loss.id {
                    let parts: Vec<&str> = lid.split(':').collect();
                    let l_tb = if parts.len() > 1 { parts[0] } else { "tax_loss" };
                    let l_id = if parts.len() > 1 { parts[1] } else { &lid };

                    let _: Option<TaxLoss> = db.0.delete((l_tb, l_id))
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        
        // 3. Delete associated DARFs to prevent orphans
        let darf_query = "SELECT *, type::string(id) as id FROM tax_darf WHERE appraisal_id = $id";
        let mut darf_result = db.0.query(darf_query)
            .bind(("id", id.clone()))
            .await
            .map_err(|e| e.to_string())?;
            
        let darfs: Vec<TaxDarf> = darf_result.take(0).map_err(|e| e.to_string())?;
        
        for darf in darfs {
            if let Some(did) = darf.id {
                 let parts: Vec<&str> = did.split(':').collect();
                 let d_tb = if parts.len() > 1 { parts[0] } else { "tax_darf" };
                 let d_id = if parts.len() > 1 { parts[1] } else { &did };

                 let _: Option<TaxDarf> = db.0.delete((d_tb, d_id))
                    .await
                    .map_err(|e: surrealdb::Error| e.to_string())?;
            }
        }

        // 4. Delete the appraisal
        let _: Option<TaxAppraisal> = db.0.delete(("tax_appraisal", &id)).await.map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// Generates a DARF from an appraisal
#[tauri::command]
pub async fn generate_darf(db: State<'_, DbState>, appraisal_id: String) -> Result<TaxDarf, String> {
    
    println!("DEBUG: generate_darf called for appraisal_id: {}", appraisal_id);
    
    // Fetch appraisal
    let appraisal_res: Option<TaxAppraisal> = db.0.select(("tax_appraisal", &appraisal_id))
        .await
        .map_err(|e| {
            println!("DEBUG: Failed to fetch appraisal: {}", e);
            e.to_string()
        })?;
        
    let appraisal = appraisal_res.ok_or("Appraisal not found")?;
    println!("DEBUG: Found appraisal for DARF: {:?}", appraisal);
        
    if appraisal.total_payable < 10.0 {
        return Err("Valor total (incluindo acumulados) inferior a R$ 10,00. Não é necessário gerar DARF ainda.".to_string());
    }

    // Define revenue code early for check
    let revenue_code = if appraisal.trade_type == "DayTrade" {
        "6015".to_string()
    } else {
        "3317".to_string()
    };
    
    let period = format!("{:02}/{}", appraisal.period_month, appraisal.period_year);

    println!("DEBUG: Checking existing DARFs by Period/Code...");
    // STRICT CHECK: One DARF per Period + Code (Prevents duplicates even if Appraisal ID changes)
    let check_query = "SELECT * FROM tax_darf WHERE period = $period AND revenue_code = $code";
    let mut check_result = db.0.query(check_query)
        .bind(("period", period.clone()))
        .bind(("code", revenue_code.clone()))
        .await
        .map_err(|e| {
            println!("DEBUG: Failed to check existing DARFs query: {}", e);
            e.to_string()
        })?;
        
    let existing_darfs: Vec<TaxDarf> = check_result.take(0).map_err(|e| {
        println!("DEBUG: Failed to deserialize existing DARFs: {}", e);
        e.to_string()
    })?;
    
    if !existing_darfs.is_empty() {
        println!("DEBUG: DARF already exists for this period/code");
        return Err("Uma DARF já existe para este período e código (seja paga ou pendente). Verifique a lista.".to_string());
    }

    // Define revenue code
    let revenue_code = if appraisal.trade_type == "DayTrade" {
        "6015".to_string()
    } else {
        "3317".to_string() // Variable Income (Swing Trade)
    };

    // Calculate due date: last business day of the following month
    let due_year = if appraisal.period_month == 12 { appraisal.period_year + 1 } else { appraisal.period_year };
    let due_month = if appraisal.period_month == 12 { 1 } else { appraisal.period_month + 1 };
    
    let due_date_obj = NaiveDate::from_ymd_opt(due_year as i32, due_month as u32, 1)
        .unwrap()
        .with_day(1)
        .unwrap()
        .checked_add_months(chrono::Months::new(1))
        .unwrap()
        .pred_opt()
        .unwrap(); // Last day of next month
      let period = format!("{:02}/{}", appraisal.period_month, appraisal.period_year);

    let darf = TaxDarf {
        id: None,
        appraisal_id,
        revenue_code: revenue_code.clone(),
        period,
        principal_value: appraisal.total_payable,
        fine: 0.0,
        interest: 0.0,
        total_value: appraisal.total_payable,
        due_date: due_date_obj.format("%Y-%m-%d").to_string(),
        payment_date: None,
        status: "Pending".to_string(),
        darf_number: None,
        account_id: None,
        transaction_id: None,
    };

    let created: Option<TaxDarf> = db.0
        .create("tax_darf")
        .content(darf)
        .await
        .map_err(|e| e.to_string())?;

    created.ok_or_else(|| "Failed to generate DARF".to_string())
}

/// Lists DARFs
#[tauri::command]
pub async fn get_darfs(db: State<'_, DbState>, year: Option<u16>) -> Result<Vec<TaxDarf>, String> {
    let query = if let Some(y) = year {
        // ALWAYS fetch Pending DARFs to ensure they don't disappear when filtering by history year
        format!("SELECT *, type::string(id) as id FROM tax_darf WHERE string::ends_with(period, '{}') OR status = 'Pending' ORDER BY due_date DESC", y)
    } else {
        "SELECT *, type::string(id) as id FROM tax_darf ORDER BY due_date DESC".to_string()
    };
    
    let mut result = db.0.query(query).await.map_err(|e| e.to_string())?;
    let darfs: Vec<TaxDarf> = result.take(0).map_err(|e| e.to_string())?;
    
    Ok(darfs)
}

/// Marks DARF as paid, updates account balance, and creates a transaction
#[tauri::command]
pub async fn mark_darf_paid(
    db: State<'_, DbState>, 
    id: String, 
    payment_date: String, 
    paid_value: f64, 
    fine: Option<f64>, 
    interest: Option<f64>,
    account_id: String, // New param
    transaction_id: String // New param
) -> Result<TaxDarf, String> {
    
    let darf_res: Option<TaxDarf> = db.0.select(("tax_darf", &id))
        .await
        .map_err(|e| e.to_string())?;
        
    let mut darf = darf_res.ok_or("DARF not found")?;
    
    // Prevent duplicate payments
    if darf.status == "Paid" {
        return Err("DARF is already marked as paid".to_string());
    }
        
    darf.payment_date = Some(payment_date.clone());
    darf.total_value = paid_value;
    darf.status = "Paid".to_string();
    
    if let Some(f) = fine {
        darf.fine = f;
    }
    
    if let Some(i) = interest {
        darf.interest = i;
    }

    darf.account_id = Some(account_id.clone());
    darf.transaction_id = Some(transaction_id.clone());

    // CRITICAL FIX: Remove ID from content to avoid "Found '...' for the 'id' field" error
    // We are updating the record *at* this ID, so we don't need to specify it in the content.
    let mut darf_content = darf.clone();
    darf_content.id = None;
    
    // --- UPDATE RECORDS FIRST ---
    // 1. Update DARF record
    let update_query = "UPDATE type::thing('tax_darf', $id) MERGE $content";
    let mut update_result = db.0.query(update_query)
        .bind(("id", id.clone())) 
        .bind(("content", darf_content))
        .await
        .map_err(|e| e.to_string())?;
        
    let updated: Option<TaxDarf> = update_result.take(0).map_err(|e| e.to_string())?;

    // 2. Update Appraisal Record
    let appraisal_id = darf.appraisal_id.clone();
    let parts: Vec<&str> = appraisal_id.split(':').collect();
    let aid_val = if parts.len() > 1 { parts[1] } else { &appraisal_id };
    
    let appraisal_result: Result<Option<TaxAppraisal>, _> = db.0.select(("tax_appraisal", aid_val)).await;
    
    if let Ok(Some(mut appraisal)) = appraisal_result {
        appraisal.status = "Paid".to_string();
        
        if let Some(aid) = &appraisal.id {
             let a_parts: Vec<&str> = aid.split(':').collect();
             let a_tb = if a_parts.len() > 1 { a_parts[0] } else { "tax_appraisal" };
             let a_id = if a_parts.len() > 1 { a_parts[1] } else { aid };

             // CRITICAL FIX: Strip ID to prevent SurrealDB error
             let mut appraisal_content = appraisal.clone();
             appraisal_content.id = None;

             let _: Option<TaxAppraisal> = db.0
                .update((a_tb, a_id))
                .content(appraisal_content)
                .await
                .map_err(|e| e.to_string())?;

             // 3. Mark all PREVIOUS PENDING appraisals of the same type as PAID (Accumulation Clear)
             let trade_type_inner = appraisal.trade_type.clone();
             let clear_query = "UPDATE tax_appraisal SET status = 'Paid' WHERE trade_type = $type AND status = 'Pending' AND (period_year < $year OR (period_year = $year AND period_month < $month))";
             
             let _: surrealdb::Response = db.0.query(clear_query)
                .bind(("type", trade_type_inner))
                .bind(("year", appraisal.period_year))
                .bind(("month", appraisal.period_month))
                .await.map_err(|e| e.to_string())?;
        }
    }

    // --- FINANCIAL INTEGRATION (SIDE EFFECTS) ---
    // 1. Update Account Balance (atomic decrement)
    let acc_parts: Vec<String> = account_id.split(':').map(|s| s.to_string()).collect();
    let acc_tb = if acc_parts.len() > 1 { acc_parts[0].clone() } else { "account".to_string() };
    let acc_id_only = if acc_parts.len() > 1 { acc_parts[1].clone() } else { account_id.clone() };

    println!("[DARF] Deducting {} from account:{}", paid_value, acc_id_only);
    let balance_query = "UPDATE type::thing($tb, $id) SET balance -= $val";

    match db.0.query(balance_query)
        .bind(("tb", acc_tb.clone()))
        .bind(("id", acc_id_only.clone()))
        .bind(("val", paid_value))
        .await {
            Ok(_) => println!("[DARF] Account balance updated successfully"),
            Err(e) => println!("[DARF] ERROR updating account balance: {}", e),
        }

    // 2. Create Cash Transaction (Expense) via bound query for Record IDs
    let tx_parts: Vec<String> = transaction_id.split(':').map(|s| s.to_string()).collect();
    let tx_tb = if tx_parts.len() > 1 { tx_parts[0].clone() } else { "cash_transaction".to_string() };
    let tx_id_only = if tx_parts.len() > 1 { tx_parts[1].clone() } else { transaction_id.clone() };

    println!("[DARF] Creating cash_transaction:{}:{} for account:{}", tx_tb, tx_id_only, acc_id_only);
    let description = format!("Pagamento DARF {} ({})", darf.period, darf.revenue_code);
    
    let tx_query = "CREATE type::thing($tx_tb, $tx_id) SET 
        date = $date, 
        amount = $amount, 
        type = 'Withdraw', 
        description = $desc, 
        account_id = type::thing($acc_tb, $acc_id)";
    
    match db.0.query(tx_query)
        .bind(("tx_tb", tx_tb))
        .bind(("tx_id", tx_id_only))
        .bind(("date", payment_date))
        .bind(("amount", 0.0 - paid_value))
        .bind(("desc", description))
        .bind(("acc_tb", acc_tb))
        .bind(("acc_id", acc_id_only))
        .await {
            Ok(_) => println!("[DARF] Cash transaction created successfully"),
            Err(e) => println!("[DARF] ERROR creating cash transaction: {}", e),
        }

    // --- END FINANCIAL INTEGRATION ---

    updated.ok_or_else(|| "Failed to update DARF".to_string())
}

#[tauri::command]
pub async fn get_darf_by_transaction(db: State<'_, DbState>, transaction_id: String) -> Result<Option<TaxDarf>, String> {
    let query = "SELECT * FROM tax_darf WHERE transaction_id = $tid";
    let mut result = db.0.query(query)
        .bind(("tid", transaction_id))
        .await
        .map_err(|e| e.to_string())?;

    let darf: Option<TaxDarf> = result.take(0).map_err(|e| e.to_string())?;
    Ok(darf)
}

/// Reverses a DARF payment (Unpay)
#[tauri::command]
pub async fn unpay_darf(db: State<'_, DbState>, id: String) -> Result<TaxDarf, String> {
    
    let darf_res: Option<TaxDarf> = db.0.select(("tax_darf", &id))
        .await
        .map_err(|e| e.to_string())?;
        
    let mut darf = darf_res.ok_or("DARF not found")?;
    
    if darf.status != "Paid" {
        return Err("DARF is not paid".to_string());
    }
    
    let paid_value = darf.total_value;
    
    // 1. Reverse Financial Transaction if linked
    let linked_acc_id = darf.account_id.clone();
    let linked_trans_id = darf.transaction_id.clone();

    if let (Some(acc_id), Some(trans_id)) = (linked_acc_id, linked_trans_id) {
        
        // Refund Account Balance (atomic)
        let acc_parts: Vec<String> = acc_id.split(':').map(|s| s.to_string()).collect();
        let acc_tb = if acc_parts.len() > 1 { acc_parts[0].clone() } else { "account".to_string() };
        let acc_id_val = if acc_parts.len() > 1 { acc_parts[1].clone() } else { acc_id.clone() };
        
        println!("[DARF UNPAY] Refunding {} to {}:{}", paid_value, acc_tb, acc_id_val);
        let refund_query = "UPDATE type::thing($tb, $id) SET balance += $val";
        match db.0.query(refund_query)
            .bind(("tb", acc_tb))
            .bind(("id", acc_id_val))
            .bind(("val", paid_value))
            .await {
                Ok(_) => println!("[DARF UNPAY] Account balance refunded successfully"),
                Err(e) => println!("[DARF UNPAY] ERROR refunding account balance: {}", e),
            }
        
        // Delete Transaction (atomic)
        let t_parts: Vec<String> = trans_id.split(':').map(|s| s.to_string()).collect();
        let t_tb = if t_parts.len() > 1 { t_parts[0].clone() } else { "cash_transaction".to_string() };
        let t_id_val = if t_parts.len() > 1 { t_parts[1].clone() } else { trans_id.clone() };
        
        println!("[DARF UNPAY] Deleting {}:{}", t_tb, t_id_val);
        let del_query = "DELETE type::thing($tb, $id)";
        match db.0.query(del_query)
            .bind(("tb", t_tb))
            .bind(("id", t_id_val))
            .await {
                Ok(_) => println!("[DARF UNPAY] Transaction deleted successfully"),
                Err(e) => println!("[DARF UNPAY] ERROR deleting transaction: {}", e),
            }
    }
    
    // 2. Reset DARF Status
    darf.status = "Pending".to_string();
    darf.payment_date = None;
    darf.total_value = darf.principal_value; // Reset to principal (remove fine/interest from total)
    darf.fine = 0.0;
    darf.interest = 0.0;
    darf.account_id = None;
    darf.transaction_id = None;
    
    // CRITICAL FIX: Strip ID
    let mut darf_content = darf.clone();
    darf_content.id = None;

    let updated: Option<TaxDarf> = db.0
        .update(("tax_darf", id.clone()))
        .content(darf_content)
        .await
        .map_err(|e| e.to_string())?;
        
    // 3. Update Appraisal Status
    let appraisal_id = darf.appraisal_id.clone();
    let parts: Vec<&str> = appraisal_id.split(':').collect();
    let aid_val = if parts.len() > 1 { parts[1] } else { &appraisal_id };
    
    let appraisal_result: Result<Option<TaxAppraisal>, _> = db.0.select(("tax_appraisal", aid_val)).await;
    
    if let Ok(Some(mut appraisal)) = appraisal_result {
        appraisal.status = "Pending".to_string();
        if let Some(aid) = &appraisal.id {
             let a_parts: Vec<&str> = aid.split(':').collect();
             let a_tb = if a_parts.len() > 1 { a_parts[0] } else { "tax_appraisal" };
             let a_id = if a_parts.len() > 1 { a_parts[1] } else { aid };

             // CRITICAL FIX: Strip ID
             let mut appraisal_content = appraisal.clone();
             appraisal_content.id = None;

             let _: Option<TaxAppraisal> = db.0
                .update((a_tb, a_id))
                .content(appraisal_content)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    
    updated.ok_or_else(|| "Failed to unpay DARF".to_string())
}

/// Deletes a DARF (blocks if paid - must use unpay_darf first)
#[tauri::command]
pub async fn delete_darf(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    
    // Check if DARF is paid - block deletion
    let darf_res: Option<TaxDarf> = db.0.select(("tax_darf", clean_id))
        .await
        .map_err(|e| e.to_string())?;
    
    if let Some(darf) = darf_res {
        if darf.status == "Paid" {
            return Err("Não é possível excluir uma DARF paga. Use 'Desfazer Pagamento' primeiro para estornar o valor.".to_string());
        }
    }
    
    let query = format!("DELETE tax_darf:{}", clean_id);
    db.0.query(&query).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Calculates interest and fine for late payment
#[tauri::command]
pub async fn calculate_interest_fine(value: f64, due_date: String, payment_date: String) -> Result<(f64, f64), String> {
    // Simplified implementation (Brazilian Federal Revenue Rule)
    // Fine: 0.33% per day of delay, capped at 20%
    // Interest: Accumulated SELIC + 1% in the month of payment
    
    let due = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d").map_err(|_| "Invalid date")?;
    let pay = NaiveDate::parse_from_str(&payment_date, "%Y-%m-%d").map_err(|_| "Invalid date")?;
    
    if pay <= due {
        return Ok((0.0, 0.0));
    }
    
    let days_delay = (pay - due).num_days();
    let fine_perc = (days_delay as f64 * 0.0033).min(0.20);
    let fine = value * fine_perc;
    
    // Simplified interest (1% per month as placeholder for SELIC)
    let months_delay = days_delay / 30;
    let interest_perc = (months_delay as f64 * 0.01) + 0.01;
    let interest = value * interest_perc;
    
    Ok((fine, interest))
}

/// Resets all IRPF data (Appraisals, DARFs, Losses)
#[tauri::command]
pub async fn reset_irpf_data(db: State<'_, DbState>) -> Result<(), String> {
    
    db.0.query("DELETE FROM tax_appraisal").await.map_err(|e| e.to_string())?;
    db.0.query("DELETE FROM tax_darf").await.map_err(|e| e.to_string())?;
    db.0.query("DELETE FROM tax_loss").await.map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_tax_loss(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let _: Option<TaxLoss> = db.0.delete(("tax_loss", &id)).await.map_err(|e| e.to_string())?;
    Ok(())
}

// --- TAX RULES & MAPPINGS (New) ---

#[tauri::command]
pub async fn get_tax_rules(db: State<'_, DbState>) -> Result<Vec<TaxRule>, String> {
    println!("[IRPF] Fetching tax_rules...");
    let mut result = match db.0.query("SELECT *, type::string(id) as id FROM tax_rule").await {
        Ok(r) => r,
        Err(e) => {
            println!("[IRPF_ERROR] Query execution failed: {}", e);
            return Err(e.to_string());
        }
    };
    
    let rules: Vec<TaxRule> = match result.take(0) {
        Ok(r) => r,
        Err(e) => {
            println!("[IRPF_ERROR] Failed to map rules: {}", e);
            return Err(e.to_string());
        }
    };
    
    println!("[IRPF] Fetched {} tax rules.", rules.len());
    Ok(rules)
}

#[tauri::command]
pub async fn save_tax_rule(db: State<'_, DbState>, rule: TaxRule) -> Result<TaxRule, String> {
    println!("[IRPF] Saving tax_rule: {:?}", rule);
    let id = rule.id.clone();
    let mut json = serde_json::to_value(&rule).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    
    let clean_id = id.split(':').last().unwrap_or(&id);

    // Using Native Rust SDK for guaranteed format parsing
    let result: Option<TaxRule> = match db.0.upsert(("tax_rule", clean_id))
        .content(json)
        .await {
            Ok(r) => r,
            Err(e) => {
                println!("[IRPF_ERROR] UPSERT Failed on tax_rule: {}", e);
                return Err(e.to_string());
            }
        };
        
    result.ok_or_else(|| "Failed to save tax rule".to_string())
}

#[tauri::command]
pub async fn delete_tax_rule(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    let query = format!("DELETE tax_rule:{}", clean_id);
    db.0.query(query).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_tax_mappings(db: State<'_, DbState>, ) -> Result<Vec<TaxMapping>, String> {
    let mut result = db.0.query("SELECT *, type::string(id) as id, type::string(asset_type_id) as asset_type_id, type::string(modality_id) as modality_id, type::string(tax_rule_id) as tax_rule_id FROM tax_mapping").await.map_err(|e| e.to_string())?;
    let mappings: Vec<TaxMapping> = result.take(0).map_err(|e| e.to_string())?;
    Ok(mappings)
}

#[tauri::command]
pub async fn save_tax_mapping(db: State<'_, DbState>, mapping: TaxMapping) -> Result<TaxMapping, String> {
    let id = mapping.id.clone();
    let mut json = serde_json::to_value(&mapping).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    
    let clean_id = id.split(':').last().unwrap_or(&id);
    
    let result: Option<TaxMapping> = db.0.upsert(("tax_mapping", clean_id))
        .content(json)
        .await
        .map_err(|e| e.to_string())?;
        
    result.ok_or_else(|| "Failed to save tax mapping".to_string())
}

#[tauri::command]
pub async fn delete_tax_mapping(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    let _r: Option<TaxMapping> = db.0.delete(("tax_mapping", clean_id))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// --- TAX PROFILES & ENTRIES (New Option B) ---

#[tauri::command]
pub async fn get_tax_profiles(db: State<'_, DbState>) -> Result<Vec<TaxProfile>, String> {
    let mut result = db.0.query("SELECT *, type::string(id) as id FROM tax_profile").await.map_err(|e| e.to_string())?;
    let profiles: Vec<TaxProfile> = result.take(0).map_err(|e| e.to_string())?;
    Ok(profiles)
}

#[tauri::command]
pub async fn save_tax_profile(db: State<'_, DbState>, profile: TaxProfile) -> Result<TaxProfile, String> {
    let id = profile.id.clone();
    let mut json = serde_json::to_value(&profile).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    
    let clean_id = id.split(':').last().unwrap_or(&id);

    let result: Option<TaxProfile> = db.0.upsert(("tax_profile", clean_id))
        .content(json)
        .await
        .map_err(|e| e.to_string())?;
        
    result.ok_or_else(|| "Failed to save tax profile".to_string())
}

#[tauri::command]
pub async fn delete_tax_profile(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    let query_p = format!("DELETE tax_profile:{}", clean_id);
    db.0.query(query_p).await.map_err(|e| e.to_string())?;

    let query_e = format!("DELETE tax_profile_entry WHERE tax_profile_id = 'tax_profile:{}' OR tax_profile_id = '{}'", clean_id, clean_id);
    let _ = db.0.query(query_e).await;
    Ok(())
}


#[tauri::command]
pub async fn get_tax_profile_entries(db: State<'_, DbState>, profile_id: Option<String>) -> Result<Vec<TaxProfileEntry>, String> {
    // Fallsback to format! to resolve persistent lifetime issues with bind in this async context
    let query = if let Some(pid) = profile_id {
        format!("SELECT *, type::string(id) as id, type::string(tax_profile_id) as tax_profile_id, type::string(modality_id) as modality_id, type::string(tax_rule_id) as tax_rule_id FROM tax_profile_entry WHERE tax_profile_id = '{}'", pid)
    } else {
        "SELECT *, type::string(id) as id, type::string(tax_profile_id) as tax_profile_id, type::string(modality_id) as modality_id, type::string(tax_rule_id) as tax_rule_id FROM tax_profile_entry".to_string()
    };
    
    let mut result = db.0.query(query)
                .await.map_err(|e| e.to_string())?;
    
    let entries: Vec<TaxProfileEntry> = result.take(0).map_err(|e| e.to_string())?;
    Ok(entries)
}

#[tauri::command]
pub async fn save_tax_profile_entry(db: State<'_, DbState>, entry: TaxProfileEntry) -> Result<TaxProfileEntry, String> {
    let id = entry.id.clone();
    let mut json = serde_json::to_value(&entry).map_err(|e| e.to_string())?;
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    
    let clean_id = id.split(':').last().unwrap_or(&id);

    let result: Option<TaxProfileEntry> = db.0.upsert(("tax_profile_entry", clean_id))
        .content(json)
        .await
        .map_err(|e| e.to_string())?;
        
    result.ok_or_else(|| "Failed to save tax profile entry".to_string())
}

#[tauri::command]
pub async fn delete_tax_profile_entry(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    let query = format!("DELETE tax_profile_entry:{}", clean_id);
    db.0.query(query).await.map_err(|e| e.to_string())?;
    Ok(())
}
