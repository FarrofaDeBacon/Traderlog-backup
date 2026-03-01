// src-tauri/src/seed/fees_seed.rs
use crate::models::FeeProfile;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn seed_fees(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Perfis de Taxas...");

    // Check if any profiles already exist to avoid "zombie" recreations after user deletion
    let mut check_all = db
        .query("SELECT count() FROM fee_profile GROUP ALL")
        .await
        .map_err(|e| e.to_string())?;
    let count: Option<i64> = check_all.take("count").map_err(|e| e.to_string())?;

    if let Some(c) = count {
        if c > 0 && filter.is_none() {
            println!("  ! Já existem perfis de taxas. Pulando seed automático.");
            return Ok(());
        }
    }

    let fees = vec![
        (
            "f1",
            "B3 - Day Trade (Ações/FIIs)",
            "Genérica",
            0.0,
            0.0,
            0.030,
            0.0,
            0.0,
            1.0,
            20.0,
            "Taxas padrão para DT em ações/FIIs",
        ),
        (
            "f2",
            "B3 - Swing Trade (Ações/FIIs)",
            "Genérica",
            0.0,
            0.0,
            0.030,
            0.0,
            0.0,
            1.0,
            15.0,
            "Taxas padrão para ST em ações/FIIs",
        ),
        (
            "f3",
            "B3 - Futuros (WIN/WDO)",
            "Genérica",
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            "Taxas de registro e emolumentos para Minicontratos",
        ),
    ];

    for (id, name, broker, fixed, percent, exchange, iss, spread, withholding, tax_rate, notes) in
        fees
    {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) {
                continue;
            }
        }

        let data = FeeProfile {
            id: id.into(),
            name: name.into(),
            broker: broker.into(),
            fixed_fee: fixed,
            percentage_fee: percent,
            exchange_fee: exchange,
            iss: iss,
            currency_spread: spread,
            withholding_tax: withholding,
            income_tax_rate: tax_rate,
            custom_items: vec![],
            tax_rule_id: None,
            notes: notes.into(),
        };
        let mut json = serde_json::to_value(&data).unwrap();
        if let Some(obj) = json.as_object_mut() {
            obj.remove("id");
        }

        // Use raw query for robust serialization
        db.query("UPSERT type::thing('fee_profile', $id) CONTENT $data")
            .bind(("id", id))
            .bind(("data", json))
            .await
            .map_err(|e| e.to_string())?;

        println!("  ✓ {}", name);
    }

    Ok(())
}
