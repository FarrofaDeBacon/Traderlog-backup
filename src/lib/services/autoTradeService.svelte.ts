import { rtdStore, type RTDTradeEvent } from "$lib/stores/rtd.svelte";
import { settingsStore } from "$lib/stores/settings.svelte";
import { tradesStore } from "$lib/stores/trades.svelte";

export interface PendingTrade {
    symbol: string;
    price: number;
    sheet?: string;
    timestamp: Date;
    account_id: string;
    isPartial: boolean;
    type: 'new' | 'partial_entry' | 'partial_exit';
    existingTradeId?: string;
}

class AutoTradeService {
    pendingTrade = $state<PendingTrade | null>(null);
    isDialogOpen = $state(false);

    constructor() {
        if (typeof window !== 'undefined') {
            rtdStore.onTradeExecuted((event) => this.handleDetection(event));
        }
    }

    private handleDetection(event: RTDTradeEvent) {
        const { quote, type, isPartial } = event;
        
        // SANITY CHECK: Ignore detections with zero or negative price
        if (quote.last <= 0) {
            console.log("[AutoTradeService] Ignoring detection due to invalid price:", quote.symbol, quote.last);
            return;
        }

        console.log("[AutoTradeService] DETECTION RECEIVED:", quote.symbol, "Price:", quote.last, "Type:", type, "Sheet:", quote.sheet);

        // Smart Account Detection
        let accountId = "";
        if (quote.sheet) {
            const sheetMatch = settingsStore.accounts.find(a =>
                a.nickname.toLowerCase().includes(quote.sheet!.toLowerCase()) ||
                a.account_type.toLowerCase().includes(quote.sheet!.toLowerCase())
            );
            if (sheetMatch) accountId = sheetMatch.id;
        }

        if (!accountId && settingsStore.accounts.length > 0) {
            accountId = settingsStore.accounts[0].id;
        }

        // Search for existing trade to link if partial
        let existingId = undefined;
        if (isPartial) {
            const openTrade = tradesStore.trades.find(t => 
                (t.asset_symbol.toUpperCase() === quote.symbol.toUpperCase() || 
                 quote.symbol.toUpperCase().startsWith(t.asset_symbol.toUpperCase())) && 
                t.exit_price === null
            );
            existingId = openTrade?.id;
        }

        this.pendingTrade = {
            symbol: quote.symbol,
            price: quote.last,
            sheet: quote.sheet,
            timestamp: new Date(),
            account_id: accountId,
            isPartial: isPartial,
            type: type,
            existingTradeId: existingId
        };

        this.isDialogOpen = true;
    }

    clear() {
        this.pendingTrade = null;
        this.isDialogOpen = false;
    }
}

export const autoTradeService = new AutoTradeService();
