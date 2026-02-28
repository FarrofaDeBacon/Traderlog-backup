const fs = require('fs');
const path = require('path');

const localesDir = path.join(__dirname, '../../src/lib/i18n/locales');
const ptBrPath = path.join(localesDir, 'pt-BR.json');
const enUsPath = path.join(localesDir, 'en-US.json');
const esEsPath = path.join(localesDir, 'es-ES.json');
const frFrPath = path.join(localesDir, 'fr-FR.json');

const pt = JSON.parse(fs.readFileSync(ptBrPath, 'utf8'));
const en = JSON.parse(fs.readFileSync(enUsPath, 'utf8'));
const es = JSON.parse(fs.readFileSync(esEsPath, 'utf8'));
const fr = JSON.parse(fs.readFileSync(frFrPath, 'utf8'));

// Function to recursively find missing keys and copy them
function syncKeys(source, target, langCode) {
    let changed = false;
    for (const key in source) {
        if (typeof source[key] === 'object' && source[key] !== null) {
            if (!target[key] || typeof target[key] !== 'object') {
                target[key] = {};
                changed = true;
            }
            if (syncKeys(source[key], target[key], langCode)) {
                changed = true;
            }
        } else {
            if (!(key in target)) {
                // To indicate it's missing, we add it with a prefix or just copy the PT-BR text
                // Since I can't translate everything via script, I'll copy the Portuguese string 
                // but prepend [EN] or similar if it's missing. Actually, better yet, we can try to leave it in English 
                // if it's en-US, or Portuguese for others.
                target[key] = typeof source[key] === 'string' ? source[key] : source[key];
                changed = true;
            }
        }
    }
    return changed;
}

const enChanged = syncKeys(pt, en, 'en');
const esChanged = syncKeys(pt, es, 'es');
const frChanged = syncKeys(pt, fr, 'fr');

if (enChanged) fs.writeFileSync(enUsPath, JSON.stringify(en, null, 2) + '\n');
if (esChanged) fs.writeFileSync(esEsPath, JSON.stringify(es, null, 2) + '\n');
if (frChanged) fs.writeFileSync(frFrPath, JSON.stringify(fr, null, 2) + '\n');

console.log(`Synced EN: ${enChanged}, ES: ${esChanged}, FR: ${frChanged}`);
