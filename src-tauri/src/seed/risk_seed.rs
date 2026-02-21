// src-tauri/src/seed/risk_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::{RiskProfile, GrowthPhase};

pub async fn seed_risk_profiles(db: &Surreal<Db>, filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] Verificando Perfis de Risco...");

    let profiles = vec![
        ("r1", "Conservador", 50.0, 100.0, 0.5, 3, 2.0, true, "All", false),
        ("r2", "Moderado", 150.0, 300.0, 1.0, 5, 1.5, true, "All", false),
        ("r3", "Agressivo", 500.0, 1000.0, 2.0, 10, 1.0, false, "All", false),
    ];

    for (id, name, max_loss, target, risk_per_trade, max_trades, min_rr, lock, account_type, growth_enabled) in profiles {
        if let Some(ref f) = filter {
            if !f.contains(&id.to_string()) { continue; }
        }

        let mut res = db.query(format!("SELECT *, type::string(id) as id FROM risk_profile:{}", id)).await.map_err(|e| e.to_string())?;
        let existing: Option<RiskProfile> = res.take(0).map_err(|e| e.to_string())?;
        
        if existing.is_none() {
            let data = RiskProfile {
                id: id.into(), name: name.into(),
                max_daily_loss: max_loss, daily_target: target, max_risk_per_trade_percent: risk_per_trade,
                max_trades_per_day: max_trades, min_risk_reward: min_rr, lock_on_loss: lock,
                account_type_applicability: account_type.into(), growth_plan_enabled: growth_enabled,
                current_phase_index: 0, growth_phases: vec![]
            };
            let mut json = serde_json::to_value(&data).unwrap();
            if let Some(obj) = json.as_object_mut() { obj.remove("id"); }

            db.query(format!("CREATE risk_profile:{} CONTENT $data", id))
                .bind(("data", json))
                .await
                .map_err(|e| e.to_string())?;
            println!("  ✓ {} (Criado)", name);
        } else {
            println!("  ✓ {} (Já existe)", name);
        }
    }

    Ok(())
}
