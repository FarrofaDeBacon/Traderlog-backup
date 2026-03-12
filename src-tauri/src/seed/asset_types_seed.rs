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
        (
            "at3",
            "Ações Internacionais (NYSE)",
            "STOCK-US",
            "market:m2",
            "Ações listadas na NYSE",
            "currency",
            Some("tax_profile:tp_acoes"),
        ),
        (
            "at4",
            "Ações Internacionais (NASDAQ)",
            "STOCK-TECH",
            "market:m3",
            "Ações listadas na NASDAQ",
            "currency",
            Some("tax_profile:tp_acoes"),
        ),
        (
            "at5",
            "Futuros Globais",
            "FUT-GL",
            "market:m4",
            "Contratos futuros globais (CME)",
            "points",
            Some("tax_profile:tp_futuros"),
        ),
        (
            "at6",
            "Pares de Moedas",
            "FOREX",
            "market:m5",
            "Mercado de câmbio",
            "pips",
            Some("tax_profile:tp_daytrade"),
        ),
        (
            "at7",
            "Criptomoedas",
            "CRYPTO",
            "market:m6",
            "Criptoativos",
            "currency",
            Some("tax_profile:tp_acoes"),
        ),
    ];

    for (id, name, code, market_id, unit_label, result_type, t_profile) in asset_types {
        if let Some(ref f) = filter {
            let required_module = market_id.replace("market:", "markets:");
            if !f.contains(&required_module) {
                continue;
            }
        }
        let asset_type_data = AssetType {
            id: Some(id.into()),
            name: name.into(),
            code: code.into(),
            market_id: Some(market_id.into()),
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
        db.query("UPSERT type::thing('asset_type', $id) CONTENT $data RETURN NONE")
            .bind(("id", id))
            .bind(("data", json_data))
            .await
            .map_err(|e| e.to_string())?;

        println!("  ✓ {}", name);
    }
    Ok(())
}
