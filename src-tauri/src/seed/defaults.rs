// src-tauri/src/seed/defaults.rs

pub struct DefaultItem {
    pub id: &'static str,
    pub label: &'static str,
}

pub struct DefaultModule {
    pub id: &'static str,
    pub label: &'static str,
    pub items: Vec<DefaultItem>,
}

pub fn get_onboarding_defaults() -> Vec<DefaultModule> {
    vec![
        DefaultModule {
            id: "markets",
            label: "Mercados",
            items: vec![
                DefaultItem { id: "m1", label: "B3 (Brasil)" },
                DefaultItem { id: "m2", label: "FOREX" },
                DefaultItem { id: "m3", label: "CRYPTO" },
                DefaultItem { id: "m4", label: "NYSE" },
                DefaultItem { id: "m5", label: "NASDAQ" },
                DefaultItem { id: "m6", label: "HKEX" },
            ],
        },
        DefaultModule {
            id: "strategies",
            label: "Estratégias",
            items: vec![
                DefaultItem { id: "s1", label: "Barra Elefante" },
                DefaultItem { id: "s2", label: "Bear 180" },
                DefaultItem { id: "s3", label: "Breakout" },
                DefaultItem { id: "s4", label: "Bull 180" },
                DefaultItem { id: "s5", label: "Clássico Perto da Média" },
                DefaultItem { id: "s6", label: "Gift" },
                DefaultItem { id: "s7", label: "Power Breakout" },
                DefaultItem { id: "s8", label: "Pullback na Média" },
                DefaultItem { id: "s9", label: "Range" },
                DefaultItem { id: "s10", label: "Setup da MM200" },
                DefaultItem { id: "s11", label: "Volta em V" },
            ],
        },
        DefaultModule {
            id: "indicators",
            label: "Indicadores",
            items: vec![
                DefaultItem { id: "i1", label: "IFR (RSI)" },
                DefaultItem { id: "i2", label: "MACD" },
                DefaultItem { id: "i3", label: "MME 21" },
                DefaultItem { id: "i4", label: "MME 8" },
                DefaultItem { id: "i5", label: "MMS 9" },
                DefaultItem { id: "i6", label: "MMS 21" },
                DefaultItem { id: "i7", label: "VWAP" },
                DefaultItem { id: "i8", label: "Bollinger Bands" },
                DefaultItem { id: "i9", label: "Volume Profile" },
                DefaultItem { id: "i10", label: "Volume" },
            ],
        },
        DefaultModule {
            id: "modalities",
            label: "Modalidades",
            items: vec![
                DefaultItem { id: "mod1", label: "Day Trade" },
                DefaultItem { id: "mod2", label: "Swing Trade" },
                DefaultItem { id: "mod3", label: "Position Trade" },
                DefaultItem { id: "mod4", label: "Scalping" },
                DefaultItem { id: "mod5", label: "Arbitrage" },
            ],
        },
        DefaultModule {
            id: "timeframes",
            label: "Timeframes",
            items: vec![
                DefaultItem { id: "tf1", label: "1 Minuto" },
                DefaultItem { id: "tf2", label: "2 Minutos" },
                DefaultItem { id: "tf3", label: "5 Minutos" },
                DefaultItem { id: "tf4", label: "10 Minutos" },
                DefaultItem { id: "tf5", label: "15 Minutos" },
                DefaultItem { id: "tf6", label: "30 Minutos" },
                DefaultItem { id: "tf7", label: "60 Minutos" },
                DefaultItem { id: "tf8", label: "4 Horas" },
                DefaultItem { id: "tf9", label: "Diário" },
                DefaultItem { id: "tf10", label: "Semanal" },
                DefaultItem { id: "tf11", label: "Mensal" },
            ],
        },
        DefaultModule {
            id: "tags",
            label: "Tags / Etiquetas",
            items: vec![
                DefaultItem { id: "t1", label: "Setup Indefinido" },
                DefaultItem { id: "t2", label: "Fora do Plano" },
                DefaultItem { id: "t3", label: "Hesitação" },
                DefaultItem { id: "t4", label: "Impulsividade" },
                DefaultItem { id: "t5", label: "FOMO" },
                DefaultItem { id: "t6", label: "Revenge Trading" },
                DefaultItem { id: "t7", label: "Bom Gerenciamento" },
                DefaultItem { id: "t8", label: "Seguiu Regras" },
                DefaultItem { id: "t9", label: "Sortudo" },
                DefaultItem { id: "t10", label: "Errou a mão" },
                DefaultItem { id: "t11", label: "Mercado Lento" },
                DefaultItem { id: "t12", label: "Alta Volatilidade" },
                DefaultItem { id: "t13", label: "Notícia Impactante" },
            ],
        },
        DefaultModule {
            id: "emotional_states",
            label: "Estados Emocionais",
            items: vec![
                DefaultItem { id: "e1", label: "Focado" },
                DefaultItem { id: "e2", label: "Confiante" },
                DefaultItem { id: "e3", label: "Disciplinado" },
                DefaultItem { id: "e4", label: "Paciente" },
                DefaultItem { id: "e5", label: "Ansioso" },
                DefaultItem { id: "e6", label: "Ganancioso" },
                DefaultItem { id: "e7", label: "Medo" },
                DefaultItem { id: "e8", label: "Vingativo" },
                DefaultItem { id: "e9", label: "Frustrado" },
                DefaultItem { id: "e10", label: "FOMO" },
                DefaultItem { id: "e11", label: "Neutro" },
                DefaultItem { id: "e12", label: "Cansado" },
            ],
        },
        DefaultModule {
            id: "chart_types",
            label: "Tipos de Gráfico",
            items: vec![
                DefaultItem { id: "ct1", label: "Candlestick 1 min" },
                DefaultItem { id: "ct2", label: "Candlestick 5 min" },
                DefaultItem { id: "ct3", label: "Candlestick 15 min" },
                DefaultItem { id: "ct4", label: "Candlestick 60 min" },
                DefaultItem { id: "ct5", label: "Candlestick Diário" },
                DefaultItem { id: "ct6", label: "Renko 5R" },
                DefaultItem { id: "ct7", label: "Renko 10R" },
                DefaultItem { id: "ct8", label: "Renko 21R" },
                DefaultItem { id: "ct9", label: "Heiken Ashi" },
                DefaultItem { id: "ct10", label: "Range 100" },
            ],
        },
        DefaultModule {
            id: "fees",
            label: "Perfis de Taxas",
            items: vec![
                DefaultItem { id: "f1", label: "Padrão B3 (Day Trade)" },
                DefaultItem { id: "f2", label: "Corretagem Zero" },
                DefaultItem { id: "f3", label: "Crypto Exchange (Binance)" },
            ],
        },
        DefaultModule {
            id: "risk",
            label: "Perfis de Risco",
            items: vec![
                DefaultItem { id: "r1", label: "Conservador" },
                DefaultItem { id: "r2", label: "Moderado" },
                DefaultItem { id: "r3", label: "Agressivo" },
            ],
        },
    ]
}
