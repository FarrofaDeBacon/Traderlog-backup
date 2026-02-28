const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');

const newCalPt = {
    monthly: "Mensal",
    monthlyShort: "M",
    weekly: "Semanal",
    weeklyShort: "S",
    weekOf: "Semana de",
    of: "de",
    days: {
        sun: "Dom",
        mon: "Seg",
        tue: "Ter",
        wed: "Qua",
        thu: "Qui",
        fri: "Sex",
        sat: "Sáb"
    }
};

const newCalEn = {
    monthly: "Monthly",
    monthlyShort: "M",
    weekly: "Weekly",
    weeklyShort: "W",
    weekOf: "Week of",
    of: "of",
    days: {
        sun: "Sun",
        mon: "Mon",
        tue: "Tue",
        wed: "Wed",
        thu: "Thu",
        fri: "Fri",
        sat: "Sat"
    }
};

const newCalEs = {
    monthly: "Mensual",
    monthlyShort: "M",
    weekly: "Semanal",
    weeklyShort: "S",
    weekOf: "Semana de",
    of: "de",
    days: {
        sun: "Dom",
        mon: "Lun",
        tue: "Mar",
        wed: "Mié",
        thu: "Jue",
        fri: "Vie",
        sat: "Sáb"
    }
};

const newCalFr = {
    monthly: "Mensuel",
    monthlyShort: "M",
    weekly: "Hebdomadaire",
    weeklyShort: "H",
    weekOf: "Semaine de",
    of: "de",
    days: {
        sun: "Dim",
        mon: "Lun",
        tue: "Mar",
        wed: "Mer",
        thu: "Jeu",
        fri: "Ven",
        sat: "Sam"
    }
};

const langs = [
    { code: 'pt-BR', cal: newCalPt },
    { code: 'en-US', cal: newCalEn },
    { code: 'es-ES', cal: newCalEs },
    { code: 'fr-FR', cal: newCalFr }
];

for (const lang of langs) {
    const filePath = path.join(localesDir, lang.code + '.json');
    if (!fs.existsSync(filePath)) continue;

    let content = fs.readFileSync(filePath, 'utf-8');
    let data = JSON.parse(content);

    if (!data.calendar) {
        data.calendar = lang.cal;
    } else {
        Object.assign(data.calendar, lang.cal);
    }

    fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
    console.log(`Updated ${lang.code}.json`);
}
