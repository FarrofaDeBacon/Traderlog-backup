const fs = require('fs');
const path = require('path');

const files = [
    'src/routes/(app)/trades/+page.svelte',
    'src/routes/(app)/strategies/[id]/+page.svelte',
    'src/routes/(app)/settings/profile/+page.svelte',
    'src/routes/(app)/settings/accounts/+page.svelte',
    'src/routes/(app)/psychology/+page.svelte',
    'src/routes/(app)/finance/+page.svelte',
    'src/lib/components/trades/TradeDetailView.svelte',
    'src/lib/components/trades/NewTradeWizard.svelte',
    'src/lib/components/strategies/StrategyPerformanceCard.svelte',
    'src/lib/components/finance/StatementTable.svelte',
    'src/lib/components/finance/TransferDialog.svelte',
    'src/lib/components/finance/DailyClosureWizard.svelte'
];

files.forEach(f => {
    if (fs.existsSync(f)) {
        let content = fs.readFileSync(f, 'utf8');
        let originalContent = content;

        // 1. Fix double $locale
        content = content.replace(/\$locale\s*\|\|\s*\$locale\s*\|\|\s*"pt-BR"/g, '$locale || "pt-BR"');

        // 2. Fix specific logic in profile page
        if (f.includes('settings/profile/+page.svelte')) {
            // Restore literal "pt-BR" for comparison and selection value
            content = content.replace(/\{#if\s*formData\.language\s*===\s*\(?\$locale\s*\|\|\s*"pt-BR"\)?\}/g, '{#if formData.language === "pt-BR"}');
            content = content.replace(/value=\{\$locale\s*\|\|\s*"pt-BR"\}/g, 'value="pt-BR"');
        }

        if (content !== originalContent) {
            fs.writeFileSync(f, content, 'utf8');
            console.log('Fixed locale usage in: ' + f);
        }
    }
});
