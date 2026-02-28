const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');

const missingData = {
    "pt-BR": {
        general: {
            dayClosure: "Fechamento do Dia",
            asset: "Ativo",
            weight: "peso",
            intensity_short: "int.",
            weeks_upper: "SEMANAS"
        },
        psychology: {
            analysis: {
                emotionalCalc: "Cálculo Emocional",
                totalWeight: "Peso Total"
            }
        }
    },
    "en-US": {
        general: {
            dayClosure: "Day Closure",
            asset: "Asset",
            weight: "weight",
            intensity_short: "int.",
            weeks_upper: "WEEKS"
        },
        psychology: {
            analysis: {
                emotionalCalc: "Emotional Calc",
                totalWeight: "Total Weight"
            }
        }
    },
    "es-ES": {
        general: {
            dayClosure: "Cierre del Día",
            asset: "Activo",
            weight: "peso",
            intensity_short: "int.",
            weeks_upper: "SEMANAS"
        },
        psychology: {
            analysis: {
                emotionalCalc: "Cálculo Emocional",
                totalWeight: "Peso Total"
            }
        }
    },
    "fr-FR": {
        general: {
            dayClosure: "Clôture du Jour",
            asset: "Actif",
            weight: "poids",
            intensity_short: "int.",
            weeks_upper: "SEMAINES"
        },
        psychology: {
            analysis: {
                emotionalCalc: "Calcul Émotionnel",
                totalWeight: "Poids Total"
            }
        }
    }
};

['pt-BR', 'en-US', 'es-ES', 'fr-FR'].forEach(lang => {
    const filePath = path.join(localesDir, lang + '.json');
    if (fs.existsSync(filePath)) {
        let content = fs.readFileSync(filePath, 'utf-8');
        let data = JSON.parse(content);

        // general
        if (!data.general) data.general = {};
        Object.assign(data.general, missingData[lang].general);

        // psychology.analysis
        if (!data.psychology) data.psychology = {};
        if (!data.psychology.analysis) data.psychology.analysis = {};
        Object.assign(data.psychology.analysis, missingData[lang].psychology.analysis);

        fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
        console.log(`Injected behavioral mapping keys into ${lang}.json`);
    }
});
