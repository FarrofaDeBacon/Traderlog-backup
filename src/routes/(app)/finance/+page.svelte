<script lang="ts">
    import {
        Wallet,
        Plus,
        Calculator,
        ArrowRightLeft,
        TrendingUp,
        Landmark,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Separator } from "$lib/components/ui/separator";
    import { toast } from "svelte-sonner";
    import * as Card from "$lib/components/ui/card";
    import StatementTable from "$lib/components/finance/StatementTable.svelte";
    import DailyClosureWizard from "$lib/components/finance/DailyClosureWizard.svelte";
    import TransactionDialog from "$lib/components/finance/TransactionDialog.svelte";
    import TransferDialog from "$lib/components/finance/TransferDialog.svelte";
    import AccountCard from "$lib/components/finance/AccountCard.svelte";
    import AccountEvolutionChart from "$lib/components/finance/AccountEvolutionChart.svelte";
    import CurrencyTicker from "$lib/components/finance/CurrencyTicker.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import * as Dialog from "$lib/components/ui/dialog";
    import { t, locale } from "svelte-i18n";
    import { cn, formatCurrency } from "$lib/utils";

    let showTransactionDialog = $state(false);
    let showTransferDialog = $state(false);
    let showClosureWizard = $state(false);
    let isSyncing = $state(false);

    let selectedChartCurrency = $state(
        settingsStore.userProfile?.main_currency || "BRL",
    );
    let selectedAccountId = $state<string>("all");

    let showBreakdown = $state(false);
    let breakdownCurrency = $state<string | null>(null);
    // Group accounts by currency for consolidated view
    let accountsByCurrency = $derived.by(() => {
        const groups: Record<string, { total: number; accounts: any[] }> = {};
        settingsStore.accounts.forEach((acc) => {
            if (!groups[acc.currency]) {
                groups[acc.currency] = { total: 0, accounts: [] };
            }
            groups[acc.currency].accounts.push(acc);
            groups[acc.currency].total += acc.balance;
        });
        return groups;
    });

    let breakdownData = $derived(
        breakdownCurrency ? accountsByCurrency[breakdownCurrency] : null,
    );

    function handleBreakdown(currency: string) {
        breakdownCurrency = currency;
        showBreakdown = true;
    }

    let totalBalanceBRL = $derived(
        settingsStore.accounts.reduce((acc, curr) => {
            const currencyObj = settingsStore.currencies.find(
                (c) => c.code === curr.currency,
            );
            const rate = currencyObj?.exchange_rate || 1;
            return acc + curr.balance * rate;
        }, 0),
    );

    const today = new Date();
    const currentMonthStr = today.toISOString().slice(0, 7); // "YYYY-MM"

    let monthResultBRL = $derived(
        tradesStore.getMonthlyTradeResult(
            currentMonthStr,
            settingsStore.accounts,
            settingsStore.currencies,
        ),
    );

    async function handleSync() {
        isSyncing = true;
        const result = await settingsStore.syncExchangeRates();
        if (result?.success) {
            toast.success(`Câmbio atualizado!`);
        } else if (result) {
            toast.error(result.error || "Erro ao sincronizar moedas.");
        }
        isSyncing = false;
    }

    const currencyColors: Record<string, string> = {
        BRL: "border-l-emerald-500",
        USD: "border-l-blue-500",
        EUR: "border-l-indigo-500",
        BTC: "border-l-orange-500",
        ETH: "border-l-purple-500",
        USDT: "border-l-cyan-500",
    };

    const currencyTextColors: Record<string, string> = {
        BRL: "text-emerald-500",
        USD: "text-blue-500",
        EUR: "text-indigo-500",
        BTC: "text-orange-500",
        ETH: "text-purple-500",
        USDT: "text-cyan-500",
    };

    function getCurrencyColor(curr: string) {
        return currencyColors[curr] || "border-l-primary";
    }

    function getCurrencyTextColor(curr: string) {
        return currencyTextColors[curr] || "text-primary";
    }
</script>

<div class="space-y-6 animate-in fade-in duration-500">
    <CurrencyTicker />

    <div class="flex-1 flex flex-col space-y-8 p-4 md:p-8">
        <div
            class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4"
        >
            <div>
                <h2 class="text-3xl font-bold tracking-tight">
                    {$t("finance.title")}
                </h2>
                <p class="text-muted-foreground">
                    {$t("finance.description")}
                </p>
            </div>
            <div class="flex flex-wrap gap-2">
                <Button
                    variant="outline"
                    onclick={() => (showClosureWizard = true)}
                >
                    <Calculator class="w-4 h-4 mr-2" />
                    {$t("finance.dailyClosure")}
                </Button>
                <Button
                    variant="outline"
                    onclick={() => (showTransferDialog = true)}
                >
                    <ArrowRightLeft class="w-4 h-4 mr-2" />
                    Transferir
                </Button>
                <Button
                    variant="outline"
                    onclick={handleSync}
                    disabled={isSyncing}
                >
                    <ArrowRightLeft
                        class={cn("w-4 h-4 mr-2", isSyncing && "animate-spin")}
                    />
                    {isSyncing ? "Sincronizando..." : "Atualizar Câmbio"}
                </Button>
                <Button onclick={() => (showTransactionDialog = true)}>
                    <Plus class="w-4 h-4 mr-2" />
                    {$t("finance.newTransaction")}
                </Button>
            </div>
        </div>

        <!-- Consolidated Balances by Currency -->
        <div
            class="grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5"
        >
            <!-- Main Converted Balance -->
            <Card.Root
                class="border-l-4 border-l-emerald-500 shadow-sm bg-card"
            >
                <Card.Header
                    class="flex flex-row items-center justify-between space-y-0 pb-2"
                >
                    <Card.Title class="text-sm font-medium">
                        Patrimônio Total (BRL)
                    </Card.Title>
                    <Wallet class="h-4 w-4 text-emerald-500" />
                </Card.Header>
                <Card.Content>
                    <div class="text-2xl font-bold text-emerald-600">
                        {formatCurrency(
                            totalBalanceBRL,
                            "BRL",
                            $locale || "pt-BR",
                        )}
                    </div>
                    <p class="text-xs text-muted-foreground">
                        Consolidado em conta principal
                    </p>
                </Card.Content>
            </Card.Root>

            {#each Object.entries(accountsByCurrency) as [currency, data]}
                <button
                    type="button"
                    onclick={() => handleBreakdown(currency)}
                    class="text-left w-full h-full"
                >
                    <Card.Root
                        class={cn(
                            "border-l-4 shadow-sm bg-card transition-colors cursor-pointer",
                            getCurrencyColor(currency),
                        )}
                    >
                        <Card.Header
                            class="flex flex-row items-center justify-between space-y-0 pb-2"
                        >
                            <Card.Title class="text-sm font-medium"
                                >Saldo em {currency}</Card.Title
                            >
                            <Landmark
                                class={cn(
                                    "h-4 w-4",
                                    getCurrencyTextColor(currency),
                                )}
                            />
                        </Card.Header>
                        <Card.Content>
                            <div class="text-xl font-bold">
                                {formatCurrency(
                                    data.total,
                                    currency,
                                    $locale || "pt-BR",
                                )}
                            </div>
                            <p class="text-[10px] text-muted-foreground mt-1">
                                Ver detalhamento de contas
                            </p>
                        </Card.Content>
                    </Card.Root>
                </button>
            {/each}

            <Card.Root
                class={cn(
                    "border-l-4 shadow-sm bg-card",
                    monthResultBRL >= 0
                        ? "border-l-emerald-500"
                        : "border-l-rose-500",
                )}
            >
                <Card.Header
                    class="flex flex-row items-center justify-between space-y-0 pb-2"
                >
                    <Card.Title class="text-sm font-medium"
                        >{$t("finance.quickStats.monthlyResult")}</Card.Title
                    >
                    <TrendingUp
                        class={cn(
                            "h-4 w-4",
                            monthResultBRL >= 0
                                ? "text-emerald-500"
                                : "text-rose-500",
                        )}
                    />
                </Card.Header>
                <Card.Content>
                    <div
                        class={cn(
                            "text-xl font-bold",
                            monthResultBRL >= 0
                                ? "text-emerald-500"
                                : "text-rose-500",
                        )}
                    >
                        {formatCurrency(
                            monthResultBRL,
                            "BRL",
                            $locale || "pt-BR",
                        )}
                    </div>
                </Card.Content>
            </Card.Root>
        </div>

        <!-- (Suas Contas section removed) -->

        <!-- Evolution Chart -->
        <Card.Root class="shadow-sm bg-card">
            <Card.Header class="pb-2">
                <div class="flex flex-row items-center justify-between w-full">
                    <div>
                        <Card.Title
                            class="text-base font-bold tracking-tight flex items-center gap-2"
                        >
                            <TrendingUp class="w-4 h-4 text-primary" />
                            Evolução Patrimonial
                        </Card.Title>
                        <Card.Description class="text-xs text-muted-foreground"
                            >Evolução acumulada em {selectedChartCurrency}</Card.Description
                        >
                    </div>
                </div>

                <!-- Account Filter Badges -->
                <div
                    class="flex items-center gap-2 overflow-x-auto pb-2 no-scrollbar px-1 mt-4"
                >
                    <button
                        type="button"
                        class={cn(
                            "px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-widest transition-all border shrink-0",
                            selectedAccountId === "all"
                                ? "bg-primary text-primary-foreground border-primary"
                                : "bg-muted/50 border-border text-muted-foreground hover:text-foreground hover:border-muted",
                        )}
                        onclick={() => (selectedAccountId = "all")}
                    >
                        Todas as Contas
                    </button>
                    {#each settingsStore.accounts as acc}
                        <button
                            type="button"
                            class={cn(
                                "px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-widest transition-all border shrink-0 flex items-center gap-1.5",
                                selectedAccountId === acc.id
                                    ? "bg-primary text-primary-foreground border-primary"
                                    : "bg-muted/50 border-border text-muted-foreground hover:text-foreground hover:border-muted",
                            )}
                            onclick={() => (selectedAccountId = acc.id)}
                        >
                            <div
                                class="w-1.5 h-1.5 rounded-full bg-current opacity-50"
                            ></div>
                            {acc.nickname}
                        </button>
                    {/each}
                </div>
            </Card.Header>
            <Card.Content>
                {#key selectedAccountId + selectedChartCurrency}
                    <AccountEvolutionChart
                        bind:accountId={selectedAccountId}
                        bind:currency={selectedChartCurrency}
                    />
                {/key}
            </Card.Content>
        </Card.Root>

        <Separator class="bg-zinc-800/50" />

        <!-- Statement Section -->
        <div class="space-y-4">
            <h3 class="text-xl font-bold tracking-tight">
                {$t("finance.statement.title")}
            </h3>
            <div class="rounded-md border bg-card/50">
                <StatementTable />
            </div>
        </div>
    </div>
</div>

<DailyClosureWizard bind:open={showClosureWizard} />
<TransactionDialog bind:open={showTransactionDialog} />
<TransferDialog bind:open={showTransferDialog} />

<Dialog.Root bind:open={showBreakdown}>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title
                >Detalhamento de Contas ({breakdownCurrency})</Dialog.Title
            >
            <Dialog.Description>
                Saldos individuais para a moeda {breakdownCurrency}.
            </Dialog.Description>
        </Dialog.Header>
        <div class="grid gap-4 py-4">
            {#if breakdownData && breakdownCurrency}
                {#each breakdownData.accounts as account}
                    <div
                        class={cn(
                            "flex items-center justify-between p-3 rounded-lg border border-l-4 bg-muted/50",
                            getCurrencyColor(breakdownCurrency),
                        )}
                    >
                        <div class="flex items-center gap-3">
                            <div
                                class="w-8 h-8 rounded-lg bg-background border flex items-center justify-center"
                            >
                                <Landmark
                                    class={cn(
                                        "w-4 h-4",
                                        getCurrencyTextColor(breakdownCurrency),
                                    )}
                                />
                            </div>
                            <div class="flex flex-col">
                                <span class="text-sm font-bold"
                                    >{account.nickname}</span
                                >
                                <span
                                    class="text-[10px] text-muted-foreground uppercase"
                                    >{account.broker}</span
                                >
                            </div>
                        </div>
                        <div class="text-right">
                            <div class="text-sm font-bold">
                                {formatCurrency(
                                    account.balance,
                                    breakdownCurrency,
                                    $locale || "pt-BR",
                                )}
                            </div>
                            <span class="text-[9px] text-muted-foreground"
                                >{account.account_number || "Sem Nu."}</span
                            >
                        </div>
                    </div>
                {/each}
            {/if}
        </div>
        <Dialog.Footer>
            <Button variant="outline" onclick={() => (showBreakdown = false)}
                >Fechar</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
