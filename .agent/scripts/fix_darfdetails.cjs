const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');
const langs = ['pt-BR', 'en-US', 'es-ES', 'fr-FR'];

const darfDetails = {
    "pt-BR": {
        title: "Detalhes da DARF",
        description: "Informações completas de apuração e pagamento.",
        loading: "Carregando DARF e Apuração...",
        period: "Período",
        statusPaid: "Status Pago",
        statusPending: "Status Pendente",
        complementaryBadge: "Guia Complementar",
        revenueCode: "Código de Receita",
        dueDate: "Data de Vencimento",
        calculationInfo: "Informações do Cálculo",
        grossResult: "Resultado Bruto",
        compensatedLosses: "Prejuízos Compensados",
        calculationBasis: "Base de Cálculo",
        taxRate: "Alíquota",
        principalValue: "Valor Principal",
        fine: "Multa",
        interest: "Juros",
        totalPaid: "Total Pago",
        paidAt: "Pago Em",
        dateNotAvailable: "Data não disponível",
        errorMessage: "Houve um problema ao carregar esta DARF.",
        unpayTitle: "Desfazer",
        unpayConfirm: "Confirmar",
        unpayCancel: "Cancelar",
        unpayAction: "Desfazer Pagamento"
    },
    "en-US": {
        title: "DARF Details",
        description: "Complete assessment and payment information.",
        loading: "Loading DARF & Appraisal...",
        period: "Period",
        statusPaid: "Paid",
        statusPending: "Pending",
        complementaryBadge: "Complementary Voucher",
        revenueCode: "Revenue Code",
        dueDate: "Due Date",
        calculationInfo: "Calculation Info",
        grossResult: "Gross Result",
        compensatedLosses: "Compensated Losses",
        calculationBasis: "Calculation Basis",
        taxRate: "Tax Rate",
        principalValue: "Principal Value",
        fine: "Fine",
        interest: "Interest",
        totalPaid: "Total Paid",
        paidAt: "Paid At",
        dateNotAvailable: "Date not available",
        errorMessage: "There was a problem loading this DARF.",
        unpayTitle: "Undo",
        unpayConfirm: "Confirm",
        unpayCancel: "Cancel",
        unpayAction: "Undo Payment"
    },
    "es-ES": {
        title: "Detalles del DARF",
        description: "Información completa de liquidación y pago.",
        loading: "Cargando DARF y Evaluación...",
        period: "Período",
        statusPaid: "Pagado",
        statusPending: "Pendiente",
        complementaryBadge: "Comprobante Complementario",
        revenueCode: "Código de Ingreso",
        dueDate: "Fecha de Vencimiento",
        calculationInfo: "Info de Cálculo",
        grossResult: "Resultado Bruto",
        compensatedLosses: "Pérdidas Compensadas",
        calculationBasis: "Base de Cálculo",
        taxRate: "Tasa de Impuesto",
        principalValue: "Valor Principal",
        fine: "Multa",
        interest: "Intereses",
        totalPaid: "Total Pagado",
        paidAt: "Pagado El",
        dateNotAvailable: "Fecha no disponible",
        errorMessage: "Hubo un problema al cargar este DARF.",
        unpayTitle: "Deshacer",
        unpayConfirm: "Confirmar",
        unpayCancel: "Cancelar",
        unpayAction: "Deshacer Pago"
    },
    "fr-FR": {
        title: "Détails du DARF",
        description: "Informations complètes sur l'évaluation et le paiement.",
        loading: "Chargement du DARF et de l'évaluation...",
        period: "Période",
        statusPaid: "Payé",
        statusPending: "En attente",
        complementaryBadge: "Justificatif Complémentaire",
        revenueCode: "Code de Revenu",
        dueDate: "Date d'échéance",
        calculationInfo: "Infos de Calcul",
        grossResult: "Résultat Brut",
        compensatedLosses: "Pertes Compensées",
        calculationBasis: "Base de Calcul",
        taxRate: "Taux d'imposition",
        principalValue: "Valeur Principale",
        fine: "Amende",
        interest: "Intérêts",
        totalPaid: "Total Payé",
        paidAt: "Payé Le",
        dateNotAvailable: "Date non disponible",
        errorMessage: "Un problème est survenu lors du chargement de ce DARF.",
        unpayTitle: "Annuler",
        unpayConfirm: "Confirmer",
        unpayCancel: "Annuler",
        unpayAction: "Annuler le Paiement"
    }
}

for (const langCode of langs) {
    const filePath = path.join(localesDir, langCode + '.json');
    if (!fs.existsSync(filePath)) continue;

    let content = fs.readFileSync(filePath, 'utf-8');
    let data;
    try {
        data = JSON.parse(content);
    } catch (e) {
        console.error("Error parsing " + langCode, e);
        continue;
    }

    // Deep merge extra properties into finance
    if (!data.finance) data.finance = {};
    if (!data.finance.darfDetails) data.finance.darfDetails = {};

    // Merge new sub-objects
    Object.assign(data.finance.darfDetails, darfDetails[langCode]);

    // Ensure general.close exists for completeness
    if (!data.general) data.general = {};
    if (!data.general.close) {
        data.general.close = langCode === 'pt-BR' ? 'Fechar' : (langCode === 'es-ES' ? 'Cerrar' : (langCode === 'fr-FR' ? 'Fermer' : 'Close'));
    }

    fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
    console.log(`Updated Darf Details translations in ${langCode}.json`);
}
