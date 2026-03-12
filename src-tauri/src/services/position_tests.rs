#[cfg(test)]
mod tests {
    use crate::services::position_service::PositionService;
    use crate::models::Trade;

    fn create_mock_trade(id: &str, symbol: &str, direction: &str, qty: f64, price: f64, fees: f64, date: &str) -> Trade {
        Trade {
            id: id.to_string(),
            date: date.to_string(),
            asset_symbol: symbol.to_string(),
            asset_type_id: Some("at1".to_string()),
            strategy_id: Some("s1".to_string()),
            account_id: Some("acc1".to_string()),
            result: 0.0,
            quantity: qty,
            direction: direction.to_string(),
            entry_price: price,
            exit_price: if direction == "Sell" { Some(price) } else { None },
            exit_date: if direction == "Sell" { Some(date.to_string()) } else { None },
            fee_total: fees,
            notes: "".to_string(),
            timeframe: "".to_string(),
            volatility: "".to_string(),
            entry_emotional_state_id: None,
            entry_emotional_state_name: None,
            exit_reason: None,
            exit_emotional_state_id: None,
            exit_emotional_state_name: None,
            entry_rationale: "".to_string(),
            confirmation_signals: "".to_string(),
            market_context: "".to_string(),
            relevant_news: "".to_string(),
            followed_plan: true,
            what_worked: "".to_string(),
            mistakes_improvements: "".to_string(),
            lessons_learned: "".to_string(),
            images: vec![],
            partial_exits: crate::models::SurrealJson(serde_json::Value::Array(vec![])),
            asset_id: None,
            modality_id: Some("mod1".to_string()),
            stop_loss: None,
            take_profit: None,
            intensity: 0.0,
        }
    }

    #[test]
    fn test_pm_calculation_basic() {
        let trades = vec![
            create_mock_trade("1", "PETR4", "Buy", 100.0, 30.0, 10.0, "2026-01-01"),
            create_mock_trade("2", "PETR4", "Buy", 100.0, 32.0, 10.0, "2026-01-02"),
        ];

        let positions = PositionService::calculate_positions(&trades);
        let petr4 = positions.get("PETR4").unwrap();

        // Custo total = (100 * 30 + 10) + (100 * 32 + 10) = 3010 + 3210 = 6220
        // PM = 6220 / 200 = 31.1
        assert!((petr4.quantity - 200.0).abs() < 0.001);
        assert!((petr4.average_price - 31.1).abs() < 0.001);
    }

    #[test]
    fn test_trade_result_with_pm() {
        let pm = 31.1;
        let sell_trade = create_mock_trade("3", "PETR4", "Sell", 100.0, 35.0, 15.0, "2026-01-05");
        
        // Resultado = 100 * (35.0 - 31.1) - 15.0 = 100 * 3.9 - 15.0 = 390 - 15 = 375
        let result = PositionService::calculate_trade_result(&sell_trade, pm);
        assert!((result - 375.0).abs() < 0.001);
    }

    #[test]
    fn test_pm_does_not_change_on_sell() {
        let trades = vec![
            create_mock_trade("1", "PETR4", "Buy", 100.0, 30.0, 10.0, "2026-01-01"),
            create_mock_trade("2", "PETR4", "Sell", 50.0, 35.0, 5.0, "2026-01-02"),
        ];

        let positions = PositionService::calculate_positions(&trades);
        let petr4 = positions.get("PETR4").unwrap();

        // PM inicial = (100 * 30 + 10) / 100 = 30.1
        // Qtd final = 50
        // PM final deve continuar 30.1
        assert!((petr4.quantity - 50.0).abs() < 0.001);
        assert!((petr4.average_price - 30.1).abs() < 0.001);
    }
}
