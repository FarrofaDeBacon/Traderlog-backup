use crate::db::DbState;
use crate::models::{Asset, Trade};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tauri::State;
use std::fs;
use encoding_rs::WINDOWS_1252;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ProfitLine {
    symbol: String,
    entry_date: String,
    exit_date: Option<String>,
    quantity: f64,
    direction: String,
    entry_price: f64,
    exit_price: Option<f64>,
    result: f64,
    line_num: usize,
}

#[tauri::command]
pub async fn import_profit_trades(
    db: State<'_, DbState>,
    file_path: String,
    account_id: String,
) -> Result<String, String> {
    let bytes = fs::read(&file_path).map_err(|e| format!("Falha ao ler arquivo: {}", e))?;
    let (content, _, _) = WINDOWS_1252.decode(&bytes);

    let lines: Vec<&str> = content.lines().collect();
    
    // Find header
    let header_line_index = lines
        .iter()
        .position(|&line| line.trim().to_lowercase().starts_with("ativo;"));

    let start_index = match header_line_index {
        Some(index) => index + 1,
        None => return Err("Cabeçalho 'Ativo' não encontrado no arquivo CSV.".to_string()),
    };

    let mut raw_lines = Vec::new();

    for (line_num, line) in lines.iter().enumerate().skip(start_index) {
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() < 17 {
            continue;
        }

        let symbol = fields[0].trim().to_uppercase();
        if symbol.is_empty() {
            continue;
        }

        let entry_date_str = fields[1].trim();
        let exit_date_str = fields[2].trim();
        let quantity: f64 = fields[4].trim().replace(".", "").replace(",", ".").parse().unwrap_or(0.0);
        let side = fields[6].trim(); // "C" or "V"
        let entry_price_raw = fields[7].trim();
        let exit_price_raw = fields[8].trim();
        let result_raw = fields[13].trim();

        if quantity == 0.0 { continue; }

        let entry_date = parse_profit_datetime(entry_date_str)
            .map_err(|e| format!("Linha {}: {}", line_num + 1, e))?;
        let exit_date = parse_profit_datetime(exit_date_str).ok();
        
        let entry_price = parse_profit_currency(entry_price_raw)?;
        let exit_price = parse_profit_currency(exit_price_raw).ok();
        let result = parse_profit_currency(result_raw).unwrap_or(0.0);

        let direction = if side == "C" { "Buy".to_string() } else { "Sell".to_string() };

        raw_lines.push(ProfitLine {
            symbol,
            entry_date,
            exit_date,
            quantity,
            direction,
            entry_price,
            exit_price,
            result,
            line_num: line_num + 1,
        });
    }

    // Grouping by (Symbol, Account, EntryDate (Minute Window), Direction)
    let mut groups: HashMap<(String, String, String, String), Vec<ProfitLine>> = HashMap::new();
    for line in raw_lines {
        // Use slice to remove seconds for grouping (fuzzy grouping within the same minute)
        // ISO string: "2024-05-15T09:30:15Z" -> "2024-05-15T09:30"
        let entry_minute = if line.entry_date.len() >= 16 {
            line.entry_date[..16].to_string()
        } else {
            line.entry_date.clone()
        };
        
        let key = (line.symbol.clone(), account_id.clone(), entry_minute, line.direction.clone());
        groups.entry(key).or_default().push(line);
    }

    let mut imported_count = 0;
    for ((symbol, _, entry_date, direction), group_lines) in groups {
        // Aggregate values
        let total_quantity: f64 = group_lines.iter().map(|l| l.quantity).sum();
        let total_result: f64 = group_lines.iter().map(|l| l.result).sum();
        
        // Weighted average for prices
        let entry_price = group_lines.iter().map(|l| l.entry_price * l.quantity).sum::<f64>() / total_quantity;
        let exit_price = group_lines.iter().filter_map(|l| l.exit_price.map(|p| p * l.quantity)).sum::<f64>() / total_quantity;
        
        // Latest exit date
        let exit_date = group_lines.iter().filter_map(|l| l.exit_date.clone()).max();

        // Partials
        let partial_exits: Vec<serde_json::Value> = group_lines
            .iter()
            .map(|l| {
                serde_json::json!({
                    "date": l.exit_date.clone().unwrap_or_else(|| l.entry_date.clone()).replace(" ", "T").chars().take(16).collect::<String>(),
                    "price": l.exit_price.unwrap_or(l.entry_price),
                    "quantity": l.quantity,
                    "result": l.result,
                    "type": "exit",
                    "notes": format!("Importado da linha {}", l.line_num)
                })
            })
            .collect();

        // 1. Get or Create Asset
        let asset_id = get_or_create_asset(&db.0, &symbol).await
            .map_err(|e| format!("Erro ao processar ativo {}: {}", symbol, e))?;

        // 2. Create Trade Record
        let trade_id = uuid::Uuid::new_v4().to_string();
        let trade = Trade {
            id: format!("trade:⟨{}⟩", trade_id),
            date: entry_date,
            asset_symbol: symbol.clone(),
            asset_type_id: Some("asset_type:at2".to_string()),
            strategy_id: Some("strategy:manual".to_string()),
            account_id: Some(account_id.clone()),
            result: total_result,
            quantity: total_quantity,
            direction: direction.clone(),
            entry_price,
            exit_price: Some(exit_price),
            exit_date,
            fee_total: 0.0,
            notes: format!("Importado do Profit (Agrupado de {} execuções)", group_lines.len()),
            timeframe: "1m".to_string(),
            volatility: "Medium".to_string(),
            entry_emotional_state_id: None,
            entry_emotional_state_name: None,
            exit_reason: Some("Imported".to_string()),
            exit_emotional_state_id: None,
            exit_emotional_state_name: None,
            entry_rationale: "Imported from Profit Report (Grouped)".to_string(),
            confirmation_signals: "External".to_string(),
            market_context: "B3".to_string(),
            relevant_news: "N/A".to_string(),
            followed_plan: true,
            what_worked: "".to_string(),
            mistakes_improvements: "".to_string(),
            lessons_learned: "".to_string(),
            images: vec![],
            partial_exits: crate::models::SurrealJson(serde_json::Value::Array(partial_exits)),
            asset_id: Some(asset_id),
            modality_id: Some("modality:m1".to_string()),
            stop_loss: None,
            take_profit: None,
            intensity: 5.0,
        };

        let mut trade_json = serde_json::to_value(&trade).map_err(|e| e.to_string())?;
        if let Some(obj) = trade_json.as_object_mut() {
            obj.remove("id");
        }

        let query = format!("UPSERT trade:⟨{}⟩ CONTENT $data RETURN NONE", trade_id);
        db.0.query(&query).bind(("data", trade_json)).await.map_err(|e| e.to_string())?;

        // 3. Relational conversion
        let update_query = format!("
            UPDATE trade:⟨{}⟩ SET 
                account_id = type::thing(account_id),
                asset_type_id = type::thing(asset_type_id),
                asset_id = type::thing(asset_id),
                strategy_id = type::thing(strategy_id),
                modality_id = type::thing(modality_id)
            WHERE id = trade:⟨{}⟩;
        ", trade_id, trade_id);
        db.0.query(&update_query).await.map_err(|e| e.to_string())?;

        imported_count += 1;
    }

    Ok(format!("{} trades importados (agrupados) com sucesso!", imported_count))
}

async fn get_or_create_asset(db: &Surreal<Db>, symbol: &str) -> Result<String, String> {
    // 1. Try to find existing asset
    let mut res = db.query("SELECT type::string(id) as id FROM asset WHERE symbol = $sym LIMIT 1")
        .bind(("sym", symbol.to_string()))
        .await
        .map_err(|e| e.to_string())?;
        
    let existing: Vec<Value> = res.take(0).map_err(|e| e.to_string())?;
    if let Some(asset) = existing.first() {
        if let Some(id) = asset.get("id").and_then(|v| v.as_str()) {
            return Ok(id.to_string());
        }
    }

    // 2. If not found, create new one
    let mut root_id: Option<String> = None;
    if symbol.len() >= 3 {
        let prefix = &symbol[..3];
        let mut root_res = db.query("SELECT type::string(id) as id FROM asset WHERE is_root = true AND symbol = $prefix LIMIT 1")
            .bind(("prefix", prefix.to_string()))
            .await
            .map_err(|e| e.to_string())?;
            
        let roots: Vec<Value> = root_res.take(0).map_err(|e| e.to_string())?;
        if let Some(root) = roots.first() {
            root_id = root.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
    }

    let new_asset = Asset {
        id: Some(format!("asset:{}", symbol.to_lowercase())),
        symbol: symbol.to_string(),
        name: symbol.to_string(),
        asset_type_id: Some("asset_type:at2".to_string()), // Default
        point_value: if symbol.starts_with("WDO") { 10.0 } else { 0.2 },
        default_fee_id: None,
        tax_profile_id: None,
        is_root: false,
        root_id,
    };

    let mut asset_json = serde_json::to_value(&new_asset).map_err(|e| e.to_string())?;
    if let Some(obj) = asset_json.as_object_mut() {
        obj.remove("id");
    }

    db.query("UPSERT type::thing('asset', $sym) CONTENT $data RETURN NONE")
        .bind(("sym", symbol.to_lowercase()))
        .bind(("data", asset_json))
        .await
        .map_err(|e| e.to_string())?;

    // 3. Relational conversion
    let update_query = format!("
        UPDATE asset:⟨{}⟩ SET 
            asset_type_id = type::thing(asset_type_id),
            root_id = (IF root_id THEN type::thing(root_id) ELSE null END)
        WHERE id = asset:⟨{}⟩;
    ", symbol.to_lowercase(), symbol.to_lowercase());
    db.query(&update_query).await.map_err(|e| e.to_string())?;

    Ok(format!("asset:{}", symbol.to_lowercase()))
}

fn parse_profit_datetime(datetime_str: &str) -> Result<String, String> {
    let s = datetime_str.trim();
    if s.is_empty() {
        return Err("Data vazia".to_string());
    }
    // Expected: 15/05/2024 09:30:15 or 15/05/2024 09:30
    let fmt_with_seconds = "%d/%m/%Y %H:%M:%S";
    let fmt_without_seconds = "%d/%m/%Y %H:%M";
    
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, fmt_with_seconds) {
        // Assume local time (America/Sao_Paulo usually for B3 traders)
        // For simplicity and matching current project style, we might just store as UTC RFC3339
        return Ok(Utc.from_local_datetime(&ndt).unwrap().to_rfc3339());
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, fmt_without_seconds) {
        return Ok(Utc.from_local_datetime(&ndt).unwrap().to_rfc3339());
    }
    Err(format!("Formato de data inválido: '{}'", datetime_str))
}

fn parse_profit_currency(currency_str: &str) -> Result<f64, String> {
    let s = currency_str.trim().replace(".", "").replace(",", ".");
    s.parse::<f64>().map_err(|e| {
        format!(
            "Formato de número inválido: '{}'. Erro: {}",
            currency_str, e
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_profit_datetime() {
        assert!(parse_profit_datetime("15/05/2024 09:30:15").is_ok());
        assert!(parse_profit_datetime("15/05/2024 09:30").is_ok());
        assert!(parse_profit_datetime("invalid").is_err());
    }

    #[test]
    fn test_parse_profit_currency() {
        assert_eq!(parse_profit_currency("1.234,56").unwrap(), 1234.56);
        assert_eq!(parse_profit_currency("100,00").unwrap(), 100.0);
        assert_eq!(parse_profit_currency("0,20").unwrap(), 0.2);
    }

    #[test]
    fn test_date_part_extraction() {
        let rfc = "2026-03-10T10:15:33Z";
        let date_part = rfc.chars().take(10).collect::<String>();
        assert_eq!(date_part, "2026-03-10");
        
        let rfc2 = "2026-03-10T15:16:36.123+00:00";
        let date_part2 = rfc2.chars().take(10).collect::<String>();
        assert_eq!(date_part2, "2026-03-10");
    }

    #[test]
    fn test_grouping_logic_simulation() {
        let account_id = "test_acc".to_string();
        let raw_lines = vec![
            ProfitLine {
                symbol: "WINJ26".to_string(),
                entry_date: "2026-03-10T10:15:33Z".to_string(),
                exit_date: Some("2026-03-10T10:28:33Z".to_string()),
                quantity: 3.0,
                direction: "Sell".to_string(),
                entry_price: 183723.33,
                exit_price: Some(183561.67),
                result: -97.0,
                line_num: 5,
            },
            ProfitLine {
                symbol: "WINJ26".to_string(),
                entry_date: "2026-03-10T15:16:36Z".to_string(),
                exit_date: Some("2026-03-10T15:26:53Z".to_string()),
                quantity: 2.0,
                direction: "Sell".to_string(),
                entry_price: 186417.50,
                exit_price: Some(186310.00),
                result: -43.0,
                line_num: 15,
            },
        ];

        let mut groups: HashMap<(String, String, String, String), Vec<ProfitLine>> = HashMap::new();
        for line in raw_lines {
            let key = (line.symbol.clone(), account_id.clone(), line.entry_date.clone(), line.direction.clone());
            groups.entry(key).or_default().push(line);
        }

        assert_eq!(groups.len(), 2); // Should be 2 separate groups now
        let group1 = groups.get(&("WINJ26".to_string(), "test_acc".to_string(), "2026-03-10T10:15:33Z".to_string(), "Sell".to_string())).unwrap();
        assert_eq!(group1.len(), 1);
        let group2 = groups.get(&("WINJ26".to_string(), "test_acc".to_string(), "2026-03-10T15:16:36Z".to_string(), "Sell".to_string())).unwrap();
        assert_eq!(group2.len(), 1);
    }
}
