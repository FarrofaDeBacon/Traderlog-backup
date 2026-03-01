// src-tauri/src/seed/markets_seed.rs
use crate::models::{Market, TradingSession};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn seed_markets(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Mercados...");

    let markets = vec![(
        "m1",
        "B3",
        "Brasil Bolsa Balcão",
        "America/Sao_Paulo",
        vec![1, 2, 3, 4, 5],
        "09:00",
        "18:00",
    )];

    for (id, code, name, timezone, trading_days, start, end) in markets {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) {
                continue;
            }
        }
        let market_data = Market {
            id: id.into(),
            code: code.into(),
            name: name.into(),
            timezone: timezone.into(),
            trading_days: trading_days.clone(),
            trading_sessions: vec![TradingSession {
                start_time: start.into(),
                end_time: end.into(),
            }],
        };
        let mut market_json = serde_json::to_value(&market_data).unwrap();
        if let Some(obj) = market_json.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('market', $id) CONTENT $data")
            .bind(("id", id))
            .bind(("data", market_json))
            .await
            .map_err(|e| {
                println!("[SEED_ERROR] Failed to seed market {}: {}", name, e);
                e.to_string()
            })?;

        println!("  ✓ {}", name);
    }

    Ok(())
}
