// src-tauri/src/seed/demo_accounts_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::{Account, CashTransaction};
use chrono::{Duration, Utc};

pub async fn seed_accounts(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Criando Contas de Demonstração (B3 Only)...");

    let accounts = vec![
        ("demo_b3_acoes", "Conta B3 - Ações", "BRL", "Demo", 50000.0, vec!["markets:m1"]),
        ("demo_b3_futuros", "Conta B3 - Futuros", "BRL", "Demo", 25000.0, vec!["markets:m1"]),
        ("simulador", "Conta Simulador", "BRL", "Demo", 100000.0, vec![]),
        ("real", "Conta Real", "BRL", "Real", 0.0, vec![]),
        ("teste", "Conta Teste", "BRL", "Demo", 100000.0, vec![]),
    ];

    for (id_suffix, name, currency, account_type, balance, required_modules) in accounts {
        if let Some(ref f) = filter {
            let full_id = format!("account:{}", id_suffix);
            let explicitly_requested = f.contains(&full_id) || f.contains(&id_suffix.to_string());
            if !explicitly_requested {
                 if !required_modules.is_empty() {
                    let has_required = required_modules.iter().any(|m| f.contains(&m.to_string()));
                    if !has_required { continue; }
                 } else { continue; }
            }
        }
        let id_part = format!("account:{}", id_suffix);
        let account = Account {
            id: id_part.clone().into(),
            nickname: name.into(),
            account_type: account_type.into(),
            broker: "Demo Broker".into(),
            account_number: format!("DEMO-{}", id_suffix.to_uppercase()),
            currency: currency.into(),
            balance: balance,
            custom_logo: None,
        };

        let create_sql = format!("CREATE {} CONTENT $data", id_part);
        let mut account_json = serde_json::to_value(&account).unwrap();
        if let Some(obj) = account_json.as_object_mut() { obj.remove("id"); }

        let response = db.query(&create_sql).bind(("data", account_json.clone())).await.map_err(|e| e.to_string())?;
        if let Err(_) = response.check() {
            let _ = db.query(format!("UPDATE {} CONTENT $data", id_part)).bind(("data", account_json)).await;
        }

        if balance > 0.0 {
             let tx_id = format!("cash_transaction:initial_{}", id_suffix);
             let tx_date = (Utc::now() - Duration::days(95)).to_rfc3339();
             let cash_tx = CashTransaction {
                id: tx_id.clone(),
                date: tx_date,
                amount: balance,
                r#type: "Deposit".into(),
                description: "Initial Balance".into(),
                account_id: id_part.clone(),
                trade_ids: None,
            };
            let mut tx_json = serde_json::to_value(&cash_tx).unwrap();
            if let Some(obj) = tx_json.as_object_mut() { obj.remove("id"); }
            let _ = db.query(format!("CREATE {} CONTENT $data", tx_id)).bind(("data", tx_json.clone())).await;
        }
    }
    Ok(())
}

pub async fn delete_demo_account_data(db: &Surreal<Db>, account_id: &str) -> Result<(), String> {
    let _ = db.query(format!("DELETE trade WHERE account_id = '{}'", account_id)).await;
    Ok(())
}

pub async fn delete_all_demo_trades(db: &Surreal<Db>) -> Result<(), String> {
    let _ = db.query("DELETE trade").await;
    Ok(())
}
