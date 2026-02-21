// src-tauri/src/seed/assets_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::Asset;

pub async fn seed_assets(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Ativos (B3 Only)...");

    let assets = vec![
        // Ações Brasil (at1) -> markets:m1
        ("a1", "PETR4", "Petrobras PN", "asset_type:at1", 1.0, "markets:m1"),
        ("a2", "VALE3", "Vale ON", "asset_type:at1", 1.0, "markets:m1"),
        ("a3", "ITUB4", "Itaú Unibanco PN", "asset_type:at1", 1.0, "markets:m1"),
        ("a4", "BBDC4", "Bradesco PN", "asset_type:at1", 1.0, "markets:m1"),
        
        // Índices B3 (at2) -> markets:m1
        ("a5", "WIN", "Mini Índice", "asset_type:at2", 0.20, "markets:m1"),
        ("a6", "WDO", "Mini Dólar", "asset_type:at2", 10.0, "markets:m1"),
        ("a7", "IND", "Índice Bovespa Cheio", "asset_type:at2", 1.0, "markets:m1"),
    ];

    for (id, symbol, name, type_id, point_value, required_module) in assets {
        if let Some(ref f) = filter {
            if !f.contains(&required_module.to_string()) { continue; }
        }
        let record_id = format!("asset:{}", id);
        let asset_data = Asset {
            id: id.into(),
            symbol: symbol.into(),
            name: name.into(),
            asset_type_id: type_id.into(),
            point_value,
            default_fee_id: None
        };

        let mut json_data = serde_json::to_value(&asset_data).unwrap();
        if let Some(obj) = json_data.as_object_mut() { obj.remove("id"); }

        let upsert_sql = format!("UPSERT {} MERGE $data", record_id);
        let _ = db.query(&upsert_sql).bind(("data", json_data)).await;
    }
    Ok(())
}
