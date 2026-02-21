import { rtdStore, type RTDQuote } from "$lib/stores/rtd.svelte";
import { settingsStore } from "$lib/stores/settings.svelte";

export interface PendingTrade {
    symbol: string;
    price: number;
    sheet?: string;
    timestamp: Date;
    account_id: string;
}

class AutoTradeService {
    pendingTrade = $state<PendingTrade | null>(null);
    isDialogOpen = $state(false);

    constructor() {
        if (typeof window !== 'undefined') {
            rtdStore.onTradeExecuted((quote) => this.handleDetection(quote));
        }
    }

    private handleDetection(quote: RTDQuote) {
        console.log("[AutoTradeService] NEW DETECTION RECEIVED:", quote.symbol, "Price:", quote.last, "Sheet:", quote.sheet);

        // Smart Account Detection
        let accountId = "";

        // 1. Try to match sheet name with account nickname or type
        if (quote.sheet) {
            const sheetMatch = settingsStore.accounts.find(a =>
                a.nickname.toLowerCase().includes(quote.sheet!.toLowerCase()) ||
                a.account_type.toLowerCase().includes(quote.sheet!.toLowerCase())
            );
            if (sheetMatch) accountId = sheetMatch.id;
        }

        // 2. Fallback to active account from a global context (or just first one)
        if (!accountId && settingsStore.accounts.length > 0) {
            accountId = settingsStore.accounts[0].id;
        }

        this.pendingTrade = {
            symbol: quote.symbol,
            price: quote.last,
            sheet: quote.sheet,
            timestamp: new Date(),
            account_id: accountId
        };

        this.isDialogOpen = true;
    }

    clear() {
        this.pendingTrade = null;
        this.isDialogOpen = false;
    }
}

export const autoTradeService = new AutoTradeService();
