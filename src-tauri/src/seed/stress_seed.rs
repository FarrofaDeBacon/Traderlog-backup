use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use chrono::{Utc, Duration};
use rand::Rng;

pub async fn seed_stress_records(db: &Surreal<Db>, count: usize) -> Result<(), String> {
    println!("[STRESS SEED] Generating {} trades...", count);
    
    // Generate all SQL synchronously to avoid ThreadRng not being Send across await
    let query = generate_stress_query(count);
    
    db.query(query).await.map_err(|e| e.to_string())?;
    
    println!("[STRESS SEED] Successfully injected {} stress records.", count);
    Ok(())
}

fn generate_stress_query(count: usize) -> String {
    let mut query = String::from("BEGIN TRANSACTION;\n");
    let assets = vec!["WINJ24", "WDOM24", "PETR4", "VALE3", "AAPL", "BTCBRL"];
    let accounts = vec!["account:⟨demo-real⟩", "account:⟨demo-sim⟩"];
    let strategies = vec!["strategy:⟨strat-price-action⟩", "strategy:⟨strat-trend-follower⟩"];
    let types = vec!["asset_type:⟨future⟩", "asset_type:⟨stock⟩", "asset_type:⟨crypto⟩"];
    
    let base_date = Utc::now() - Duration::days(365);
    let mut rng = rand::thread_rng();

    for i in 0..count {
        let trade_id = format!("stress_{}", i);
        let date = base_date + Duration::minutes(rng.gen_range(0..525600));
        let exit_date = date + Duration::minutes(rng.gen_range(1..480));
        
        let asset = assets[rng.gen_range(0..assets.len())];
        let qty = rng.gen_range(1..100) as f64;
        
        let entry = if asset.contains("WIN") {
            rng.gen_range(110000.0..130000.0)
        } else {
            rng.gen_range(10.0..150.0)
        };
        
        let exit_p = entry + rng.gen_range(-5.0..10.0);
        let result = (exit_p - entry) * qty * if asset.contains("WIN") { 0.2 } else { 1.0 };
        
        query.push_str(&format!(
            "UPSERT trade:⟨{}⟩ CONTENT {{ \
                account_id: type::thing('{}'), \
                asset_symbol: '{}', \
                asset_type_id: type::thing('{}'), \
                date: '{}', \
                entry_price: {}, \
                exit_date: '{}', \
                exit_price: {}, \
                followed_plan: true, \
                quantity: {}, \
                result: {}, \
                status: 'Closed', \
                strategy_id: type::thing('{}'), \
                notes: 'Stress generated', \
                modality_id: type::thing('modality:⟨day-trade⟩') \
            }};\n",
            trade_id,
            accounts[rng.gen_range(0..accounts.len())],
            asset,
            types[rng.gen_range(0..types.len())],
            date.to_rfc3339(),
            entry,
            exit_date.to_rfc3339(),
            exit_p,
            qty,
            result,
            strategies[rng.gen_range(0..strategies.len())]
        ));

        if i % 1000 == 0 && i > 0 {
            query.push_str("COMMIT TRANSACTION;\nBEGIN TRANSACTION;\n");
        }
    }
    
    query.push_str("COMMIT TRANSACTION;");
    query
}
