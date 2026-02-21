// src-tauri/src/seed/tags_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::Tag;

pub async fn seed_tags(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    let mut result = db.query("SELECT *, meta::id(id) as id FROM tag LIMIT 1").await.map_err(|e| e.to_string())?;
    let _existing: Vec<Tag> = result.take(0).map_err(|e| e.to_string())?;
    
    // Verificação inicial removida para permitir upserts individuais
    // if !existing.is_empty() { ... }

    println!("[SEED] Populando Tags...");

    let tags = vec![
        ("t1", "Setup Indefinido", "#9E9E9E"),
        ("t2", "Fora do Plano", "#F44336"),
        ("t3", "Hesitação", "#FF9800"),
        ("t4", "Impulsividade", "#E91E63"),
        ("t5", "Fomo", "#9C27B0"),
        ("t6", "Revenge Trading", "#D32F2F"),
        ("t7", "Bom Gerenciamento", "#4CAF50"),
        ("t8", "Seguiu Regras", "#2196F3"),
        ("t9", "Sortudo", "#00BCD4"),
        ("t10", "Errou a mão", "#FF5722"),
        ("t11", "Mercado Lento", "#607D8B"),
        ("t12", "Alta Volatilidade", "#795548"),
        ("t13", "Notícia Impactante", "#FFC107"),
    ];

    for (id, name, color) in tags {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) { continue; }
        }
        // Fix: Use query with type::string(id) to match struct string ID
        let sql_check = format!("SELECT *, type::string(id) as id FROM tag:{}", id);
        let mut result = db.query(&sql_check).await.map_err(|e| e.to_string())?;
        let existing: Option<Tag> = result.take(0).map_err(|e| e.to_string())?;

        if existing.is_none() {
            let tag_data = Tag {
                    id: id.into(),
                    name: name.into(),
                    color: color.into(),
                };
            let mut json_data = serde_json::to_value(&tag_data).unwrap();
            if let Some(obj) = json_data.as_object_mut() { obj.remove("id"); }

            db.query(format!("CREATE tag:{} CONTENT $data", id))
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
