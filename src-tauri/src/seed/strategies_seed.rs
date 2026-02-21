// src-tauri/src/seed/strategies_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::Strategy;

pub async fn seed_strategies(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    let mut result = db.query("SELECT *, meta::id(id) as id FROM strategy LIMIT 1").await.map_err(|e| e.to_string())?;
    let _existing: Vec<Strategy> = result.take(0).map_err(|e| e.to_string())?;
    
    // Verificação inicial removida para permitir upserts individuais
    // if !existing.is_empty() { ... }

    println!("[SEED] Populando Estratégias...");

    let strategies = vec![
        ("s1", "Barra Elefante", "Candle com corpo muito maior que os anteriores", "Volume acima da média"),
        ("s2", "Bear 180", "Reversão baixista forte", "Stop acima da máxima"),
        ("s3", "Breakout", "Rompimento de resistência/suporte", "Volume confirmatório"),
        ("s4", "Bull 180", "Reversão altista forte", "Stop abaixo da mínima"),
        ("s5", "Clássico Perto da Média", "Pullback em tendência", "Retorno à média móvel"),
        ("s6", "Gift", "Pequeno candle após Barra Elefante", "Continuação esperada"),
        ("s7", "Power Breakout", "Rompimento explosivo", "Grande amplitude"),
        ("s8", "Pullback na Média", "Retração temporária", "Alinhamento de médias"),
        ("s9", "Range", "Lateralização", "Topos e fundos definidos"),
        ("s10", "Setup da MM200", "Operação na MM200", "Nível forte de S/R"),
        ("s11", "Volta em V", "Reversão rápida", "Movimento abrupto"),
    ];

    for (id, name, description, entry) in strategies {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) { continue; }
        }
        let sql_check = format!("SELECT *, type::string(id) as id FROM strategy:{}", id);
        let mut result = db.query(&sql_check).await.map_err(|e| e.to_string())?;
        let existing: Option<Strategy> = result.take(0).map_err(|e| e.to_string())?;

        if existing.is_none() {
            let strategy_data = Strategy {
                    id: id.into(),
                    name: name.into(),
                    description: description.into(),
                    market_ids: vec![],
                    timeframes: vec![],
                    asset_types: vec![],
                    indicators: vec![],
                    specific_assets: vec![],
                    entry_criteria: entry.into(),
                    exit_criteria: "".into(),
                    management_criteria: "".into(),
                    has_partial: false,
                    partial_description: "".into(),
                    images: vec![],
                };
            let mut json_data = serde_json::to_value(&strategy_data).unwrap();
            if let Some(obj) = json_data.as_object_mut() { obj.remove("id"); }

            db.query(format!("CREATE strategy:{} CONTENT $data", id))
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
