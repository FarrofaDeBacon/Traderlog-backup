use crate::models::Trade;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PartialExit {
    pub date: Option<String>,
    pub price: f64,
    pub quantity: f64,
    pub r#type: String, // "entry" | "exit"
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub average_price: f64,
}

pub struct PositionService;

impl PositionService {
    /// Calcula o Preço Médio (PM) e o resultado líquido de uma série de trades.
    /// segue as regras da Receita Federal:
    /// - Compras aumentam o PM (incluindo taxas).
    /// - Vendas realizam lucro/prejuízo com base no PM atual e não alteram o PM.
    pub fn calculate_positions(trades: &[Trade]) -> HashMap<String, Position> {
        let mut positions: HashMap<String, Position> = HashMap::new();
        
        // Ordenar trades por data para garantir o cálculo cronológico
        let mut sorted_trades = trades.to_vec();
        sorted_trades.sort_by(|a, b| a.date.cmp(&b.date));

        for trade in sorted_trades {
            let entry = positions.entry(trade.asset_symbol.clone()).or_insert(Position {
                symbol: trade.asset_symbol.clone(),
                quantity: 0.0,
                average_price: 0.0,
            });

            if trade.direction == "Buy" {
                // Nova Qtd = Qtd Atual + Qtd Comprada
                let new_qty = entry.quantity + trade.quantity;
                if new_qty > 0.0 {
                    // Novo PM = (Custo Total Anterior + Custo da Nova Compra + Taxas) / Nova Qtd
                    let current_cost = entry.quantity * entry.average_price;
                    let new_cost = (trade.quantity * trade.entry_price) + trade.fee_total;
                    entry.average_price = (current_cost + new_cost) / new_qty;
                }
                entry.quantity = new_qty;
            } else {
                // Venda: Apenas reduz a quantidade no estoque
                // O PM não muda na venda pela regra da Receita Federal
                entry.quantity -= trade.quantity;
                
                // Se a quantidade zerar, o PM vira zero (posição encerrada)
                if entry.quantity <= 0.0 {
                    entry.quantity = 0.0;
                    entry.average_price = 0.0;
                }
            }
        }

        positions
    }

    /// Calculates the profit or loss for a single trade.
    /// Incorporates Moving Average logic for partial exits/additions if they exist.
    pub fn calculate_trade_result(trade: &Trade, current_pm: f64, point_value: f64) -> f64 {
        // Parse partial exits if any
        let partials: Vec<PartialExit> = serde_json::from_value(trade.partial_exits.0.clone())
            .unwrap_or_default();

        let multiplier = if trade.direction == "Buy" { 1.0 } else { -1.0 };
        
        // Complex case: Moving Average calculation
        // If partials exist, we follow the chronological moving average realization.
        if !partials.is_empty() {
            let mut current_avg_price = if trade.modality_id.as_deref() == Some("mod2") {
                current_pm
            } else {
                trade.entry_price
            };
            
            let mut current_qty = trade.quantity;
            let mut total_entry_qty = trade.quantity;
            let mut total_exit_qty = 0.0;
            let mut gross_currency_total = 0.0;

            let mut sorted_partials = partials;
            sorted_partials.sort_by(|a, b| a.date.cmp(&b.date));

            for p in sorted_partials {
                if p.r#type == "entry" {
                    let new_qty = current_qty + p.quantity;
                    if new_qty > 0.0 {
                        current_avg_price = ((current_avg_price * current_qty) + (p.price * p.quantity)) / new_qty;
                    }
                    current_qty = new_qty;
                    total_entry_qty += p.quantity;
                } else {
                    let diff = (p.price - current_avg_price) * multiplier;
                    gross_currency_total += diff * p.quantity * point_value;
                    current_qty -= p.quantity;
                    total_exit_qty += p.quantity;
                }
            }

            // Final exit portion
            let remaining_qty = total_entry_qty - total_exit_qty;
            if let Some(exit_price) = trade.exit_price {
                if remaining_qty > 0.0 {
                    let diff = (exit_price - current_avg_price) * multiplier;
                    gross_currency_total += diff * remaining_qty * point_value;
                }
            }

            return gross_currency_total - trade.fee_total;
        }

        // Base case: No partial exits
        if let Some(exit_price) = trade.exit_price {
            // Complete trade (usually DayTrade)
            let basis = if trade.modality_id.as_deref() == Some("mod2") {
                current_pm
            } else {
                trade.entry_price
            };

            let gross_pts = (exit_price - basis) * multiplier;
            (trade.quantity * gross_pts * point_value) - trade.fee_total
        } else if trade.direction == "Sell" {
            // Standalone Sell operation (SwingTrade exit)
            // Result = (SellingPrice - PM) * Quantity
            (trade.quantity * (trade.entry_price - current_pm) * point_value) - trade.fee_total
        } else {
            // Standalone Buy operation in SwingTrade (doesn't realize result)
            0.0
        }
    }
}
