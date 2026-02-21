// src-tauri/src/seed/emotional_states_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::EmotionalState;

pub async fn seed_emotional_states(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Estados Emocionais...");

    let states = vec![
        ("e1", "Focado", "Positive", "Concentração total", "Boa execução", 10.0),
        ("e2", "Confiante", "Positive", "Segurança na análise", "Entradas precisas", 9.0),
        ("e3", "Disciplinado", "Positive", "Seguindo o plano", "Consistência", 9.0),
        ("e4", "Paciente", "Positive", "Aguardando o setup", "Evita overtrading", 8.0),
        
        ("e5", "Ansioso", "Negative", "Pressa para operar", "Entradas antecipadas", 4.0),
        ("e6", "Ganancioso", "Negative", "Querendo ganhar muito rápido", "Exposição excessiva", 3.0),
        ("e7", "Medo", "Negative", "Receio de perder", "Perda de oportunidades", 2.0),
        ("e8", "Vingativo", "Negative", "Tentando recuperar perdas", "Aumento de prejuízo", 1.0),
        ("e9", "Frustrado", "Negative", "Irritação com o mercado", "Decisões emocionais", 2.0),
        ("e10", "FOMO", "Negative", "Medo de ficar de fora", "Entras ruins", 3.0),

        ("e11", "Neutro", "Neutral", "Sem emoção forte", "Operação técnica", 6.0),
        ("e12", "Cansado", "Neutral", "Baixa energia", "Risco de desatenção", 5.0),
    ];

    for (id, name, impact, description, potential, weight) in states {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) { continue; }
        }

        let es_struct = EmotionalState {
            id: id.into(), name: name.into(), impact: impact.into(),
            description: description.into(), potential_impact: potential.into(),
            weight
        };

        // Converter para JSON e remover o ID para evitar duplicidade no SurrealDB
        let mut es_data = serde_json::to_value(&es_struct).unwrap();
        if let Some(obj) = es_data.as_object_mut() {
            obj.remove("id");
        }

        // 1. Tentar selecionar por ID para ver se já existe
        let mut result = db.query(format!("SELECT *, type::string(id) as id FROM emotional_state:{}", id))
            .await.map_err(|e| e.to_string())?;
        let existing: Option<EmotionalState> = result.take(0).map_err(|e| e.to_string())?;

        if let Some(_) = existing {
            // Existe por ID, atualiza
            db.query(format!("UPDATE emotional_state:{} CONTENT $data", id))
                .bind(("data", es_data))
                .await
                .map_err(|e| e.to_string())?;
            println!("  ✓ {} (Atualizado por ID)", name);
        } else {
            // Não existe por ID, tentar por Nome
            let mut result = db.query("SELECT *, type::string(id) as id FROM emotional_state WHERE name = $name")
                .bind(("name", name))
                .await
                .map_err(|e| e.to_string())?;
            
            let found_by_name: Vec<EmotionalState> = result.take(0).map_err(|e| e.to_string())?;

            if let Some(existing_by_name) = found_by_name.first() {
                // Existe por nome, atualiza esse registro específico usando o ID encontrado
                db.query("UPDATE $id CONTENT $data")
                    .bind(("id", existing_by_name.id.clone()))
                    .bind(("data", es_data))
                    .await
                    .map_err(|e| e.to_string())?;
                println!("  ✓ {} (Atualizado por Nome)", name);
            } else {
                // Não existe de forma alguma, cria novo com o ID pretendido
                db.query(format!("CREATE emotional_state:{} CONTENT $data", id))
                    .bind(("data", es_data))
                    .await
                    .map_err(|e| e.to_string())?;
                println!("  ✓ {} (Criado Novo)", name);
            }
        }
    }

    Ok(())
}
