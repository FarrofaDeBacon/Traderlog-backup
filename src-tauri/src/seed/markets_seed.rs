// src-tauri/src/seed/markets_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::{Market, TradingSession};

pub async fn seed_markets(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Mercados...");

    let markets = vec![
        ("m1", "B3", "Brasil Bolsa Balcão", "America/Sao_Paulo", vec![1,2,3,4,5], "09:00", "18:00"),
    ];

    for (id, code, name, timezone, trading_days, start, end) in markets {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) { continue; }
        }
        let create_sql = format!("CREATE market:{} CONTENT $data", id);
        let market_data = Market {
                id: id.into(), code: code.into(), name: name.into(),
                timezone: timezone.into(), trading_days: trading_days.clone(),
                trading_sessions: vec![TradingSession {
                    start_time: start.into(), end_time: end.into()
                }]
            };
        let mut json_data = serde_json::to_value(&market_data).unwrap();
        if let Some(obj) = json_data.as_object_mut() { obj.remove("id"); }

        if let Err(_) = db.query(&create_sql)
            .bind(("data", json_data.clone())) // Bind clone for create
            .await 
        {
            let update_sql = format!("UPDATE market:{} CONTENT $data", id);
             db.query(&update_sql)
                .bind(("data", json_data))
                .await
                .map_err(|e| e.to_string())?;
            println!("  ✓ {} (Atualizado)", name);
        } else {
             println!("  ✓ {} (Criado)", name);
        }
    }

    Ok(())
}
