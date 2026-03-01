// src-tauri/src/seed/asset_types_seed.rs
use crate::models::AssetType;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn seed_asset_types(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Tipos de Ativo (B3 Only)...");

    let asset_types = vec![
        (
            "at1",
            "Ações Brasil",
            "STOCK-BR",
            "market:m1",
            "Ação ordinária ou preferencial",
            "currency",
            Some("tax_profile:tp_acoes"),
        ),
        (
            "at2",
            "Índices B3",
            "INDEX-BR",
            "market:m1",
            "Contratos futuros de índice",
            "points",
            Some("tax_profile:tp_futuros"),
        ),
    ];

    for (id, name, code, market_id, unit_label, result_type, t_profile) in asset_types {
        if let Some(ref f) = filter {
            let required_module = market_id.replace("market:", "markets:");
            if !f.contains(&required_module) {
                continue;
            }
        }
        let record_id = format!("asset_type:{}", id);
        let asset_type_data = AssetType {
            id: id.into(),
            name: name.into(),
            code: code.into(),
            market_id: market_id.into(),
            default_fee_id: None,
            tax_profile_id: t_profile.map(|s| s.into()),
            unit_label: unit_label.into(),
            result_type: result_type.into(),
        };

        let mut json_data = serde_json::to_value(&asset_type_data).unwrap();
        if let Some(obj) = json_data.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('asset_type', $id) CONTENT $data")
            .bind(("id", id))
            .bind(("data", json_data))
            .await
            .map_err(|e| e.to_string())?;

        println!("  ✓ {}", name);
    }
    Ok(())
}
