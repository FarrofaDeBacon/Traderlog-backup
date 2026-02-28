const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');
const langs = ['pt-BR', 'en-US', 'es-ES', 'fr-FR'];

const months = {
    "pt-BR": {
        january: "Janeiro",
        february: "Fevereiro",
        march: "Março",
        april: "Abril",
        may: "Maio",
        june: "Junho",
        july: "Julho",
        august: "Agosto",
        september: "Setembro",
        october: "Outubro",
        november: "Novembro",
        december: "Dezembro"
    },
    "en-US": {
        january: "January",
        february: "February",
        march: "March",
        april: "April",
        may: "May",
        june: "June",
        july: "July",
        august: "August",
        september: "September",
        october: "October",
        november: "November",
        december: "December"
    },
    "es-ES": {
        january: "Enero",
        february: "Febrero",
        march: "Marzo",
        april: "Abril",
        may: "Mayo",
        june: "Junio",
        july: "Julio",
        august: "Agosto",
        september: "Septiembre",
        october: "Octubre",
        november: "Noviembre",
        december: "Diciembre"
    },
    "fr-FR": {
        january: "Janvier",
        february: "Février",
        march: "Mars",
        april: "Avril",
        may: "Mai",
        june: "Juin",
        july: "Juillet",
        august: "Août",
        september: "Septembre",
        october: "Octobre",
        november: "Novembre",
        december: "Décembre"
    }
}

const extras = {
    "pt-BR": {
        darf: {
            status: {
                late: "Atrasado",
                pending: "Pendente",
                paid: "Pago"
            },
            labels: {
                valueToPay: "Valor a Pagar",
                dueDate: "Vencimento",
                principal: "Principal",
                fine: "Multa (R$)",
                interest: "Juros (R$)",
                paymentDate: "Data do Pagamento",
                totalPaid: "Valor Total Pago"
            },
            buttons: {
                pay: "Pagar",
                cancel: "Cancelar",
                confirmPay: "Confirmar Pagamento",
                delete: "Excluir",
                confirmUnpay: "Confirmar Estorno"
            },
            messages: {
                loading: "Carregando...",
                emptyHistory: "Nenhuma DARF paga em {year}."
            },
            modal: {
                pay: {
                    title: "Registrar Pagamento",
                    description: "Insira os detalhes para registrar o pagamento desta DARF.",
                    noAccount: "Selecione uma conta para debitar."
                },
                delete: {
                    title: "Confirmar Exclusão",
                    prompt: "Tem certeza que deseja excluir esta DARF?"
                },
                unpay: {
                    title: "Desfazer Pagamento",
                    descriptionLine1: "Isso irá reverter o status da DARF para Pendente e estornar o valor de",
                    descriptionLine2: "para a conta de origem.",
                    descriptionLine3: "A transação de saída será removida do Hub Financeiro."
                }
            }
        }
    },
    "en-US": {
        darf: {
            status: {
                late: "Late",
                pending: "Pending",
                paid: "Paid"
            },
            labels: {
                valueToPay: "Value to Pay",
                dueDate: "Due Date",
                principal: "Principal",
                fine: "Fine (R$)",
                interest: "Interest (R$)",
                paymentDate: "Payment Date",
                totalPaid: "Total Value Paid"
            },
            buttons: {
                pay: "Pay",
                cancel: "Cancel",
                confirmPay: "Confirm Payment",
                delete: "Delete",
                confirmUnpay: "Confirm Refund"
            },
            messages: {
                loading: "Loading...",
                emptyHistory: "No DARFs paid in {year}."
            },
            modal: {
                pay: {
                    title: "Register Payment",
                    description: "Enter the details to register the payment for this DARF.",
                    noAccount: "Select an account to debit."
                },
                delete: {
                    title: "Confirm Deletion",
                    prompt: "Are you sure you want to delete this DARF?"
                },
                unpay: {
                    title: "Undo Payment",
                    descriptionLine1: "This will revert the DARF status to Pending and refund the amount of",
                    descriptionLine2: "to the source account.",
                    descriptionLine3: "The outgoing transaction will be removed from the Finance Hub."
                }
            }
        }
    },
    "es-ES": {},
    "fr-FR": {}
}

extras["es-ES"] = extras["en-US"];
extras["fr-FR"] = extras["en-US"];

for (const langCode of langs) {
    const filePath = path.join(localesDir, langCode + '.json');
    if (!fs.existsSync(filePath)) continue;

    let content = fs.readFileSync(filePath, 'utf-8');
    let data = JSON.parse(content);

    // Deep merge extra properties into fiscal
    if (!data.fiscal) data.fiscal = {};
    if (!data.fiscal.darf) data.fiscal.darf = {};

    // Merge new sub-objects
    Object.assign(data.fiscal.darf, {
        ...data.fiscal.darf,
        ...extras[langCode].darf
    });

    if (!data.months) {
        data.months = months[langCode];
    } else {
        Object.assign(data.months, months[langCode]);
    }

    fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
    console.log(`Updated DARF and months translations in ${langCode}.json`);
}
