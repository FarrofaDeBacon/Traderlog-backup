// src-tauri/src/seed/timeframes_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::Timeframe;

pub async fn seed_timeframes(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Timeframes...");

    let timeframes = vec![
        ("tf1", "1 Minuto", "1m"),
        ("tf2", "2 Minutos", "2m"),
        ("tf3", "5 Minutos", "5m"),
        ("tf4", "10 Minutos", "10m"),
        ("tf5", "15 Minutos", "15m"),
        ("tf6", "30 Minutos", "30m"),
        ("tf7", "60 Minutos", "60m"),
        ("tf8", "4 Horas", "4H"),
        ("tf9", "Diário", "1D"),
        ("tf10", "Semanal", "1W"),
        ("tf11", "Mensal", "1M"),
    ];

    for (id, name, value) in timeframes {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) { continue; }
        }
        let create_sql = format!("CREATE timeframe:{} CONTENT $data", id);
        let tf_data = Timeframe {
                id: id.into(), name: name.into(), value: value.into()
            };

        if let Ok(mut json_data) = serde_json::to_value(&tf_data) {
             if let Some(obj) = json_data.as_object_mut() {
                obj.remove("id");
            }

            if let Err(_) = db.query(&create_sql)
                .bind(("data", json_data.clone()))
                .await 
            {
                let update_sql = format!("UPDATE timeframe:{} CONTENT $data", id);
                db.query(&update_sql)
                    .bind(("data", json_data))
                    .await
                    .map_err(|e| e.to_string())?;
                 println!("  ✓ {} (Atualizado)", name);
            } else {
                 println!("  ✓ {} (Criado)", name);
            }
        }
    }

    Ok(())
}
