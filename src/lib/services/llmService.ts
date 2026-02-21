import { fetch } from "@tauri-apps/plugin-http";
import { settingsStore } from "$lib/stores/settings.svelte";

export interface EconomicEvent {
    time: string;
    currency: string;
    event: string;
    importance: 'high' | 'medium' | 'low';
}

export const llmService = {
    async extractEconomicEvents(html: string): Promise<EconomicEvent[]> {
        const configId = settingsStore.psychologyApiId;
        const config = settingsStore.apiConfigs.find(c => c.id === configId && c.enabled);

        if (!config || !config.api_key) {
            // Fallback: try to find any enabled gemini config if the explicitly chosen one is missing/disabled/mock
            const fallbackConfig = settingsStore.apiConfigs.find(c => c.provider === 'google_gemini' && c.enabled);
            if (!fallbackConfig || !fallbackConfig.api_key) {
                throw new Error("Gemini API Key not configured or disabled");
            }
            return this.processWithConfig(html, fallbackConfig);
        }

        return this.processWithConfig(html, config);
    },

    async analyzeJournal(content: string, emotion: string, intensity: number): Promise<string> {
        const configId = settingsStore.psychologyApiId;
        const config = settingsStore.apiConfigs.find(c => c.id === configId && c.enabled);

        if (!config || !config.api_key) {
            const fallbackConfig = settingsStore.apiConfigs.find(c => (c.provider === 'openai' || c.provider === 'google_gemini') && c.enabled);
            if (!fallbackConfig) return "Nenhuma IA configurada para Psicologia. Vá em Configurações > API & Integrações.";
            return this.processJournalAnalysis(content, emotion, intensity, fallbackConfig);
        }

        return this.processJournalAnalysis(content, emotion, intensity, config);
    },

    async processJournalAnalysis(content: string, emotion: string, intensity: number, config: any): Promise<string> {
        const prompt = `
            Você é um psicólogo especializado em trading de alta performance (Mental Game Coach).
            Analise este registro de diário de um trader:
            
            Emoção: ${emotion} (Intensidade: ${intensity}/10)
            Relato: "${content}"
            
            Se o relato for curto, seja breve. Se for detalhado, forneça insights mais profundos.
            Sua meta é:
            1. Validar a emoção (sem julgamento).
            2. Identificar vieses cognitivos ou falhas de disciplina, se houver.
            3. Dar uma dica prática para a próxima sessão.
            
            Responda em Português do Brasil. Tom profissional porém próximo ("coach").
        `;

        if (config.provider === 'google_gemini') {
            const API_KEY = config.api_key.trim();
            // Try visible models
            const models = ['gemini-1.5-flash', 'gemini-2.0-flash-exp'];

            for (const model of models) {
                try {
                    const ENDPOINT = `https://generativelanguage.googleapis.com/v1beta/models/${model}:generateContent?key=${API_KEY}`;
                    const response = await fetch(ENDPOINT, {
                        method: "POST",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ contents: [{ parts: [{ text: prompt }] }] })
                    });

                    if (response.ok) {
                        const data = await response.json();
                        return data.candidates?.[0]?.content?.parts?.[0]?.text || "Sem resposta da IA.";
                    }
                } catch (e) { console.error(e); }
            }
            // Last attempt error
            return "Erro ao contatar Gemini. Verifique a chave.";
        }

        if (config.provider === 'openai') {
            const API_KEY = config.api_key.trim();
            try {
                const response = await fetch("https://api.openai.com/v1/chat/completions", {
                    method: "POST",
                    headers: { "Content-Type": "application/json", "Authorization": `Bearer ${API_KEY}` },
                    body: JSON.stringify({
                        model: "gpt-3.5-turbo",
                        messages: [{ role: "user", content: prompt }]
                    })
                });
                if (response.ok) {
                    const data = await response.json();
                    return data.choices?.[0]?.message?.content || "Sem resposta.";
                }
            } catch (e) { return "Erro API OpenAI"; }
        }

        return "Modo Simulação: Ótimo registro! Continue monitorando suas emoções. (Configure uma chave real para análise real)";
    },

    async processWithConfig(html: string, config: any): Promise<EconomicEvent[]> {

        const API_KEY = config.api_key.trim();
        // NOVA ABORDAGEM: Descoberta Dinâmica de Modelos
        // Em vez de adivinhar, vamos perguntar quais modelos estão disponíveis para esta chave.
        let attempts: { ver: string, model: string }[] = [];

        try {
            console.log("LLM Service: Buscando modelos disponíveis...");
            const listModelsUrl = `https://generativelanguage.googleapis.com/v1beta/models?key=${API_KEY}`;
            const listResp = await fetch(listModelsUrl);

            if (listResp.ok) {
                const listData = await listResp.json();
                const availableModels = listData.models || [];

                // Filtra apenas modelos 'generateContent' e 'gemini'
                const viableModels = availableModels.filter((m: any) =>
                    m.name.includes("gemini") && m.supportedGenerationMethods?.includes("generateContent")
                );

                // Ordena por preferência (Flash > Pro) para evitar Rate Limit (429) no plano gratuito
                viableModels.sort((a: any, b: any) => {
                    const aIsFlash = a.name.includes("flash");
                    const bIsFlash = b.name.includes("flash");
                    // Queremos Flash primeiro
                    if (aIsFlash && !bIsFlash) return -1;
                    if (!aIsFlash && bIsFlash) return 1;
                    return 0;
                });

                // Cria lista de tentativas com os nomes REAIS retornados pela API
                attempts = viableModels.map((m: any) => ({
                    ver: "v1beta",
                    model: m.name.replace("models/", "")
                })).slice(0, 3); // Pega top 3

                console.log("LLM Service: Modelos encontrados:", attempts);
            }
        } catch (e) {
            console.warn("Falha ao listar modelos, usando fallback fixo.", e);
        }

        // Fallback se a listagem falhar ou não achar nada
        if (attempts.length === 0) {
            attempts = [
                { ver: "v1beta", model: "gemini-1.5-flash" },
                { ver: "v1", model: "gemini-pro" }
            ];
        }

        let lastError = "";

        for (const attempt of attempts) {
            try {
                const ENDPOINT = `https://generativelanguage.googleapis.com/${attempt.ver}/models/${attempt.model}:generateContent?key=${API_KEY}`;
                console.log(`LLM Service: Tentando ${attempt.model}...`);

                const response = await fetch(ENDPOINT, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        contents: [{
                            parts: [{
                                text: `ATENÇÃO: Aja como um EXTRAÍDOR DE DADOS DE CALENDÁRIO ECONÔMICO.
                                Analise o HTML da tabela abaixo.
                                
                                PROBLEMA A RESOLVER: Os horários estão vindo errado (00:00).
                                SOLUÇÃO: Em cada linha (<tr>), procure por QUALQUER texto que pareça um horário (Formato HH:MM).
                                
                                REGRAS DE EXTRAÇÃO:
                                1. TIME (Hora): 
                                   - Procure um padrão de hora "XX:XX" (ex: "09:30", "14:00") na linha.
                                   - Se não achar, procure atributos como 'data-event-datetime'.
                                   - Se a linha tiver "Tentative", "All Day" ou "Dia Todo", use isso.
                                   - NÃO invente "00:00" se não tiver hora.
                                
                                2. CURRENCY (Moeda): USD, BRL, EUR, GBP, etc.
                                
                                3. IMPORTANCE (Importância):
                                   - High: 3 touros, 3 estrelas, ou texto vermellho/negrito indicando alta relevância.
                                   - Medium: 2 touros/estrelas.
                                   - Low: 1 touro/estrela.
                                   - IMPORTANTE: Eventos como "Payroll", "CPI", "PPI", "FOMC", "Taxa de Juros" são SEMPRE "high".
                                
                                4. EVENT (Evento): O nome do indicador.
                                
                                SAÍDA: JSON Array.
                                [{"time":"10:30","currency":"USD","event":"Payroll","importance":"high"}, ...]
                                
                                HTML ALVO: ${html.substring(0, 50000)}`
                            }]
                        }]
                    })
                });

                if (response.ok) {
                    const data = await response.json();
                    const text = data.candidates?.[0]?.content?.parts?.[0]?.text || "[]";
                    console.log(`LLM Resposta Bruta (${attempt.model}):`, text);
                    const jsonStr = text.replace(/```json/g, "").replace(/```/g, "").trim();
                    const result = JSON.parse(jsonStr) as EconomicEvent[];
                    return result;
                } else {
                    // Tratar erros específicos
                    if (response.status === 429) {
                        throw new Error("Cota de requisições excedida (429). Aguarde 1 minuto.");
                    }

                    const err = await response.json().catch(() => ({}));
                    lastError = err.error?.message || response.statusText;
                    console.warn(`LLM Service: Falha em ${attempt.model}: ${lastError}`);

                    // Se erro fatal de chave ou cota, não adianta tentar outro modelo
                    if (lastError.includes("key") || lastError.includes("quota") || response.status === 403) {
                        throw new Error(lastError);
                    }
                    continue;
                }
            } catch (e) {
                lastError = (e as Error).message;
                // Se for cota ou chave, aborta loop
                if (lastError.includes("Cota") || lastError.includes("key")) throw e;
                continue;
            }
        }

        throw new Error(`Falha na IA: ${lastError}`);
    }
};
