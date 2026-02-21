// src-tauri/src/seed/modalities_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::Modality;

pub async fn seed_modalities(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Modalidades...");

    let modalities = vec![
        ("mod1", "Day Trade", "Operações iniciadas e encerradas no mesmo dia"),
        ("mod2", "Swing Trade", "Operações que duram de 2 dias a algumas semanas"),
    ];

    for (id, name, description) in modalities {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) { continue; }
        }
        let create_sql = format!("CREATE modality:{} CONTENT $data", id);
        let modality_data = Modality {
            id: id.into(), name: name.into(), description: description.into()
        };

        if let Ok(mut json_data) = serde_json::to_value(&modality_data) {
             if let Some(obj) = json_data.as_object_mut() {
                obj.remove("id");
            }
            if let Err(_) = db.query(&create_sql)
                .bind(("data", json_data.clone()))
                .await 
            {
                let update_sql = format!("UPDATE modality:{} CONTENT $data", id);
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
