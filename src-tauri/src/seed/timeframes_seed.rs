// src-tauri/src/seed/timeframes_seed.rs
use crate::models::Timeframe;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

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
            if !f.contains(&id.to_string()) {
                continue;
            }
        }
        let create_sql = format!("CREATE timeframe:{} CONTENT $data", id);
        let tf_data = Timeframe {
            id: id.into(),
            name: name.into(),
            value: value.into(),
        };

        let mut tf_json = serde_json::to_value(&tf_data).unwrap();
        if let Some(obj) = tf_json.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('timeframe', $id) CONTENT $data")
            .bind(("id", id))
            .bind(("data", tf_json))
            .await
            .map_err(|e| {
                println!("[SEED_ERROR] Failed to seed timeframe {}: {}", name, e);
                e.to_string()
            })?;

        println!("  ✓ {}", name);
    }

    Ok(())
}
