
import { invoke } from "@tauri-apps/api/core";
import { getLocalDatePart } from "$lib/utils";
import type { Trade, Account, Currency, UserProfile } from "$lib/types";
import { settingsStore } from "$lib/stores/settings.svelte";
import { calculateAverageTimeBetweenTrades } from "$lib/utils/gann";

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
            await settingsStore.loadCashTransactions();

            // Sync account balances because save_trade might have updated them
            invoke("get_accounts").then(res => {
                if (res) settingsStore.accounts = res as Account[];
            }).catch(e => console.error("Failed to sync accounts after trade addition", e));

            return { success: true, trade: newTrade };
        } catch (e: any) {
            console.error("[TradesStore] Error adding trade:", e);
            return { success: false, error: e.toString() };
        }
    }

    /**
     * Helper to normalize IDs for comparison (extracts clean ID from 'table:id' or '{String: "id"}' or {String: "id"})
     */
    normalizeId(id: any): string {
        if (!id) return "";

        // Handle if it's already an object (Tauri/Serde can return this)
        if (typeof id === 'object') {
            if (id.String) return this.normalizeId(id.String);
            if (id.id) return this.normalizeId(id.id);
        }

        let str = id.toString();
        // Handle Rust/SurrealDB stringified object-like representations if they leak
        if (str.includes("String:")) {
            str = str.split("String:").pop() || str;
        }
        // Remove brackets, quotes and whitespace
        // Normalized for Svelte/JS comparison: keep only the hex/uuid part
        str = str.replace(/[{}\"\'\s⟨⟩`]/g, "");

        // Take the last part of a prefixed ID
        return str.split(":").pop() || str;
    }

    async updateTrade(id: string, trade: Partial<Trade>) {
        try {
            const normalizedInputId = this.normalizeId(id);
            console.log("[TradesStore] updateTrade: Normalized Input ID:", normalizedInputId, "Original:", id);

            const existingIdx = this.trades.findIndex(t => {
                const tid = this.normalizeId(t.id);
                // Extra log for debugging
                const isMatch = tid === normalizedInputId;
                if (isMatch) {
                    console.log(`[TradesStore] MATCH FOUND: '${tid}' === '${normalizedInputId}'`);
                }
                return isMatch;
            });
            console.log("[TradesStore] Existing index found:", existingIdx);

            if (existingIdx === -1) {
                console.error("[TradesStore] Could not find trade to update. Input ID:", id, "Normalized:", normalizedInputId);
                console.log("[TradesStore] Available trade IDs:", this.trades.map(t => this.normalizeId(t.id)));
                throw new Error(`Trade with ID ${id} not found in store.`);
            }

            const existing = this.trades[existingIdx];
            // CRITICAL: Ensure the ID is preserved and correctly formatted for backend
            // We use the full original ID (potentially with prefix) from the store
            const updatedTrade = { ...existing, ...trade, id: existing.id };

            console.log("[TradesStore] Calling save_trade with ID:", updatedTrade.id);
            const result = await invoke("save_trade", { trade: $state.snapshot(updatedTrade) });
            console.log("[TradesStore] save_trade result:", result);

            this.trades = this.trades.map((t, i) => i === existingIdx ? updatedTrade : t);

            // Trigger financial refresh
            await settingsStore.loadCashTransactions();

            // Sync account balances
            invoke("get_accounts").then(res => {
                if (res) settingsStore.accounts = res as Account[];
            }).catch(e => console.error("Failed to sync accounts after trade update", e));

            return { success: true };
        } catch (e: any) {
            console.error("[TradesStore] Error updating trade:", e);
            return { success: false, error: e.toString() };
        }
    }

    async removeTrade(id: string) {
        try {
            const normalizedInputId = this.normalizeId(id);
            console.log("[TradesStore] removeTrade: Normalized Input ID:", normalizedInputId);

            await invoke("delete_trade", { id });

            // Optimistic local update
            this.trades = this.trades.filter(t => {
                const tid = this.normalizeId(t.id);
                return tid !== normalizedInputId;
            });

            await this.loadTrades();
            await settingsStore.loadCashTransactions();

            // Sync account balances
            invoke("get_accounts").then(res => {
                if (res) settingsStore.accounts = res as Account[];
            }).catch(e => console.error("Failed to sync accounts after trade deletion", e));

            return { success: true };
        } catch (e: any) {
            console.error("[TradesStore] Error deleting trade:", e);
            return { success: false, error: e.toString() };
        }
    }

    async removeTrades(ids: string[]) {
        try {
            console.log("[TradesStore] removeTrades: count=", ids.length);
            await invoke("delete_trades_by_ids", { ids });

            const normalizedIds = ids.map(id => this.normalizeId(id));

            // Optimistic local update
            this.trades = this.trades.filter(t => {
                const tid = this.normalizeId(t.id);
                return !normalizedIds.includes(tid);
            });

            await this.loadTrades();
            await settingsStore.loadCashTransactions();

            // Sync account balances
            invoke("get_accounts").then(res => {
                if (res) settingsStore.accounts = res as Account[];
            }).catch(e => console.error("Failed to sync accounts after batch trade deletion", e));

            return { success: true };
        } catch (e: any) {
            console.error("[TradesStore] Error deleting trades:", e);
            return { success: false, error: e.toString() };
        }
    }

    getDailyResultByAccount(date: string, accounts: Account[]) {
        const normalizedTargetDate = getLocalDatePart(date);
        console.log(`[TradesStore] getDailyResultByAccount for date: ${date} (normalized: ${normalizedTargetDate})`);

        return accounts.map(acc => {
            const accCleanId = this.normalizeId(acc.id);
            const accountTrades = this.trades.filter(t => {
                const isClosed = t.exit_price !== null && t.exit_price !== undefined;
                if (!isClosed) return false;

                const dateToUse = getLocalDatePart(t.exit_date || t.date);

                const tAccId = this.normalizeId(t.account_id);
                const match = tAccId === accCleanId && dateToUse === normalizedTargetDate;

                return match;
            });
            const totalResult = accountTrades.reduce((sum, t) => sum + (Number(t.result) || 0), 0);
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

            // Use startsWith but also support date objects if they ever leak here
            const dateStr = t.exit_date || t.date || "";
            return dateStr.substring(0, 7) === yearMonth;
        });

        for (const trade of monthlyTrades) {
            total += this.getConvertedTradeResult(trade, accounts, currencies);
        }
        return total;
    }

    getTotalBalanceBRL(accounts: Account[], currencies: Currency[]) {
        return accounts.reduce((acc, curr) => {
            // Find currency by code or by ID
            const currencyObj = currencies.find(c => 
                c.code === curr.currency || 
                c.id === curr.currency_id ||
                c.id === (curr.currency_id?.includes(":") ? curr.currency_id : `currency:${curr.currency_id}`)
            );
            const rate = currencyObj?.exchange_rate || 1;
            return acc + (Number(curr.balance) || 0) * rate;
        }, 0);
    }

    getConvertedTradeResult(trade: Trade, accounts: Account[], currencies: Currency[]): number {
        const account = accounts.find(a => a.id === trade.account_id);

        let rawResult = trade.result;
        if (typeof rawResult === 'string') {
            rawResult = parseFloat(String(rawResult).replace(/[^\d,-]/g, "").replace(",", "."));
        }
        const numericResult = Number(rawResult) || 0;

        if (!account) return numericResult;

        const currency = currencies.find(c => 
            c.code === account.currency || 
            c.id === account.currency_id ||
            c.id === (account.currency_id?.includes(":") ? account.currency_id : `currency:${account.currency_id}`)
        );
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
                avg_interval: 0,
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
            avg_interval: calculateAverageTimeBetweenTrades(strategyTrades),
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
