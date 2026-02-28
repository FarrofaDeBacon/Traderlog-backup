const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');
const langs = ['pt-BR', 'en-US', 'es-ES', 'fr-FR'];
const allKeys = {};

function flattenKeys(obj, prefix = '') {
    let result = [];
    for (const key in obj) {
        if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
            result = result.concat(flattenKeys(obj[key], prefix + key + '.'));
        } else {
            result.push(prefix + key);
        }
    }
    return result;
}

for (const lang of langs) {
    const filePath = path.join(localesDir, lang + '.json');
    if (fs.existsSync(filePath)) {
        const data = JSON.parse(fs.readFileSync(filePath, 'utf-8'));
        allKeys[lang] = flattenKeys(data);
    }
}

const baseLang = 'pt-BR';
const baseKeys = new Set(allKeys[baseLang]);

for (const lang of langs) {
    if (lang === baseLang) continue;
    const langKeys = new Set(allKeys[lang]);

    const missingInLang = [...baseKeys].filter(x => !langKeys.has(x));
    const extraInLang = [...langKeys].filter(x => !baseKeys.has(x));

    console.log(`\n=== Language: ${lang} ===`);
    console.log(`Total Keys: ${langKeys.size} (pt-BR has ${baseKeys.size})`);

    if (missingInLang.length > 0) {
        console.log(`\nMissing keys relative to pt-BR (${missingInLang.length}):`);
        missingInLang.slice(0, 10).forEach(k => console.log('  - ' + k));
        if (missingInLang.length > 10) console.log(`  ... and ${missingInLang.length - 10} more.`);
    }
    if (extraInLang.length > 0) {
        console.log(`\nExtra keys relative to pt-BR (${extraInLang.length}):`);
        extraInLang.slice(0, 10).forEach(k => console.log('  - ' + k));
        if (extraInLang.length > 10) console.log(`  ... and ${extraInLang.length - 10} more.`);
    }
}
