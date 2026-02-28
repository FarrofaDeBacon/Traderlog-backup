const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');
const langs = ['pt-BR', 'en-US', 'es-ES', 'fr-FR'];

const translations = {
    "pt-BR": {
        darf: {
            title: "Painel de DARFs",
            description: "Gerencie e armazene seus comprovantes de arrecadação.",
            history: "Histórico de DARFs",
            accountError: "Erro na Conta",
            active: "Ativo",
            fineInterest: "Multa e Juros",
            principalValue: "Valor Principal",
            registerPayment: "Registrar Pagamento",
            revenueCode: "Código de Receita",
            selectAccount: "Selecionar Conta",
            sourceAccount: "Conta de Origem",
            totalPaid: "Total Pago",
            table: {
                period: "Período",
                revenueCode: "Cód. Receita",
                principalValue: "Valor Principal",
                fineInterest: "Multas/Juros",
                totalPaid: "Total Pago",
                status: "Status"
            }
        },
        irpf: {
            title: "Central de IRPF",
            description: "Acompanhe e consolide suas notas e tributos mensais (Day Trade e Swing).",
            calculate: "Calcular",
            calculating: "Calculando...",
            evolution: "Evolução Tributária",
            history: "Histórico de Apurações",
            manageDarfs: "Gerenciar DARFs",
            newAppraisal: "Nova Apuração",
            periodFilter: "Filtrar por Ano",
            saveBeforeDarf: "Salve a apuração antes de gerar uma DARF.",
            kpis: {
                totalDue: "Imposto Devido",
                dueYearHint: "Acumulado a pagar neste exercício.",
                totalPaid: "Total Pago",
                paidYearHint: "Soma das guias já quitadas.",
                pending: "Guias Pendentes",
                pendingHint: "Você possui {count} DARF(s) pendente(s).",
                losses: "Prejuízos Acumulados"
            },
            modal: {
                title: "Nova Apuração",
                description: "Calcule os resultados líquidos isolados de um mês específico.",
                month: "Mês Base",
                year: "Ano-calendário"
            },
            table: {
                period: "Mês/Ano",
                type: "Modalidade",
                netProfit: "Lucro Líq.",
                toPay: "IR Devido",
                compensated: "Compensado",
                status: "Situação",
                actions: "Ações",
                paid: "Quitado",
                pending: "Pendente",
                ok: "Sem Imposto",
                alreadyGenerated: "DARF gerada",
                generateDarf: "Gerar DARF",
                minTaxHint: "Valor inferior ou igual à R$ 10",
                minTaxAlert: "IRPF mínimo",
                cannotDelete: "Não é possível excluir"
            }
        }
    },
    "en-US": {
        darf: {
            title: "DARF Dashboard",
            description: "Manage and store your tax collection vouchers.",
            history: "DARF History",
            accountError: "Account Error",
            active: "Active",
            fineInterest: "Fine & Interest",
            principalValue: "Principal Value",
            registerPayment: "Register Payment",
            revenueCode: "Revenue Code",
            selectAccount: "Select Account",
            sourceAccount: "Source Account",
            totalPaid: "Total Paid",
            table: {
                period: "Period",
                revenueCode: "Rev. Code",
                principalValue: "Principal",
                fineInterest: "Fines/Interest",
                totalPaid: "Total Paid",
                status: "Status"
            }
        },
        irpf: {
            title: "IRPF Hub",
            description: "Track and consolidate your monthly tax assessments (Day Trade and Swing).",
            calculate: "Calculate",
            calculating: "Calculating...",
            evolution: "Tax Evolution",
            history: "Appraisal History",
            manageDarfs: "Manage DARFs",
            newAppraisal: "New Appraisal",
            periodFilter: "Year Filter",
            saveBeforeDarf: "Save the appraisal before generating a DARF.",
            kpis: {
                totalDue: "Tax Due",
                dueYearHint: "Accumulated payable this year.",
                totalPaid: "Total Paid",
                paidYearHint: "Sum of paid vouchers.",
                pending: "Pending Vouchers",
                pendingHint: "You have {count} pending DARF(s).",
                losses: "Accumulated Losses"
            },
            modal: {
                title: "New Appraisal",
                description: "Calculate the net isolated results for a specific month.",
                month: "Base Month",
                year: "Calendar Year"
            },
            table: {
                period: "Month/Year",
                type: "Modality",
                netProfit: "Net Profit",
                toPay: "Tax Due",
                compensated: "Compensated",
                status: "Status",
                actions: "Actions",
                paid: "Paid",
                pending: "Pending",
                ok: "No Tax",
                alreadyGenerated: "DARF Generated",
                generateDarf: "Generate DARF",
                minTaxHint: "Value lesser than or equal to R$ 10",
                minTaxAlert: "Min. IRPF",
                cannotDelete: "Cannot delete"
            }
        }
    },
    // Mirroring English into ES and FR temporarily for ease, 
    // it's better than placeholders like 'Title'. 
    "es-ES": {},
    "fr-FR": {}
};

// Fallback to EN if ES or FR isn't defined explicitly
translations["es-ES"] = translations["en-US"];
translations["fr-FR"] = translations["en-US"];

for (const langCode of langs) {
    const filePath = path.join(localesDir, langCode + '.json');
    if (!fs.existsSync(filePath)) continue;

    let content = fs.readFileSync(filePath, 'utf-8');
    let data = JSON.parse(content);

    // Deep merge fiscal
    if (!data.fiscal) {
        data.fiscal = translations[langCode];
    } else {
        // We override since the original 'fiscal' had placeholders
        data.fiscal = { ...data.fiscal, ...translations[langCode] };
    }

    fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
    console.log(`Updated fiscal translations in ${langCode}.json`);
}
