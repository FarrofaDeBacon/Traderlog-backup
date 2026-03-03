use crate::db::DbState;
use crate::logic::RuleBucket;
use crate::models::irpf::{TaxAppraisal, TaxDarf, TaxLoss};
use crate::models::Trade;
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use serde_json;
use tauri::State;

#[derive(Debug, Deserialize)]
struct IrpfTrade {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub asset_symbol: String,
    #[serde(default)]
    pub exit_date: Option<String>,
    #[serde(default)]
    pub result: Option<f64>,
    #[serde(default)]
    pub fee_total: Option<f64>,
    #[serde(default)]
    pub quantity: Option<f64>,
    #[serde(default)]
    pub exit_price: Option<f64>,
    #[serde(default)]
    pub modality_id: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub asset_type_id: Option<String>,
    #[serde(default)]
    pub point_value: Option<f64>,
}

use crate::models::{Asset, AssetType, Modality, TaxMapping, TaxProfile, TaxProfileEntry, TaxRule};

#[tauri::command]
pub async fn calculate_monthly_tax(
    db: State<'_, DbState>,
    month: u8,
    year: u16,
) -> Result<Vec<TaxAppraisal>, String> {
    println!(
        ">>> ENTRY: calculate_monthly_tax(month={}, year={})",
        month, year
    );

    // TEMPORARY: Force diagnostic dump
    if let Err(e) = crate::commands::diagnostic_closure_dump(db.clone()).await {
        println!("[IRPF] diagnostic dump failed: {}", e);
    }

    // 1. Fetch data with subqueries for completeness
    let query_trades = "SELECT 
            type::string(id) as id,
            type::string(date) as date,
            (IF exit_date != NONE THEN type::string(exit_date) ELSE null END) as exit_date,
            (IF account_id != NONE THEN type::string(account_id) ELSE null END) as account_id,
            (IF asset_id != NONE THEN type::string(asset_id) ELSE null END) as asset_id,
            (IF modality_id != NONE THEN type::string(modality_id) ELSE null END) as modality_id,
            type::string(asset_type_id) as trade_asset_type_id,
            type::float(result) as result,
            type::float(fee_total) as fee_total,
            type::float(quantity) as quantity,
            type::float(exit_price) as exit_price,
            (SELECT VALUE type::string(symbol) FROM asset WHERE id = $parent.asset_id LIMIT 1)[0] as asset_symbol,
            (SELECT VALUE (IF asset_type_id != NONE THEN type::string(asset_type_id) ELSE null END) FROM asset WHERE id = $parent.asset_id LIMIT 1)[0] as linked_asset_type_id,
            (SELECT VALUE type::float(point_value) FROM asset WHERE id = $parent.asset_id LIMIT 1)[0] as point_value
         FROM trade WHERE (exit_date IS NOT NONE) AND (result IS NOT NONE)";

    let mut trade_res = db.0.query(query_trades).await.map_err(|e| e.to_string())?;
    println!("DEBUG: Query executed, taking raw results...");

    let raw_trades: Vec<serde_json::Value> = trade_res.take(0).map_err(|e| {
        let err_msg = format!("RAW DESERIALIZATION ERROR: {}", e);
        println!("{}", err_msg);
        err_msg
    })?;

    println!("[IRPF] Found {} raw trades in DB.", raw_trades.len());

    let mut all_trades = Vec::new();
    for rt in raw_trades {
        let id_str = rt["id"].as_str().unwrap_or("").to_string();
        let date_str = rt["date"].as_str().unwrap_or("").to_string();
        let exit_date_str = rt["exit_date"].as_str().map(|s| s.to_string());
        let res_val = rt["result"].as_f64();
        let fee_val = rt["fee_total"].as_f64().or_else(|| rt["fees"].as_f64());
        let qty_val = rt["quantity"].as_f64();
        let exit_p = rt["exit_price"].as_f64();
        let mod_id = rt["modality_id"].as_str().map(|s| s.to_string());
        let asset_id = rt["asset_id"].as_str().map(|s| s.to_string());
        let linked_at_id = rt["linked_asset_type_id"].as_str().map(|s| s.to_string());
        let trade_at_id = rt["trade_asset_type_id"].as_str().map(|s| s.to_string());
        let at_id = linked_at_id.or(trade_at_id);

        let symbol = rt["asset_symbol"].as_str().unwrap_or("").to_string();
        let pv = rt["point_value"].as_f64();

        all_trades.push(IrpfTrade {
            id: id_str,
            date: date_str,
            asset_symbol: symbol,
            exit_date: exit_date_str,
            result: res_val,
            fee_total: fee_val,
            quantity: qty_val,
            exit_price: exit_p,
            modality_id: mod_id,
            asset_id,
            asset_type_id: at_id,
            point_value: pv,
        });
    }

    // In-memory format agnostic filter by month/year
    let prefix = format!("{}-{:02}-", year, month);
    println!(
        "[IRPF] Filtering {} trades with prefix: '{}'",
        all_trades.len(),
        prefix
    );

    let trades_for_period: Vec<IrpfTrade> = all_trades
        .into_iter()
        .filter(|t| {
            let date_to_use = t.exit_date.as_ref().unwrap_or(&t.date);
            let matches = date_to_use.starts_with(&prefix);
            if matches {
                println!("[IRPF] Trade {} MATCHED period prefix {}", t.id, prefix);
            }
            matches
        })
        .collect();
    println!(
        "[IRPF] Trades matching period {}/{}: {}",
        month,
        year,
        trades_for_period.len()
    );

    // Existing Appraisals (needed for preservation and incremental logic)
    let exist_query = "SELECT *, type::string(id) as id FROM tax_appraisal WHERE period_month = $month AND period_year = $year";
    let mut exist_res =
        db.0.query(exist_query)
            .bind(("month", month))
            .bind(("year", year))
            .await
            .map_err(|e| e.to_string())?;
    let existing_appraisals: Vec<TaxAppraisal> = exist_res.take(0).map_err(|e| e.to_string())?;

    // --- INCREMENTAL LOGIC ---
    // Identify trades already appraised in "Paid" appraisals for this period
    let mut already_appraised_ids = std::collections::HashSet::new();
    for app in &existing_appraisals {
        // Only exclude trades from appraisals that are "Paid" or "Ok" (settled)
        if app.status == "Paid" || app.status == "Ok" {
            for tid in &app.trade_ids {
                already_appraised_ids.insert(tid.clone());
            }
        }
    }

    // Filter out already appraised trades
    let trades: Vec<IrpfTrade> = if !already_appraised_ids.is_empty() {
        println!(
            "[IRPF] Already appraised IDs found: {:?}",
            already_appraised_ids
        );
        let final_list: Vec<IrpfTrade> = trades_for_period
            .into_iter()
            .filter(|t| {
                let is_excluded = already_appraised_ids.contains(&t.id);
                if is_excluded {
                    println!("[IRPF] Excluding trade {} - already appraised", t.id);
                }
                !is_excluded
            })
            .collect();
        println!("[IRPF] After exclusion, {} trades remain", final_list.len());
        final_list
    } else {
        println!("[IRPF] No existing 'Paid'/'Ok' appraisals found. All {} trades from period will be processed.", trades_for_period.len());
        trades_for_period
    };

    if already_appraised_ids.len() > 0 && trades.is_empty() {
        println!("[IRPF] No NEW trades to appraise for {}/{}", month, year);
        return Ok(vec![]);
    }

    println!(
        "[IRPF] Total de trades a processar (novos): {}",
        trades.len()
    );

    // Assets & Asset Types
    let mut asset_res = db.0.query("SELECT 
        type::string(id) as id, 
        symbol, 
        name, 
        type::string(asset_type_id) as asset_type_id, 
        type::float(point_value) as point_value,
        (IF default_fee_id != NONE THEN type::string(default_fee_id) ELSE null END) as default_fee_id,
        (IF tax_profile_id != NONE THEN type::string(tax_profile_id) ELSE null END) as tax_profile_id 
        FROM asset").await.map_err(|e| e.to_string())?;
    let assets: Vec<Asset> = asset_res.take(0).map_err(|e| e.to_string())?;
    println!("[IRPF] Loaded {} assets", assets.len());

    let mut at_res = db.0.query("SELECT 
        type::string(id) as id, 
        name, 
        code, 
        type::string(market_id) as market_id, 
        unit_label, 
        result_type,
        (IF default_fee_id != NONE THEN type::string(default_fee_id) ELSE null END) as default_fee_id,
        (IF tax_profile_id != NONE THEN type::string(tax_profile_id) ELSE null END) as tax_profile_id 
        FROM asset_type").await.map_err(|e| e.to_string())?;
    let mut asset_types: Vec<AssetType> = at_res.take(0).map_err(|e| e.to_string())?;
    println!("[IRPF] Loaded {} asset_types", asset_types.len());

    // AUTO-HEALING: If standard asset types lack profiles, fix them in-memory
    for at in asset_types.iter_mut() {
        if at.tax_profile_id.is_none() {
            let clean_id = at.id.split(':').last().unwrap_or(&at.id);
            if clean_id == "at1" {
                at.tax_profile_id = Some("tax_profile:tp_acoes".to_string());
                println!("[IRPF] Auto-healing: Linked at1 to tp_acoes");
            } else if clean_id == "at2" {
                at.tax_profile_id = Some("tax_profile:tp_futuros".to_string());
                println!("[IRPF] Auto-healing: Linked at2 to tp_futuros");
            }
        }
        println!(
            "[IRPF] ASSET_TYPE: id={}, name={}, tax_profile_id={:?}",
            at.id, at.name, at.tax_profile_id
        );
    }

    // Modalities
    let mut mod_res = db.0.query("SELECT type::string(id) as id, name, type::string(description ?? '') as description FROM modality").await.map_err(|e| e.to_string())?;
    let modalities: Vec<Modality> = mod_res.take(0).map_err(|e| e.to_string())?;

    // Tax Profiles
    let mut entry_res = db.0.query("SELECT type::string(id) as id, type::string(tax_profile_id) as tax_profile_id, type::string(modality_id) as modality_id, type::string(tax_rule_id) as tax_rule_id FROM tax_profile_entry").await.map_err(|e| e.to_string())?;
    let profile_entries: Vec<TaxProfileEntry> = entry_res.take(0).map_err(|e| e.to_string())?;

    // Tax Rules
    let mut rule_res = db.0.query("SELECT type::string(id) as id, name, trade_type, revenue_code, tax_rate, withholding_rate, withholding_basis, exemption_threshold, cumulative_losses, basis FROM tax_rule").await.map_err(|e| e.to_string())?;
    let rules: Vec<TaxRule> = rule_res.take(0).map_err(|e| e.to_string())?;

    // Accumulated Losses
    let mut loss_res = db.0.query("SELECT type::string(id) as id, trade_type, amount, balance, origin_date FROM tax_loss WHERE balance > 0").await.map_err(|e| e.to_string())?;
    let losses: Vec<TaxLoss> = loss_res.take(0).map_err(|e| e.to_string())?;

    println!(
        "[IRPF] DEBUG: Loaded {} profile_entries",
        profile_entries.len()
    );
    for (i, e) in profile_entries.iter().enumerate() {
        println!(
            "[IRPF] DEBUG: Entry {}: profile_id={}, modality_id={}, rule_id={}",
            i, e.tax_profile_id, e.modality_id, e.tax_rule_id
        );
    }

    // 2. Processing Buckets

    // NOTE: Tax Profile resolution logic is inlined in the trade processing loop below.

    // Helper: Robust ID normalization (extracts clean ID from 'table:id' or '{String: "id"}')
    let normalize_id = |label: &str, s: &str| -> String {
        let mut clean = s.to_string();
        if clean.contains("String:") {
            clean = clean.split("String:").last().unwrap_or(&clean).to_string();
        }
        clean = clean.replace(['{', '}', '"', '\'', ' ', '`'], "");
        let res = clean
            .split(':')
            .last()
            .unwrap_or(clean.as_str())
            .to_string();
        if !s.is_empty() {
            println!("[IRPF] normalize_id({}): '{}' -> '{}'", label, s, res);
        }
        res
    };

    // Helper: Find Rule for (Profile + Modality ID)
    let find_rule = |profile_id: &str, target_mod_id: &str| -> Option<TaxRule> {
        let clean_pid = normalize_id("ProfileParam", profile_id);
        let clean_target_mid = normalize_id("ModalityParam", target_mod_id);

        println!(
            "[IRPF] find_rule looking for clean_pid='{}' and clean_target_mid='{}'",
            clean_pid, clean_target_mid
        );

        let entry = profile_entries.iter().find(|e| {
            let e_pid = normalize_id("EntryProfile", &e.tax_profile_id);
            let e_mid = normalize_id("EntryModality", &e.modality_id);

            e_pid == clean_pid && e_mid == clean_target_mid
        });

        if let Some(e) = entry {
            let clean_rule_id = normalize_id("EntryRule", &e.tax_rule_id);
            let rule = rules
                .iter()
                .find(|r| normalize_id("RuleTableId", &r.id) == clean_rule_id)
                .cloned();

            if rule.is_none() {
                println!("[IRPF] Found entry for Profile {}, but Rule ID {} not found in total {} rules.", clean_pid, e.tax_rule_id, rules.len());
            } else {
                println!(
                    "[IRPF] Successfully matched Rule {} for Profile {} and Modality {}",
                    clean_rule_id, clean_pid, clean_target_mid
                );
            }
            rule
        } else {
            println!(
                "[IRPF] No Tax Profile Entry found for Profile {} and Modality {}",
                clean_pid, clean_target_mid
            );
            None
        }
    };

    // Group Trades into Buckets keyed by Tax Rule ID
    use std::collections::HashMap;
    let mut buckets: HashMap<String, RuleBucket> = HashMap::new();

    println!(
        "[IRPF] Total de trades a processar no loop final: {}",
        trades.len()
    );
    for trade in &trades {
        let mut trade_mod_id = trade.modality_id.as_deref().unwrap_or("").to_string();

        // AUTO-MODALITY DETECTION: If missing, compare entry and exit dates
        if trade_mod_id.is_empty() || trade_mod_id == "null" {
            let entry_date = trade.date.split('T').next().unwrap_or("");
            let exit_date = trade
                .exit_date
                .as_deref()
                .unwrap_or("")
                .split('T')
                .next()
                .unwrap_or("");

            if !entry_date.is_empty() && entry_date == exit_date {
                trade_mod_id = "mod1".to_string(); // Day Trade
                println!(
                    "[IRPF] Auto-detected modality for trade {}: mod1 (Day Trade)",
                    trade.id
                );
            } else {
                trade_mod_id = "mod2".to_string(); // Swing Trade
                println!(
                    "[IRPF] Auto-detected modality for trade {}: mod2 (Swing Trade)",
                    trade.id
                );
            }
        }

        println!(
            "[IRPF] Processando Trade: id={}, modality_id={}, asset_type_id={:?}",
            trade.id, trade_mod_id, trade.asset_type_id
        );

        // We will determine result_val after finding the rule to check for trade_type robustly
        let initial_result = trade.result.unwrap_or(0.0) - trade.fee_total.unwrap_or(0.0);

        // RESOLUTION LOGIC: Asset -> AssetType fallback
        let mut final_profile_id = None;

        // 1. Try Asset Specific Profile
        if let Some(aid) = &trade.asset_id {
            let clean_aid = normalize_id("TradeAssetParam", aid);
            if let Some(asset) = assets.iter().find(|a| {
                normalize_id("AssetTableId", &a.id) == clean_aid || a.symbol == trade.asset_symbol
            }) {
                if let Some(pid) = &asset.tax_profile_id {
                    final_profile_id = Some(pid.clone());
                    println!(
                        "[IRPF] Trade {} -> Tax Profile {} resolvido via Ativo",
                        trade.id,
                        normalize_id("ResolvedProfile", pid)
                    );
                }
            }
        }

        // 2. Fallback to Trade's AssetType directly if still None
        if final_profile_id.is_none() {
            if let Some(atid) = &trade.asset_type_id {
                let clean_atid = normalize_id("TradeAssetTypeParam", atid);
                if let Some(at) = asset_types
                    .iter()
                    .find(|t| normalize_id("AssetTypeTableId", &t.id) == clean_atid)
                {
                    final_profile_id = at.tax_profile_id.clone();
                    println!(
                        "[IRPF] Trade {} -> Tax Profile {} resolvido via Tipo de Ativo",
                        trade.id,
                        final_profile_id
                            .as_deref()
                            .map(|pid| normalize_id("ResolvedProfileFallback", pid))
                            .unwrap_or("None".to_string())
                    );
                }
            }
        }

        if let Some(profile_id) = final_profile_id {
            if let Some(rule) = find_rule(&profile_id, &trade_mod_id) {
                println!(
                    "[IRPF] Trade {} -> Regra Fiscal {} encontrada",
                    trade.id, rule.id
                );
                let bucket_key = format!("{}:{}", rule.id, trade_mod_id);
                let entry = buckets.entry(bucket_key).or_insert(RuleBucket {
                    rule: rule.clone(),
                    modality_id: trade_mod_id.to_string(),
                    gross_profit: 0.0,
                    gross_loss: 0.0,
                    sales_total: 0.0,
                    trade_ids: Vec::new(),
                });

                if initial_result > 0.0 {
                    entry.gross_profit += initial_result;
                } else {
                    entry.gross_loss += initial_result.abs();
                }

                if rule.trade_type != "DayTrade" {
                    if let (Some(price), Some(qty)) = (trade.exit_price, trade.quantity) {
                        let pv = trade.point_value.unwrap_or(1.0);
                        entry.sales_total += price * qty * pv;
                    }
                }
                entry.trade_ids.push(trade.id.clone());
            }
        } else {
            println!(
                "[IRPF] Trade {} skipped: No Tax Profile found (Asset: {:?}, Type: {:?})",
                trade.id, trade.asset_id, trade.asset_type_id
            );
        }
    }

    println!("[IRPF] Distribution finished. Buckets: {}", buckets.len());

    // 3. Calculate Appraisal per Bucket
    // 1. Resolve Rules (Process all buckets regardless of profit/loss to ensure we have trade_ids)
    let mut appraisals = Vec::new();

    for (rule_id, bucket) in buckets {
        let loss_category = bucket.rule.trade_type.clone();

        // 3.3 Compensation Logic: Fetch available loss
        let current_month_start = format!("{}-{:02}-01", year, month);
        let available_loss: f64 = losses
            .iter()
            .filter(|l| {
                l.trade_type == loss_category && l.origin_date < current_month_start
            })
            .map(|l| l.balance)
            .sum();

        // 3.5 IRRF Credit Carryover Logic
        let credit_query = "SELECT type::float(withholding_credit_remaining) as withholding_credit_remaining FROM tax_appraisal WHERE trade_type = $type AND (`period_year` < $year OR (`period_year` = $year AND `period_month` < $month)) ORDER BY `period_year` DESC, `period_month` DESC LIMIT 1";
        let mut credit_res =
            db.0.query(credit_query)
                .bind(("type", loss_category.clone()))
                .bind(("year", year))
                .bind(("month", month))
                .await
                .map_err(|e| e.to_string())?;

        let prev_credit_vec: Vec<serde_json::Value> =
            credit_res.take(0).map_err(|e| e.to_string())?;
        let previous_credit = if let Some(v) = prev_credit_vec.first() {
            v.get("withholding_credit_remaining")
                .and_then(|t| t.as_f64())
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // 3.5 Accumulation Logic (< R$ 10)
        let accum_query = "SELECT type::float(total_payable) as total_payable FROM tax_appraisal WHERE trade_type = $type AND status = 'Pending' AND total_payable < 10.0 AND (`period_year` < $year OR (`period_year` = $year AND `period_month` < $month)) ORDER BY `period_year` DESC, `period_month` DESC LIMIT 1";
        let mut accum_res =
            db.0.query(accum_query)
                .bind(("type", loss_category.clone()))
                .bind(("year", year))
                .bind(("month", month))
                .await
                .map_err(|e| e.to_string())?;

        let prev_accum: Vec<serde_json::Value> = accum_res.take(0).map_err(|e| e.to_string())?;
        let tax_accumulated = if let Some(v) = prev_accum.first() {
            v.get("total_payable")
                .and_then(|t| t.as_f64())
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let mut appraisal = crate::logic::calculate_appraisal(
            &bucket,
            month,
            year,
            available_loss,
            previous_credit,
            tax_accumulated,
        );
        appraisal.tax_rule_id = rule_id; 
        appraisal.calculation_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        appraisals.push(appraisal);
    }

    println!("DEBUG: Generated {} appraisals (Profile)", appraisals.len());
    Ok(appraisals)
}

/// Lists saved appraisals
#[tauri::command]
pub async fn get_appraisals(
    db: State<'_, DbState>,
    year: Option<u16>,
) -> Result<Vec<TaxAppraisal>, String> {
    // PRE-MIGRATION: Robust multi-stage field renaming
    let migrations = vec![
        "UPDATE tax_appraisal SET period_month = `month`, period_year = `year`, `month` = NONE, `year` = NONE WHERE `month` != NONE",
        "UPDATE tax_appraisal SET period_month = appraisal_month, period_year = appraisal_year, appraisal_month = NONE, appraisal_year = NONE WHERE appraisal_month != NONE"
    ];
    for m in migrations {
        let _ = db.0.query(m).await;
    }

    let base_select = "SELECT \
        type::string(id) as id, \
        type::number(period_month) as period_month, \
        type::number(period_year) as period_year, \
        trade_type, \
        (IF tax_rule_id != NONE THEN type::string(tax_rule_id) ELSE null END) as tax_rule_id, \
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
        type::string(calculation_date) as calculation_date, \
        status, \
        trade_ids \
        FROM tax_appraisal ";

    let query = if let Some(y) = year {
        format!("{} WHERE `period_year` = {} OR status = 'Pending' ORDER BY `period_year` DESC, `period_month` DESC", base_select, y)
    } else {
        format!(
            "{} ORDER BY `period_year` DESC, `period_month` DESC",
            base_select
        )
    };

    let mut result = db.0.query(query).await.map_err(|e| e.to_string())?;
    let appraisals: Vec<TaxAppraisal> = result.take(0).map_err(|e| {
        println!(
            "[ERROR] get_appraisals (irpf) deserialization failure: {}",
            e
        );
        e.to_string()
    })?;

    Ok(appraisals)
}

#[tauri::command]
pub async fn save_appraisal(
    db: State<'_, DbState>,
    data: TaxAppraisal,
) -> Result<TaxAppraisal, String> {
    // --- NEW: LOSS RESTORATION (Undo previous compensation if updating) ---
    let query_existing = "SELECT *, type::string(id) as id FROM tax_appraisal WHERE `period_month` = $month AND `period_year` = $year AND trade_type = $type AND status = 'Pending' LIMIT 1";
    let mut check_res =
        db.0.query(query_existing)
            .bind(("month", data.period_month))
            .bind(("year", data.period_year))
            .bind(("type", data.trade_type.clone()))
            .await
            .map_err(|e| e.to_string())?;

    let existing_opt: Option<TaxAppraisal> = check_res.take(0).map_err(|e| e.to_string())?;

    if let Some(existing) = existing_opt {
        if existing.compensated_loss > 0.0 {
            println!(
                "[IRPF] Restoring {} losses from existing appraisal before update",
                existing.compensated_loss
            );
            let mut to_restore = existing.compensated_loss;

            // Restore LIFO (Newest first, reverse of FIFO usage)
            let loss_query = "SELECT *, type::string(id) as id FROM tax_loss WHERE trade_type = $type AND balance < amount ORDER BY origin_date DESC";
            let mut loss_result =
                db.0.query(loss_query)
                    .bind(("type", data.trade_type.clone()))
                    .await
                    .map_err(|e| e.to_string())?;

            let losses_to_restore: Vec<TaxLoss> = loss_result.take(0).map_err(|e| e.to_string())?;
            for mut loss_record in losses_to_restore {
                if to_restore <= 0.0 {
                    break;
                }

                let can_restore = loss_record.amount - loss_record.balance;
                let restore_amount = if can_restore >= to_restore {
                    to_restore
                } else {
                    can_restore
                };

                loss_record.balance += restore_amount;
                to_restore -= restore_amount;

                if let Some(id_val) = &loss_record.id.clone() {
                    let parts: Vec<&str> = id_val.split(':').collect();
                    let l_tb = if parts.len() > 1 {
                        parts[0]
                    } else {
                        "tax_loss"
                    };
                    let l_id = if parts.len() > 1 { parts[1] } else { id_val };
                    loss_record.id = None;
                    let _: Option<serde_json::Value> =
                        db.0.update((l_tb, l_id))
                            .content(loss_record)
                            .await
                            .map_err(|e| e.to_string())?;
                }
            }
        }
    }

    // --- LOSS COMPENSATION LOGIC (Deducting new value) ---
    if data.compensated_loss > 0.0 {
        let mut remaining_compensation = data.compensated_loss;

        let loss_query = "SELECT *, type::string(id) as id FROM tax_loss WHERE balance > 0 AND trade_type = $type ORDER BY origin_date ASC";
        let mut loss_result =
            db.0.query(loss_query)
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
            if let Some(id_val) = &loss_record.id.clone() {
                let parts: Vec<&str> = id_val.split(':').collect();
                let l_tb = if parts.len() > 1 {
                    parts[0]
                } else {
                    "tax_loss"
                };
                let l_id = if parts.len() > 1 { parts[1] } else { id_val };
                loss_record.id = None;
                let _: Option<serde_json::Value> =
                    db.0.update((l_tb, l_id))
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
        let mut check_result =
            db.0.query(check_query)
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
                let l_tb = if parts.len() > 1 {
                    parts[0]
                } else {
                    "tax_loss"
                };
                let l_id = if parts.len() > 1 { parts[1] } else { lid };
                loss.id = None;

                let _: Option<TaxLoss> =
                    db.0.update((l_tb, l_id))
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

            let _: Option<TaxLoss> =
                db.0.create("tax_loss")
                    .content(new_loss)
                    .await
                    .map_err(|e| e.to_string())?;
        }
    }
    // --- END LOSS COMPENSATION LOGIC ---

    // 1. If ID is present, update existing
    if let Some(id) = &data.id {
        let parts: Vec<&str> = id.split(':').collect();
        let tb = if parts.len() > 1 {
            parts[0].to_string()
        } else {
            "tax_appraisal".to_string()
        };
        let tid = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            id.to_string()
        };

        let mut content_for_update = data.clone();
        content_for_update.id = None;

        let query_str = "UPDATE type::thing($tb, $id) MERGE $data;";
        let mut update_res =
            db.0.query(query_str)
                .bind(("tb", tb.clone()))
                .bind(("id", tid.clone()))
                .bind(("data", content_for_update))
                .await
                .map_err(|e| {
                    println!("[IRPF] Error updating appraisal SDK: {}", e);
                    e.to_string()
                })?;

        let updated: Option<TaxAppraisal> = update_res.take(0).map_err(|e| e.to_string())?;

        if updated.is_none() {
            println!(
                "[IRPF] Warning: SDK update returned None for {}:{}",
                tb, tid
            );
            // Fallback to CREATE if UPDATE failed (Upsert behavior)
            let created: Option<TaxAppraisal> =
                db.0.create(tb)
                    .content(data.clone())
                    .await
                    .map_err(|e| e.to_string())?;
            return created.ok_or_else(|| "Failed to upsert appraisal".to_string());
        }
        return Ok(updated.unwrap());
    }

    // 2. If no ID, check if one exists for this Month/Year/Type to avoid duplicates
    // CRITICAL: Only match PENDING appraisals. If a PAID one exists, we create a COMPLEMENTARY record.
    let query = "SELECT *, type::string(id) as id FROM tax_appraisal WHERE `period_month` = $month AND `period_year` = $year AND trade_type = $type AND status = 'Pending'";
    let mut result =
        db.0.query(query)
            .bind(("month", data.period_month))
            .bind(("year", data.period_year))
            .bind(("type", data.trade_type.clone()))
            .await
            .map_err(|e| e.to_string())?;

    let existing: Option<TaxAppraisal> = result.take(0).map_err(|e| e.to_string())?;

    if let Some(existing_record) = existing {
        if let Some(id) = &existing_record.id {
            let parts: Vec<&str> = id.split(':').collect();
            let tb = if parts.len() > 1 {
                parts[0]
            } else {
                "tax_appraisal"
            };
            let tid = if parts.len() > 1 { parts[1] } else { id };

            println!(
                "[IRPF] Updating EXISTING record found by month/year/type: {}:{}",
                tb, tid
            );

            let mut new_data = data.clone();
            new_data.id = None;

            let updated: Option<TaxAppraisal> =
                db.0.update((tb, tid))
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

    let created_opt: Option<TaxAppraisal> =
        db.0.create("tax_appraisal")
            .content(new_data)
            .await
            .map_err(|e| e.to_string())?;

    let created = created_opt.ok_or_else(|| "Failed to create new appraisal".to_string())?;

    // --- NEW: ACCUMULATION MERGE / COMPLEMENTARY DETECTION ---
    // If a PAID appraisal already exists for this same period/type, this NEW one is complementary.
    let comp_query = "SELECT count() FROM tax_appraisal WHERE `period_month` = $month AND `period_year` = $year AND trade_type = $type AND status = 'Paid'";
    let mut comp_res = db.0.query(comp_query)
        .bind(("month", created.period_month))
        .bind(("year", created.period_year))
        .bind(("type", created.trade_type.clone()))
        .await
        .map_err(|e| e.to_string())?;
    
    let paid_count: i64 = comp_res.take::<Option<i64>>(0).map_err(|e| e.to_string())?.unwrap_or(0);
    
    if paid_count > 0 {
        if let Some(id_str) = created.id.as_ref() {
            let clean_id = id_str.split(':').last().unwrap_or(id_str).to_string();
            println!("[IRPF] Marking NEW appraisal as COMPLEMENTARY because a Paid one exists");
            let update_comp = "UPDATE type::thing('tax_appraisal', $id) SET is_complementary = true";
            let _ = db.0.query(update_comp)
                .bind(("id", clean_id))
                .await;
        }
    }

    if created.tax_accumulated > 0.0 {
        let mark_query = "UPDATE tax_appraisal SET status = 'Paid', calculation_date = $now WHERE trade_type = $type AND status = 'Pending' AND total_payable < 10.0 AND (`period_year` < $year OR (`period_year` = $year AND `period_month` < $month))";
        db.0.query(mark_query)
            .bind(("type", created.trade_type.clone()))
            .bind(("year", created.period_year))
            .bind(("month", created.period_month))
            .bind((
                "now",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            ))
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(created)
}

#[tauri::command]
pub async fn get_accumulated_losses(db: State<'_, DbState>) -> Result<Vec<TaxLoss>, String> {
    let query = "SELECT \
        type::string(id) as id, \
        trade_type, \
        type::float(amount) as amount, \
        type::float(balance) as balance, \
        type::string(origin_date) as origin_date \
        FROM tax_loss WHERE balance > 0 ORDER BY origin_date ASC";

    let mut result = db.0.query(query).await.map_err(|e| e.to_string())?;

    let losses: Vec<TaxLoss> = result.take(0).map_err(|e| e.to_string())?;

    Ok(losses)
}

/// Deletes an appraisal and its associated tax loss if applicable
#[tauri::command]
pub async fn delete_appraisal(db: State<'_, DbState>, id: String) -> Result<(), String> {
    // 1. Fetch the appraisal to check if it generated a loss
    let parts: Vec<&str> = id.split(':').collect();
    let tb = if parts.len() > 1 {
        parts[0]
    } else {
        "tax_appraisal"
    };
    let tid = if parts.len() > 1 { parts[1] } else { &id };

    let appraisal: Option<TaxAppraisal> =
        db.0.select((tb, tid)).await.map_err(|e| e.to_string())?;

    if let Some(appraisal) = appraisal {
        // 2. If it generated a loss, delete the corresponding TaxLoss record
        if appraisal.net_profit < 0.0 {
            let origin_date = format!("{}-{:02}-01", appraisal.period_year, appraisal.period_month);

            // Find the tax_loss record
            let query = "SELECT * FROM tax_loss WHERE origin_date = $date AND trade_type = $type";
            let mut result =
                db.0.query(query)
                    .bind(("date", origin_date))
                    .bind(("type", appraisal.trade_type.clone()))
                    .await
                    .map_err(|e| e.to_string())?;

            let loss_record: Option<TaxLoss> = result.take(0).map_err(|e| e.to_string())?;

            if let Some(loss) = loss_record {
                if let Some(lid) = loss.id {
                    let parts: Vec<&str> = lid.split(':').collect();
                    let l_tb = if parts.len() > 1 {
                        parts[0]
                    } else {
                        "tax_loss"
                    };
                    let l_id = if parts.len() > 1 { parts[1] } else { &lid };

                    let _: Option<TaxLoss> =
                        db.0.delete((l_tb, l_id)).await.map_err(|e| e.to_string())?;
                }
            }
        }

        // 3. Delete associated DARFs to prevent orphans (but block if any is Paid)
        let darf_query = "SELECT *, type::string(id) as id FROM tax_darf WHERE appraisal_id = $id";
        let mut darf_result =
            db.0.query(darf_query)
                .bind(("id", id.clone()))
                .await
                .map_err(|e| e.to_string())?;

        let darfs: Vec<TaxDarf> = darf_result.take(0).map_err(|e| e.to_string())?;

        // CHECK: If any DARF is paid, block appraisal deletion
        if darfs.iter().any(|d| d.status == "Paid") {
            return Err("Não é possível excluir uma apuração que possui DARF marcada como Paga. Estorne o pagamento primeiro.".to_string());
        }

        for darf in darfs {
            if let Some(did) = darf.id {
                let parts: Vec<&str> = did.split(':').collect();
                let d_tb = if parts.len() > 1 {
                    parts[0]
                } else {
                    "tax_darf"
                };
                let d_id = if parts.len() > 1 { parts[1] } else { &did };

                let _: Option<TaxDarf> =
                    db.0.delete((d_tb, d_id))
                        .await
                        .map_err(|e: surrealdb::Error| e.to_string())?;
            }
        }

        // 4. Delete the appraisal
        let parts: Vec<&str> = id.split(':').collect();
        let tb = if parts.len() > 1 {
            parts[0]
        } else {
            "tax_appraisal"
        };
        let tid = if parts.len() > 1 { parts[1] } else { &id };

        let _: Option<TaxAppraisal> = db.0.delete((tb, tid)).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Generates a DARF from an appraisal
#[tauri::command]
pub async fn generate_darf(
    db: State<'_, DbState>,
    appraisal_id: String,
) -> Result<TaxDarf, String> {
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

    let appraisal_res: Option<TaxAppraisal> = db.0.select((tb, tid)).await.map_err(|e| {
        println!("DEBUG: Failed to fetch appraisal: {}", e);
        e.to_string()
    })?;

    let appraisal = appraisal_res.ok_or("Appraisal not found")?;
    println!("DEBUG: Found appraisal for DARF: {:?}", appraisal);

    if appraisal.is_exempt {
        return Err("Esta apuração está marcada como ISENTA. Não há imposto a pagar.".to_string());
    }

    if appraisal.total_payable < 10.0 {
        return Err("Valor total (incluindo acumulados) inferior a R$ 10,00. Não é necessário gerar DARF ainda.".to_string());
    }

    // Use revenue code from appraisal with fallback for older records
    let revenue_code = if !appraisal.revenue_code.is_empty() {
        appraisal.revenue_code.clone()
    } else {
        // Fallback for old records: 6015 is standard for individuals in stock market
        "6015".to_string()
    };

    let period = format!("{:02}/{}", appraisal.period_month, appraisal.period_year);

    println!("DEBUG: Checking existing DARFs for this Appraisal ID...");
    // Identify if a DARF already exists for THIS appraisal
    let check_query = "SELECT *, \
        type::string(id) as id, \
        (IF appraisal_id != NONE THEN type::string(appraisal_id) ELSE null END) as appraisal_id, \
        (IF account_id != NONE THEN type::string(account_id) ELSE null END) as account_id, \
        (IF transaction_id != NONE THEN type::string(transaction_id) ELSE null END) as transaction_id \
        FROM tax_darf WHERE (IF appraisal_id != NONE THEN type::string(appraisal_id) ELSE '' END) = $appraisal_id";
    let mut check_result =
        db.0.query(check_query)
            .bind(("appraisal_id", appraisal_id.clone()))
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
        is_complementary: appraisal.is_complementary,
    };

    let created: Option<TaxDarf> = db.0.create("tax_darf").content(darf).await.map_err(|e| {
        println!("DEBUG: Failed to create DARF: {}", e);
        e.to_string()
    })?;

    if let Some(ref d) = created {
        println!("DEBUG: DARF generated successfully with ID: {:?}", d.id);
    }

    created.ok_or_else(|| "Failed to generate DARF".to_string())
}

/// Lists DARFs
#[tauri::command]
pub async fn get_darfs(db: State<'_, DbState>, year: Option<u16>) -> Result<Vec<TaxDarf>, String> {
    println!("DEBUG: get_darfs called for year: {:?}", year);
    let query = if let Some(y) = year {
        // ALWAYS fetch Pending DARFs to ensure they don't disappear when filtering by history year
        format!("SELECT *, \
            type::string(id) as id, \
            (IF appraisal_id != NONE THEN type::string(appraisal_id) ELSE null END) as appraisal_id, \
            (IF account_id != NONE THEN type::string(account_id) ELSE null END) as account_id, \
            (IF transaction_id != NONE THEN type::string(transaction_id) ELSE null END) as transaction_id \
            FROM tax_darf WHERE string::ends_with(period, '{}') OR status = 'Pending' ORDER BY due_date DESC", y)
    } else {
        "SELECT *, \
            type::string(id) as id, \
            (IF appraisal_id != NONE THEN type::string(appraisal_id) ELSE null END) as appraisal_id, \
            (IF account_id != NONE THEN type::string(account_id) ELSE null END) as account_id, \
            (IF transaction_id != NONE THEN type::string(transaction_id) ELSE null END) as transaction_id \
            FROM tax_darf ORDER BY due_date DESC".to_string()
    };

    let mut result = db.0.query(query).await.map_err(|e| e.to_string())?;
    let darfs: Vec<TaxDarf> = result.take(0).map_err(|e| {
        println!("[ERROR] get_darfs (irpf) deserialization failure: {}", e);
        e.to_string()
    })?;

    println!("DEBUG: Found {} DARFs", darfs.len());
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
    account_id: String,     // New param
    transaction_id: String, // New param
) -> Result<TaxDarf, String> {
    let parts: Vec<&str> = id.split(':').collect();
    let tb = if parts.len() > 1 {
        parts[0]
    } else {
        "tax_darf"
    };
    let tid = if parts.len() > 1 { parts[1] } else { &id };

    let darf_res: Option<TaxDarf> = db.0.select((tb, tid)).await.map_err(|e| e.to_string())?;

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
    let clean_id = id.split(':').last().unwrap_or(&id).to_string();
    let update_query = "UPDATE type::thing('tax_darf', $id) MERGE $content";
    let mut update_result =
        db.0.query(update_query)
            .bind(("id", clean_id.clone()))
            .bind(("content", darf_content))
            .await
            .map_err(|e| e.to_string())?;

    let updated: Option<TaxDarf> = update_result.take(0).map_err(|e| e.to_string())?;

    // 2. Update Appraisal Record
    let appraisal_id = darf.appraisal_id.clone();
    let parts: Vec<&str> = appraisal_id.split(':').collect();
    let aid_val = if parts.len() > 1 {
        parts[1]
    } else {
        &appraisal_id
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
            let clear_query = "UPDATE tax_appraisal SET status = 'Paid' WHERE trade_type = $type AND status = 'Pending' AND (`period_year` < $year OR (`period_year` = $year AND `period_month` < $month))";

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
    let description = format!("Pagamento DARF {} ({})", darf.period, darf.revenue_code);

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

    updated.ok_or_else(|| "Failed to update DARF".to_string())
}

#[tauri::command]
pub async fn diagnostic_dump_darfs(db: State<'_, DbState>) -> Result<(), String> {
    println!("[DIAGNOSTIC] Dumping all TAX_DARF records...");
    let mut result = db.0.query("SELECT *, type::string(id) as id, type::string(transaction_id) as tx_id_str FROM tax_darf").await.map_err(|e| e.to_string())?;
    let darfs: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;
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
pub async fn get_appraisal_by_id(
    db: State<'_, DbState>,
    id: String,
) -> Result<Option<TaxAppraisal>, String> {
    let clean_id = id.split(':').last().unwrap_or(&id).to_string();
    let query = "SELECT *, type::string(id) as id, type::string(tax_rule_id) as tax_rule_id FROM tax_appraisal WHERE type::string(id) = $id LIMIT 1";
    let mut result =
        db.0.query(query)
            .bind(("id", format!("tax_appraisal:{}", clean_id)))
            .await
            .map_err(|e| e.to_string())?;

    Ok(result.take(0).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn get_darf_by_id(db: State<'_, DbState>, id: String) -> Result<Option<TaxDarf>, String> {
    let clean_id = id.split(':').last().unwrap_or(&id).to_string();
    let query = "SELECT *, \
        type::string(id) as id, \
        (IF appraisal_id != NONE THEN type::string(appraisal_id) ELSE null END) as appraisal_id, \
        (IF account_id != NONE THEN type::string(account_id) ELSE null END) as account_id, \
        (IF transaction_id != NONE THEN type::string(transaction_id) ELSE null END) as transaction_id \
        FROM tax_darf WHERE type::string(id) = $id LIMIT 1";
    let mut result =
        db.0.query(query)
            .bind(("id", format!("tax_darf:{}", clean_id)))
            .await
            .map_err(|e| e.to_string())?;

    Ok(result.take(0).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn get_darf_by_transaction(
    db: State<'_, DbState>,
    transaction_id: String,
) -> Result<Option<TaxDarf>, String> {
    println!(
        "[DARF LOOKUP] Looking for DARF with transaction_id: {}",
        transaction_id
    );

    // 1. Try Direct Lookup (Normal way)
    let clean_id = transaction_id
        .split(':')
        .last()
        .unwrap_or(&transaction_id)
        .trim_matches('`')
        .to_string();
    let prefixed_id = format!("cash_transaction:{}", clean_id);

    let query = "SELECT *, \
        type::string(id) as id, \
        (IF appraisal_id != NONE THEN type::string(appraisal_id) ELSE null END) as appraisal_id, \
        (IF account_id != NONE THEN type::string(account_id) ELSE null END) as account_id, \
        (IF transaction_id != NONE THEN type::string(transaction_id) ELSE null END) as transaction_id \
        FROM tax_darf WHERE type::string(transaction_id) = $clean OR type::string(transaction_id) = $prefixed LIMIT 1";
    let mut result =
        db.0.query(query)
            .bind(("clean", clean_id.clone()))
            .bind(("prefixed", prefixed_id))
            .await
            .map_err(|e| e.to_string())?;

    if let Ok(Some(darf)) = result.take::<Option<TaxDarf>>(0) {
        println!("[DARF LOOKUP] Found via direct ID lookup");
        return Ok(Some(darf));
    }

    // 2. Fallback: Search by description parsing (For historical or unlinked records)
    // We need to fetch the transaction first to get its description
    let tx_res: Option<serde_json::Value> =
        db.0.select(("cash_transaction", &clean_id))
            .await
            .map_err(|e| e.to_string())?;

    if let Some(tx) = tx_res {
        if let Some(desc) = tx.get("description").and_then(|v| v.as_str()) {
            println!("[DARF LOOKUP] Fallback: Parsing description '{}'", desc);

            // Expected patterns: "Pagamento DARF MM/YYYY (CODE)" OR "Estorno - Pagamento DARF MM/YYYY (CODE)"
            if desc.contains("DARF") {
                // Find period (MM/YYYY) - Convert to String to fix borrowing error
                let period_match = desc
                    .split_whitespace()
                    .find(|s| s.contains('/') && s.len() >= 7)
                    .map(|s| s.to_string());
                // Find code (numbers inside parentheses or as a 4-digit token)
                let code_match = desc
                    .split_whitespace()
                    .find(|s| {
                        s.len() >= 4 && s.chars().all(|c| c.is_numeric() || c == '(' || c == ')')
                    })
                    .map(|s| s.trim_matches('(').trim_matches(')').to_string());

                if let (Some(period), Some(code)) = (period_match, code_match) {
                    println!(
                        "[DARF LOOKUP] Fallback: Searching for period={} code={}",
                        period, code
                    );
                    let fallback_query = "SELECT *, \
                        type::string(id) as id, \
                        (IF appraisal_id != NONE THEN type::string(appraisal_id) ELSE null END) as appraisal_id, \
                        (IF account_id != NONE THEN type::string(account_id) ELSE null END) as account_id, \
                        (IF transaction_id != NONE THEN type::string(transaction_id) ELSE null END) as transaction_id \
                        FROM tax_darf WHERE period = $period AND revenue_code = $code LIMIT 1";
                    let mut fb_result =
                        db.0.query(fallback_query)
                            .bind(("period", period))
                            .bind(("code", code))
                            .await
                            .map_err(|e| e.to_string())?;

                    if let Ok(Some(darf)) = fb_result.take::<Option<TaxDarf>>(0) {
                        println!("[DARF LOOKUP] Found via description fallback patterns");
                        return Ok(Some(darf));
                    }
                }
            }
        }
    }

    println!("[DARF LOOKUP] Not found after all attempts");
    Ok(None)
}

/// Reverses a DARF payment (Unpay)
#[tauri::command]
pub async fn unpay_darf(db: State<'_, DbState>, id: String) -> Result<TaxDarf, String> {
    let parts: Vec<&str> = id.split(':').collect();
    let tb = if parts.len() > 1 {
        parts[0]
    } else {
        "tax_darf"
    };
    let tid = if parts.len() > 1 { parts[1] } else { &id };

    let darf_res: Option<TaxDarf> = db.0.select((tb, tid)).await.map_err(|e| e.to_string())?;

    let mut darf = darf_res.ok_or("DARF not found")?;

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
            darf.period, darf.revenue_code
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
    let appraisal_id = darf.appraisal_id.clone();
    let parts: Vec<&str> = appraisal_id.split(':').collect();
    let aid_val = if parts.len() > 1 {
        parts[1]
    } else {
        &appraisal_id
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
                AND (`period_year` < $year OR (`period_year` = $year AND `period_month` < $month))";

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

    updated.ok_or_else(|| "Failed to unpay DARF".to_string())
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

// --- TAX RULES & MAPPINGS (New) ---

#[tauri::command]
pub async fn get_tax_rules(db: State<'_, DbState>) -> Result<Vec<TaxRule>, String> {
    println!("[IRPF] Fetching tax_rules...");
    let mut result = match db
        .0
        .query("SELECT *, type::string(id) as id FROM tax_rule")
        .await
    {
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
    println!("[IRPF] save_tax_rule START for ID: {:?}", rule.id);
    let mut json = serde_json::to_value(&rule).map_err(|e| e.to_string())?;

    let id_str = rule.id.clone();
    let clean_id = id_str.split(':').last().unwrap_or(&id_str).to_string();

    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }

    // Use raw query for robust serialization with custom IdVisitor
    let mut response =
        db.0.query("UPSERT type::thing('tax_rule', $id) CONTENT $data")
            .bind(("id", clean_id))
            .bind(("data", json))
            .await
            .map_err(|e| {
                println!("[IRPF_ERROR] UPSERT Failed on tax_rule: {}", e);
                e.to_string()
            })?;

    let saved: TaxRule = response
        .take::<Option<TaxRule>>(0)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to save tax rule".to_string())?;
    Ok(saved)
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
pub async fn get_tax_mappings(db: State<'_, DbState>) -> Result<Vec<TaxMapping>, String> {
    let mut result = db.0.query("SELECT *, type::string(id) as id, type::string(asset_type_id) as asset_type_id, type::string(modality_id) as modality_id, type::string(tax_rule_id) as tax_rule_id FROM tax_mapping").await.map_err(|e| e.to_string())?;
    let mappings: Vec<TaxMapping> = result.take(0).map_err(|e| e.to_string())?;
    Ok(mappings)
}

#[tauri::command]
pub async fn save_tax_mapping(
    db: State<'_, DbState>,
    mapping: TaxMapping,
) -> Result<TaxMapping, String> {
    let mut json = serde_json::to_value(&mapping).map_err(|e| e.to_string())?;

    let id_str = mapping.id.clone();
    let clean_id = id_str.split(':').last().unwrap_or(&id_str).to_string();

    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }

    // Use raw query for robust serialization with custom IdVisitor
    let mut response =
        db.0.query("UPSERT type::thing('tax_mapping', $id) CONTENT $data")
            .bind(("id", clean_id))
            .bind(("data", json))
            .await
            .map_err(|e| e.to_string())?;

    let saved: TaxMapping = response
        .take::<Option<TaxMapping>>(0)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to save tax mapping".to_string())?;
    Ok(saved)
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
pub async fn get_tax_profiles(db: State<'_, DbState>) -> Result<Vec<TaxProfile>, String> {
    let mut result =
        db.0.query("SELECT *, type::string(id) as id FROM tax_profile")
            .await
            .map_err(|e| e.to_string())?;
    let profiles: Vec<TaxProfile> = result.take(0).map_err(|e| e.to_string())?;
    Ok(profiles)
}

#[tauri::command]
pub async fn save_tax_profile(
    db: State<'_, DbState>,
    profile: TaxProfile,
) -> Result<TaxProfile, String> {
    let mut json = serde_json::to_value(&profile).map_err(|e| e.to_string())?;

    let id_str = profile.id.clone();
    let clean_id = id_str.split(':').last().unwrap_or(&id_str).to_string();

    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }

    // Use raw query for robust serialization with custom IdVisitor
    let mut response =
        db.0.query("UPSERT type::thing('tax_profile', $id) CONTENT $data")
            .bind(("id", clean_id))
            .bind(("data", json))
            .await
            .map_err(|e| e.to_string())?;

    let saved: TaxProfile = response
        .take::<Option<TaxProfile>>(0)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to save tax profile".to_string())?;
    Ok(saved)
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

#[tauri::command]
pub async fn get_tax_profile_entries(
    db: State<'_, DbState>,
    profile_id: Option<String>,
) -> Result<Vec<TaxProfileEntry>, String> {
    // Fallsback to format! to resolve persistent lifetime issues with bind in this async context
    let query = if let Some(pid) = profile_id {
        format!("SELECT *, type::string(id) as id, type::string(tax_profile_id) as tax_profile_id, type::string(modality_id) as modality_id, type::string(tax_rule_id) as tax_rule_id FROM tax_profile_entry WHERE tax_profile_id = '{}'", pid)
    } else {
        "SELECT *, type::string(id) as id, type::string(tax_profile_id) as tax_profile_id, type::string(modality_id) as modality_id, type::string(tax_rule_id) as tax_rule_id FROM tax_profile_entry".to_string()
    };

    let mut result = db.0.query(query).await.map_err(|e| e.to_string())?;

    let entries: Vec<TaxProfileEntry> = result.take(0).map_err(|e| e.to_string())?;
    Ok(entries)
}

#[tauri::command]
pub async fn save_tax_profile_entry(
    db: State<'_, DbState>,
    entry: TaxProfileEntry,
) -> Result<TaxProfileEntry, String> {
    let mut json = serde_json::to_value(&entry).map_err(|e| e.to_string())?;

    let id_str = entry.id.clone();
    let clean_id = id_str.split(':').last().unwrap_or(&id_str).to_string();

    if let Some(obj) = json.as_object_mut() {
        obj.remove("id");
    }

    // Use raw query for robust serialization with custom IdVisitor
    let mut response =
        db.0.query("UPSERT type::thing('tax_profile_entry', $id) CONTENT $data")
            .bind(("id", clean_id))
            .bind(("data", json))
            .await
            .map_err(|e| e.to_string())?;

    let saved: TaxProfileEntry = response
        .take::<Option<TaxProfileEntry>>(0)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to save tax profile entry".to_string())?;
    Ok(saved)
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
