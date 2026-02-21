// src-tauri/src/seed/currencies_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::Currency;

pub async fn seed_currencies(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Verificando Moedas...");

    let currencies = vec![
        ("BRL", "BRL", "R$", "Real Brasileiro", 1.0),
        ("USD", "USD", "$", "Dólar Americano", 5.0),
        ("EUR", "EUR", "€", "Euro", 5.40),
        ("GBP", "GBP", "£", "Libra Esterlina", 6.30),
        ("JPY", "JPY", "¥", "Iene Japonês", 0.034),
        ("AUD", "AUD", "A$", "Dólar Australiano", 3.25),
        ("CAD", "CAD", "C$", "Dólar Canadense", 3.70),
        ("CHF", "CHF", "Fr", "Franco Suíço", 5.60),
    ];

    for (id, code, symbol, name, rate) in currencies {
        // Tenta CRIAR. Se falhar (já existe), faz UPDATE.
        let create_sql = format!("CREATE currency:{} CONTENT $data", id);
        let currency_data = Currency {
            id: id.into(), code: code.into(), symbol: symbol.into(),
            name: name.into(), exchange_rate: rate
        };
        let mut json_data = serde_json::to_value(&currency_data).unwrap();
        if let Some(obj) = json_data.as_object_mut() { obj.remove("id"); }

        if let Err(_) = db.query(&create_sql)
            .bind(("data", json_data.clone()))
            .await 
        {
            // Se falhou o CREATE, tenta UPDATE
            let update_sql = format!("UPDATE currency:{} CONTENT $data", id);
             db.query(&update_sql)
                .bind(("data", json_data))
                .await
                .map_err(|e| e.to_string())?;
            println!("  ✓ {} (Atualizado)", name);
        } else {
            println!("  ✓ {} (Criado)", name);
        }
    }

    Ok(())
}
