// src-tauri/src/seed/chart_types_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::ChartType;

pub async fn seed_chart_types(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    let mut result = db.query("SELECT *, meta::id(id) as id FROM chart_type LIMIT 1").await.map_err(|e| e.to_string())?;
    let _existing: Vec<ChartType> = result.take(0).map_err(|e| e.to_string())?;
    
    // Verificação inicial removida para permitir upserts individuais
    // if !existing.is_empty() { ... }

    println!("[SEED] Populando Tipos de Gráfico...");

    let chart_types = vec![
        ("ct1", "Candlestick 1 min", "TimeBased", "1m"),
        ("ct2", "Candlestick 5 min", "TimeBased", "5m"),
        ("ct3", "Candlestick 15 min", "TimeBased", "15m"),
        ("ct4", "Candlestick 60 min", "TimeBased", "60m"),
        ("ct5", "Candlestick Diário", "TimeBased", "1D"),
        ("ct6", "Renko 5R", "Renko", "5R"),
        ("ct7", "Renko 10R", "Renko", "10R"),
        ("ct8", "Renko 21R", "Renko", "21R"),
        ("ct9", "Heiken Ashi", "TimeBased", "HA"),
        ("ct10", "Range 100", "Range", "100"),
    ];

    for (id, name, base_type, parameter) in chart_types {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) { continue; }
        }
        let sql_check = format!("SELECT *, type::string(id) as id FROM chart_type:{}", id);
        let mut result = db.query(&sql_check).await.map_err(|e| e.to_string())?;
        let existing: Option<ChartType> = result.take(0).map_err(|e| e.to_string())?;

        if existing.is_none() {
            let chart_data = ChartType {
                    id: id.into(),
                    name: name.into(),
                    base_type: base_type.into(),
                    parameter: parameter.into(),
                };
            let mut json_data = serde_json::to_value(&chart_data).unwrap();
            if let Some(obj) = json_data.as_object_mut() { obj.remove("id"); }

            db.query(format!("CREATE chart_type:{} CONTENT $data", id))
                .bind(("data", json_data))
                .await
                .map_err(|e| e.to_string())?;
            println!("  ✓ {} (Criado)", name);
        } else {
             println!("  ✓ {} (Já existe)", name);
        }
    }

    Ok(())
}
