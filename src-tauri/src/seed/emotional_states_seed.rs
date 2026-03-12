// src-tauri/src/seed/emotional_states_seed.rs
use crate::models::EmotionalState;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn seed_emotional_states(
    db: &Surreal<Db>,
    filter: Option<Vec<String>>,
) -> Result<(), String> {
    println!("[SEED] Verificando Estados Emocionais...");

    let states = vec![
        (
            "e1",
            "Focado",
            "Positive",
            "Concentração total",
            "Boa execução",
            10.0,
        ),
        (
            "e2",
            "Confiante",
            "Positive",
            "Segurança na análise",
            "Entradas precisas",
            9.0,
        ),
        (
            "e3",
            "Disciplinado",
            "Positive",
            "Seguindo o plano",
            "Consistência",
            9.0,
        ),
        (
            "e4",
            "Paciente",
            "Positive",
            "Aguardando o setup",
            "Evita overtrading",
            8.0,
        ),
        (
            "e5",
            "Ansioso",
            "Negative",
            "Pressa para operar",
            "Entradas antecipadas",
            4.0,
        ),
        (
            "e6",
            "Ganancioso",
            "Negative",
            "Querendo ganhar muito rápido",
            "Exposição excessiva",
            3.0,
        ),
        (
            "e7",
            "Medo",
            "Negative",
            "Receio de perder",
            "Perda de oportunidades",
            2.0,
        ),
        (
            "e8",
            "Vingativo",
            "Negative",
            "Tentando recuperar perdas",
            "Aumento de prejuízo",
            1.0,
        ),
        (
            "e9",
            "Frustrado",
            "Negative",
            "Irritação com o mercado",
            "Decisões emocionais",
            2.0,
        ),
        (
            "e10",
            "FOMO",
            "Negative",
            "Medo de ficar de fora",
            "Entras ruins",
            3.0,
        ),
        (
            "e11",
            "Neutro",
            "Neutral",
            "Sem emoção forte",
            "Operação técnica",
            6.0,
        ),
        (
            "e12",
            "Cansado",
            "Neutral",
            "Baixa energia",
            "Risco de desatenção",
            5.0,
        ),
    ];

    for (id, name, impact, description, potential, weight) in states {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) {
                continue;
            }
        }

        let es_struct = EmotionalState {
            id: Some(id.into()),
            name: name.into(),
            impact: impact.into(),
            description: description.into(),
            potential_impact: potential.into(),
            weight,
        };

        // Converter para JSON e remover o ID para evitar duplicidade no SurrealDB
        let mut es_data = serde_json::to_value(&es_struct).unwrap();
        if let Some(obj) = es_data.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('emotional_state', $id) CONTENT $data")
            .bind(("id", id))
            .bind(("data", es_data))
            .await
            .map_err(|e| {
                println!(
                    "[SEED_ERROR] Failed to seed emotional state {}: {}",
                    name, e
                );
                e.to_string()
            })?;

        println!("  ✓ {}", name);
    }

    Ok(())
}
