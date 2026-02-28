const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');

const newLicensePt = {
    banner: {
        trialActive: "Você tem {days} dias restantes no período de trial.",
        getLicensePrompt: "Garanta seu acesso.",
        activateNow: "Ativar Agora",
        expired: "Seu período de trial expirou. Renove sua licença.",
        getLicenseNow: "Obter Licença"
    }
};

const newLicenseEn = {
    banner: {
        trialActive: "You have {days} days left in your trial period.",
        getLicensePrompt: "Secure your access.",
        activateNow: "Activate Now",
        expired: "Your trial period has expired. Renew your license.",
        getLicenseNow: "Get License"
    }
};

const newLicenseEs = {
    banner: {
        trialActive: "Te quedan {days} días de prueba.",
        getLicensePrompt: "Asegura tu acceso.",
        activateNow: "Activar Ahora",
        expired: "Tu período de prueba ha expirado.",
        getLicenseNow: "Obtener Licencia"
    }
};

const newLicenseFr = {
    banner: {
        trialActive: "Il vous reste {days} jours d'essai.",
        getLicensePrompt: "Sécurisez votre accès.",
        activateNow: "Activer Maintenant",
        expired: "Votre période d'essai a expiré.",
        getLicenseNow: "Obtenir une Licence"
    }
};

const newSurvivorPt = {
    survivor: {
        title: "Survivor Hub & Monitoring",
        activePlan: "Plano Ativo:",
        select: "Selecione",
        operationalStatus: "Status Operacional",
        statusBlocked: "Bloqueado (Loss Max)",
        statusRisk: "Alerta de Risco",
        statusAlpha: "Alpha Operational",
        currentPhase: "Fase Atual",
        dailyEvolution: "Evolução Diária",
        target: "Meta: ",
        tradesUsage: "Uso de Operações",
        avgRFactor: "Fator R Médio",
        roadmapProgress: "Roadmap Progress",
        pnl: "Pnl",
        consistency: "Consist.",
        riskMonth: "Survivor Risk (Mês)",
        limit: "Limite",
        eligibility: "Elegibilidade",
        greenZone: "Green Zone",
        operateTrade: "OPERAR 2% TRADE"
    }
};

const newSurvivorEn = {
    survivor: {
        title: "Survivor Hub & Monitoring",
        activePlan: "Active Plan:",
        select: "Select",
        operationalStatus: "Operational Status",
        statusBlocked: "Blocked (Max Loss)",
        statusRisk: "Risk Alert",
        statusAlpha: "Alpha Operational",
        currentPhase: "Current Phase",
        dailyEvolution: "Daily Evolution",
        target: "Target: ",
        tradesUsage: "Trades Usage",
        avgRFactor: "Avg R-Factor",
        roadmapProgress: "Roadmap Progress",
        pnl: "PnL",
        consistency: "Consist.",
        riskMonth: "Survivor Risk (Month)",
        limit: "Limit",
        eligibility: "Eligibility",
        greenZone: "Green Zone",
        operateTrade: "OPERATE 2% TRADE"
    }
};

const newSurvivorEs = {
    survivor: {
        title: "Survivor Hub & Monitoring",
        activePlan: "Plan Activo:",
        select: "Seleccionar",
        operationalStatus: "Estado Operativo",
        statusBlocked: "Bloqueado (Pérdida Máx)",
        statusRisk: "Alerta de Riesgo",
        statusAlpha: "Alpha Operational",
        currentPhase: "Fase Actual",
        dailyEvolution: "Evolución Diaria",
        target: "Meta: ",
        tradesUsage: "Uso de Operaciones",
        avgRFactor: "Factor R Promedio",
        roadmapProgress: "Progreso Roadmap",
        pnl: "PnL",
        consistency: "Consist.",
        riskMonth: "Riesgo Survivor (Mes)",
        limit: "Límite",
        eligibility: "Elegibilidad",
        greenZone: "Green Zone",
        operateTrade: "OPERAR 2% TRADE"
    }
};

const newSurvivorFr = {
    survivor: {
        title: "Survivor Hub & Monitoring",
        activePlan: "Plan Actif:",
        select: "Sélectionner",
        operationalStatus: "Statut Opérationnel",
        statusBlocked: "Bloqué (Perte Max)",
        statusRisk: "Alerte de Risque",
        statusAlpha: "Alpha Operational",
        currentPhase: "Phase Actuelle",
        dailyEvolution: "Évolution Quotidienne",
        target: "Cible: ",
        tradesUsage: "Utilisation des Opérations",
        avgRFactor: "Facteur R Moyen",
        roadmapProgress: "Progression de la Feuille de Route",
        pnl: "PnL",
        consistency: "Consist.",
        riskMonth: "Risque Survivor (Mois)",
        limit: "Limite",
        eligibility: "Éligibilité",
        greenZone: "Zone Verte",
        operateTrade: "OPÉRER 2% TRADE"
    }
};

const newChartsPt = { wins: "VITÓRIAS", losses: "DERROTAS" };
const newChartsEn = { wins: "WINS", losses: "LOSSES" };
const newChartsEs = { wins: "VICTORIAS", losses: "DERROTAS" };
const newChartsFr = { wins: "VICTOIRES", losses: "DÉFAITES" };

const langs = [
    { code: 'pt-BR', license: newLicensePt, survivor: newSurvivorPt, charts: newChartsPt },
    { code: 'en-US', license: newLicenseEn, survivor: newSurvivorEn, charts: newChartsEn },
    { code: 'es-ES', license: newLicenseEs, survivor: newSurvivorEs, charts: newChartsEs },
    { code: 'fr-FR', license: newLicenseFr, survivor: newSurvivorFr, charts: newChartsFr }
];

for (const lang of langs) {
    const filePath = path.join(localesDir, lang.code + '.json');
    if (!fs.existsSync(filePath)) continue;

    let content = fs.readFileSync(filePath, 'utf-8');
    let data = JSON.parse(content);

    // 1. License block
    if (!data.license) {
        data.license = lang.license;
    } else {
        if (!data.license.banner) {
            data.license.banner = lang.license.banner;
        } else {
            Object.assign(data.license.banner, lang.license.banner);
        }
    }

    // 2. Dashboard Survivor block
    if (!data.dashboard) {
        data.dashboard = {};
    }

    if (!data.dashboard.survivor) {
        data.dashboard.survivor = lang.survivor.survivor;
    } else {
        Object.assign(data.dashboard.survivor, lang.survivor.survivor);
    }

    // 3. Dashboard Charts extension
    if (!data.dashboard.charts) {
        data.dashboard.charts = {};
    }
    Object.assign(data.dashboard.charts, lang.charts);

    fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
    console.log(`Updated ${lang.code}.json`);
}
