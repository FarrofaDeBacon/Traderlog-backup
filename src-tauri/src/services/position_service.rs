use crate::models::Trade;
use std::collections::HashMap;

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

    /// Calcula o lucro líquido de uma operação baseada no PM atual.
    /// Funciona tanto para Vendas (Exit) quanto para Compras (Exit - Short).
    pub fn calculate_trade_result(trade: &Trade, current_pm: f64) -> f64 {
        if let Some(exit_price) = trade.exit_price {
            // Se temos preço de saída, é uma operação completa (ex: DayTrade do Profit)
            // Resultado = Qtd * (Preço de Saída - Preço de Entrada) - Taxas
            // Nota: Se for Buy, entrada é entry_price. Se for Sell (Short), entrada é entry_price.
            // Para simplificar: exit - entry sempre dá o lucro se direction for respeitado.
            let gross = if trade.direction == "Buy" {
                exit_price - trade.entry_price
            } else {
                trade.entry_price - exit_price
            };
            (trade.quantity * gross) - trade.fee_total
        } else if trade.direction == "Sell" {
            // Venda parcial ou saída de posição SwingTrade usando o PM acumulado
            // Resultado = Qtd * (Preço de Venda - PM) - Taxas
            (trade.quantity * (trade.entry_price - current_pm)) - trade.fee_total
        } else {
            // Compras comuns não geram resultado imediato no SwingTrade
            0.0
        }
    }
}
