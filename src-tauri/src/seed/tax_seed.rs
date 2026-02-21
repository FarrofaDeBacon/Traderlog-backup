// src-tauri/src/seed/tax_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::{TaxRule, TaxProfile, TaxProfileEntry};

pub async fn seed_tax_rules(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Criando Regras e Perfis Tributários (B3)...");

    // 1. Tax Rules
    let rules = vec![
        ("rule_swing_acoes", "Swing Trade Ações (15%)", 15.0, 0.005, 20000.0, "NetProfit", true),
        ("rule_day_acoes", "Day Trade Ações (20%)", 20.0, 1.0, 0.0, "NetProfit", true),
        ("rule_swing_futuros", "Swing Trade Futuros (15%)", 15.0, 0.005, 0.0, "NetProfit", true),
        ("rule_day_futuros", "Day Trade Futuros (20%)", 20.0, 1.0, 0.0, "NetProfit", true),
        ("rule_fiis", "FIIs (20%)", 20.0, 0.0, 0.0, "NetProfit", true),
    ];

    for (id, name, rate, w_rate, exemption, basis, cumulative) in rules {
        let rule = TaxRule {
            id: id.into(),
            name: name.into(),
            tax_rate: rate,
            withholding_rate: w_rate,
            exemption_threshold: exemption,
            basis: basis.into(),
            cumulative_losses: cumulative,
        };
        
        let mut data = serde_json::to_value(&rule).unwrap();
        if let Some(obj) = data.as_object_mut() { obj.remove("id"); }
        let _ = db.query(format!("UPSERT tax_rule:{} MERGE $data", id)).bind(("data", data)).await;
    }

    // 2. Tax Profiles
    let profiles = vec![
        ("tp_acoes", "Perfil Tributário Ações", "Tributação padrão de ações (Isenção 20k no Swing)"),
        ("tp_futuros", "Perfil Tributário Futuros", "Tributação de Índices e Dólar (Sem isenção)"),
    ];

    for (id, name, desc) in profiles {
        let profile = TaxProfile {
            id: id.into(),
            name: name.into(),
            description: Some(desc.into()),
        };
        let mut data = serde_json::to_value(&profile).unwrap();
        if let Some(obj) = data.as_object_mut() { obj.remove("id"); }
        let _ = db.query(format!("UPSERT tax_profile:{} MERGE $data", id)).bind(("data", data)).await;
    }

    // 3. Tax Profile Entries (Linking Profile -> Modality -> Rule)
    // Modalities IDs: 'mod1' (Day Trade) and 'mod2' (Swing Trade) — from modalities_seed.rs
    let entries = vec![
        ("tpe_acoes_swing", "tax_profile:tp_acoes", "modality:mod2", "tax_rule:rule_swing_acoes"),
        ("tpe_acoes_day", "tax_profile:tp_acoes", "modality:mod1", "tax_rule:rule_day_acoes"),
        
        ("tpe_futuros_swing", "tax_profile:tp_futuros", "modality:mod2", "tax_rule:rule_swing_futuros"),
        ("tpe_futuros_day", "tax_profile:tp_futuros", "modality:mod1", "tax_rule:rule_day_futuros"),
    ];

    for (id, p_id, m_id, r_id) in entries {
        let entry = TaxProfileEntry {
            id: id.into(),
            tax_profile_id: p_id.into(),
            modality_id: m_id.into(),
            tax_rule_id: r_id.into(),
        };
        let mut data = serde_json::to_value(&entry).unwrap();
        if let Some(obj) = data.as_object_mut() { obj.remove("id"); }
        let _ = db.query(format!("UPSERT tax_profile_entry:{} MERGE $data", id)).bind(("data", data)).await;
    }

    Ok(())
}
