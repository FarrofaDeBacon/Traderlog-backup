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
            exit_price: None,
            exit_date: None,
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
        
        // Resultado = 100 * (35.0 - 31.1) * 1.0 (Stock) - 15.0 = 100 * 3.9 - 15.0 = 390 - 15 = 375
        let result = PositionService::calculate_trade_result(&sell_trade, pm, 1.0);
        assert!((result - 375.0).abs() < 0.001);
    }

    #[test]
    fn test_trade_result_mini_indice() {
        // WIN (Mini Índice)
        // Buy @ 120.000, Sell @ 121.000 (1.000 points)
        // Multiplier (point_value) = 0.2
        // Profit = 1 contract * 1.000 pts * 0.2 R$/pt = R$ 200,00
        let mut trade = create_mock_trade("4", "WINJ24", "Buy", 1.0, 120000.0, 0.0, "2026-03-01");
        trade.exit_price = Some(121000.0);
        
        let result = PositionService::calculate_trade_result(&trade, 0.0, 0.2);
        assert!((result - 200.0).abs() < 0.001);
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

    #[test]
    fn test_trade_result_with_partial_exit_discrepancy() {
        // Reproducing the R$ 485 discrepancy found in March
        // Initial Entry: Buy 3 @ 184455
        let mut trade = create_mock_trade("5", "WINJ24", "Buy", 3.0, 184455.0, 0.0, "2026-03-13T10:00:00");
        
        // Final Exit: 2 @ 186240 (the logic assumes remaining_qty exits here)
        trade.exit_price = Some(186240.0);
        
        // Partial Exit: 1 @ 183815
        let partial_json = serde_json::json!([
            {
                "date": "2026-03-13T10:15:00",
                "price": 183815.0,
                "quantity": 1.0,
                "type": "exit",
                "notes": "Parcial"
            }
        ]);
        trade.partial_exits = crate::models::SurrealJson(partial_json);

        // Calculation:
        // 1. Partial Exit (1 qty): (183815 - 184455) * 1 * 0.2 = -640 * 0.2 = -128.0
        // 2. Final Exit (2 qty): (186240 - 184455) * 2 * 0.2 = 1785 * 2 * 0.2 = 357 * 2 = 714.0
        // Total = 714.0 - 128.0 = 586.0
        
        // Old Logic (without partials): (3 * (186240 - 184455) * 0.2) = 1071.0
        // Discrepancy = 1071 - 586 = 485.0 (Bingo!)

        let result = PositionService::calculate_trade_result(&trade, 0.0, 0.2);
        assert!((result - 586.0).abs() < 0.001, "Expected 586.0, got {}", result);
    }
}
