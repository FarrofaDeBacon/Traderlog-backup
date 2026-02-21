import { fetch } from "@tauri-apps/plugin-http";
import { llmService, type EconomicEvent } from "./llmService";

export const marketMock = {
    async checkEconomicCalendar(): Promise<EconomicEvent[]> {
        // BYPASS DE SEGURANÇA: Retorna mock imediatamente para não travar
        return this.getMockData("Modo Segurança: Reinicie o App");

        const url = "https://sslecal2.forexprostools.com/?columns=exc_flags,exc_currency,exc_importance,exc_actual,exc_forecast,exc_previous&features=datepicker,timezone&countries=110,17,37,12,25,32,6,36,5,72&calType=day&timeZone=12&lang=12";

        console.log("MarketMock: Buscando calendário via Parser Local (Sem IA)...");

        try {
            // Timeout de 10 segundos para não travar o app
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 10000);

            const response = await fetch(url, {
                method: "GET",
                headers: {
                    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                },
                signal: controller.signal
            });

            clearTimeout(timeoutId);

            if (!response.ok) throw new Error("Falha ao baixar calendário");

            const html = await response.text();
            const events: EconomicEvent[] = [];

            // REGEX POWER: Extração manual, rápida e sem custo
            // Padrão para pegar linhas da tabela
            const rowRegex = /<tr[^>]*id="eventRowId_(\d+)"[^>]*>([\s\S]*?)<\/tr>/gi;
            let match;

            while ((match = rowRegex.exec(html)) !== null) {
                const rowContent = match[2];

                // 1. Extrair Hora (HH:MM)
                const timeMatch = rowContent.match(/(\d{2}:\d{2})/);
                // Se não tem hora, pode ser 'Tentative' ou já passou (mas pegamos o que der)
                const time = timeMatch ? timeMatch[1] : "--:--";

                // 2. Extrair Moeda (USD, BRL, EUR)
                const currencyMatch = rowContent.match(/(USD|BRL|EUR|GBP|JPY|CNY)/);
                const currency = currencyMatch ? currencyMatch[1] : "???";

                // 3. Extrair Importância (High/Medium/Low)
                // O widget usa title="High Volatility Expected" ou ícones
                let importance: 'low' | 'medium' | 'high' = 'low';
                if (rowContent.includes('High Volatility Expected') || (rowContent.match(/grayFullBullishIcon/g) || []).length === 3) {
                    importance = 'high';
                } else if (rowContent.includes('Moderate Volatility') || (rowContent.match(/grayFullBullishIcon/g) || []).length === 2) {
                    importance = 'medium';
                }

                // 4. Extrair Evento (Nome)
                // Tenta achar o nome limpando as tags
                const eventNameMatch = rowContent.match(/<td[^>]*class="[^"]*left event[^"]*"[^>]*>([\s\S]*?)<\/td>/i);
                let eventTitle = "Evento Econômico";

                if (eventNameMatch) {
                    // Limpa tags HTML do conteúdo da célula e decodifica entities básicas
                    eventTitle = eventNameMatch[1]
                        .replace(/<[^>]+>/g, '')
                        .replace(/&nbsp;/g, ' ')
                        .replace(/&amp;/g, '&')
                        .trim();
                }

                // Adiciona apenas se achou moeda válida
                if (currency !== "???") {
                    events.push({
                        time,
                        currency,
                        event: eventTitle,
                        importance
                    });
                }
            }

            console.log(`Parser Local: ${events.length} eventos encontrados.`);

            if (events.length === 0) {
                return this.getMockData("Nenhum evento detectado. Verifique Widget.");
            }

            // Ordenar por horário
            return events.sort((a, b) => a.time.localeCompare(b.time));

        } catch (error) {
            console.error("Erro no Parser Local:", error);
            return this.getMockData("Falha ao ler dados do Widget.");
        }
    },

    getMockData(errorMessage?: string): EconomicEvent[] {
        if (errorMessage) {
            return [
                { time: "--:--", currency: "ERR", event: errorMessage, importance: "high" }
            ];
        }

        const day = new Date().getDay();
        if (day === 0 || day === 6) return [];

        return [
            { time: "09:30", currency: "USD", event: "Aguardando sincronização real...", importance: "medium" }
        ];
    }
};
