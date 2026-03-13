use crate::db::DbState;
use std::collections::HashMap;
use crate::logic::RuleBucket;
use crate::models::irpf::{TaxAppraisal, TaxDarf, TaxLoss};
use crate::models::{Asset, AssetType, TaxMapping, TaxProfile, TaxProfileEntry, TaxRule, SurrealJson};
use chrono::{Datelike, NaiveDate};
use serde_json::{self, Value};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use crate::services::position_service::{PositionService, Position};
use tauri::State;



#[tauri::command]
pub async fn calculate_monthly_tax(db: State<'_, DbState>, month: u8, year: u16) -> Result<Vec<crate::models::dto::TaxAppraisalDto>, String> {
    println!(">>> ENTRY: calculate_monthly_tax(month={}, year={})", month, year);

    // 0. Load reference data first so we can use it during normalization
    let mut asset_res = db.0.query("SELECT id, symbol, name, asset_type_id, type::float(point_value) as point_value, default_fee_id, tax_profile_id FROM asset").await.map_err(|e| e.to_string())?;
    let assets_json: Vec<SurrealJson> = asset_res.take(0).map_err(|e| e.to_string())?;
    let assets: Vec<Asset> = assets_json.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();

    let mut at_res = db.0.query("SELECT id, name, code, market_id, unit_label, result_type, default_fee_id, tax_profile_id FROM asset_type").await.map_err(|e| e.to_string())?;
    let at_json: Vec<SurrealJson> = at_res.take(0).map_err(|e| e.to_string())?;
    let mut asset_types: Vec<AssetType> = at_json.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();

    // Auto-heal missing tax profiles for standard asset types
    for at in asset_types.iter_mut() {
        if at.tax_profile_id.is_none() {
            let id_str = at.id.as_deref().unwrap_or("");
            let clean = id_str.split(':').last().unwrap_or(id_str);
            if clean == "at1" { at.tax_profile_id = Some("tax_profile:tp_acoes".to_string()); }
            else if clean == "at2" { at.tax_profile_id = Some("tax_profile:tp_futuros".to_string()); }
        }
    }

    // 1. Fetch ALL trades for the assets involved in this month to calculate accurate PM
    let month_end = if month == 12 { format!("{}-12-31", year) } else { format!("{}-{:02}-01", year, month + 1) };
    
    // First, find which assets have activity in the period
    let period_prefix = format!("{}-{:02}-", year, month);
    // DEBUG: Check some exit_date values
    if let Ok(mut debug_res) = db.0.query("SELECT exit_date, type::string(exit_date) as s_date FROM trade WHERE exit_date != NONE LIMIT 5").await {
        let debug_json: Vec<SurrealJson> = debug_res.take(0).unwrap_or_default();
        println!("[IRPF] DEBUG: Sample exit_dates: {:?}", debug_json);
    }

    let mut symbols_res = db.0.query("SELECT asset_symbol FROM trade WHERE exit_date != NONE AND (IF type::is::datetime(exit_date) THEN string::starts_with(type::string(exit_date), $prefix) ELSE string::starts_with(exit_date, $prefix) END) GROUP BY asset_symbol")
        .bind(("prefix", period_prefix.clone()))
        .await
        .map_err(|e| e.to_string())?;
    
    let symbols: Vec<String> = symbols_res.take::<Vec<SurrealJson>>(0)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|sj| sj.0["asset_symbol"].as_str().map(|s| s.to_string()))
        .collect();

    if symbols.is_empty() {
        println!("[IRPF] No asset activity for period {}.", period_prefix);
        return Ok(vec![]);
    }

    // Now fetch full history for these assets up to the end of the month
    let mut history_res = db.0.query("SELECT * FROM trade WHERE asset_symbol INSIDE $symbols AND date < $end ORDER BY date ASC")
        .bind(("symbols", symbols))
        .bind(("end", month_end))
        .await
        .map_err(|e| e.to_string())?;
    
    let raw_history: Vec<SurrealJson> = history_res.take(0).map_err(|e| e.to_string())?;
    let all_history: Vec<crate::models::Trade> = raw_history.into_iter()
        .filter_map(|sj| serde_json::from_value(sj.0).ok())
        .collect();

    println!("[IRPF] Loaded {} history trades for calculation.", all_history.len());

    // 2. Identify which trades are already in Paid/Ok appraisals
    let exist_query = "SELECT *, id FROM tax_appraisal WHERE period_month = $month AND period_year = $year";
    let mut exist_res = db.0.query(exist_query).bind(("month", month)).bind(("year", year)).await.map_err(|e| e.to_string())?;
    let existing_raw: Vec<SurrealJson> = exist_res.take(0).map_err(|e| e.to_string())?;
    let existing_appraisals: Vec<TaxAppraisal> = existing_raw.into_iter()
        .filter_map(|sj| serde_json::from_value(sj.0).ok())
        .collect();

    let mut already_appraised_ids = std::collections::HashSet::new();
    for app in &existing_appraisals {
        if app.status == "Paid" || app.status == "Ok" {
            for tid in &app.trade_ids {
                already_appraised_ids.insert(tid.clone());
            }
        }
    }

    if all_history.is_empty() {
        println!("[IRPF] No trade history for period {}/{}.", month, year);
        return Ok(vec![]);
    }

    let mut entry_res = db.0.query("SELECT id, tax_profile_id, modality_id, tax_rule_id FROM tax_profile_entry").await.map_err(|e| e.to_string())?;
    let entries_json: Vec<SurrealJson> = entry_res.take(0).map_err(|e| e.to_string())?;
    let profile_entries: Vec<TaxProfileEntry> = entries_json.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();

    let mut rule_res = db.0.query("SELECT id, name, trade_type, revenue_code, tax_rate, withholding_rate, withholding_basis, exemption_threshold, cumulative_losses, basis FROM tax_rule").await.map_err(|e| e.to_string())?;
    let rules_json: Vec<SurrealJson> = rule_res.take(0).map_err(|e| e.to_string())?;
    let rules: Vec<TaxRule> = rules_json.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();

    let mut loss_res = db.0.query("SELECT id, trade_type, amount, balance, origin_date FROM tax_loss WHERE balance > 0").await.map_err(|e| e.to_string())?;
    let losses_json: Vec<SurrealJson> = loss_res.take(0).map_err(|e| e.to_string())?;
    let losses: Vec<TaxLoss> = losses_json.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();

    println!("[IRPF] Loaded: {} assets, {} types, {} profile_entries, {} rules", assets.len(), asset_types.len(), profile_entries.len(), rules.len());

    // 6. Normalize IDs from SurrealDB format
    let normalize_id = |label: &str, s: &str| -> String {
        let mut clean = s.to_string();
        if clean.contains("String:") { clean = clean.split("String:").last().unwrap_or(&clean).to_string(); }
        clean = clean.replace(['{', '}', '"', '\'', ' ', '`'], "");
        let res = clean.split(':').last().unwrap_or(clean.as_str()).to_string();
        if !s.is_empty() { println!("[IRPF] normalize_id({}): '{}' -> '{}'", label, s, res); }
        res
    };

    // 7. Find tax rule for (profile, modality) pair
    let find_rule = |profile_id: &str, target_mod_id: &str| -> Option<TaxRule> {
        let clean_pid = normalize_id("ProfileParam", profile_id);
        let clean_mid = normalize_id("ModalityParam", target_mod_id);
        let entry = profile_entries.iter().find(|e| {
            normalize_id("EntryProfile", e.tax_profile_id.as_deref().unwrap_or("")) == clean_pid &&
            normalize_id("EntryModality", e.modality_id.as_deref().unwrap_or("")) == clean_mid
        });
        if let Some(e) = entry {
            let clean_rid = normalize_id("EntryRule", e.tax_rule_id.as_deref().unwrap_or(""));
            rules.iter().find(|r| normalize_id("RuleId", r.id.as_deref().unwrap_or("")) == clean_rid).cloned()
        } else {
            println!("[IRPF] No entry for profile={} modality={}", clean_pid, clean_mid);
            None
        }
    };

    // 8. Group trades into rule buckets
    // 8. Group trades into rule buckets using PM from PositionService
    let mut positions: HashMap<String, Position> = HashMap::new();
    let mut buckets: HashMap<String, RuleBucket> = HashMap::new();
    let period_prefix = format!("{}-{:02}-", year, month);

    for trade in all_history {
        let symbol = trade.asset_symbol.clone();
        let current_pm = positions.get(&symbol).map(|p| p.average_price).unwrap_or(0.0);

        // Check if this trade belongs to the target period for tax calculation
        let is_in_period = trade.exit_date.as_ref().map(|d| d.starts_with(&period_prefix)).unwrap_or(false);
        let is_not_appraised = !already_appraised_ids.contains(&trade.id);

        if is_in_period && is_not_appraised {
            // Find modality (DayTrade vs SwingTrade)
            let mut trade_mod_id = trade.modality_id.as_deref().unwrap_or("").to_string();
            if trade_mod_id.is_empty() || trade_mod_id == "null" {
                let ed = trade.date.split('T').next().unwrap_or("");
                let xd = trade.exit_date.as_deref().unwrap_or("").split('T').next().unwrap_or("");
                trade_mod_id = if !ed.is_empty() && ed == xd { "mod1".to_string() } else { "mod2".to_string() };
            }

            // Calculate ACTUAL result using PM (if SwingTrade) or Entry (if DayTrade)
            let basis = if trade_mod_id == "mod1" { trade.entry_price } else { current_pm };
            let real_result = PositionService::calculate_trade_result(&trade, basis);
            
            println!("[IRPF] Trade {} result calculated: basis={}, result={}", trade.id, basis, real_result);

            // Find tax profile
            let mut final_profile_id: Option<String> = None;
            if let Some(aid) = &trade.asset_id {
                let clean_aid = normalize_id("AssetId", aid);
                if let Some(asset) = assets.iter().find(|a| normalize_id("AssetTblId", a.id.as_deref().unwrap_or("")) == clean_aid || a.symbol == symbol) {
                    if let Some(pid) = &asset.tax_profile_id { final_profile_id = Some(pid.clone()); }
                }
            }
            if final_profile_id.is_none() {
                if let Some(atid) = &trade.asset_type_id {
                    let clean_atid = normalize_id("AssetTypeId", atid);
                    if let Some(at) = asset_types.iter().find(|t| normalize_id("AssetTypeTblId", t.id.as_deref().unwrap_or("")) == clean_atid) {
                        final_profile_id = at.tax_profile_id.clone();
                    }
                }
            }

            if let Some(profile_id) = final_profile_id {
                if let Some(rule) = find_rule(&profile_id, &trade_mod_id) {
                    let bucket_key = format!("{}:{}", rule.id.as_deref().unwrap_or(""), trade_mod_id);
                    let entry = buckets.entry(bucket_key).or_insert(RuleBucket {
                        rule: rule.clone(),
                        gross_profit: 0.0,
                        gross_loss: 0.0,
                        sales_total: 0.0,
                        trade_ids: Vec::new(),
                    });

                    if real_result > 0.0 { entry.gross_profit += real_result; }
                    else { entry.gross_loss += real_result.abs(); }

                    if rule.trade_type != "DayTrade" {
                        let point_value = trade.asset_id.as_ref().and_then(|aid| {
                            let clean_aid = normalize_id("AssetId", aid);
                            assets.iter().find(|a| normalize_id("AssetTblId", a.id.as_deref().unwrap_or("")) == clean_aid).map(|a| a.point_value)
                        }).unwrap_or(1.0);
                        
                        if let Some(price) = trade.exit_price {
                             entry.sales_total += price * trade.quantity * point_value;
                        }
                    }
                    entry.trade_ids.push(trade.id.clone());
                }
            }
        }

        // Always update PM for the next trades, UNLESS it's a DayTrade
        // DayTrades don't change the PM of the SwingTrade position.
        let is_daytrade = trade.modality_id.as_deref().map(|m| m == "mod1").unwrap_or(false);
        if !is_daytrade {
            let updated_pos = PositionService::calculate_positions(&[trade]);
            if let Some(up) = updated_pos.get(&symbol) {
                positions.insert(symbol, up.clone());
            }
        } else {
            println!("[IRPF] Skipping PM update for DayTrade: {}", trade.id);
        }
    }

    println!("[IRPF] {} buckets built.", buckets.len());

    // 9. Calculate appraisal per bucket
    let mut appraisals = Vec::new();
    let current_month_start = format!("{}-{:02}-01", year, month);

    for (rule_id, bucket) in buckets {
        let loss_category = bucket.rule.trade_type.clone();
        let norm_cat = loss_category.replace(' ', "").to_lowercase();

        let available_loss: f64 = losses.iter()
            .filter(|l| l.trade_type.replace(' ', "").to_lowercase() == norm_cat && l.origin_date < current_month_start)
            .map(|l| l.balance).sum();

        // Previous IRRF credit
        let mut credit_res = db.0.query("SELECT type::float(withholding_credit_remaining) as withholding_credit_remaining, period_year, period_month FROM tax_appraisal WHERE trade_type = $type")
            .bind(("type", loss_category.clone())).await.map_err(|e| e.to_string())?;
        let credit_json: Vec<SurrealJson> = credit_res.take(0).map_err(|e| e.to_string())?;
        let mut prev_credit_vec: Vec<serde_json::Value> = credit_json.into_iter().map(|sj| sj.0).collect();
        prev_credit_vec.retain(|v| {
            let py = v.get("period_year").and_then(|x| x.as_u64()).unwrap_or(0) as u16;
            let pm = v.get("period_month").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
            py < year || (py == year && pm <= month)
        });
        prev_credit_vec.sort_by(|a, b| {
            let (ay, am) = (a.get("period_year").and_then(|x| x.as_u64()).unwrap_or(0), a.get("period_month").and_then(|x| x.as_u64()).unwrap_or(0));
            let (by, bm) = (b.get("period_year").and_then(|x| x.as_u64()).unwrap_or(0), b.get("period_month").and_then(|x| x.as_u64()).unwrap_or(0));
            (by, bm).cmp(&(ay, am))
        });
        let previous_credit = prev_credit_vec.first().and_then(|v| v.get("withholding_credit_remaining")).and_then(|t| t.as_f64()).unwrap_or(0.0);

        // Accumulated sub-R$10 appraisals
        let mut accum_res = db.0.query("SELECT type::float(total_payable) as total_payable, period_year, period_month FROM tax_appraisal WHERE trade_type = $type AND status = 'Pending'")
            .bind(("type", loss_category.clone())).await.map_err(|e| e.to_string())?;
        let accum_json: Vec<SurrealJson> = accum_res.take(0).map_err(|e| e.to_string())?;
        let mut prev_accum: Vec<serde_json::Value> = accum_json.into_iter().map(|sj| sj.0).collect();
        prev_accum.retain(|v| {
            let py = v.get("period_year").and_then(|x| x.as_u64()).unwrap_or(0) as u16;
            let pm = v.get("period_month").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
            let tp = v.get("total_payable").and_then(|x| x.as_f64()).unwrap_or(0.0);
            tp < 10.0 && (py < year || (py == year && pm <= month))
        });
        prev_accum.sort_by(|a, b| {
            let (ay, am) = (a.get("period_year").and_then(|x| x.as_u64()).unwrap_or(0), a.get("period_month").and_then(|x| x.as_u64()).unwrap_or(0));
            let (by, bm) = (b.get("period_year").and_then(|x| x.as_u64()).unwrap_or(0), b.get("period_month").and_then(|x| x.as_u64()).unwrap_or(0));
            (by, bm).cmp(&(ay, am))
        });
        let tax_accumulated = prev_accum.first().and_then(|v| v.get("total_payable")).and_then(|t| t.as_f64()).unwrap_or(0.0);

        // Complementary check
        let mut paid_res = db.0.query("SELECT type::float(total_payable) as total_payable FROM tax_appraisal WHERE period_month = $month AND period_year = $year AND trade_type = $type AND status = 'Paid'")
            .bind(("month", month)).bind(("year", year)).bind(("type", loss_category.clone())).await.map_err(|e| e.to_string())?;
        let paid_json: Vec<SurrealJson> = paid_res.take(0).map_err(|e| e.to_string())?;
        let already_paid: f64 = paid_json.into_iter().map(|sj| sj.0)
            .map(|v| v.get("total_payable").and_then(|t| t.as_f64()).unwrap_or(0.0)).sum();

        let mut appraisal = crate::logic::calculate_appraisal(&bucket, month, year, available_loss, previous_credit, tax_accumulated, 0.0);
        appraisal.tax_rule_id = Some(rule_id);
        appraisal.calculation_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        appraisal.is_complementary = already_paid > 0.0;
        appraisals.push(appraisal);
    }

    println!("DEBUG: Generated {} appraisals", appraisals.len());
    println!("Result: {:?}", appraisals);
    Ok(appraisals.into_iter().map(|a| crate::models::ToDto::to_dto(&a)).collect())
}

/// Robust helper to find a single record by field or ID
pub async fn find_one_robust<T>(
    db: &Surreal<Db>,
    table: &str,
    field: &str,
    value: &str,
) -> Result<Option<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    // Normalize value (remove backticks that sometimes appear in SurrealDB 2.x stringified IDs)
    let clean_value = value.replace('`', "");
    
    // Strategy 1: Direct ID match if field is "id"
    if field == "id" {
        let tid = if clean_value.contains(':') {
            clean_value.split(':').last().unwrap_or(&clean_value)
        } else {
            &clean_value
        };
        let res: Option<SurrealJson> = db.select((table, tid)).await.map_err(|e| e.to_string())?;
        if let Some(sj) = res {
            return serde_json::from_value(sj.0).map_err(|e| {
                println!("[ERROR] find_one_robust deserialization failed for {}: {}. Error: {}", table, clean_value, e);
                format!("Erro ao processar dados de {}: {}", table, e)
            });
        }
    }

    // Strategy 2: Query by field (handles both string and thing types in DB)
    let q = format!("SELECT * FROM {} WHERE type::string({}) = $val OR {} = type::thing($val)", table, field, field);
    let mut result = db.query(q).bind(("val", clean_value.clone())).await.map_err(|e| e.to_string())?;
    let list: Vec<SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    
    if let Some(sj) = list.into_iter().next() {
        return serde_json::from_value(sj.0).map_err(|e| {
            println!("[ERROR] find_one_robust (query) deserialization failed for {}: {}. Error: {}", table, clean_value, e);
            format!("Erro ao processar dados de {}: {}", table, e)
        });
    }

    Ok(None)
}

/// Lists saved appraisals
#[tauri::command]
pub async fn get_appraisals(
    db: State<'_, DbState>,
    year: Option<u16>,
) -> Result<Vec<crate::models::dto::TaxAppraisalDto>, String> {
    // PRE-MIGRATION: Robust multi-stage field renaming
    let migrations = vec![
        "UPDATE tax_appraisal SET period_month = month, period_year = year, month = NONE, year = NONE WHERE month != NONE",
        "UPDATE tax_appraisal SET period_month = appraisal_month, period_year = appraisal_year, appraisal_month = NONE, appraisal_year = NONE WHERE appraisal_month != NONE"
    ];
    for m in migrations {
        let _ = db.0.query(m).await;
    }

    let base_select = "SELECT \
        id, \
        type::number(period_month) as period_month, \
        type::number(period_year) as period_year, \
        trade_type, \
        tax_rule_id, \
        revenue_code, \
        type::float(gross_profit) as gross_profit, \
        type::float(loss) as loss, \
        type::float(net_profit) as net_profit, \
        type::float(compensated_loss) as compensated_loss, \
        type::float(calculation_basis) as calculation_basis, \
        type::float(tax_rate) as tax_rate, \
        type::float(tax_due) as tax_due, \
        type::float(withheld_tax) as withheld_tax, \
        type::float(withholding_credit_used) as withholding_credit_used, \
        type::float(withholding_credit_remaining) as withholding_credit_remaining, \
        type::float(tax_payable) as tax_payable, \
        type::float(tax_accumulated) as tax_accumulated, \
        type::float(total_payable) as total_payable, \
        is_exempt, \
        calculation_date, \
        status, \
        trade_ids, \
        is_complementary \
        FROM tax_appraisal ";

    let query = if let Some(y) = year {
        format!("{} WHERE period_year = {} OR status = 'Pending'", base_select, y)
    } else {
        format!("{}", base_select)
    };

    let mut result = db.0.query(query).await.map_err(|e| e.to_string())?;
    let appraisals_json_sj: Vec<SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    
    let mut appraisals: Vec<TaxAppraisal> = appraisals_json_sj.into_iter()
        .filter_map(|sj| serde_json::from_value(sj.0).ok())
        .collect();

    // Sort descending by year then month
    appraisals.sort_by(|a, b| {
        b.period_year.cmp(&a.period_year)
            .then(b.period_month.cmp(&a.period_month))
    });

    Ok(appraisals.into_iter().map(|a| crate::models::ToDto::to_dto(&a)).collect())
}

#[tauri::command]
pub async fn save_appraisal(
    db: tauri::State<'_, DbState>,
    data: TaxAppraisal,
) -> Result<crate::models::dto::TaxAppraisalDto, String> {
    let client = db.0.clone();
    println!("[IRPF] Starting save_appraisal for month {}/{}", data.period_month, data.period_year);
    
    let mut data_to_save = data.clone();
    
    // 1. Determine if this should be marked as complementary
    let sibling_id = data.id.clone().unwrap_or_default();
    
    let mut comp_res = client.query("SELECT count() FROM tax_appraisal WHERE period_month = $month AND period_year = $year AND trade_type = $type AND status = 'Paid' AND id != $id")
        .bind(serde_json::json!({
            "month": data.period_month,
            "year": data.period_year,
            "type": data.trade_type,
            "id": sibling_id
        }))
        .await
        .map_err(|e| {
            println!("[IRPF] Complementary check query failed: {:?}", e);
            e.to_string()
        })?;

    let paid_count_res: Vec<SurrealJson> = comp_res.take(0).map_err(|e| e.to_string())?;
    let paid_count = paid_count_res.first()
        .and_then(|sj| sj.0.get("count").and_then(|c| c.as_i64()))
        .unwrap_or(0);
    
    if paid_count > 0 {
        println!("[IRPF] Marking as complementary because {} paid siblings found", paid_count);
        data_to_save.is_complementary = true;
    }

    // 2. Save the record
    let saved: TaxAppraisal = if let Some(id_str) = &data_to_save.id {
        let clean_id = id_str.split(':').last().unwrap_or(id_str).to_string();
        println!("[IRPF] Updating existing record: {}", clean_id);
        
        client.update(("tax_appraisal", clean_id))
            .merge(data_to_save)
            .await
            .map_err(|e| {
                println!("[IRPF] UPDATE ERROR: {:?}", e);
                e.to_string()
            })?
            .ok_or_else(|| "Record not found during update".to_string())?
    } else {
        println!("[IRPF] Creating new record");
        client.create("tax_appraisal")
            .content(data_to_save)
            .await
            .map_err(|e| {
                println!("[IRPF] CREATE ERROR: {:?}", e);
                e.to_string()
            })?
            .ok_or_else(|| "Failed to create new record".to_string())?
    };

    // 3. Post-save logic: Mark old pending items as paid if there's accumulated tax
    if saved.tax_accumulated > 0.0 {
        let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        client.query("UPDATE tax_appraisal SET status = 'Paid', calculation_date = $now WHERE trade_type = $type AND status = 'Pending' AND total_payable < 10.0 AND (period_year < $year OR (period_year = $year AND period_month < $month))")
            .bind(serde_json::json!({
                "type": saved.trade_type,
                "year": saved.period_year,
                "month": saved.period_month,
                "now": now_str
            }))
            .await
            .map_err(|e| {
                println!("[IRPF] Post-save status update failed: {:?}", e);
                e.to_string()
            })?;
    }

    println!("[IRPF] Save completed successfully for {}", saved.id.as_deref().unwrap_or("unknown"));
    Ok(crate::models::ToDto::to_dto(&saved))
}

#[tauri::command]
pub async fn get_accumulated_losses(db: State<'_, DbState>) -> Result<Vec<crate::models::dto::TaxLossDto>, String> {
    let mut loss_res = db.0.query("SELECT id, trade_type, type::float(amount) as amount, type::float(balance) as balance, origin_date FROM tax_loss WHERE balance > 0 ORDER BY origin_date ASC").await.map_err(|e| e.to_string())?;
    let losses_sj: Vec<SurrealJson> = loss_res.take(0).map_err(|e| e.to_string())?;
    let losses: Vec<TaxLoss> = losses_sj.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();
    Ok(losses.into_iter().map(|l| crate::models::ToDto::to_dto(&l)).collect())
}

#[tauri::command]
pub async fn get_appraisal_by_id(
    db: State<'_, DbState>,
    id: String,
) -> Result<Option<crate::models::dto::TaxAppraisalDto>, String> {
    let appraisal: Option<TaxAppraisal> = find_one_robust(&db.0, "tax_appraisal", "id", &id).await?;
    Ok(appraisal.map(|a| crate::models::ToDto::to_dto(&a)))
}

/// Deletes an appraisal and its associated tax loss if applicable
#[tauri::command]
pub async fn delete_appraisal(db: State<'_, DbState>, id: String) -> Result<(), String> {
    // 1. Fetch the appraisal to check if it generated a loss
    let tid = if id.contains(':') { id.split(':').last().unwrap_or(&id) } else { &id };

    let appraisal_sj: Option<SurrealJson> =
        db.0.select(("tax_appraisal", tid)).await.map_err(|e| e.to_string())?;
    
    let sj = appraisal_sj.ok_or_else(|| "Appraisal not found".to_string())?;
    let appraisal: TaxAppraisal = serde_json::from_value(sj.0).map_err(|e| {
        println!("[ERROR] delete_appraisal deserialization failed: {}", e);
        format!("Erro ao processar dados da apuração: {}", e)
    })?;

    // 2. If it generated a loss, delete the corresponding TaxLoss record
    if appraisal.net_profit < 0.0 {
        let origin_date = format!("{}-{:02}-01", appraisal.period_year, appraisal.period_month);
        let query = "SELECT * FROM tax_loss WHERE origin_date = $date AND trade_type = $type";
        let mut loss_res = db.0.query(query)
            .bind(("date", origin_date))
            .bind(("type", appraisal.trade_type.clone()))
            .await
            .map_err(|e| e.to_string())?;
        
        let loss_record_sj: Option<SurrealJson> = loss_res.take(0).map_err(|e| e.to_string())?;
        if let Some(lsj) = loss_record_sj {
            let loss_record: TaxLoss = serde_json::from_value(lsj.0).map_err(|e| {
                println!("[ERROR] delete_appraisal (loss) deserialization failed: {}", e);
                e.to_string()
            })?;
            
            // Only delete if it hasn't been used yet
            if loss_record.amount == loss_record.balance {
                if let Some(lid) = loss_record.id {
                    let l_tid = if lid.contains(':') { lid.split(':').last().unwrap_or(&lid) } else { &lid };
                    let _: Option<TaxLoss> = db.0.delete(("tax_loss", l_tid)).await.map_err(|e| e.to_string())?;
                }
            }
        }
    }

    // 3. Delete associated DARFs to prevent orphans (but block if any is Paid)
    let darf_query = "SELECT * FROM tax_darf WHERE appraisal_id = $id";
    let mut darf_result = db.0.query(darf_query).bind(("id", id.clone())).await.map_err(|e| e.to_string())?;
    let darfs_json: Vec<SurrealJson> = darf_result.take(0).map_err(|e| e.to_string())?;
    let darfs: Vec<TaxDarf> = darfs_json.into_iter().filter_map(|s| serde_json::from_value(s.0).ok()).collect();

    if darfs.iter().any(|d| d.status == "Paid") {
        return Err("Não é possível excluir uma apuração que possui DARF marcada como Paga. Estorne o pagamento primeiro.".to_string());
    }

    for darf in darfs {
        if let Some(did) = darf.id {
            let d_tid = if did.contains(':') { did.split(':').last().unwrap_or(&did) } else { &did };
            let _: Option<TaxDarf> = db.0.delete(("tax_darf", d_tid)).await.map_err(|e| e.to_string())?;
        }
    }

    // 4. Finally delete the appraisal itself
    let _: Option<TaxAppraisal> = db.0.delete(("tax_appraisal", tid)).await.map_err(|e| e.to_string())?;

    Ok(())
}

/// Generates a DARF from an appraisal
#[tauri::command]
pub async fn generate_darf(
    db: State<'_, DbState>,
    appraisal_id: String,
) -> Result<crate::models::dto::TaxDarfDto, String> {
    println!(
        "DEBUG: generate_darf called for appraisal_id: {}",
        appraisal_id
    );

    // Fetch appraisal
    let parts: Vec<&str> = appraisal_id.split(':').collect();
    let tb = if parts.len() > 1 {
        parts[0]
    } else {
        "tax_appraisal"
    };
    let tid = if parts.len() > 1 {
        parts[1]
    } else {
        &appraisal_id
    };

    let appraisal_res: Option<crate::models::SurrealJson> = db.0.select((tb, tid)).await.map_err(|e| {
        println!("DEBUG: Failed to fetch appraisal: {}", e);
        e.to_string()
    })?;
    
    let sj = appraisal_res.ok_or("Appraisal not found")?;
    let appraisal: TaxAppraisal = serde_json::from_value(sj.0).map_err(|e| {
        println!("DEBUG: Failed to deserialize appraisal: {}", e);
        format!("Falha na desserialização da apuração: {}", e)
    })?;
    println!("DEBUG: Found appraisal for DARF: {:?}", appraisal);

    if appraisal.is_exempt {
        return Err("Esta apuração está marcada como ISENTA. Não há imposto a pagar.".to_string());
    }

    if appraisal.total_payable < 10.0 {
        return Err("Valor total (incluindo acumulados) inferior a R$ 10,00. Não é necessário gerar DARF ainda.".to_string());
    }

    // Use revenue code from appraisal with fallback for older records
    let revenue_code = if let Some(rc) = appraisal.revenue_code.as_ref() {
        if !rc.is_empty() { rc.clone() } else { "6015".to_string() }
    } else {
        "6015".to_string()
    };

    let period = format!("{:02}/{}", appraisal.period_month, appraisal.period_year);

    println!("DEBUG: Checking existing DARFs for this Appraisal ID...");
    // Identify if a DARF already exists for THIS appraisal
    let check_query = "SELECT * FROM tax_darf WHERE appraisal_id = $appraisal_id";
    let mut check_result =
        db.0.query(check_query)
            .bind(("appraisal_id", appraisal_id.clone()))
            .await
            .map_err(|e| {
                println!("DEBUG: Failed to check existing DARFs query: {}", e);
                e.to_string()
            })?;

    let existing_darfs_json: Vec<crate::models::SurrealJson> = check_result.take(0).map_err(|e| {
        println!("DEBUG: Failed to deserialize existing DARFs: {}", e);
        e.to_string()
    })?;

    let existing_darfs: Vec<TaxDarf> = existing_darfs_json.into_iter().filter_map(|v| serde_json::from_value(v.0).ok()).collect();

    if !existing_darfs.is_empty() {
        println!("DEBUG: DARF already exists for this appraisal");
        return Err(
            "Uma DARF já existe para esta apuração. Verifique a Central de DARFs.".to_string(),
        );
    }

    // Calculate due date: last business day of the following month

    // Calculate due date: last business day of the following month
    let due_year = if appraisal.period_month == 12 {
        appraisal.period_year + 1
    } else {
        appraisal.period_year
    };
    let due_month = if appraisal.period_month == 12 {
        1
    } else {
        appraisal.period_month + 1
};

    let due_date_obj = NaiveDate::from_ymd_opt(due_year as i32, due_month as u32, 1)
        .unwrap()
        .with_day(1)
        .unwrap()
        .checked_add_months(chrono::Months::new(1))
        .unwrap()
        .pred_opt()
        .unwrap(); // Last day of next month
                   // period already defined above

    let darf = TaxDarf {
        id: None,
        appraisal_id: Some(appraisal_id),
        revenue_code: Some(revenue_code.clone()),
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
        is_complementary: appraisal.is_complementary,
        metadata: std::collections::HashMap::new(),
    };

    let mut created_res = db.0.query("CREATE tax_darf SET 
        appraisal_id = $appraisal_id,
        revenue_code = $revenue_code,
        period = $period,
        principal_value = $principal_value,
        fine = $fine,
        interest = $interest,
        total_value = $total_value,
        due_date = $due_date,
        payment_date = $payment_date,
        status = $status,
        darf_number = $darf_number,
        account_id = $account_id,
        transaction_id = $transaction_id,
        is_complementary = $is_complementary
        RETURN *, type::string(id) as id")
        .bind(("appraisal_id", darf.appraisal_id))
        .bind(("revenue_code", darf.revenue_code))
        .bind(("period", darf.period))
        .bind(("principal_value", darf.principal_value))
        .bind(("fine", darf.fine))
        .bind(("interest", darf.interest))
        .bind(("total_value", darf.total_value))
        .bind(("due_date", darf.due_date))
        .bind(("payment_date", darf.payment_date))
        .bind(("status", darf.status))
        .bind(("darf_number", darf.darf_number))
        .bind(("account_id", darf.account_id))
        .bind(("transaction_id", darf.transaction_id))
        .bind(("is_complementary", darf.is_complementary))
        .await
        .map_err(|e| {
            println!("DEBUG: Failed to create DARF query: {}", e);
            e.to_string()
        })?;

    let created_sj: Option<SurrealJson> = created_res
        .take(0)
        .map_err(|e| e.to_string())?;
    
    let created: Option<TaxDarf> = created_sj.and_then(|sj| serde_json::from_value(sj.0).ok());

    if let Some(ref d) = created {
        println!("DEBUG: DARF generated successfully with ID: {:?}", d.id);
    }

    created.map(|d| crate::models::ToDto::to_dto(&d)).ok_or_else(|| "Failed to generate DARF".to_string())
}

/// Lists DARFs
#[tauri::command]
pub async fn get_darfs(db: State<'_, DbState>, year: Option<u16>) -> Result<Vec<crate::models::dto::TaxDarfDto>, String> {
    println!("DEBUG: get_darfs called for year: {:?}", year);
    let (base_query, query_params) = if let Some(y) = year {
        let q = "SELECT * FROM tax_darf WHERE string::ends_with(period, $year) OR status = 'Pending' OR (payment_date != NONE AND string::starts_with(type::string(payment_date), $year)) ORDER BY due_date DESC";
        (q, Some(y.to_string()))
    } else {
        let q = "SELECT * FROM tax_darf ORDER BY due_date DESC";
        (q, None)
    };

    let mut result = if let Some(y) = query_params {
        db.0.query(base_query).bind(("year", y)).await.map_err(|e| e.to_string())?
    } else {
        db.0.query(base_query).await.map_err(|e| e.to_string())?
    };
    
    let darfs_json: Vec<SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    let darfs: Vec<TaxDarf> = darfs_json.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();

    Ok(darfs.into_iter().map(|d| crate::models::ToDto::to_dto(&d)).collect())
}

#[tauri::command]
pub async fn get_darf_by_id(
    db: State<'_, DbState>,
    id: String,
) -> Result<Option<crate::models::dto::TaxDarfDto>, String> {
    let darf: Option<TaxDarf> = find_one_robust(&db.0, "tax_darf", "id", &id).await?;
    Ok(darf.map(|d| crate::models::ToDto::to_dto(&d)))
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
    account_id: String,
    transaction_id: String,
) -> Result<crate::models::dto::TaxDarfDto, String> {
    let tid = if id.contains(':') { id.split(':').last().unwrap_or(&id) } else { &id };
    println!("[DEBUG] mark_darf_paid: Fetching DARF tid={} (original id={})", tid, id);
    
    let darf_res_json: Option<crate::models::SurrealJson> = db.0.select(("tax_darf", tid)).await.map_err(|e| e.to_string())?;
    
    let sj = darf_res_json.ok_or_else(|| {
        println!("[ERROR] mark_darf_paid: DARF {} not found in database", tid);
        "DARF not found".to_string()
    })?;
    
    let mut darf: TaxDarf = serde_json::from_value(sj.0).map_err(|e| {
        println!("[ERROR] mark_darf_paid deserialization failed: {}", e);
        format!("Erro ao processar dados da DARF: {}", e)
    })?;

    println!("[DEBUG] mark_darf_paid: Current DARF status in DB: '{}'", darf.status);

    // Prevent duplicate payments
    if darf.status == "Paid" {
        println!("[WARNING] mark_darf_paid: Rejecting payment because status is already 'Paid'");
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
    // --- ULTRA-SAFE: Build manual map for TaxDarf update ---
    let mut content_map = serde_json::Map::new();
    content_map.insert("appraisal_id".to_string(), darf.appraisal_id.clone().into());
    content_map.insert("revenue_code".to_string(), darf.revenue_code.clone().into());
    content_map.insert("period".to_string(), darf.period.clone().into());
    content_map.insert("principal_value".to_string(), darf.principal_value.into());
    content_map.insert("fine".to_string(), darf.fine.into());
    content_map.insert("interest".to_string(), darf.interest.into());
    content_map.insert("total_value".to_string(), darf.total_value.into());
    content_map.insert("due_date".to_string(), darf.due_date.clone().into());
    content_map.insert("payment_date".to_string(), serde_json::json!(darf.payment_date));
    content_map.insert("status".to_string(), darf.status.clone().into());
    content_map.insert("darf_number".to_string(), serde_json::json!(darf.darf_number));
    content_map.insert("account_id".to_string(), serde_json::json!(darf.account_id));
    content_map.insert("transaction_id".to_string(), serde_json::json!(darf.transaction_id));
    content_map.insert("is_complementary".to_string(), darf.is_complementary.into());

    // 1. Update DARF record
    let clean_id = id.split(':').last().unwrap_or(&id).to_string();
    let update_query = "UPDATE type::thing('tax_darf', $id) MERGE $content RETURN *";
    let mut update_result =
        db.0.query(update_query)
            .bind(("id", clean_id.clone()))
            .bind(("content", serde_json::Value::Object(content_map)))
            .await
            .map_err(|e| e.to_string())?;

    let updated_json: Option<crate::models::SurrealJson> = update_result.take(0).map_err(|e| e.to_string())?;
    let updated: Option<TaxDarf> = updated_json.and_then(|v| serde_json::from_value(v.0).ok());

    // 2. Update Appraisal Record
    let appraisal_id_str = darf.appraisal_id.as_deref().unwrap_or("");
    let parts: Vec<&str> = appraisal_id_str.split(':').collect();
    let aid_val = if parts.len() > 1 {
        parts[1]
    } else {
        appraisal_id_str
    };

    let appraisal_result: Result<Option<TaxAppraisal>, _> =
        db.0.select(("tax_appraisal", aid_val)).await;

    if let Ok(Some(mut appraisal)) = appraisal_result {
        appraisal.status = "Paid".to_string();

        if let Some(aid) = &appraisal.id {
            let a_parts: Vec<&str> = aid.split(':').collect();
            let a_tb = if a_parts.len() > 1 {
                a_parts[0]
            } else {
                "tax_appraisal"
            };
            let a_id = if a_parts.len() > 1 { a_parts[1] } else { aid };

            // CRITICAL FIX: Strip ID to prevent SurrealDB error
            let mut appraisal_content = appraisal.clone();
            appraisal_content.id = None;

            let _: Option<TaxAppraisal> =
                db.0.update((a_tb, a_id))
                    .content(appraisal_content)
                    .await
                    .map_err(|e| e.to_string())?;

            // 3. Mark all PREVIOUS PENDING appraisals of the same type as PAID (Accumulation Clear)
            let trade_type_inner = appraisal.trade_type.clone();
            let clear_query = "UPDATE tax_appraisal SET status = 'Paid' WHERE trade_type = $type AND status = 'Pending' AND (period_year < $year OR (period_year = $year AND period_month < $month))";

            let _: surrealdb::Response =
                db.0.query(clear_query)
                    .bind(("type", trade_type_inner))
                    .bind(("year", appraisal.period_year))
                    .bind(("month", appraisal.period_month))
                    .await
                    .map_err(|e| e.to_string())?;
        }
    }

    println!(
        "[IRPF] Status updated for DARF {} and its appraisals.",
        clean_id
    );



    // --- FINANCIAL INTEGRATION (SIDE EFFECTS) ---
    // 1. Update Account Balance (atomic decrement)
    let acc_parts: Vec<String> = account_id.split(':').map(|s| s.to_string()).collect();
    let acc_tb = if acc_parts.len() > 1 {
        acc_parts[0].clone()
    } else {
        "account".to_string()
    };
    let acc_id_only = if acc_parts.len() > 1 {
        acc_parts[1].clone()
    } else {
        account_id.clone()
    };

    println!(
        "[DARF] Deducting {} from account:{}",
        paid_value, acc_id_only
    );
    let balance_query = "UPDATE type::thing($tb, $id) SET balance -= $val";

    match db
        .0
        .query(balance_query)
        .bind(("tb", acc_tb.clone()))
        .bind(("id", acc_id_only.clone()))
        .bind(("val", paid_value))
        .await
    {
        Ok(_) => println!("[DARF] Account balance updated successfully"),
        Err(e) => println!("[DARF] ERROR updating account balance: {}", e),
    }

    // 2. Create Cash Transaction (Expense) via bound query for Record IDs
    let tx_parts: Vec<String> = transaction_id.split(':').map(|s| s.to_string()).collect();
    let tx_tb = if tx_parts.len() > 1 {
        tx_parts[0].clone()
    } else {
        "cash_transaction".to_string()
    };
    let tx_id_only = if tx_parts.len() > 1 {
        tx_parts[1].clone()
    } else {
        transaction_id.clone()
    };

    println!(
        "[DARF] Creating cash_transaction:{}:{} for account:{}",
        tx_tb, tx_id_only, acc_id_only
    );
    let description = format!("Pagamento DARF {} ({})", darf.period, darf.revenue_code.as_deref().unwrap_or(""));

    // Use full payment_date string (which may contain time)
    let tx_query = "CREATE type::thing($tx_tb, $tx_id) SET 
        date = $date, 
        amount = $amount, 
        type = 'Withdraw', 
        description = $desc, 
        account_id = type::thing($acc_tb, $acc_id),
        category = 'TaxPayment',
        system_linked = true";

    match db
        .0
        .query(tx_query)
        .bind(("tx_tb", tx_tb))
        .bind(("tx_id", tx_id_only))
        .bind(("date", payment_date.clone()))
        .bind(("amount", 0.0 - paid_value))
        .bind(("desc", description))
        .bind(("acc_tb", acc_tb))
        .bind(("acc_id", acc_id_only))
        .await
    {
        Ok(_) => println!("[DARF] Cash transaction created successfully"),
        Err(e) => println!("[DARF] ERROR creating cash transaction: {}", e),
    }

    // --- END FINANCIAL INTEGRATION ---

    updated.map(|d| crate::models::ToDto::to_dto(&d)).ok_or_else(|| "Failed to update DARF".to_string())
}

#[tauri::command]
pub async fn diagnostic_dump_darfs(db: State<'_, DbState>) -> Result<(), String> {
    println!("[DIAGNOSTIC] Dumping all TAX_DARF records...");
    let mut result = db.0.query("SELECT * FROM tax_darf").await.map_err(|e| e.to_string())?;
    let darfs: Vec<SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    println!("[DIAGNOSTIC] Total DARFs found: {}", darfs.len());
    for d in darfs {
        println!(
            "[DIAGNOSTIC] DARF: {}",
            serde_json::to_string(&d).unwrap_or_default()
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn diagnostic_fix_darf_status(
    db: State<'_, DbState>,
    id: String,
    target_status: String,
) -> Result<(), String> {
    let tid = if id.contains(':') { id.split(':').last().unwrap_or(&id) } else { &id };
    println!("[DIAGNOSTIC] Force updating DARF {} status to '{}'", tid, target_status);
    
    let _: surrealdb::Response = db.0.query("UPDATE type::thing('tax_darf', $id) SET status = $status")
        .bind(("id", tid.to_string()))
        .bind(("status", target_status))
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(())
}

#[tauri::command]
pub async fn get_darf_by_transaction(
    db: State<'_, DbState>,
    transaction_id: String,
) -> Result<Option<crate::models::dto::TaxDarfDto>, String> {
    // Robust normalization: handle backticks and table prefix
    let clean_tx_id = transaction_id.replace('`', "").split(':').last().unwrap_or(&transaction_id).to_string();
    println!("[DEBUG] get_darf_by_transaction: Searching for normalized tx_id='{}' (original='{}')", clean_tx_id, transaction_id);
    
    let darf: Option<TaxDarf> = find_one_robust(&db.0, "tax_darf", "transaction_id", &clean_tx_id).await?;
    Ok(darf.map(|d| crate::models::ToDto::to_dto(&d)))
}

/// Reverses a DARF payment (Unpay)
#[tauri::command]
pub async fn unpay_darf(db: State<'_, DbState>, id: String) -> Result<crate::models::dto::TaxDarfDto, String> {
    let parts: Vec<&str> = id.split(':').collect();
    let tb = if parts.len() > 1 {
        parts[0]
    } else {
        "tax_darf"
    };
    let tid = if parts.len() > 1 { parts[1] } else { &id };

    let darf_res: Option<SurrealJson> = db.0.select((tb, tid)).await.map_err(|e| e.to_string())?;
    
    let sj = darf_res.ok_or_else(|| "DARF not found".to_string())?;
    let mut darf: TaxDarf = serde_json::from_value(sj.0).map_err(|e| {
        println!("[ERROR] unpay_darf deserialization failed: {}", e);
        format!("Erro ao processar dados da DARF: {}", e)
    })?;

    if darf.status != "Paid" {
        return Err("DARF is not paid".to_string());
    }

    let paid_value = darf.total_value;

    // 1. Reverse Financial Transaction if linked
    let linked_acc_id = darf.account_id.clone();
    let linked_trans_id = darf.transaction_id.clone();

    println!(
        "[DARF UNPAY] Linked Account: {:?}, Linked Transaction: {:?}",
        linked_acc_id, linked_trans_id
    );

    // 1. Create Refund Transaction (Estorno) if we have the account info
    if let Some(acc_id) = linked_acc_id {
        let acc_parts: Vec<String> = acc_id.split(':').map(|s| s.to_string()).collect();
        let acc_tb = if acc_parts.len() > 1 {
            acc_parts[0].clone()
        } else {
            "account".to_string()
        };
        let acc_id_val = if acc_parts.len() > 1 {
            acc_parts[1].clone()
        } else {
            acc_id.clone()
        };

        // Refund Account Balance (atomic)
        println!(
            "[DARF UNPAY] Refunding {} to {}:{}",
            paid_value, acc_tb, acc_id_val
        );
        let refund_acc_query = "UPDATE type::thing($tb, $id) SET balance += $val";
        match db
            .0
            .query(refund_acc_query)
            .bind(("tb", acc_tb.clone()))
            .bind(("id", acc_id_val.clone()))
            .bind(("val", paid_value))
            .await
        {
            Ok(_) => println!("[DARF UNPAY] Account balance refunded successfully"),
            Err(e) => println!("[DARF UNPAY] ERROR refunding account balance: {}", e),
        }

        // Create a separate Deposit transaction for the refund (historical audit trail)
        // If we don't have a linked transaction ID (e.g., from seeder), derive one from the DARF ID
        let raw_tid = match linked_trans_id {
            Some(tid) => tid.split(':').last().unwrap_or(&tid).to_string(),
            None => format!("legacy_unpay_{}", tid), // tid is from darf lookup at start
        };

        let refund_tx_id = format!("refund_{}", raw_tid);
        let description = format!(
            "Estorno - Pagamento DARF {} ({})",
            darf.period, darf.revenue_code.as_deref().unwrap_or("")
        );

        // Use original payment date for the refund so they appear together in the statement
        // payment_date might be full ISO or just YYYY-MM-DD
        let refund_date = darf
            .payment_date
            .clone()
            .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

        println!(
            "[DARF UNPAY] Creating refund transaction cash_transaction:{}",
            refund_tx_id
        );
        let refund_tx_query = "CREATE type::thing('cash_transaction', $id) SET 
            date = $date, 
            amount = $amount, 
            type = 'Deposit', 
            description = $desc, 
            account_id = type::thing($acc_tb, $acc_id),
            category = 'TaxRefund',
            system_linked = true";

        match db
            .0
            .query(refund_tx_query)
            .bind(("id", refund_tx_id))
            .bind(("date", refund_date))
            .bind(("amount", paid_value))
            .bind(("desc", description))
            .bind(("acc_tb", acc_tb.clone()))
            .bind(("acc_id", acc_id_val.clone()))
            .await
        {
            Ok(_) => println!("[DARF UNPAY] Refund cash transaction created successfully"),
            Err(e) => {
                println!("[DARF UNPAY] ERROR creating refund transaction: {}", e);
                // Don't error out the whole unpay, but log it
            }
        }
    } else {
        println!("[DARF UNPAY] WARNING: No linked account found for refund. Skipping financial reversal.");
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

    let clean_id = id.split(':').last().unwrap_or(&id).to_string();
    let updated: Option<TaxDarf> =
        db.0.update(("tax_darf", clean_id))
            .content(darf_content)
            .await
            .map_err(|e| e.to_string())?;

    // 3. Update Appraisal Status
    let appraisal_id_str = darf.appraisal_id.as_deref().unwrap_or("");
    let parts: Vec<&str> = appraisal_id_str.split(':').collect();
    let aid_val = if parts.len() > 1 {
        parts[1]
    } else {
        appraisal_id_str
    };

    let appraisal_result: Result<Option<TaxAppraisal>, _> =
        db.0.select(("tax_appraisal", aid_val)).await;

    if let Ok(Some(mut appraisal)) = appraisal_result {
        appraisal.status = "Pending".to_string();
        if let Some(aid) = &appraisal.id {
            let a_parts: Vec<&str> = aid.split(':').collect();
            let a_tb = if a_parts.len() > 1 {
                a_parts[0]
            } else {
                "tax_appraisal"
            };
            let a_id = if a_parts.len() > 1 { a_parts[1] } else { aid };

            // CRITICAL FIX: Strip ID
            let mut appraisal_content = appraisal.clone();
            appraisal_content.id = None;

            let _: Option<TaxAppraisal> =
                db.0.update((a_tb, a_id))
                    .content(appraisal_content)
                    .await
                    .map_err(|e| e.to_string())?;

            // 3. Mark all PREVIOUS PAID appraisals of the same type as PENDING (Revert Accumulation)
            // Only those that were accumulated (don't have their own specific Paid DARF)
            // However, to keep it simple and consistent with how mark_darf_paid works,
            // we revert all previous 'Paid' ones of the same type that are before this period.
            // A more precise check would ensure they don't have another paid DARF, but
            // since accumulation is strictly chronological, this is generally safe.
            let trade_type_inner = appraisal.trade_type.clone();
            let revert_query = "UPDATE tax_appraisal SET status = 'Pending' \
                WHERE trade_type = $type AND status = 'Paid' \
                AND (period_year < $year OR (period_year = $year AND period_month < $month))";

            let _: surrealdb::Response =
                db.0.query(revert_query)
                    .bind(("type", trade_type_inner))
                    .bind(("year", appraisal.period_year))
                    .bind(("month", appraisal.period_month))
                    .await
                    .map_err(|e| e.to_string())?;

            println!(
                "[DARF UNPAY] Reverted accumulation for appraisals before {}/{} ({})",
                appraisal.period_month, appraisal.period_year, appraisal.trade_type
            );
        }
    }

    updated.map(|d| crate::models::ToDto::to_dto(&d)).ok_or_else(|| "Failed to unpay DARF".to_string())
}

/// Deletes a DARF (blocks if paid - must use unpay_darf first)
#[tauri::command]
pub async fn delete_darf(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);

    // Check if DARF is paid - block deletion
    let darf_res: Option<TaxDarf> =
        db.0.select(("tax_darf", clean_id))
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
pub async fn calculate_interest_fine(
    value: f64,
    due_date: String,
    payment_date: String,
) -> Result<(f64, f64), String> {
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
    db.0.query("DELETE FROM tax_appraisal")
        .await
        .map_err(|e| e.to_string())?;
    db.0.query("DELETE FROM tax_darf")
        .await
        .map_err(|e| e.to_string())?;
    db.0.query("DELETE FROM tax_loss")
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_tax_loss(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let _: Option<TaxLoss> =
        db.0.delete(("tax_loss", &id))
            .await
            .map_err(|e| e.to_string())?;
    Ok(())
}


#[tauri::command]
pub async fn get_tax_rules(db: State<'_, DbState>) -> Result<Vec<crate::models::dto::TaxRuleDto>, String> {
    let mut result = db.0.query("SELECT * FROM tax_rule").await.map_err(|e| e.to_string())?;
    let rules_sj: Vec<SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    let rules: Vec<TaxRule> = rules_sj.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();
    Ok(rules.into_iter().map(|r| crate::models::ToDto::to_dto(&r)).collect())
}

#[tauri::command]
pub async fn save_tax_rule(db: State<'_, DbState>, rule: TaxRule) -> Result<crate::models::dto::TaxRuleDto, String> {
    let mut json = serde_json::to_value(&rule).map_err(|e| e.to_string())?;
    let id_str = rule.id.as_deref().unwrap_or("");
    let clean_id = id_str.split(':').last().unwrap_or(id_str).to_string();
    if let Some(obj) = json.as_object_mut() { obj.remove("id"); }
    let mut response = db.0.query("UPSERT type::thing('tax_rule', $id) CONTENT $data RETURN *")
            .bind(("id", clean_id)).bind(("data", json)).await.map_err(|e| e.to_string())?;
    let saved_sj: Option<SurrealJson> = response.take(0).map_err(|e| e.to_string())?;
    let sj = saved_sj.ok_or_else(|| "Failed to save tax rule".to_string())?;
    let saved: TaxRule = serde_json::from_value(sj.0).map_err(|e| {
        println!("[ERROR] save_tax_rule deserialization failed: {}", e);
        e.to_string()
    })?;
    Ok(crate::models::ToDto::to_dto(&saved))
}

#[tauri::command]
pub async fn delete_tax_rule(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    let _ =
        db.0.query("DELETE type::thing('tax_rule', $id)")
            .bind(("id", clean_id.to_string()))
            .await;
    Ok(())
}

#[tauri::command]
pub async fn get_tax_mappings(db: State<'_, DbState>) -> Result<Vec<crate::models::dto::TaxMappingDto>, String> {
    let mut result = db.0.query("SELECT * FROM tax_mapping").await.map_err(|e| e.to_string())?;
    let mappings_sj: Vec<SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    let mappings: Vec<TaxMapping> = mappings_sj.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();
    Ok(mappings.into_iter().map(|m| crate::models::ToDto::to_dto(&m)).collect())
}

#[tauri::command]
pub async fn save_tax_mapping(
    db: State<'_, DbState>,
    mapping: TaxMapping,
) -> Result<crate::models::dto::TaxMappingDto, String> {
    let mut json = serde_json::to_value(&mapping).map_err(|e| e.to_string())?;

    let id_str = mapping.id.as_deref().unwrap_or("");
    let clean_id = id_str.split(':').last().unwrap_or(id_str).to_string();

    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }

    // Use raw query for robust serialization with custom IdVisitor
    let mut response =
        db.0.query("UPSERT type::thing('tax_mapping', $id) CONTENT $data RETURN *")
            .bind(("id", clean_id))
            .bind(("data", json))
            .await
            .map_err(|e| e.to_string())?;

    let saved_sj: Option<SurrealJson> = response.take(0).map_err(|e| e.to_string())?;
    let sj = saved_sj.ok_or_else(|| "Failed to save tax mapping".to_string())?;
    let saved: TaxMapping = serde_json::from_value(sj.0).map_err(|e| {
        println!("[ERROR] save_tax_mapping deserialization failed: {}", e);
        e.to_string()
    })?;
    Ok(crate::models::ToDto::to_dto(&saved))
}

#[tauri::command]
pub async fn delete_tax_mapping(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    let _ =
        db.0.query("DELETE type::thing('tax_mapping', $id)")
            .bind(("id", clean_id.to_string()))
            .await;
    Ok(())
}

// --- TAX PROFILES & ENTRIES (New Option B) ---

#[tauri::command]
pub async fn get_tax_profiles(db: State<'_, DbState>) -> Result<Vec<crate::models::dto::TaxProfileDto>, String> {
    let mut result =
        db.0.query("SELECT * FROM tax_profile")
            .await
            .map_err(|e| e.to_string())?;
    let profiles_sj: Vec<SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    let profiles: Vec<TaxProfile> = profiles_sj.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();
    Ok(profiles.into_iter().map(|p| crate::models::ToDto::to_dto(&p)).collect())
}

#[tauri::command]
pub async fn save_tax_profile(
    db: State<'_, DbState>,
    profile: TaxProfile,
) -> Result<crate::models::dto::TaxProfileDto, String> {
    let mut json = serde_json::to_value(&profile).map_err(|e| e.to_string())?;

    let id_str = profile.id.as_deref().unwrap_or("");
    let clean_id = id_str.split(':').last().unwrap_or(id_str).to_string();

    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }

    // Use raw query for robust serialization with custom IdVisitor
    let mut response =
        db.0.query("UPSERT type::thing('tax_profile', $id) CONTENT $data RETURN *")
            .bind(("id", clean_id))
            .bind(("data", json))
            .await
            .map_err(|e| e.to_string())?;

    let saved_sj: Option<SurrealJson> = response.take(0).map_err(|e| e.to_string())?;
    let sj = saved_sj.ok_or_else(|| "Failed to save tax profile".to_string())?;
    let saved: TaxProfile = serde_json::from_value(sj.0).map_err(|e| {
        println!("[ERROR] save_tax_profile deserialization failed: {}", e);
        e.to_string()
    })?;
    Ok(crate::models::ToDto::to_dto(&saved))
}

#[tauri::command]
pub async fn delete_tax_profile(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);

    // Delete profile
    let _ =
        db.0.query("DELETE type::thing('tax_profile', $id)")
            .bind(("id", clean_id.to_string()))
            .await;
    // Delete related entries (raw query handles relations fine if we use type::thing properly or direct strings)
    // Note: The related entries IDs aren't known, so we delete by condition, wrapping in tick marks to be safe
    let query_e = format!("DELETE tax_profile_entry WHERE tax_profile_id = type::thing('tax_profile:{}') OR tax_profile_id = 'tax_profile:{}' OR tax_profile_id = '{}'", clean_id, clean_id, clean_id);
    let _ = db.0.query(query_e).await;
    Ok(())
}

// --- TAX PROFILES & ENTRIES (New Option B) ---

#[tauri::command]
pub async fn get_tax_profile_entries(
    db: State<'_, DbState>,
    profile_id: Option<String>,
) -> Result<Vec<crate::models::dto::TaxProfileEntryDto>, String> {
    // Fallsback to format! to resolve persistent lifetime issues with bind in this async context
    let query = if let Some(pid) = profile_id {
        format!("SELECT * FROM tax_profile_entry WHERE tax_profile_id = '{}'", pid)
    } else {
        "SELECT * FROM tax_profile_entry".to_string()
    };

    let mut result = db.0.query(query).await.map_err(|e| e.to_string())?;

    let entries_sj: Vec<SurrealJson> = result.take(0).map_err(|e| e.to_string())?;
    let entries: Vec<TaxProfileEntry> = entries_sj.into_iter().filter_map(|sj| serde_json::from_value(sj.0).ok()).collect();
    Ok(entries.into_iter().map(|e| crate::models::ToDto::to_dto(&e)).collect())
}

#[tauri::command]
pub async fn save_tax_profile_entry(
    db: State<'_, DbState>,
    entry: TaxProfileEntry,
) -> Result<crate::models::dto::TaxProfileEntryDto, String> {
    let mut json = serde_json::to_value(&entry).map_err(|e| e.to_string())?;

    let id_str = entry.id.as_deref().unwrap_or("");
    let clean_id = id_str.split(':').last().unwrap_or(id_str).to_string();

    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }

    // Use raw query for robust serialization with custom IdVisitor
    let mut response =
        db.0.query("UPSERT type::thing('tax_profile_entry', $id) CONTENT $data RETURN *")
            .bind(("id", clean_id))
            .bind(("data", json))
            .await
            .map_err(|e| e.to_string())?;

    let saved_sj: Option<SurrealJson> = response.take(0).map_err(|e| e.to_string())?;
    let sj = saved_sj.ok_or_else(|| "Failed to save tax profile entry".to_string())?;
    let saved: TaxProfileEntry = serde_json::from_value(sj.0).map_err(|e| {
        println!("[ERROR] save_tax_profile_entry deserialization failed: {}", e);
        e.to_string()
    })?;
    Ok(crate::models::ToDto::to_dto(&saved))
}
#[tauri::command]
pub async fn delete_tax_profile_entry(db: State<'_, DbState>, id: String) -> Result<(), String> {
    let clean_id = id.split(':').last().unwrap_or(&id);
    let _ =
        db.0.query("DELETE type::thing('tax_profile_entry', $id)")
            .bind(("id", clean_id.to_string()))
            .await;
    Ok(())
}
