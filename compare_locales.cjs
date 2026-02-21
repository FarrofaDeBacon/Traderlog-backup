const fs = require('fs');
const path = require('path');

const files = [
    'src/lib/i18n/locales/pt-BR.json',
    'src/lib/i18n/locales/en-US.json',
    'src/lib/i18n/locales/es-ES.json',
    'src/lib/i18n/locales/fr-FR.json'
].map(f => path.join('c:/PROJETOS/TraderLogPro', f));

function getKeys(obj, parentKey = '') {
    let keys = new Set();
    for (let k in obj) {
        let newKey = parentKey ? `${parentKey}.${k}` : k;
        keys.add(newKey);
        if (typeof obj[k] === 'object' && obj[k] !== null && !Array.isArray(obj[k])) {
            const childKeys = getKeys(obj[k], newKey);
            childKeys.forEach(ck => keys.add(ck));
        }
    }
    return keys;
}

let allKeysPerFile = {};
let supersetKeys = new Set();

files.forEach(f => {
    try {
        const content = fs.readFileSync(f, 'utf8');
        const data = JSON.parse(content);
        const keys = getKeys(data);
        allKeysPerFile[f] = keys;
        keys.forEach(k => supersetKeys.add(k));
    } catch (e) {
        console.error(`Error reading ${f}: ${e.message}`);
    }
});

console.log(`Total unique keys across all files: ${supersetKeys.size}`);

files.forEach(f => {
    const fileName = path.basename(f);
    const fileKeys = allKeysPerFile[f];
    if (!fileKeys) return;

    let missing = [];
    supersetKeys.forEach(k => {
        if (!fileKeys.has(k)) {
            missing.push(k);
        }
    });

    if (missing.length > 0) {
        console.log(`\nMissing keys in ${fileName} (${missing.length}):`);
        missing.sort().slice(0, 50).forEach(m => console.log(`  - ${m}`));
        if (missing.length > 50) console.log(`  ... and ${missing.length - 50} more.`);
    } else {
        console.log(`\nNo keys missing in ${fileName}`);
    }
});
