const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');

// We will extract the missing keys from es-ES and inject into pt-BR / en-US.
const esPath = path.join(localesDir, 'es-ES.json');
const esData = JSON.parse(fs.readFileSync(esPath, 'utf-8'));

// The extra keys are in `psychology.kpi`, `psychology.charts`, `psychology.checkin`
// Instead of copying Spanish text, we will manually define them in PT and EN to be correct.

const ptMissing = {
    psychology: {
        kpi: {
            unregistered: "Não Registrado",
            unregisteredDescription: "Trades sem check-in"
        },
        charts: {
            winRateTitle: "Taxa de Acerto vs Emocional",
            resultTitle: "Resultado vs Emocional"
        },
        checkin: {
            title: "Check-in Emocional",
            description: "Registre como você estava se sentindo no momento deste trade.",
            date: "Data do Trade",
            stateLabel: "Estado Emocional",
            statePlaceholder: "Selecione a emoção predominante",
            intensityLabel: "Intensidade",
            intensityPlaceholder: "Qual a intensidade (1-10)?",
            notesLabel: "Observações Adicionais",
            notesPlaceholder: "O que desencadeou isso? (Opcional)",
            saveCheckin: "Salvar Check-in"
        }
    }
};

const enMissing = {
    psychology: {
        kpi: {
            unregistered: "Unregistered",
            unregisteredDescription: "Trades without check-in"
        },
        charts: {
            winRateTitle: "Win Rate vs Emotional",
            resultTitle: "Result vs Emotional"
        },
        checkin: {
            title: "Emotional Check-in",
            description: "Log how you were feeling at the time of this trade.",
            date: "Trade Date",
            stateLabel: "Emotional State",
            statePlaceholder: "Select the predominant emotion",
            intensityLabel: "Intensity",
            intensityPlaceholder: "Intensity level (1-10)?",
            notesLabel: "Additional Notes",
            notesPlaceholder: "What triggered this? (Optional)",
            saveCheckin: "Save Check-in"
        }
    }
};

['pt-BR', 'en-US'].forEach(lang => {
    const filePath = path.join(localesDir, lang + '.json');
    if (fs.existsSync(filePath)) {
        let content = fs.readFileSync(filePath, 'utf-8');
        let data = JSON.parse(content);

        if (!data.psychology) data.psychology = {};

        // Deep merge
        const sources = lang === 'pt-BR' ? ptMissing.psychology : enMissing.psychology;

        Object.keys(sources).forEach(category => {
            if (!data.psychology[category]) {
                data.psychology[category] = {};
            }
            Object.assign(data.psychology[category], sources[category]);
        });

        fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
        console.log(`Injected missing psychology keys into ${lang}.json`);
    }
});
