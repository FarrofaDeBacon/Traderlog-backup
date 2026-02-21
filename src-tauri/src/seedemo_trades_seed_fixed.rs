// src-tauri/src/seed/demo_trades_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use crate::models::Trade;
use chrono::{DateTime, Duration, Utc, TimeZone};
use rand::Rng;

// Helper function to generate random date in the last N days
fn random_date_in_last_days(days: i64)-> String {
    let mut rng = rand::thread_rng();
    let days_ago = rng.gen_range(0..days);
    let date = Utc::now() - Duration::days(days_ago);
    date.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// Helper to generate realistic P&L based on market
fn generate_result(market: &str, is_win: bool, is_breakeven: bool) -> f64 {
    let mut rng = rand::thread_rng();
    if is_breakeven {
        return rng.gen_range(-10.0..10.0);
    }
    
    match market {
        "FOREX" => if is_win{ rng.gen_range(50.0..500.0) } else { rng.gen_range(-400.0..-30.0) },
        "B3_ACOES" => if is_win { rng.gen_range(100.0..1500.0) } else { rng.gen_range(-1200.0..-80.0) },
        "B3_FUTUROS" => if is_win { rng.gen_range(80.0..800.0) } else { rng.gen_range(-700.0..-50.0) },
        "NASDAQ" => if is_win { rng.gen_range(200.0..2500.0) } else { rng.gen_range(-2000.0..-150.0) },
        "CRYPTO" => if is_win { rng.gen_range(100.0..1000.0) } else { rng.gen_range(-900.0..-80.0) },
        _ => if is_win { 100.0 } else { -100.0 },
    }
}

pub async fn seed_demo_forex_trades(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Gerando trades de demonstração para FOREX...");
    
    let assets = vec!["EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD"];
    let strategies = vec!["strategy:price_action", "strategy:scalping", "strategy:breakout", "strategy:trend"];
    let timeframes = vec!["5m", "15m", "1H", "4H", "1D"];
    let emotional_states = vec![
        ("emotional_state:confidence", "Confiança"),
        ("emotional_state:discipline", "Disciplina"),
        ("emotional_state:fear", "Medo"),
        ("emotional_state:greed", "Ganância"),
        ("emotional_state:patience", "Paciência"),
    ];
    
    let mut rng = rand::thread_rng();
    
    for i in 1..=60 {
        let is_win = rng.gen_bool(0.55); // 55% win rate
        let is_breakeven = !is_win && rng.gen_bool(0.22); // 10% of total
        let is_long = rng.gen_bool(0.6);
        
        let asset = assets[rng.gen_range(0..assets.len())];
        let strategy = strategies[rng.gen_range(0..strategies.len())];
        let timeframe = timeframes[rng.gen_range(0..timeframes.len())];
        let (entry_state_id, entry_state_name) = emotional_states[rng.gen_range(0..emotional_states.len())];
        let (exit_state_id, exit_state_name) = emotional_states[rng.gen_range(0..emotional_states.len())];
        
        let entry_price: f64 = rng.gen_range(1.05..1.35);
        let result = generate_result("FOREX", is_win, is_breakeven);
        let quantity: f64 = rng.gen_range(1000.0..10000.0);
        let exit_price = if is_long {
            entry_price + (result / quantity)
        } else {
            entry_price - (result / quantity)
        };
        
        let entry_date = random_date_in_last_days(90);
        let exit_date = {
            let entry_dt = DateTime::parse_from_rfc3339(&entry_date).unwrap();
            let hours = rng.gen_range(1..12);
            (entry_dt + Duration::hours(hours)).format("%Y-%m-%dT%H:%M:%SZ").to_string()
        };
        
        let trade = Trade {
            id: format!("trade:demo_forex_{:03}", i),
            date: entry_date.clone(),
            asset_symbol: asset.to_string(),
            strategy_id: strategy.to_string(),
            account_id: "account:demo_forex".to_string(),
            result,
            quantity,
            direction: if is_long { "Buy".to_string() } else { "Sell".to_string() },
            entry_price,
            exit_price: Some(exit_price),
            exit_date: Some(exit_date),
            fee_total: rng.gen_range(5.0..25.0),
            notes: format!("Trade demo {} em {}", if is_win { "WIN" } else if is_breakeven { "BREAKEVEN" } else { "LOSS" }, asset),
            timeframe: timeframe.to_string(),
            volatility: if rng.gen_bool(0.5) { "Alta".to_string() } else { "Média".to_string() },
            entry_emotional_state_id: Some(entry_state_id.to_string()),
            entry_emotional_state_name: Some(entry_state_name.to_string()),
            exit_reason: Some(if is_win {"Take Profit".to_string() } else { "Stop Loss".to_string() }),
            exit_emotional_state_id: Some(exit_state_id.to_string()),
            exit_emotional_state_name: Some(exit_state_name.to_string()),
            entry_rationale: "Análise técnica indica oportunidade".to_string(),
            confirmation_signals: "Confluência de indicadores".to_string(),
            market_context: "Tendência de alta no par".to_string(),
            relevant_news: "Notícias econômicas favoráveis".to_string(),
            followed_plan: rng.gen_bool(0.8),
            what_worked: if is_win { "Estratégia bem executada".to_string() } else { "".to_string() },
            mistakes_improvements: if !is_win { "Melhorar gestão de risco".to_string() } else { "".to_string() },
            lessons_learned: "Manter disciplina no plano".to_string(),
            images: vec![],
            partial_exits: serde_json::json!([]),
        };
        
        // Check if exists
        let mut query_result = db.query(&format!("SELECT * FROM {}", trade.id)).await.map_err(|e| e.to_string())?;
        let existing: Vec<Trade> = query_result.take(0).map_err(|e| e.to_string())?;
        
        if existing.is_empty() {
            db.query(&format!("CREATE {} CONTENT $data", trade.id))
                .bind(("data", &trade))
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    
    println!("  ✓ 60 trades FOREX criados");
    Ok(())
}

pub async fn seed_demo_b3_acoes_trades(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Gerando trades de demonstração para B3 Ações...");
    
    let assets = vec!["PETR4", "VALE3", "ITUB4", "BBDC4", "ABEV3", "MGLU3", "B3SA3"];
    let strategies = vec!["strategy:tape_reading", "strategy:order_flow", "strategy:swing"];
    let timeframes = vec!["15m", "1H", "1D", "1W"];
    let emotional_states = vec![
        ("emotional_state:confidence", "Confiança"),
        ("emotional_state:discipline", "Disciplina"),
        ("emotional_state:focus", "Foco"),
        ("emotional_state:anxiety", "Ansiedade"),
    ];
    
    let mut rng = rand::thread_rng();
    
    for i in 1..=55 {
        let is_win = rng.gen_bool(0.55);
        let is_breakeven = !is_win && rng.gen_bool(0.22);
        let is_long = rng.gen_bool(0.65);
        
        let asset = assets[rng.gen_range(0..assets.len())];
        let strategy = strategies[rng.gen_range(0..strategies.len())];
        let timeframe = timeframes[rng.gen_range(0..timeframes.len())];
        let (entry_state_id, entry_state_name) = emotional_states[rng.gen_range(0..emotional_states.len())];
        
        let entry_price: f64 = rng.gen_range(10.0..45.0);
        let result = generate_result("B3_ACOES", is_win, is_breakeven);
        let quantity: f64 = rng.gen_range(100.0..1000.0);
        let exit_price = if is_long {
            entry_price + (result / quantity)
        } else {
            entry_price - (result / quantity)
        };
        
        let entry_date = random_date_in_last_days(90);
        let exit_date = {
            let entry_dt = DateTime::parse_from_rfc3339(&entry_date).unwrap();
            let hours = if timeframe == "1D" || timeframe == "1W" { rng.gen_range(24..168) } else { rng.gen_range(1..8) };
            (entry_dt + Duration::hours(hours)).format("%Y-%m-%dT%H:%M:%SZ").to_string()
        };
        
        let trade = Trade {
            id: format!("trade:demo_b3_acoes_{:03}", i),
            date: entry_date.clone(),
            asset_symbol: asset.to_string(),
            strategy_id: strategy.to_string(),
            account_id: "account:demo_b3_acoes".to_string(),
            result,
            quantity,
            direction: if is_long { "Buy".to_string() } else { "Sell".to_string() },
            entry_price,
            exit_price: Some(exit_price),
            exit_date: Some(exit_date),
            fee_total: rng.gen_range(15.0..80.0),
            notes: format!("Operação {} em {}", if is_win { "lucrativa" } else { "com perda" }, asset),
            timeframe: timeframe.to_string(),
            volatility: "Média".to_string(),
            entry_emotional_state_id: Some(entry_state_id.to_string()),
            entry_emotional_state_name: Some(entry_state_name.to_string()),
            exit_reason: Some(if is_win { "Alvo atingido".to_string() } else { "Stop acionado".to_string() }),
            exit_emotional_state_id: Some(entry_state_id.to_string()),
            exit_emotional_state_name: Some(entry_state_name.to_string()),
            entry_rationale: "Setup conforme estratégia".to_string(),
            confirmation_signals: "Volume e price action confirmaram".to_string(),
            market_context: "Mercado em tendência".to_string(),
            relevant_news: "Balanço trimestral divulgado".to_string(),
            followed_plan: rng.gen_bool(0.75),
            what_worked: if is_win { "Entrada precisa".to_string() } else { "".to_string() },
            mistakes_improvements: if !is_win { "Ajustar stop loss".to_string() } else { "".to_string() },
            lessons_learned: "Paciência e timing são fundamentais".to_string(),
            images: vec![],
            partial_exits: serde_json::json!([]),
        };
        
        let mut query_result = db.query(&format!("SELECT * FROM {}", trade.id)).await.map_err(|e| e.to_string())?;
        let existing: Vec<Trade> = query_result.take(0).map_err(|e| e.to_string())?;
        
        if existing.is_empty() {
            db.query(&format!("CREATE {} CONTENT $data", trade.id))
                .bind(("data", &trade))
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    
    println!("  ✓ 55 trades B3 Ações criados");
    Ok(())
}

pub async fn seed_demo_b3_futuros_trades(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Gerando trades de demonstração para B3 Futuros...");
    
    let assets = vec!["WIN", "WDO", "BGI"];
    let strategies = vec!["strategy:scalping", "strategy:fluxo", "strategy:volume_profile"];
    let timeframes = vec!["1m", "5m", "15m"];
    
    let mut rng = rand::thread_rng();
    
    for i in 1..=70 {
        let is_win = rng.gen_bool(0.57);
        let is_breakeven = !is_win && rng.gen_bool(0.18);
        let is_long = rng.gen_bool(0.6);
        
        let asset = assets[rng.gen_range(0..assets.len())];
        let strategy = strategies[rng.gen_range(0..strategies.len())];
        let timeframe = timeframes[rng.gen_range(0..timeframes.len())];
        
        let entry_price: f64 = if asset == "WIN" {
            rng.gen_range(100000.0..120000.0)
        } else if asset == "WDO" {
            rng.gen_range(5000.0..5800.0)
        } else {
            rng.gen_range(110000.0..130000.0)
        };
        
        let result = generate_result("B3_FUTUROS", is_win, is_breakeven);
        let quantity: f64 = rng.gen_range(1.0..5.0);
        let exit_price = if is_long {
            entry_price + (result / quantity)
        } else {
            entry_price - (result / quantity)
        };
        
        let entry_date = random_date_in_last_days(60);
        let exit_date = {
            let entry_dt = DateTime::parse_from_rfc3339(&entry_date).unwrap();
            let minutes = rng.gen_range(5..180);
            (entry_dt + Duration::minutes(minutes)).format("%Y-%m-%dT%H:%M:%SZ").to_string()
        };
        
        let trade = Trade {
            id: format!("trade:demo_b3_futuros_{:03}", i),
            date: entry_date.clone(),
            asset_symbol: asset.to_string(),
            strategy_id: strategy.to_string(),
            account_id: "account:demo_b3_futuros".to_string(),
            result,
            quantity,
            direction: if is_long { "Buy".to_string() } else { "Sell".to_string() },
            entry_price,
            exit_price: Some(exit_price),
            exit_date: Some(exit_date),
            fee_total: rng.gen_range(2.0..15.0),
            notes: format!("Scalping em {}", asset),
            timeframe: timeframe.to_string(),
            volatility: "Alta".to_string(),
            entry_emotional_state_id: Some("emotional_state:discipline".to_string()),
            entry_emotional_state_name: Some("Disciplina".to_string()),
            exit_reason: Some("Objetivo alcançado".to_string()),
            exit_emotional_state_id: Some("emotional_state:discipline".to_string()),
            exit_emotional_state_name: Some("Disciplina".to_string()),
            entry_rationale: "Rompimento de suporte/resistência".to_string(),
            confirmation_signals: "Volume acima da média".to_string(),
            market_context: "Alta volatilidade intraday".to_string(),
            relevant_news: "".to_string(),
            followed_plan: rng.gen_bool(0.85),
            what_worked: if is_win { "Execução rápida".to_string() } else { "".to_string() },
            mistakes_improvements: if !is_win { "Melhorar timing de entrada".to_string() } else { "".to_string() },
            lessons_learned: "Scalping requer disciplina máxima".to_string(),
            images: vec![],
            partial_exits: serde_json::json!([]),
        };
        
        let mut query_result = db.query(&format!("SELECT * FROM {}", trade.id)).await.map_err(|e| e.to_string())?;
        let existing: Vec<Trade> = query_result.take(0).map_err(|e| e.to_string())?;
        
        if existing.is_empty() {
            db.query(&format!("CREATE {} CONTENT $data", trade.id))
                .bind(("data", &trade))
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    
    println!("  ✓ 70 trades B3 Futuros criados");
    Ok(())
}

pub async fn seed_demo_nasdaq_trades(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Gerando trades de demonstração para NASDAQ...");
    
    let assets = vec!["AAPL", "TSLA", "NVDA", "MSFT", "GOOGL", "AMZN", "META"];
    let strategies = vec!["strategy:momentum", "strategy:gap_trading", "strategy:technical"];
    let timeframes = vec!["15m", "1H", "4H", "1D"];
    
    let mut rng = rand::thread_rng();
    
    for i in 1..=60 {
        let is_win = rng.gen_bool(0.54);
        let is_breakeven = !is_win && rng.gen_bool(0.2);
        let is_long = rng.gen_bool(0.62);
        
        let asset = assets[rng.gen_range(0..assets.len())];
        let strategy = strategies[rng.gen_range(0..strategies.len())];
        let timeframe = timeframes[rng.gen_range(0..timeframes.len())];
        
        let entry_price: f64 = match asset {
            "AAPL" => rng.gen_range(180.0..220.0),
            "TSLA" => rng.gen_range(200.0..350.0),
            "NVDA" => rng.gen_range(500.0..900.0),
            "MSFT" => rng.gen_range(380.0..450.0),
            "GOOGL" => rng.gen_range(140.0..180.0),
            "AMZN" => rng.gen_range(160.0..200.0),
            "META" => rng.gen_range(400.0..550.0),
            _ => 100.0,
        };
        
        let result = generate_result("NASDAQ", is_win, is_breakeven);
        let quantity: f64 = rng.gen_range(10.0..100.0);
        let exit_price = if is_long {
            entry_price + (result / quantity)
        } else {
            entry_price - (result / quantity)
        };
        
        let entry_date = random_date_in_last_days(90);
        let exit_date = {
            let entry_dt = DateTime::parse_from_rfc3339(&entry_date).unwrap();
            let hours = if timeframe == "1D" { rng.gen_range(24..120) } else { rng.gen_range(2..12) };
            (entry_dt + Duration::hours(hours)).format("%Y-%m-%dT%H:%M:%SZ").to_string()
        };
        
        let trade = Trade {
            id: format!("trade:demo_nasdaq_{:03}", i),
            date: entry_date.clone(),
            asset_symbol: asset.to_string(),
            strategy_id: strategy.to_string(),
            account_id: "account:demo_nasdaq".to_string(),
            result,
            quantity,
            direction: if is_long { "Buy".to_string() } else { "Sell".to_string() },
            entry_price,
            exit_price: Some(exit_price),
            exit_date: Some(exit_date),
            fee_total: rng.gen_range(10.0..50.0),
            notes: format!("Trade {} em {}", if is_win { "positivo" } else { "negativo" }, asset),
            timeframe: timeframe.to_string(),
            volatility: if rng.gen_bool(0.6) { "Alta".to_string() } else { "Média".to_string() },
            entry_emotional_state_id: Some("emotional_state:confidence".to_string()),
            entry_emotional_state_name: Some("Confiança".to_string()),
            exit_reason: Some(if is_win { "Target reached".to_string() } else { "Cut loss".to_string() }),
            exit_emotional_state_id: Some("emotional_state:discipline".to_string()),
            exit_emotional_state_name: Some("Disciplina".to_string()),
            entry_rationale: "Technical setup aligned".to_string(),
            confirmation_signals: "Multiple indicators confirmed".to_string(),
            market_context: "US market showing strength".to_string(),
            relevant_news: "Earnings report positive".to_string(),
            followed_plan: rng.gen_bool(0.78),
            what_worked: if is_win { "Good entry timing".to_string() } else { "".to_string() },
            mistakes_improvements: if !is_win { "Better risk management needed".to_string() } else { "".to_string() },
            lessons_learned: "Stay patient for setup".to_string(),
            images: vec![],
            partial_exits: serde_json::json!([]),
        };
        
        let mut query_result = db.query(&format!("SELECT * FROM {}", trade.id)).await.map_err(|e| e.to_string())?;
        let existing: Vec<Trade> = query_result.take(0).map_err(|e| e.to_string())?;
        
        if existing.is_empty() {
            db.query(&format!("CREATE {} CONTENT $data", trade.id))
                .bind(("data", &trade))
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    
    println!("  ✓ 60 trades NASDAQ criados");
    Ok(())
}

pub async fn seed_demo_crypto_trades(db: &Surreal<Db>) -> Result<(), String> {
    println!("[SEED] Gerando trades de demonstração para Crypto...");
    
    let assets = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT"];
    let strategies = vec!["strategy:breakout", "strategy:support_resistance", "strategy:moving_averages"];
    let timeframes = vec!["15m", "1H", "4H", "1D"];
    
    let mut rng = rand::thread_rng();
    
    for i in 1..=65 {
        let is_win = rng.gen_bool(0.53);
        let is_breakeven = !is_win && rng.gen_bool(0.21);
        let is_long = rng.gen_bool(0.58);
        
        let asset = assets[rng.gen_range(0..assets.len())];
        let strategy = strategies[rng.gen_range(0..strategies.len())];
        let timeframe = timeframes[rng.gen_range(0..timeframes.len())];
        
        let entry_price: f64 = match asset {
            "BTCUSDT" => rng.gen_range(60000.0..95000.0),
            "ETHUSDT" => rng.gen_range(3000.0..4500.0),
            "BNBUSDT" => rng.gen_range(500.0..700.0),
            "SOLUSDT" => rng.gen_range(100.0..200.0),
            "ADAUSDT" => rng.gen_range(0.8..1.2),
            _ => 100.0,
        };
        
        let result = generate_result("CRYPTO", is_win, is_breakeven);
        let quantity: f64 = rng.gen_range(0.1..2.0);
        let exit_price = if is_long {
            entry_price + (result / quantity)
        } else {
            entry_price - (result / quantity)
        };
        
        let entry_date = random_date_in_last_days(90);
        let exit_date = {
            let entry_dt = DateTime::parse_from_rfc3339(&entry_date).unwrap();
            let hours = rng.gen_range(2..72);
            (entry_dt + Duration::hours(hours)).format("%Y-%m-%dT%H:%M:%SZ").to_string()
        };
        
        let trade = Trade {
            id: format!("trade:demo_crypto_{:03}", i),
            date: entry_date.clone(),
            asset_symbol: asset.to_string(),
            strategy_id: strategy.to_string(),
            account_id: "account:demo_crypto".to_string(),
            result,
            quantity,
            direction: if is_long { "Buy".to_string() } else { "Sell".to_string() },
            entry_price,
            exit_price: Some(exit_price),
            exit_date: Some(exit_date),
            fee_total: rng.gen_range(5.0..40.0),
            notes: format!("Crypto trade em {}", asset),
            timeframe: timeframe.to_string(),
            volatility: "Alta".to_string(),
            entry_emotional_state_id: Some("emotional_state:confidence".to_string()),
            entry_emotional_state_name: Some("Confiança".to_string()),
            exit_reason: Some(if is_win { "TP hit".to_string() } else { "SL triggered".to_string() }),
            exit_emotional_state_id: Some("emotional_state:discipline".to_string()),
            exit_emotional_state_name: Some("Disciplina".to_string()),
            entry_rationale: "Clear breakout pattern".to_string(),
            confirmation_signals: "Volume spike confirmed".to_string(),
            market_context: "Bullish crypto sentiment".to_string(),
            relevant_news: "Positive crypto news".to_string(),
            followed_plan: rng.gen_bool(0.72),
            what_worked: if is_win { "Breakout strategy worked".to_string() } else { "".to_string() },
            mistakes_improvements: if !is_win { "Tighten stop loss".to_string() } else { "".to_string() },
            lessons_learned: "Crypto volatility requires wider stops".to_string(),
            images: vec![],
            partial_exits: serde_json::json!([]),
        };
        
        let mut query_result = db.query(&format!("SELECT * FROM {}", trade.id)).await.map_err(|e| e.to_string())?;
        let existing: Vec<Trade> = query_result.take(0).map_err(|e| e.to_string())?;
        
        if existing.is_empty() {
            db.query(&format!("CREATE {} CONTENT $data", trade.id))
                .bind(("data", &trade))
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    
    println!("  ✓ 65 trades Crypto criados");
    Ok(())
}

pub async fn seed_all_demo_trades(db: &Surreal<Db>) -> Result<(), String> {
    seed_demo_forex_trades(db).await?;
    seed_demo_b3_acoes_trades(db).await?;
    seed_demo_b3_futuros_trades(db).await?;
    seed_demo_nasdaq_trades(db).await?;
    seed_demo_crypto_trades(db).await?;
    
    println!("[SEED] ✅ Total de 310 trades de demonstração criados!");
    Ok(())
}
