// src-tauri/src/seed/demo_accounts_seed.rs
use crate::models::{Account, CashTransaction};
use chrono::{Duration, Utc};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn seed_accounts(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Criando Contas de Demonstração (Filter: {:?})...", filter);

    let accounts = vec![
        (
            "sim_brl",
            "Simulador (BRL)",
            "BRL",
            "Demo",
            10000.0,
        ),
        (
            "sim_usd",
            "Simulador (USD)",
            "USD",
            "Demo",
            10000.0,
        ),
        (
            "sim_usdt",
            "Simulador (USDT)",
            "USDT",
            "Demo",
            10000.0,
        ),
        (
            "real_brl",
            "Conta Real (BRL)",
            "BRL",
            "Real",
            0.0,
        ),
        (
            "real_usd",
            "Conta Real (USD)",
            "USD",
            "Real",
            0.0,
        ),
        (
            "real_usdt",
            "Conta Real (USDT)",
            "USDT",
            "Real",
            0.0,
        ),
    ];

    for (id_suffix, name, currency, account_type, balance) in accounts {
        let id_part = format!("account:{}", id_suffix);
        
        // Skip if filter is present and doesn't contain this ID
        if let Some(ref f) = filter {
            if !f.contains(&id_part) {
                // If filter is "account:real", we include all "Real" accounts
                if f.contains(&"account:real".to_string()) && account_type == "Real" {
                    // keep going
                } else if f.contains(&"account:demo".to_string()) && account_type == "Demo" {
                    // keep going
                } else {
                    continue;
                }
            }
        }

        let account = Account {
            id: id_part.clone().into(),
            nickname: name.into(),
            account_type: account_type.into(),
            broker: "Demo Broker".into(),
            account_number: format!("DEMO-{}", id_suffix.to_uppercase()),
            currency_id: Some(format!("currency:{}", currency)),
            currency: None,
            balance: balance,
            custom_logo: None,
        };

        let mut account_json = serde_json::to_value(&account).unwrap();
        if let Some(obj) = account_json.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('account', $id) CONTENT $data")
            .bind(("id", id_suffix))
            .bind(("data", account_json))
            .await
            .map_err(|e| e.to_string())?;

        if balance > 0.0 {
            let tx_id = format!("cash_transaction:initial_{}", id_suffix);
            let tx_date = (Utc::now() - Duration::days(95)).to_rfc3339();
            let cash_tx = CashTransaction {
                id: tx_id.clone(),
                date: tx_date,
                amount: balance,
                r#type: "Deposit".into(),
                description: "Initial Balance".into(),
                account_id: Some(id_part.clone()),
                trade_ids: None,
                category: None,
                system_linked: None,
            };
            let mut tx_json = serde_json::to_value(&cash_tx).unwrap();
            if let Some(obj) = tx_json.as_object_mut() {
                obj.remove("id");
            }

            let tx_parts: Vec<&str> = tx_id.split(':').collect();
            let tx_clean_id = (if tx_parts.len() > 1 {
                tx_parts[1]
            } else {
                &tx_id
            })
            .to_string();

            db.query("UPSERT type::thing('cash_transaction', $id) CONTENT $data")
                .bind(("id", tx_clean_id))
                .bind(("data", tx_json))
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub async fn delete_demo_account_data(db: &Surreal<Db>, account_id: &str) -> Result<(), String> {
    let _ = db
        .query("DELETE trade WHERE account_id = type::thing('account', $id)")
        .bind((
            "id",
            account_id
                .split(':')
                .last()
                .unwrap_or(account_id)
                .to_string(),
        ))
        .await;
    Ok(())
}

pub async fn delete_all_demo_trades(db: &Surreal<Db>) -> Result<(), String> {
    let _ = db.query("DELETE trade").await;
    Ok(())
}
