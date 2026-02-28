import json
import os

def set_nested(d, path, value):
    keys = path.split('.')
    current = d
    for key in keys[:-1]:
        if key not in current or not isinstance(current[key], dict):
            current[key] = {}
        current = current[key]
    current[keys[-1]] = value

locales_dir = "src/lib/i18n/locales"

pt_BR = {
    "finance.quickStats.balanceIn": "Saldo em {currency}",
    "finance.quickStats.totalEquity": "Patrimônio Total",
    "finance.quickStats.consolidated": "Consolidado",
    "finance.quickStats.viewBreakdown": "Ver Detalhes",
    "finance.charts.evolutionSubtitle": "Evolução do Patrimônio e Aportes",
    
    "trades.quickStats.totalBalance": "Saldo Total",
    "trades.quickStats.winRate": "Taxa de Acerto",
    "trades.quickStats.winRateDesc": "{winners} de {total} Trades",
    "trades.quickStats.profitFactor": "Profit Factor",
    "trades.quickStats.profitFactorDesc": "Fator de Lucratividade",
    "trades.quickStats.openTrades": "Em Aberto",
    "trades.quickStats.openTradesDesc": "Posições Atuais",
    
    "trades.dashboard.weekOf": "Semana de {date}",
    "trades.sidebar.summary": "Resumo Lateral",
    "trades.sidebar.performance": "Desempenho",
    "trades.sidebar.distribution": "Distribuição",
    "trades.sidebar.equityCurve": "Curva de Capital - Drawdown",
    "trades.sidebar.gain": "Gain"
}

en_US = {
    "finance.quickStats.balanceIn": "Balance in {currency}",
    "finance.quickStats.totalEquity": "Total Equity",
    "finance.quickStats.consolidated": "Consolidated",
    "finance.quickStats.viewBreakdown": "View Breakdown",
    "finance.charts.evolutionSubtitle": "Equity and Deposits Evolution",
    
    "trades.quickStats.totalBalance": "Total Balance",
    "trades.quickStats.winRate": "Win Rate",
    "trades.quickStats.winRateDesc": "{winners} of {total} Trades",
    "trades.quickStats.profitFactor": "Profit Factor",
    "trades.quickStats.profitFactorDesc": "Profitability Factor",
    "trades.quickStats.openTrades": "Open Trades",
    "trades.quickStats.openTradesDesc": "Current Positions",
    
    "trades.dashboard.weekOf": "Week of {date}",
    "trades.sidebar.summary": "Side Summary",
    "trades.sidebar.performance": "Performance",
    "trades.sidebar.distribution": "Distribution",
    "trades.sidebar.equityCurve": "Equity Curve - Drawdown",
    "trades.sidebar.gain": "Gain"
}

es_ES = {
    "finance.quickStats.balanceIn": "Saldo en {currency}",
    "finance.quickStats.totalEquity": "Patrimonio Total",
    "finance.quickStats.consolidated": "Consolidado",
    "finance.quickStats.viewBreakdown": "Ver Detalles",
    "finance.charts.evolutionSubtitle": "Evolución del Patrimonio",
    
    "trades.quickStats.totalBalance": "Saldo Total",
    "trades.quickStats.winRate": "Tasa de Acierto",
    "trades.quickStats.winRateDesc": "{winners} de {total} Trades",
    "trades.quickStats.profitFactor": "Profit Factor",
    "trades.quickStats.profitFactorDesc": "Factor de Rentabilidad",
    "trades.quickStats.openTrades": "Abiertos",
    "trades.quickStats.openTradesDesc": "Posiciones Actuales",
    
    "trades.dashboard.weekOf": "Semana del {date}",
    "trades.sidebar.summary": "Resumen Lateral",
    "trades.sidebar.performance": "Rendimiento",
    "trades.sidebar.distribution": "Distribución",
    "trades.sidebar.equityCurve": "Curva de Capital - Drawdown",
    "trades.sidebar.gain": "Gain"
}

fr_FR = {
    "finance.quickStats.balanceIn": "Solde en {currency}",
    "finance.quickStats.totalEquity": "Capitaux Totaux",
    "finance.quickStats.consolidated": "Consolidé",
    "finance.quickStats.viewBreakdown": "Voir les détails",
    "finance.charts.evolutionSubtitle": "Évolution du Solde",
    
    "trades.quickStats.totalBalance": "Solde Total",
    "trades.quickStats.winRate": "Taux de Réussite",
    "trades.quickStats.winRateDesc": "{winners} sur {total} Trades",
    "trades.quickStats.profitFactor": "Facteur de Profit",
    "trades.quickStats.profitFactorDesc": "Facteur de Rentabilité",
    "trades.quickStats.openTrades": "Ouvert",
    "trades.quickStats.openTradesDesc": "Positions Actuelles",
    
    "trades.dashboard.weekOf": "Semaine du {date}",
    "trades.sidebar.summary": "Résumé Latéral",
    "trades.sidebar.performance": "Performances",
    "trades.sidebar.distribution": "Distribution",
    "trades.sidebar.equityCurve": "Courbe de Capital - Drawdown",
    "trades.sidebar.gain": "Gain"
}

def inject(lang, dic):
    file_path = os.path.join(locales_dir, f"{lang}.json")
    if not os.path.exists(file_path):
        return
    with open(file_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    for k, v in dic.items():
        set_nested(data, k, v)
        
    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=4, ensure_ascii=False)
        print(f"Injected into {lang}")

inject('pt-BR', pt_BR)
inject('en-US', en_US)
inject('es-ES', es_ES)
inject('fr-FR', fr_FR)
print("Done.")
