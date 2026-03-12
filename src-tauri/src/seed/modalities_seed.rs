// src-tauri/src/seed/modalities_seed.rs
use crate::models::Modality;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn seed_modalities(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Modalidades...");

    let modalities = vec![
        (
            "mod1",
            "Day Trade",
            "Operações iniciadas e encerradas no mesmo dia",
        ),
        (
            "mod2",
            "Swing Trade",
            "Operações que duram de 2 dias a algumas semanas",
        ),
    ];

    for (id, name, description) in modalities {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) {
                continue;
            }
        }
        let modality_data = Modality {
            id: Some(id.into()),
            name: name.into(),
            description: description.into(),
        };
        let mut modality_json = serde_json::to_value(&modality_data).unwrap();
        if let Some(obj) = modality_json.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('modality', $id) CONTENT $data")
            .bind(("id", id))
            .bind(("data", modality_json))
            .await
            .map_err(|e| {
                println!("[SEED_ERROR] Failed to seed modality {}: {}", name, e);
                e.to_string()
            })?;

        println!("  ✓ {}", name);
    }

    Ok(())
}
