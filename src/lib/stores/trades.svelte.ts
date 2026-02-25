
import { invoke } from "@tauri-apps/api/core";
import { getLocalDatePart } from "$lib/utils";
import type { Trade, Account, Currency, UserProfile } from "$lib/types";

class TradesStore {
    trades = $state<Trade[]>([]);
    isLoading = $state<boolean>(false);

    async loadTrades() {
        if (this.isLoading) return;
        this.isLoading = true;
        try {
            console.log("[TradesStore] Calling get_trades...");
            const trades = await invoke("get_trades") as Trade[];
            console.log("[TradesStore] Received trades from backend:", trades ? trades.length : 0);

            // Critical: always update trades even if empty, to clear the list correctly
            this.trades = trades || [];
        } catch (e) {
            console.error("[TradesStore] Error loading trades:", e);
        } finally {
            this.isLoading = false;
        }
    }

    async addTrade(trade: Omit<Trade, 'id'>) {
        try {
            const newTrade = { ...trade, id: crypto.randomUUID() } as Trade;
            console.log("[TradesStore] Adding new trade:", newTrade.id);

            await invoke("save_trade", { trade: $state.snapshot(newTrade) });

            // Re-fetch instead of just pushing to ensure we have the DB's true state
            await this.loadTrades();

            return { success: true, trade: newTrade };
        } catch (e: any) {
            console.error("[TradesStore] Error adding trade:", e);
            return { success: false, error: e.toString() };
        }
    }

    async updateTrade(id: string, trade: Partial<Trade>) {
        try {
            const existing = this.trades.find(t => t.id === id);
            if (!existing) throw new Error("Trade not found");

            const updatedTrade = { ...existing, ...trade };
            await invoke("save_trade", { trade: $state.snapshot(updatedTrade) });

            // CRITICAL: Re-fetch to ensure we have the DB's true state (prevents duplication/desync)
            await this.loadTrades();
            return { success: true };
        } catch (e: any) {
            console.error("[TradesStore] Error updating trade:", e);
            return { success: false, error: e.toString() };
        }
    }

    async removeTrade(id: string) {
        try {
            await invoke("delete_trade", { id });
            this.trades = this.trades.filter(t => t.id !== id);
            return { success: true };
        } catch (e: any) {
            console.error("[TradesStore] Error deleting trade:", e);
            return { success: false, error: e.toString() };
        }
    }

    getDailyResultByAccount(date: string, accounts: Account[]) {
        const normalizedTargetDate = getLocalDatePart(date);
        console.log(`[TradesStore] getDailyResultByAccount for date: ${date} (normalized: ${normalizedTargetDate})`);

        // Helper to normalize IDs for comparison (extracts clean ID from 'table:id' or '{String: "id"}' or {String: "id"})
        const normalizeId = (id: any) => {
            if (!id) return "";

            // Handle if it's already an object (Tauri/Serde can return this)
            if (typeof id === 'object') {
                if (id.String) return normalizeId(id.String);
                if (id.id) return normalizeId(id.id);
            }

            let str = id.toString();
            // Handle Rust/SurrealDB stringified object-like representations if they leak
            if (str.includes("String:")) {
                str = str.split("String:").pop() || str;
            }
            // Remove brackets, quotes and whitespace
            str = str.replace(/[{}\"\'\s]/g, "");
            // Take the last part of a prefixed ID
            return str.split(":").pop();
        };

        return accounts.map(acc => {
            const accCleanId = normalizeId(acc.id);
            const accountTrades = this.trades.filter(t => {
                const isClosed = t.exit_price !== null && t.exit_price !== undefined;
                if (!isClosed) return false;

                const dateToUse = getLocalDatePart(t.exit_date || t.date);

                const tAccId = normalizeId(t.account_id);
                console.log(`[TradesStore] Filtering Trade ${t.id}: AccID(${t.account_id}) -> ${tAccId} vs Account(${acc.nickname}) -> ${accCleanId}. Date: ${dateToUse} vs ${normalizedTargetDate}`);

                const match = tAccId === accCleanId && dateToUse === normalizedTargetDate;

                if (match) {
                    console.log(`[TradesStore] MATCH FOUND! Trade ${t.id} for account ${acc.nickname}`);
                }

                return match;
            });
            const totalResult = accountTrades.reduce((sum, t) => sum + (Number(t.result) || 0), 0);
            console.log(`[TradesStore] Account ${acc.nickname} result: ${totalResult} (${accountTrades.length} trades)`);
            return {
                account_id: acc.id,
                account_name: acc.nickname,
                currency: acc.currency,
                result: totalResult,
                trades_count: accountTrades.length
            };
        });
    }

    getTradesCountForDate(date: string) {
        const normalizedTargetDate = getLocalDatePart(date);
        return this.trades.filter(t => {
            const isClosed = t.exit_price !== null && t.exit_price !== undefined;
            if (!isClosed) return false;

            const dateToUse = getLocalDatePart(t.exit_date || t.date);
            return dateToUse === normalizedTargetDate;
        }).length;
    }

    getMonthlyTradeResult(yearMonth: string, accounts: Account[], currencies: Currency[]) {
        let total = 0;
        const monthlyTrades = this.trades.filter(t => {
            const isClosed = t.exit_price !== null && t.exit_price !== undefined;
            if (!isClosed) return false;

            const dateToUse = t.exit_date || t.date;
            return dateToUse && dateToUse.startsWith(yearMonth);
        });

        for (const trade of monthlyTrades) {
            total += this.getConvertedTradeResult(trade, accounts, currencies);
        }
        return total;
    }

    getConvertedTradeResult(trade: Trade, accounts: Account[], currencies: Currency[]): number {
        const account = accounts.find(a => a.id === trade.account_id);

        // Robust parsing for result, handling potential string formats like "R$ 1.000,00"
        let rawResult = trade.result;
        if (typeof rawResult === 'string') {
            // Remove non-numeric chars except minus and dot/comma
            // Assuming standard format, we might need to handle localization more carefully if mixed
            // But for now, stripping currency symbols is key.
            // If comma is used as decimal separator (pt-BR), replace with dot
            rawResult = parseFloat(String(rawResult).replace(/[^\d,-]/g, "").replace(",", "."));
        }
        const numericResult = Number(rawResult) || 0;

        if (!account) return numericResult;

        const currency = currencies.find(c => c.code === account.currency);
        const rate = currency?.exchange_rate || 1.0;

        return numericResult * rate;
    }

    getStrategyStats(strategyId: string, accounts: Account[], currencies: Currency[], userProfile: UserProfile, currencyMode: 'original' | 'main' = 'main') {
        const strategyTrades = this.trades.filter(t => t.strategy_id === strategyId);
        const total_trades = strategyTrades.length;

        if (total_trades === 0) {
            return {
                total_trades: 0,
                win_rate: 0,
                profit_factor: 0,
                total_profit: 0,
                average_profit: 0,
                payoff: 0,
                currency: userProfile.main_currency
            };
        }

        let total_profit = 0;
        let winnersCount = 0;
        let losersCount = 0;
        let totalWin = 0;
        let totalLoss = 0;

        let usedCurrency = userProfile.main_currency;
        if (currencyMode === 'original') {
            const firstAccId = strategyTrades[0].account_id;
            const firstAcc = accounts.find(a => a.id === firstAccId);
            const firstCurrency = firstAcc?.currency || "BRL";

            const allSame = strategyTrades.every(t => {
                const acc = accounts.find(a => a.id === t.account_id);
                return (acc?.currency || "BRL") === firstCurrency;
            });

            if (allSame) {
                usedCurrency = firstCurrency;
            } else {
                usedCurrency = userProfile.main_currency;
                currencyMode = 'main';
            }
        }

        for (const trade of strategyTrades) {
            const account = accounts.find(a => a.id === trade.account_id);
            const accCurrency = account?.currency || "BRL";

            let result = trade.result;
            if (currencyMode === 'main') {
                const currency = currencies.find(c => c.code === accCurrency);
                const rate = currency?.exchange_rate || 1.0;
                result = trade.result * rate;
            }

            total_profit += result;
            if (result > 0) {
                winnersCount++;
                totalWin += result;
            } else if (result < 0) {
                losersCount++;
                totalLoss += Math.abs(result);
            }
        }

        const win_rate = (winnersCount / total_trades) * 100;
        const profit_factor = totalLoss === 0 ? (totalWin > 0 ? 99.99 : 0) : totalWin / totalLoss;
        const average_profit = total_profit / total_trades;

        const avg_win = winnersCount > 0 ? totalWin / winnersCount : 0;
        const avg_loss = losersCount > 0 ? totalLoss / losersCount : 0;
        const payoff = avg_loss === 0 ? (avg_win > 0 ? 99.99 : 0) : avg_win / avg_loss;

        return {
            total_trades,
            win_rate,
            profit_factor,
            total_profit,
            average_profit,
            payoff,
            currency: usedCurrency
        };
    }

    constructor() {
        if (typeof window !== "undefined") {
            this.loadTrades();
        }
    }
}

export const tradesStore = new TradesStore();
