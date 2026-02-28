// src-tauri/src/seed/_master_seeder.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

use super::{
    currencies_seed, markets_seed, asset_types_seed, assets_seed,
    strategies_seed, indicators_seed, modalities_seed,
    emotional_states_seed, timeframes_seed, user_profile_seed,
    tags_seed, chart_types_seed, fees_seed, risk_seed,
    demo_accounts_seed, demo_trades_seed, tax_seed
};

/// Executa todos os seeds na ordem correta (respeitando dependências)

/// Inicialização mínima (apenas perfil de usuário) para permitir o Onboarding
pub async fn run_minimal_seeds(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] 🏎️ Inicialização mínima...");
    user_profile_seed::seed_user_profile(db).await?;
    Ok(())
}

pub async fn run_all_seeds(db: &Surreal<Db>) -> Result<(), String> {
    println!("\n[SEED] 🌱 Iniciando seeding do banco de dados...\n");

    // Nível 1: Dados básicos sem dependências
    currencies_seed::seed_currencies(db).await?;
    markets_seed::seed_markets(db, None).await?;
    emotional_states_seed::seed_emotional_states(db, None).await?;
    modalities_seed::seed_modalities(db, None).await?;
    timeframes_seed::seed_timeframes(db, None).await?;
    user_profile_seed::seed_user_profile(db).await?;
    tags_seed::seed_tags(db, None).await?;
    chart_types_seed::seed_chart_types(db, None).await?;
    fees_seed::seed_fees(db, None).await?;
    risk_seed::seed_risk_profiles(db, None).await?;
    tax_seed::seed_tax_rules(db).await?;

    // Nível 2: Dados que dependem do Nível 1
    asset_types_seed::seed_asset_types(db, None).await?;

    // Nível 3: Dados que dependem do Nível 2
    assets_seed::seed_assets(db, None).await?;
    indicators_seed::seed_indicators(db, None).await?;
    strategies_seed::seed_strategies(db, None).await?;

    // Nível 4: Dados Operacionais (Contas e Trades)
    demo_accounts_seed::seed_accounts(db, None).await?;
    demo_trades_seed::seed_all_demo_trades(db, None).await?;

    println!("\n[SEED] ✅ Seeding concluído com sucesso!\n");
    Ok(())
}

/// Inicialização base (Configurações, Sem dados operacionais/trades)
pub async fn run_base_seeds(db: &Surreal<Db>) -> Result<(), String> {
    println!("\n[SEED] 🚀 Iniciando seeding base (Configurações)...\n");

    // Nível 1: Dados básicos sem dependências
    currencies_seed::seed_currencies(db).await?;
    markets_seed::seed_markets(db, None).await?;
    emotional_states_seed::seed_emotional_states(db, None).await?;
    modalities_seed::seed_modalities(db, None).await?;
    timeframes_seed::seed_timeframes(db, None).await?;
    user_profile_seed::seed_user_profile(db).await?;
    tags_seed::seed_tags(db, None).await?;
    chart_types_seed::seed_chart_types(db, None).await?;
    fees_seed::seed_fees(db, None).await?;
    risk_seed::seed_risk_profiles(db, None).await?;
    tax_seed::seed_tax_rules(db).await?;

    // Nível 2: Dados que dependem do Nível 1
    asset_types_seed::seed_asset_types(db, None).await?;

    // Nível 3: Dados que dependem do Nível 2
    assets_seed::seed_assets(db, None).await?;
    indicators_seed::seed_indicators(db, None).await?;
    strategies_seed::seed_strategies(db, None).await?;

    println!("\n[SEED] ✅ Seeding base concluído!\n");
    Ok(())
}

/// Força o reseeding completo (deleta tudo e popula novamente)
pub async fn force_reseed_all(db: &Surreal<Db>) -> Result<(), String> {
    println!("\n[SEED] 🔥 FORCE RESEED: Deletando todos os dados...\n");

    // Deleta todas as tabelas na ordem inversa (para respeitar foreign keys)
    // Primeiro: dados dependentes (trades, transactions, journal)
    let result1 = db.query("
        REMOVE TABLE trade;
        REMOVE TABLE cash_transaction;
        REMOVE TABLE journal_entry;
        REMOVE TABLE account;
    ").await;
    
    if let Err(e) = result1 {
        println!("[SEED] ⚠️  Warning deleting user data: {}", e);
    } else {
        println!("[SEED] 🗑️ Trades, transactions e accounts removidos COMPLETO.");
    }

    // Segundo: dados configuracionais
    let result2 = db.query("
        REMOVE TABLE strategy;
        REMOVE TABLE indicator;
        REMOVE TABLE asset;
        REMOVE TABLE asset_type;
        REMOVE TABLE user_profile;
        REMOVE TABLE timeframe;
        REMOVE TABLE modality;
        REMOVE TABLE emotional_state;
        REMOVE TABLE market;
        REMOVE TABLE currency;
        REMOVE TABLE tag;
        REMOVE TABLE chart_type;
        REMOVE TABLE fee_profile;
        REMOVE TABLE risk_profile;
        REMOVE TABLE tax_rule;
        REMOVE TABLE tax_profile;
        REMOVE TABLE tax_profile_entry;
        REMOVE TABLE tax_mapping;
        REMOVE TABLE tax_appraisal;
        REMOVE TABLE tax_darf;
        REMOVE TABLE tax_loss;
    ").await;
    
    if let Err(e) = result2 {
        println!("[SEED] ⚠️  Warning deleting configuration data: {}", e);
    } else {
        println!("[SEED] 🗑️ Configurações deletadas.");
    }

    println!("[SEED] Iniciando reseeding...\n");

    // Executa seeding completo
    run_all_seeds(db).await?;

    println!("\n[SEED] ✅ Force reseed concluído com sucesso!\n");

    Ok(())
}

/// Executa apenas os seeds selecionados
pub async fn run_selective_seeds(db: &Surreal<Db>, selection: Vec<String>) -> Result<(), String> {
    println!("\n[SEED] 🧩 Iniciando seeding seletivo: {:?}\n", selection);

    let get_filter = |prefix: &str| -> Option<Vec<String>> {
        if selection.contains(&prefix.to_string()) {
            return None; // Se o módulo literal está presente, pega tudo
        }
        let filtered: Vec<String> = selection.iter()
            .filter(|s| s.starts_with(&format!("{}:", prefix)))
            .map(|s| s.replace(&format!("{}:", prefix), ""))
            .collect();
        
        if filtered.is_empty() { None } else { Some(filtered) }
    };

    let has_module = |name: &str| -> bool {
        selection.contains(&name.to_string()) || selection.iter().any(|s| s.starts_with(&format!("{}:", name)))
    };

    // Nível 1
    if has_module("currencies") { currencies_seed::seed_currencies(db).await?; }
    if has_module("markets") { markets_seed::seed_markets(db, get_filter("markets")).await?; }
    if has_module("emotional_states") { emotional_states_seed::seed_emotional_states(db, get_filter("emotional_states")).await?; }
    if has_module("modalities") { modalities_seed::seed_modalities(db, get_filter("modalities")).await?; }
    if has_module("timeframes") { timeframes_seed::seed_timeframes(db, get_filter("timeframes")).await?; }
    if has_module("user_profile") { user_profile_seed::seed_user_profile(db).await?; } // user_profile doesn't have filters
    if has_module("tags") { tags_seed::seed_tags(db, get_filter("tags")).await?; }
    if has_module("chart_types") { chart_types_seed::seed_chart_types(db, get_filter("chart_types")).await?; }
    if has_module("fees") { fees_seed::seed_fees(db, get_filter("fees")).await?; }
    if has_module("risk") { risk_seed::seed_risk_profiles(db, get_filter("risk")).await?; }

    // Nível 2
    if has_module("asset_types") || has_module("markets") || has_module("assets") { 
         // Derive filter for asset types from markets selection if possible, otherwise use selection
         // FIX: We need full strings like "markets:m1" for asset_types_seed filtering to work
         let market_filter = if selection.iter().any(|s| s.starts_with("markets:")) {
             Some(selection.iter().filter(|s| s.starts_with("markets:")).cloned().collect())
         } else {
             Some(selection.clone())
         };
         asset_types_seed::seed_asset_types(db, market_filter).await?;
    }

    // Always ensure accounts are created based on selected markets (even if demo data is off)
    // If "accounts" is explicitly selected, use its filter. Otherwise use market selection to determine accounts.
     // FIX: Account seeding also relies on filtering by full module name (markets:mX) OR demo_accounts logic
    // But demo_accounts logic expects "markets:mX" in required_modules check against filter
    // AND now also checking for account:id_suffix
    let account_filter = if selection.iter().any(|s| s.starts_with("markets:") || s.starts_with("account:")) {
             Some(selection.iter().filter(|s| s.starts_with("markets:") || s.starts_with("account:")).cloned().collect())
         } else {
             Some(selection.clone())
         };
    demo_accounts_seed::seed_accounts(db, account_filter).await?;

    // Nível 3
    // FIX: Assets are implied by Markets, so check has_module("markets") too
    if selection.contains(&"assets".to_string()) || has_module("assets") || has_module("markets") { 
        // FIX: Use full strings for assets_seed filtering too
        let market_filter = if selection.iter().any(|s| s.starts_with("markets:")) {
             Some(selection.iter().filter(|s| s.starts_with("markets:")).cloned().collect())
         } else {
             Some(selection.clone())
         };
        assets_seed::seed_assets(db, market_filter).await?; 
    }
    if has_module("indicators") { indicators_seed::seed_indicators(db, get_filter("indicators")).await?; }
    if has_module("strategies") { strategies_seed::seed_strategies(db, get_filter("strategies")).await?; }

    println!("\n[SEED] ✅ Seeding seletivo concluído!\n");
    Ok(())
}
