const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');
const ptBrPath = path.join(localesDir, 'pt-BR.json');
const enUsPath = path.join(localesDir, 'en-US.json');

const newDashboardTranslationsPt = {
    title: "Dashboard",
    subtitle: "Visão geral de desempenho.",
    terminalHub: "Terminal Hub",
    newTrade: "Novo Trade",
    kpis: {
        netResult: "Resultado Líquido",
        winRate: "Taxa de Acerto",
        profitFactor: "Fator de Lucro",
        discipline: "Disciplina",
        payoff: "Payoff",
        maxDrawdown: "Drawdown Máximo"
    },
    charts: {
        equity: "Evolução do Patrimônio",
        equitySubtitle: "Evolução gráfica",
        consistency: "CONSISTÊNCIA"
    },
    dailyDetail: {
        title: "Detalhe Diário",
        subtitle: "Visão detalhada do dia",
        summary: "Resumo",
        tradeQty: "Total de Operações",
        efficiency: "Eficiência",
        operationsList: "Lista de Operações",
        asset: "Ativo",
        direction: "Direção",
        result: "Resultado",
        buy: "Compra",
        sell: "Venda",
        noTrades: "Nenhum trade"
    },
    schedule: {
        title: "Horários de Maior Eficiência",
        morning: "Manhã",
        afternoon: "Tarde",
        basedOn: "Baseado nos últimos {count} dias úteis"
    }
};

const newDashboardTranslationsEn = {
    title: "Dashboard",
    subtitle: "Welcome to your command center.",
    terminalHub: "Terminal Hub",
    newTrade: "New Trade",
    kpis: {
        netResult: "Net Result",
        winRate: "Win Rate",
        profitFactor: "Profit Factor",
        discipline: "Discipline",
        payoff: "Payoff",
        maxDrawdown: "Max Drawdown"
    },
    charts: {
        equity: "Net Equity",
        equitySubtitle: "Equity over time",
        consistency: "CONSISTENCY"
    },
    dailyDetail: {
        title: "Daily Detail",
        subtitle: "Detailed view of the day",
        summary: "Summary",
        tradeQty: "Total Trades",
        efficiency: "Efficiency",
        operationsList: "Operations List",
        asset: "Asset",
        direction: "Direction",
        result: "Result",
        buy: "Buy",
        sell: "Sell",
        noTrades: "No trades"
    },
    schedule: {
        title: "Most Efficient Schedules",
        morning: "Morning",
        afternoon: "Afternoon",
        basedOn: "Based on the last {count} business days"
    }
};

const newPsychologyTranslationsPt = {
    title: "Análise Psicológica",
    description: "Entenda como suas emoções impactam seu resultado financeiro.",
    syncWeights: "Sincronizar Pesos",
    checkin: {
        button: "Novo Registro"
    },
    kpi: {
        bestMindset: "Melhor Estado Mental",
        consolidated: "Consolidado",
        worstMindset: "Pior Estado Mental",
        accumulatedLoss: "Perda Acumulada",
        tiltCost: "O Custo do Tilt",
        tiltDescription: "A soma total dos prejuízos operando fora da zona de controle é de {value}",
        records: "Registros de Diário",
        recordsDescription: "Avaliações preenchidas na plataforma"
    },
    charts: {
        pnlByEmotion: "Resultado Financeiro por Emoção",
        winRate: "Eficiência %",
        correlation: "Correlação: Emoção x Resultado (Diário)"
    },
    analysis: {
        title: "Mapeamento Comportamental",
        hierarchy: "Hierarquia"
    },
    journal: {
        title: "Registros e Entradas Diretas",
        score: "Nota (0-10)",
        entryStateShort: "ESTADO INT. (CURTO)",
        exitStateShort: "ESTADO EXT. (CURTO)",
        intensityShort: "INTENSIDADE"
    },
    insight: {
        title: "Detalhe do Cálculo e Impacto",
        opScore: "Nota",
        formulaTitle: "Como chegamos nessa nota?"
    }
};

const newPsychologyTranslationsEn = {
    title: "Psychological Analysis",
    description: "Understand how your emotions impact your financial result.",
    syncWeights: "Sync Weights",
    checkin: {
        button: "New Entry"
    },
    kpi: {
        bestMindset: "Best Mindset",
        consolidated: "Consolidated",
        worstMindset: "Worst Mindset",
        accumulatedLoss: "Accumulated Loss",
        tiltCost: "Cost of Tilt",
        tiltDescription: "Total loss while operating out of control zone is {value}",
        records: "Journal Records",
        recordsDescription: "Evaluations filled in the platform"
    },
    charts: {
        pnlByEmotion: "Financial Result by Emotion",
        winRate: "Efficiency %",
        correlation: "Correlation: Emotion x Result (Daily)"
    },
    analysis: {
        title: "Behavioral Mapping",
        hierarchy: "Hierarchy"
    },
    journal: {
        title: "Records and Direct Entries",
        score: "Score (0-10)",
        entryStateShort: "ENTRY STATE (SHORT)",
        exitStateShort: "EXIT STATE (SHORT)",
        intensityShort: "INTENSITY"
    },
    insight: {
        title: "Calculation Details & Impact",
        opScore: "Score",
        formulaTitle: "How did we reach this score?"
    }
};

function addTranslations(filePath, langCode) {
    let content = fs.readFileSync(filePath, 'utf-8');
    let data = JSON.parse(content);

    // Add dashboard
    if (!data.dashboard || !data.dashboard.kpis) {
        if (!data.dashboard) {
            data.dashboard = langCode === 'pt' ? newDashboardTranslationsPt : newDashboardTranslationsEn;
        } else {
            Object.assign(data.dashboard, langCode === 'pt' ? newDashboardTranslationsPt : newDashboardTranslationsEn);
        }
    } else {
        Object.assign(data.dashboard, langCode === 'pt' ? newDashboardTranslationsPt : newDashboardTranslationsEn);
    }

    // Add psychology
    if (!data.psychology) {
        data.psychology = langCode === 'pt' ? newPsychologyTranslationsPt : newPsychologyTranslationsEn;
    } else {
        Object.assign(data.psychology, langCode === 'pt' ? newPsychologyTranslationsPt : newPsychologyTranslationsEn);
    }

    // Add finance.closureWizard.step1.noTradesWarning
    if (data.finance && data.finance.closureWizard && data.finance.closureWizard.step1) {
        if (!data.finance.closureWizard.step1.noTradesWarning) {
            data.finance.closureWizard.step1.noTradesWarning = langCode === 'pt' ?
                "Nenhum trade encontrado para o dia selecionado. Selecione outra data ou registre trades." :
                "No trades found for the selected day. Select another date or register trades.";
        }
    }

    fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
    console.log('Updated ' + filePath);
}

try {
    addTranslations(ptBrPath, 'pt');
    addTranslations(enUsPath, 'en');
} catch (e) {
    console.error(e);
}
