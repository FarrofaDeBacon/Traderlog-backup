import { rtdStore } from "$lib/stores/rtd.svelte";
import { bcbService } from "./bcbService";

export interface MarketSentiment {
    winIntention: number; // 0 to 100
    wdoIntention: number; // 0 to 100
    lastUpdate: Date | null;
}

class QuantEngine {
    // BCB Data State
    selic = $state(0);
    cdi = $state(0);
    ptax = $state(0);
    bcbLastUpdate = $state<Date | null>(null);

    // Context & Derived Sentiment
    sentiment = $derived.by((): MarketSentiment => {
        const win = rtdStore.winBook;
        const wdo = rtdStore.wdoBook;

        const calcIntention = (bid: number, ask: number) => {
            const total = bid + ask;
            if (total === 0) return 50;
            return (bid / total) * 100;
        };

        return {
            winIntention: calcIntention(win?.bid || 0, win?.ask || 0),
            wdoIntention: calcIntention(wdo?.bid || 0, wdo?.ask || 0),
            lastUpdate: rtdStore.lastUpdate
        };
    });

    constructor() {
        this.initializeMacroPolling();
    }

    private async initializeMacroPolling() {
        // First fetch
        await this.refreshMacroData();

        // Polling every 15 minutes for macro data (conservative)
        if (typeof window !== 'undefined') {
            setInterval(() => {
                this.refreshMacroData();
            }, 15 * 60 * 1000);
        }
    }

    async refreshMacroData() {
        console.log("[QuantEngine] Syncing macro context from BCB...");
        const data = await bcbService.fetchMarketData();
        this.selic = data.selic;
        this.cdi = data.cdi;
        this.ptax = data.ptax;
        this.bcbLastUpdate = data.lastUpdate;
    }
}

export const quantEngine = new QuantEngine();
