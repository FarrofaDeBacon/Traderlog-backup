// src-tauri/src/seed/indicators_seed.rs
use crate::models::Indicator;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn seed_indicators(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    let mut result = db
        .query("SELECT *, meta::id(id) as id FROM indicator LIMIT 1")
        .await
        .map_err(|e| e.to_string())?;
    let _existing: Vec<Indicator> = result.take(0).map_err(|e| e.to_string())?;

    // Verificação inicial removida
    // if !existing.is_empty() { ... }

    println!("[SEED] Populando Indicadores...");

    let indicators = vec![
        (
            "i1",
            "IFR (14)",
            "Oscillator",
            "SubWindow",
            "#9C27B0",
            "Índice de Força Relativa",
        ),
        (
            "i2",
            "MACD Padrão",
            "Trend",
            "SubWindow",
            "#2196F3",
            "Moving Average Convergence Divergence",
        ),
        (
            "i3",
            "MME 21",
            "Trend",
            "Overlay",
            "#FF9800",
            "Média Móvel Exponencial 21",
        ),
        (
            "i4",
            "MME 8",
            "Trend",
            "Overlay",
            "#4CAF50",
            "Média Móvel Exponencial 8",
        ),
        (
            "i5",
            "MMS 9",
            "Trend",
            "Overlay",
            "#F44336",
            "Média Móvel Simples 9",
        ),
        (
            "i6",
            "MMS 21",
            "Trend",
            "Overlay",
            "#E91E63",
            "Média Móvel Simples 21",
        ),
        (
            "i7",
            "VWAP",
            "Volume",
            "Overlay",
            "#00BCD4",
            "Volume Weighted Average Price",
        ),
        (
            "i8",
            "Bollinger Bands",
            "Volatility",
            "Overlay",
            "#795548",
            "Bandas de Bollinger",
        ),
        (
            "i9",
            "Volume Profile",
            "Volume",
            "SubWindow",
            "#607D8B",
            "Perfil de Volume",
        ),
        (
            "i10",
            "Volume",
            "Volume",
            "SubWindow",
            "#9E9E9E",
            "Volume de negociação",
        ),
    ];

    for (id, name, category, plot_type, color, description) in indicators {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) {
                continue;
            }
        }

        let indicator_data = Indicator {
            id: id.into(),
            name: name.into(),
            category: category.into(),
            plot_type: plot_type.into(),
            default_color: color.into(),
            usage_description: description.into(),
            parameters: vec![],
        };
        let mut json_data = serde_json::to_value(&indicator_data).unwrap();
        if let Some(obj) = json_data.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('indicator', $id) CONTENT $data")
            .bind(("id", id))
            .bind(("data", json_data))
            .await
            .map_err(|e| e.to_string())?;

        println!("  ✓ {}", name);
    }

    Ok(())
}
