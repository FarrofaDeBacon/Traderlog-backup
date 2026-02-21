// src-tauri/src/seed/demo_trades_seed.rs
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use rand::Rng;
use chrono::{NaiveDate, Duration, Datelike, Weekday};
use std::collections::BTreeMap;

/// Pre-generates all trade SQL strings synchronously, then executes them
pub async fn seed_all_demo_trades(db: &Surreal<Db>, _filter: Option<Vec<String>>) -> Result<(), String> {
    println!("[SEED] 🎯 Gerando trades demo de Jan/2024 a Fev/2026...");

    // Generate all SQL statements synchronously (rng is not Send)
    let (trade_sqls, closing_sqls) = generate_all_sqls();
    let total_trades = trade_sqls.len();
    let total_closings = closing_sqls.len();
    println!("[SEED] Gerados {} trades + {} fechamentos diários. Inserindo...", total_trades, total_closings);

    // Insert trades
    for (i, sql) in trade_sqls.iter().enumerate() {
        if let Err(e) = db.query(sql.as_str()).await {
            println!("[SEED] ⚠️ Error trade {}: {}", i + 1, e);
        }
        if (i + 1) % 100 == 0 {
            println!("[SEED] ... {}/{} trades", i + 1, total_trades);
        }
    }

    // Insert daily closings
    for (i, sql) in closing_sqls.iter().enumerate() {
        if let Err(e) = db.query(sql.as_str()).await {
            println!("[SEED] ⚠️ Error closing {}: {}", i + 1, e);
        }
    }

    println!("[SEED] ✅ {} trades + {} fechamentos criados!", total_trades, total_closings);
    Ok(())
}

/// Generates all trade + closing SQL strings synchronously
fn generate_all_sqls() -> (Vec<String>, Vec<String>) {
    let strategies = ["s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11"];
    let emotionals = ["e1", "e2", "e3", "e4", "e5", "e6", "e7", "e8", "e9", "e10", "e11", "e12"];
    let timeframes = ["1m", "5m", "15m", "60m"];
    let contexts = [
        "Tendencia de Alta", "Consolidacao", "Rompimento", "Reversao", "Pullback",
        "Abertura Forte", "Leilao de Fechamento", "Volatilidade Elevada",
    ];
    let signals = [
        "Candle de Forca", "Divergencia RSI", "Volume Alto", "Medias Alinhadas",
        "Fibonacci 61.8", "Tape Reading", "Fluxo Comprador", "Rompimento VWAP",
    ];

    let mut rng = rand::thread_rng();
    let mut trade_sqls = Vec::new();
    let mut daily_results: BTreeMap<NaiveDate, (f64, Vec<String>)> = BTreeMap::new(); // date -> (net_result, [trade_ids])
    let mut trade_count = 0u32;

    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 2, 21).unwrap();

    // Find first Monday
    let mut week_start = start_date;
    while week_start.weekday() != Weekday::Mon {
        week_start = week_start + Duration::days(1);
    }

    while week_start < end_date {
        let trades_this_week = rng.gen_range(5..=8);

        for _ in 0..trades_this_week {
            trade_count += 1;

            let day_offset = rng.gen_range(0..5i64);
            let trade_date = week_start + Duration::days(day_offset);
            if trade_date >= end_date { break; }

            let is_win = rng.gen_bool(0.6);
            let (symbol, base_price, tick_size, point_value) = if is_win {
                ("WIN", 128000.0_f64, 5.0_f64, 0.20_f64)
            } else {
                ("WDO", 5100.0_f64, 0.5_f64, 10.0_f64)
            };

            let is_day_trade = rng.gen_bool(0.65);
            let is_buy = rng.gen_bool(0.5);
            let direction = if is_buy { "Buy" } else { "Sell" };
            let quantity = rng.gen_range(1..=5) as f64;

            let price_var = rng.gen_range(-0.02_f64..0.02) * base_price;
            let entry_price = ((base_price + price_var) / tick_size).round() * tick_size;

            let is_win_trade = rng.gen_bool(0.60);
            let ticks = rng.gen_range(2..=40) as f64;

            let exit_price = if is_win_trade {
                if is_buy { entry_price + ticks * tick_size } else { entry_price - ticks * tick_size }
            } else {
                if is_buy { entry_price - ticks * tick_size } else { entry_price + ticks * tick_size }
            };

            let gross_result = if is_win_trade {
                ticks * tick_size * point_value * quantity
            } else {
                -(ticks * tick_size * point_value * quantity)
            };

            let fee_total = (quantity * 1.50 + 0.50).round();
            let result = ((gross_result - fee_total) * 100.0).round() / 100.0;

            let entry_hour = rng.gen_range(9..=16);
            let entry_min = rng.gen_range(0..=59);
            let date_str = format!("{}T{:02}:{:02}:00Z", trade_date, entry_hour, entry_min);

            let exit_day = if is_day_trade {
                trade_date
            } else {
                let mut d = trade_date + Duration::days(rng.gen_range(1..=5));
                while d.weekday() == Weekday::Sat || d.weekday() == Weekday::Sun {
                    d = d + Duration::days(1);
                }
                d
            };
            let exit_hour = rng.gen_range(9..=17);
            let exit_min = rng.gen_range(0..=59);
            let exit_date_str = format!("{}T{:02}:{:02}:00Z", exit_day, exit_hour, exit_min);

            let strategy = strategies[rng.gen_range(0..strategies.len())];
            let entry_emo = emotionals[rng.gen_range(0..emotionals.len())];
            let exit_emo = emotionals[rng.gen_range(0..emotionals.len())];
            let timeframe = timeframes[rng.gen_range(0..timeframes.len())];
            let context = contexts[rng.gen_range(0..contexts.len())];
            let signal = signals[rng.gen_range(0..signals.len())];

            let sl_ticks = rng.gen_range(5..=20) as f64;
            let tp_ticks = rng.gen_range(10..=40) as f64;
            let (stop_loss, take_profit) = if is_buy {
                (entry_price - sl_ticks * tick_size, entry_price + tp_ticks * tick_size)
            } else {
                (entry_price + sl_ticks * tick_size, entry_price - tp_ticks * tick_size)
            };

            let volatility = if ticks > 25.0 { "Alta" } else if ticks > 10.0 { "Media" } else { "Baixa" };
            let followed = if is_win_trade { rng.gen_bool(0.8) } else { rng.gen_bool(0.4) };
            let intensity = rng.gen_range(3..=9) as f64;
            let mod_id = if is_day_trade { "mod1" } else { "mod2" };

            let trade_id = format!("trade:{}", trade_count);

            // Track daily results for closing entries
            let entry = daily_results.entry(trade_date).or_insert((0.0, Vec::new()));
            entry.0 += result;
            entry.1.push(trade_id.clone());

            let sql = format!(
                "CREATE trade:{} SET \
                    date = '{date}', \
                    asset_symbol = '{symbol}', \
                    asset_type_id = type::thing('asset_type', 'at2'), \
                    strategy_id = type::thing('strategy', '{strategy}'), \
                    account_id = type::thing('account', 'demo_b3_futuros'), \
                    result = {result}, \
                    quantity = {qty}, \
                    direction = '{direction}', \
                    entry_price = {entry_p}, \
                    exit_price = {exit_p}, \
                    exit_date = '{exit_date}', \
                    fee_total = {fee}, \
                    notes = '', \
                    timeframe = '{tf}', \
                    volatility = '{vol}', \
                    modality_id = type::thing('modality', '{mod_id}'), \
                    stop_loss = {sl}, \
                    take_profit = {tp}, \
                    intensity = {intensity}, \
                    entry_emotional_state_id = type::thing('emotional_state', '{entry_emo}'), \
                    exit_emotional_state_id = type::thing('emotional_state', '{exit_emo}'), \
                    entry_rationale = '{signal}', \
                    confirmation_signals = '{signal}', \
                    market_context = '{context}', \
                    relevant_news = '', \
                    followed_plan = {followed}, \
                    what_worked = '', \
                    mistakes_improvements = '', \
                    lessons_learned = '', \
                    images = [], \
                    partial_exits = []",
                trade_count,
                date = date_str,
                symbol = symbol,
                strategy = strategy,
                result = result,
                qty = quantity,
                direction = direction,
                entry_p = entry_price,
                exit_p = exit_price,
                exit_date = exit_date_str,
                fee = fee_total,
                tf = timeframe,
                vol = volatility,
                mod_id = mod_id,
                sl = stop_loss,
                tp = take_profit,
                intensity = intensity,
                entry_emo = entry_emo,
                exit_emo = exit_emo,
                signal = signal,
                context = context,
                followed = followed,
            );

            trade_sqls.push(sql);
        }

        week_start = week_start + Duration::weeks(1);
    }

    // Generate daily closing cash_transactions (Fechamento Diário)
    let mut closing_sqls = Vec::new();
    let mut closing_count = 0u32;

    for (date, (net_result, trade_ids)) in &daily_results {
        closing_count += 1;
        let rounded_result = (net_result * 100.0).round() / 100.0;
        let tx_type = if rounded_result >= 0.0 { "Deposit" } else { "Withdraw" };
        let date_str = format!("{}T17:30:00Z", date);
        let desc = format!("Fechamento Diario {}", date.format("%d/%m/%Y"));
        let num_trades = trade_ids.len();

        // Amount: positive for deposit, use (0 - X) for negative to avoid SQL parse error
        let amount_expr = if rounded_result >= 0.0 {
            format!("{}", rounded_result)
        } else {
            format!("(0 - {})", rounded_result.abs())
        };

        let sql = format!(
            "CREATE cash_transaction:daily_closing_{} SET \
                date = '{date}', \
                amount = {amount}, \
                type = '{tx_type}', \
                description = '{desc} - {count} trades', \
                account_id = 'account:demo_b3_futuros'",
            closing_count,
            date = date_str,
            amount = amount_expr,
            tx_type = tx_type,
            desc = desc,
            count = num_trades,
        );

        closing_sqls.push(sql);
    }

    (trade_sqls, closing_sqls)
}

// Legacy compatibility stubs
pub async fn seed_demo_forex_trades(db: &Surreal<Db>) -> Result<(), String> { 
    seed_all_demo_trades(db, None).await 
}
pub async fn seed_demo_b3_acoes_trades(db: &Surreal<Db>) -> Result<(), String> { 
    seed_all_demo_trades(db, None).await 
}
pub async fn seed_demo_b3_futuros_trades(db: &Surreal<Db>) -> Result<(), String> { 
    seed_all_demo_trades(db, None).await 
}
pub async fn seed_demo_nasdaq_trades(db: &Surreal<Db>) -> Result<(), String> { 
    seed_all_demo_trades(db, None).await 
}
pub async fn seed_demo_crypto_trades(db: &Surreal<Db>) -> Result<(), String> { 
    seed_all_demo_trades(db, None).await 
}
