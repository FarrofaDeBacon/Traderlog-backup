import { fetch } from "@tauri-apps/plugin-http";

export interface BcbDataPoint {
    data: string;
    valor: string;
}

export interface BcbContext {
    selic: number;
    cdi: number;
    ptax: number;
    lastUpdate: Date | null;
}

class BcbService {
    private cache: BcbContext = {
        selic: 0,
        cdi: 0,
        ptax: 0,
        lastUpdate: null
    };

    private CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

    async fetchMarketData(): Promise<BcbContext> {
        const now = Date.now();
        if (this.cache.lastUpdate && (now - this.cache.lastUpdate.getTime()) < this.CACHE_DURATION) {
            return this.cache;
        }

        console.log("[BcbService] Fetching fresh data from BCB API...");

        try {
            const [selicRes, cdiRes, ptaxRes] = await Promise.all([
                this.fetchSerie(432), // SELIC
                this.fetchSerie(12),  // CDI
                this.fetchSerie(1)    // PTAX (Dólar)
            ]);

            this.cache = {
                selic: selicRes,
                cdi: cdiRes,
                ptax: ptaxRes,
                lastUpdate: new Date()
            };

            return this.cache;
        } catch (error) {
            console.error("[BcbService] Error fetching BCB data:", error);
            return this.cache; // Return stale cache or defaults on error
        }
    }

    private async fetchSerie(serieId: number): Promise<number> {
        const url = `https://api.bcb.gov.br/dados/serie/bcdata.sgs.${serieId}/dados/ultimos/1?formato=json`;
        try {
            const response = await fetch(url, {
                method: 'GET',
                connectTimeout: 5000
            });

            if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);

            const data = await response.json() as BcbDataPoint[];
            if (data && data.length > 0) {
                return parseFloat(data[0].valor);
            }
            return 0;
        } catch (e) {
            console.warn(`[BcbService] Failed to fetch serie ${serieId}:`, e);
            return 0;
        }
    }
}

export const bcbService = new BcbService();
