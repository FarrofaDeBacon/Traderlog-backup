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
        let lines = content.split('\n');
        lines.forEach((line, i) => {
            if (line.includes('$locale || "pt-BR"')) {
                console.log(`${f}:${i + 1}: ${line.trim()}`);
            }
        });
    }
});
