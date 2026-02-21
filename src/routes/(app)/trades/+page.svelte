<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import * as Card from "$lib/components/ui/card";
    import * as Table from "$lib/components/ui/table";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { t, locale } from "svelte-i18n";
    import {
        ChevronDown,
        ChevronRight,
        TrendingUp,
        TrendingDown,
        Calendar as CalendarIcon,
        Search,
        Filter,
        BookOpen,
        Pencil,
        Trash2,
        ArrowRightLeft,
        Eye,
        X,
        CheckCircle2,
        Clock,
        Plus,
        Coins,
        Activity,
    } from "lucide-svelte";
    import { Badge } from "$lib/components/ui/badge";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";
    import {
        cn,
        formatCurrency,
        formatNumber,
        getLocalDatePart,
    } from "$lib/utils";
    import * as Dialog from "$lib/components/ui/dialog";
    import NewTradeWizard from "$lib/components/trades/NewTradeWizard.svelte";
    import TradeDetailView from "$lib/components/trades/TradeDetailView.svelte";
    import * as Select from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";
    import TradeOutcomePieChart from "$lib/components/trades/TradeOutcomePieChart.svelte";
    import TradeEquityChart from "$lib/components/trades/TradeEquityChart.svelte";

    import { untrack } from "svelte";
    import { format } from "date-fns";

    let searchQuery = $state("");
    let expandedMonths = $state<Set<string>>(new Set());
    let expandedWeeks = $state<Set<string>>(new Set());
    let expandedDays = $state<Record<string, boolean>>({});

    // Filter states
    let filterStatus = $state("all"); // all, open, closed
    let filterAccount = $state("all");
    let filterStrategy = $state("all");
    let filterAssetType = $state("all");
    let filterCurrency = $state("all");
    let showFilters = $state(false);
    let currencyMode = $state<"original" | "main">("original");

    // Dialog/Modal states
    let isDeleteOpen = $state(false);
    let tradeToDelete = $state<string | null>(null);
    let isEditOpen = $state(false);
    let isViewOpen = $state(false);
    let selectedTrade = $state<any>(null);

    // Dynamic Chart Context
    let activeContext = $state<{
        type: "global" | "month" | "week" | "day" | "trade";
        key: string | null;
        label: string;
        data?: any;
    }>({ type: "global", key: null, label: "Todo o Período" });

    const allTrades = $derived(tradesStore.trades);

    // Base Filtering logic
    const filteredTrades = $derived.by(() => {
        return allTrades.filter((t) => {
            // Search filter
            if (searchQuery) {
                const q = searchQuery.toLowerCase();
                const strategyName =
                    settingsStore.strategies
                        .find((s) => s.id === t.strategy_id)
                        ?.name.toLowerCase() || "";
                const matchesSearch =
                    t.asset_symbol.toLowerCase().includes(q) ||
                    strategyName.includes(q);
                if (!matchesSearch) return false;
            }

            // Status filter
            if (filterStatus !== "all") {
                const isClosed = !!t.exit_price;
                if (filterStatus === "open" && isClosed) return false;
                if (filterStatus === "closed" && !isClosed) return false;
            }

            // Account filter
            if (filterAccount !== "all" && t.account_id !== filterAccount)
                return false;

            // Strategy filter
            if (filterStrategy !== "all" && t.strategy_id !== filterStrategy)
                return false;

            // Asset Type filter
            if (
                filterAssetType !== "all" &&
                t.asset_type_id !== filterAssetType
            )
                return false;

            // Currency filter
            if (filterCurrency !== "all") {
                const acc = settingsStore.accounts.find(
                    (a) => a.id === t.account_id,
                );
                if (acc?.currency !== filterCurrency) return false;
            }

            return true;
        });
    });

    function getWeekKey(date: Date) {
        const d = new Date(date);
        const day = d.getDay();
        const diff = d.getDate() - day + (day === 0 ? -6 : 1);
        const monday = new Date(d.setDate(diff));
        return monday.toISOString().split("T")[0];
    }

    const hierarchicalTradesData = $derived.by(() => {
        const monthsMap: Record<string, any> = {};

        // Build the hierarchy: Month -> Week -> Day
        for (const trade of filteredTrades) {
            const dateStr = getLocalDatePart(trade.exit_date || trade.date);
            const date = new Date(dateStr + "T12:00:00");
            const monthKey = dateStr.slice(0, 7); // YYYY-MM
            const weekKey = getWeekKey(date);

            // Month Level
            if (!monthsMap[monthKey]) {
                monthsMap[monthKey] = {
                    key: monthKey,
                    label: date.toLocaleDateString($locale || "pt-BR", {
                        month: "long",
                        year: "numeric",
                    }),
                    weeks: {},
                    totalPnlByCurrency: {},
                    trades: [],
                };
            }
            const month = monthsMap[monthKey];
            month.trades.push(trade);

            // Week Level
            if (!month.weeks[weekKey]) {
                month.weeks[weekKey] = {
                    key: weekKey,
                    label: `Semana de ${new Date(weekKey + "T12:00:00").toLocaleDateString($locale || "pt-BR", { day: "numeric", month: "short" })}`,
                    days: {},
                    totalPnlByCurrency: {},
                    trades: [],
                };
            }
            const week = month.weeks[weekKey];
            week.trades.push(trade);

            // Day Level
            if (!week.days[dateStr]) {
                week.days[dateStr] = {
                    key: dateStr,
                    date: dateStr,
                    label: date.toLocaleDateString($locale || "pt-BR", {
                        weekday: "long",
                        day: "numeric",
                    }),
                    trades: [],
                    totalPnlByCurrency: {},
                };
            }
            const day = week.days[dateStr];
            day.trades.push(trade);

            // Calculate PnL across levels
            const acc = settingsStore.accounts.find(
                (a) => a.id === trade.account_id,
            );
            const curr = acc?.currency || "BRL";
            const res = trade.result || 0;

            day.totalPnlByCurrency[curr] =
                (day.totalPnlByCurrency[curr] || 0) + res;
            week.totalPnlByCurrency[curr] =
                (week.totalPnlByCurrency[curr] || 0) + res;
            month.totalPnlByCurrency[curr] =
                (month.totalPnlByCurrency[curr] || 0) + res;
        }

        // Convert Objects back to sorted arrays
        return Object.values(monthsMap)
            .sort((a, b) => b.key.localeCompare(a.key))
            .map((month) => {
                const weeks = Object.values(month.weeks)
                    .sort((a: any, b: any) => b.key.localeCompare(a.key))
                    .map((week: any) => {
                        const days = Object.values(week.days)
                            .sort((a: any, b: any) =>
                                b.date.localeCompare(a.date),
                            )
                            .map((day: any) => ({
                                ...day,
                                pnlEntries: Object.entries(
                                    day.totalPnlByCurrency,
                                ).map(([curr, val]) => ({ curr, val })),
                            }));

                        return {
                            ...week,
                            days,
                            pnlEntries: Object.entries(
                                week.totalPnlByCurrency,
                            ).map(([curr, val]) => ({ curr, val })),
                        };
                    });

                return {
                    ...month,
                    weeks,
                    pnlEntries: Object.entries(month.totalPnlByCurrency).map(
                        ([curr, val]) => ({ curr, val }),
                    ),
                };
            });
    });

    $effect(() => {
        if (hierarchicalTradesData.length > 0) {
            untrack(() => {
                if (expandedMonths.size === 0) {
                    const firstMonth = hierarchicalTradesData[0];
                    expandedMonths.add(firstMonth.key);
                    if (firstMonth.weeks.length > 0) {
                        expandedWeeks.add(firstMonth.weeks[0].key);
                    }
                    expandedMonths = new Set(expandedMonths);
                    expandedWeeks = new Set(expandedWeeks);
                }
            });
        }
    });

    function toggleMonth(key: string) {
        if (expandedMonths.has(key)) {
            expandedMonths.delete(key);
            activeContext = {
                type: "global",
                key: null,
                label: "Todo o Período",
            };
        } else {
            expandedMonths.add(key);
            const month = hierarchicalTradesData.find((m) => m.key === key);
            if (month) {
                activeContext = {
                    type: "month",
                    key: key,
                    label: month.label,
                };
            }
        }
        expandedMonths = new Set(expandedMonths);
    }

    function toggleWeek(key: string) {
        if (expandedWeeks.has(key)) {
            expandedWeeks.delete(key);
            // Revert to month context if week closed
            const month = hierarchicalTradesData.find((m) =>
                m.weeks.some((w: any) => w.key === key),
            );
            if (month && expandedMonths.has(month.key)) {
                activeContext = {
                    type: "month",
                    key: month.key,
                    label: month.label,
                };
            } else {
                activeContext = {
                    type: "global",
                    key: null,
                    label: "Todo o Período",
                };
            }
        } else {
            expandedWeeks.add(key);
            const week = hierarchicalTradesData
                .flatMap((m) => m.weeks)
                .find((w) => w.key === key);
            if (week) {
                activeContext = {
                    type: "week",
                    key: key,
                    label: week.label,
                };
            }
        }
        expandedWeeks = new Set(expandedWeeks);
    }

    function getDayResult(trades: any[]) {
        if (currencyMode === "main") {
            return trades.reduce(
                (sum, t) =>
                    sum +
                    tradesStore.getConvertedTradeResult(
                        t,
                        settingsStore.accounts,
                        settingsStore.currencies,
                    ),
                0,
            );
        }
        const summary: Record<string, number> = {};
        trades.forEach((t) => {
            const acc = settingsStore.accounts.find(
                (a) => a.id === t.account_id,
            );
            const curr = acc?.currency || "BRL";
            summary[curr] = (summary[curr] || 0) + (t.result || 0);
        });
        return summary;
    }

    const kpis = $derived.by(() => {
        const total = filteredTrades.length;
        const mainCurrency = settingsStore.userProfile?.main_currency || "BRL";

        let profitTotal = 0;
        let winners = 0;
        let winnersProfit = 0;
        let totalLoss = 0;
        let consolidatedTotal = 0;
        const pnlByCurrency: Record<string, number> = {};

        filteredTrades.forEach((t) => {
            const acc = settingsStore.accounts.find(
                (a) => a.id === t.account_id,
            );
            const curr = acc?.currency || "BRL";
            const rawResult = t.result || 0;

            // PnL by currency
            pnlByCurrency[curr] = (pnlByCurrency[curr] || 0) + rawResult;

            // Converted result for consolidated metrics
            const convertedRes = tradesStore.getConvertedTradeResult(
                t,
                settingsStore.accounts,
                settingsStore.currencies,
            );

            consolidatedTotal += convertedRes;

            const metricResult =
                currencyMode === "main" ? convertedRes : rawResult;

            profitTotal += metricResult;

            if (metricResult > 0) {
                winners++;
                winnersProfit += metricResult;
            } else if (metricResult < 0) {
                totalLoss += Math.abs(metricResult);
            }
        });

        const winRate = total > 0 ? (winners / total) * 100 : 0;
        const profitFactor =
            totalLoss === 0
                ? winnersProfit > 0
                    ? 99.99
                    : 0
                : winnersProfit / totalLoss;

        return {
            total,
            profitTotal,
            pnlByCurrency,
            mainCurrency,
            consolidatedTotal,
            winRate: winRate.toFixed(1),
            winners,
            openCount: allTrades.filter((t) => !t.exit_price).length,
            profitFactor: profitFactor.toFixed(2),
        };
    });

    function toggleDay(date: string) {
        expandedDays[date] = !expandedDays[date];
        if (expandedDays[date]) {
            const day = hierarchicalTradesData
                .flatMap((m) => m.weeks)
                .flatMap((w) => w.days)
                .find((d) => d.key === date);
            if (day) {
                activeContext = {
                    type: "day",
                    key: date,
                    label: day.label,
                };
            }
        }
    }

    const filteredTradesForChart = $derived.by(() => {
        let baseTrades = [];

        if (activeContext.type === "global") {
            baseTrades = filteredTrades;
        } else if (activeContext.type === "month") {
            const month = hierarchicalTradesData.find(
                (m) => m.key === activeContext.key,
            );
            baseTrades = month?.trades || [];
        } else if (activeContext.type === "week") {
            const week = hierarchicalTradesData
                .flatMap((m) => m.weeks)
                .find((w) => w.key === activeContext.key);
            baseTrades = week?.trades || [];
        } else if (activeContext.type === "day") {
            const day = hierarchicalTradesData
                .flatMap((m) => m.weeks)
                .flatMap((w) => w.days)
                .find((d) => d.key === activeContext.key);
            baseTrades = day?.trades || [];
        } else if (activeContext.type === "trade") {
            baseTrades = activeContext.data ? [activeContext.data] : [];
        } else {
            baseTrades = filteredTrades;
        }

        // CRITICAL: Sort chronologically for accurate Drawdown and Equity Curve calculation
        return [...baseTrades].sort(
            (a, b) =>
                new Date(a.exit_date || a.date).getTime() -
                new Date(b.exit_date || b.date).getTime(),
        );
    });

    const activeContextStats = $derived.by(() => {
        const trades = filteredTradesForChart;
        const total = trades.length;

        if (total === 0) {
            return {
                total: 0,
                pnlEntries: [],
                winRate: "0.0",
                profitFactor: "0.00",
                avgGain: 0,
                avgLoss: 0,
                riskReward: "0.00",
                maxDrawdown: 0,
            };
        }

        let winners = 0;
        let losers = 0;
        let grossProfit = 0;
        let grossLoss = 0;
        let cumulativePnl = 0;
        let maxPeak = 0;
        let maxDd = 0;
        const pnlByCurrency: Record<string, number> = {};

        trades.forEach((t: any) => {
            const acc = settingsStore.accounts.find(
                (a) => a.id === t.account_id,
            );
            const curr = acc?.currency || "BRL";
            const res = t.result || 0;

            pnlByCurrency[curr] = (pnlByCurrency[curr] || 0) + res;

            // Use converted result for cross-currency calculations
            const convertedRes = tradesStore.getConvertedTradeResult(
                t,
                settingsStore.accounts,
                settingsStore.currencies,
            );

            if (convertedRes > 0) {
                winners++;
                grossProfit += convertedRes;
            } else if (convertedRes < 0) {
                losers++;
                grossLoss += Math.abs(convertedRes);
            }

            // Drawdown calculation
            cumulativePnl += convertedRes;
            if (cumulativePnl > maxPeak) maxPeak = cumulativePnl;
            const dd = maxPeak - cumulativePnl;
            if (dd > maxDd) maxDd = dd;
        });

        const pnlEntries = Object.entries(pnlByCurrency).map(([curr, val]) => ({
            curr,
            val: val as number,
        }));

        const winRate = (winners / total) * 100;
        const profitFactor =
            grossLoss === 0
                ? grossProfit > 0
                    ? 99.99
                    : 0
                : grossProfit / grossLoss;
        const avgGain = winners > 0 ? grossProfit / winners : 0;
        const avgLoss = losers > 0 ? grossLoss / losers : 0;
        const rr =
            avgLoss === 0 ? (avgGain > 0 ? 99.99 : 0) : avgGain / avgLoss;

        return {
            total,
            pnlEntries,
            winRate: winRate.toFixed(1),
            profitFactor: profitFactor.toFixed(2),
            avgGain,
            avgLoss,
            riskReward: rr.toFixed(2),
            maxDrawdown: maxDd,
            mainCurrency: settingsStore.userProfile?.main_currency || "BRL",
        };
    });

    function handleSelectTrade(trade: any) {
        activeContext = {
            type: "trade",
            key: trade.id,
            label: `${trade.asset_symbol} (${format(new Date(trade.date), "dd/MM")})`,
            data: trade,
        };
    }

    function handleEdit(trade: any) {
        selectedTrade = trade;
        isEditOpen = true;
    }

    function handleView(trade: any) {
        selectedTrade = trade;
        isViewOpen = true;
    }

    function requestDelete(id: string) {
        tradeToDelete = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (tradeToDelete) {
            const result = await tradesStore.removeTrade(tradeToDelete);
            if (result.success) {
                toast.success(
                    $t("trades.messages.delete_success") ||
                        "Operação excluída com sucesso.",
                );
            } else {
                toast.error(result.error || "Erro ao excluir operação.");
            }
            tradeToDelete = null;
        }
        isDeleteOpen = false;
    }

    function clearFilters() {
        filterStatus = "all";
        filterAccount = "all";
        filterStrategy = "all";
        filterAssetType = "all";
        filterCurrency = "all";
        searchQuery = "";
    }
</script>

<div class="flex flex-col gap-6 p-4 md:p-8">
    <div
        class="flex flex-col md:flex-row md:items-center justify-between gap-4"
    >
        <div class="mb-8">
            <h1 class="text-3xl font-black tracking-tight">
                {$t("trades.title")}
            </h1>
            <p class="text-muted-foreground">
                {$t("trades.subtitle")}
            </p>
        </div>
        <div class="flex gap-2">
            <Button
                variant={showFilters ? "secondary" : "outline"}
                size="sm"
                class="h-9"
                onclick={() => (showFilters = !showFilters)}
            >
                <Filter class="w-4 h-4 mr-2" />
                {$t("trades.filters.title") || "Filtrar"}
                {#if filterStatus !== "all" || filterAccount !== "all" || filterStrategy !== "all" || filterAssetType !== "all"}
                    <Badge
                        class="ml-2 h-4 w-4 p-0 flex items-center justify-center bg-primary text-primary-foreground"
                        >!</Badge
                    >
                {/if}
            </Button>

            <Button
                size="sm"
                class="h-9"
                variant="outline"
                onclick={() => {
                    selectedTrade = null;
                    isEditOpen = true;
                }}
            >
                <Plus class="w-4 h-4 mr-2" />
                {$t("trades.actions.new_trade") || "Novo Trade"}
            </Button>
            <Button size="sm" class="h-9">
                <ArrowRightLeft class="w-4 h-4 mr-2" />
                {$t("trades.actions.import")}
            </Button>
        </div>
    </div>

    <!-- KPI Row -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-2">
        <Card.Root
            class="border-l-4 {kpis.profitTotal >= 0
                ? 'border-l-emerald-500'
                : 'border-l-rose-500'} shadow-sm bg-card"
        >
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium">Saldo Total</Card.Title>
                <Coins
                    class="w-4 h-4 {kpis.profitTotal >= 0
                        ? 'text-emerald-500'
                        : 'text-rose-500'}"
                />
            </Card.Header>
            <Card.Content>
                <div class="flex items-center justify-between">
                    <div>
                        {#each Object.entries(kpis.pnlByCurrency) as [curr, val]}
                            <div class="flex items-baseline gap-1.5">
                                <span
                                    class="text-[10px] font-medium text-muted-foreground uppercase"
                                    >{curr}</span
                                >
                                <span
                                    class="text-sm font-bold {(val as number) >=
                                    0
                                        ? 'text-emerald-600'
                                        : 'text-rose-500'}"
                                >
                                    {formatNumber(val as number)}
                                </span>
                            </div>
                        {/each}
                    </div>
                    {#if Object.keys(kpis.pnlByCurrency).length > 1}
                        <div class="text-right border-l border-border pl-3">
                            <span
                                class="text-sm font-bold {kpis.consolidatedTotal >=
                                0
                                    ? 'text-emerald-600'
                                    : 'text-rose-500'}"
                            >
                                {formatCurrency(
                                    kpis.consolidatedTotal,
                                    kpis.mainCurrency,
                                )}
                            </span>
                            <p class="text-[10px] text-muted-foreground">
                                Convertido em {kpis.mainCurrency}
                            </p>
                        </div>
                    {/if}
                </div>
            </Card.Content>
        </Card.Root>

        <Card.Root class="border-l-4 border-l-blue-500 shadow-sm bg-card">
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium"
                    >Taxa de Acerto</Card.Title
                >
                <Activity class="w-4 h-4 text-blue-500" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">
                    {kpis.winRate}%
                </div>
                <p class="text-xs text-muted-foreground mt-1">
                    {kpis.winners} de {kpis.total} Trades
                </p>
            </Card.Content>
        </Card.Root>

        <Card.Root class="border-l-4 border-l-teal-500 shadow-sm bg-card">
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium"
                    >Profit Factor</Card.Title
                >
                <TrendingUp class="w-4 h-4 text-teal-500" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">
                    {kpis.profitFactor}
                </div>
                <p class="text-xs text-muted-foreground mt-1">
                    Fator de Lucratividade
                </p>
            </Card.Content>
        </Card.Root>

        <Card.Root class="border-l-4 border-l-amber-500 shadow-sm bg-card">
            <Card.Header
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <Card.Title class="text-sm font-medium">Em Aberto</Card.Title>
                <Clock class="w-4 h-4 text-amber-500" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">
                    {kpis.openCount}
                </div>
                <p class="text-xs text-muted-foreground mt-1">
                    Posições Atuais
                </p>
            </Card.Content>
        </Card.Root>
    </div>

    {#if showFilters}
        <Card.Root
            class="bg-card/30 backdrop-blur-md border-primary/10 shadow-xl overflow-visible"
        >
            <Card.Content class="p-4 flex flex-wrap gap-4 items-end">
                <div class="space-y-1 w-[150px]">
                    <label
                        for="filter-status"
                        class="text-[10px] font-bold uppercase text-muted-foreground"
                        >{$t("trades.filters.status")}</label
                    >
                    <Select.Root type="single" bind:value={filterStatus}>
                        <Select.Trigger
                            id="filter-status"
                            class="bg-background/50 h-9"
                        >
                            {#if filterStatus === "all"}
                                {$t("general.all") || "Todos"}
                            {:else if filterStatus === "open"}
                                {$t("trades.table.status_open") || "Aberto"}
                            {:else if filterStatus === "closed"}
                                {$t("trades.table.status_closed") || "Fechado"}
                            {:else}
                                {filterStatus}
                            {/if}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="all"
                                >{$t("general.all") || "Todos"}</Select.Item
                            >
                            <Select.Item value="open"
                                >{$t("trades.table.status_open") ||
                                    "Aberto"}</Select.Item
                            >
                            <Select.Item value="closed"
                                >{$t("trades.table.status_closed") ||
                                    "Fechado"}</Select.Item
                            >
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="space-y-1 w-[150px]">
                    <label
                        for="filter-account"
                        class="text-[10px] font-bold uppercase text-muted-foreground"
                        >{$t("trades.filters.account")}</label
                    >
                    <Select.Root type="single" bind:value={filterAccount}>
                        <Select.Trigger
                            id="filter-account"
                            class="bg-background/50 h-9"
                        >
                            {settingsStore.accounts.find(
                                (a) => a.id === filterAccount,
                            )?.nickname ||
                                $t("general.all") ||
                                "Todas"}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="all"
                                >{$t("general.all") || "Todas"}</Select.Item
                            >
                            {#each settingsStore.accounts as acc}
                                <Select.Item value={acc.id}
                                    >{acc.nickname}</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="space-y-1 w-[150px]">
                    <label
                        for="filter-strategy"
                        class="text-[10px] font-bold uppercase text-muted-foreground"
                        >{$t("trades.filters.strategy")}</label
                    >
                    <Select.Root type="single" bind:value={filterStrategy}>
                        <Select.Trigger
                            id="filter-strategy"
                            class="bg-background/50 h-9"
                        >
                            {settingsStore.strategies.find(
                                (s) => s.id === filterStrategy,
                            )?.name ||
                                $t("general.all") ||
                                "Todas"}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="all"
                                >{$t("general.all") || "Todas"}</Select.Item
                            >
                            {#each settingsStore.strategies as strat}
                                <Select.Item value={strat.id}
                                    >{strat.name}</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="space-y-1 w-[150px]">
                    <label
                        for="filter-asset-type"
                        class="text-[10px] font-bold uppercase text-muted-foreground"
                        >{$t("trades.filters.assetType")}</label
                    >
                    <Select.Root type="single" bind:value={filterAssetType}>
                        <Select.Trigger
                            id="filter-asset-type"
                            class="bg-background/50 h-9"
                        >
                            {settingsStore.assetTypes.find(
                                (at) => at.id === filterAssetType,
                            )?.name ||
                                $t("general.all") ||
                                "Todos"}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="all"
                                >{$t("general.all") || "Todos"}</Select.Item
                            >
                            {#each settingsStore.assetTypes as type}
                                <Select.Item value={type.id}
                                    >{type.name}</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="space-y-1 w-[150px]">
                    <label
                        for="filter-currency"
                        class="text-[10px] font-bold uppercase text-muted-foreground"
                    >
                        {$t("trades.filters.currency") || "Moeda"}
                    </label>
                    <Select.Root type="single" bind:value={filterCurrency}>
                        <Select.Trigger
                            id="filter-currency"
                            class="bg-background/50 h-9"
                        >
                            {filterCurrency === "all"
                                ? $t("general.all") || "Todas"
                                : filterCurrency}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="all"
                                >{$t("general.all") || "Todas"}</Select.Item
                            >
                            {#each [...new Set(settingsStore.accounts.map((a) => a.currency))] as curr}
                                <Select.Item value={curr}>{curr}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="flex-1 flex justify-end gap-2">
                    <Button
                        variant="ghost"
                        size="sm"
                        class="text-xs h-8"
                        onclick={clearFilters}
                    >
                        <X class="w-3 h-3 mr-1" />
                        {$t("trades.actions.clear_filters")}
                    </Button>
                </div>
            </Card.Content>
        </Card.Root>
    {/if}

    <!-- Main Split Content -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
        <!-- Left: Month/Week/Day Trade Hierarchy (MAIN LIST) -->
        <div class="lg:col-span-9 space-y-4">
            {#if hierarchicalTradesData.length === 0}
                <div
                    class="mt-8 flex flex-1 flex-col items-center justify-center rounded-2xl border-2 border-dashed border-muted-foreground/20 p-12 text-center bg-accent/5"
                >
                    <div
                        class="p-6 rounded-full bg-zinc-950 border border-primary/10 mb-4 shadow-xl"
                    >
                        <BookOpen class="w-12 h-12 text-zinc-500/50" />
                    </div>
                    <h3 class="text-xl font-bold tracking-tight">
                        {$t("trades.empty.title")}
                    </h3>
                    <p
                        class="text-sm text-muted-foreground max-w-xs mx-auto mt-2"
                    >
                        {$t("trades.empty.description")}
                    </p>
                    <Button
                        class="mt-8 h-11 px-8 rounded-full shadow-lg shadow-primary/20 transition-all hover:scale-105 active:scale-95"
                        onclick={() => {
                            selectedTrade = null;
                            isEditOpen = true;
                        }}
                    >
                        {$t("trades.actions.new_trade")}
                    </Button>
                </div>
            {:else}
                <div class="space-y-4">
                    {#each hierarchicalTradesData as month (month.key)}
                        {@const isMonthExpanded = expandedMonths.has(month.key)}
                        <div
                            class="rounded-xl border border-zinc-800/50 overflow-hidden bg-zinc-900/20"
                        >
                            <!-- Month Header -->
                            <button
                                class="w-full flex items-center justify-between p-4 hover:bg-white/5 transition-colors border-none bg-transparent cursor-pointer {activeContext.key ===
                                month.key
                                    ? 'bg-primary/5 ring-1 ring-inset ring-primary/20'
                                    : ''}"
                                onclick={() => toggleMonth(month.key)}
                            >
                                <div class="flex items-center gap-4">
                                    <div
                                        class="p-2.5 rounded-xl bg-primary/10 border border-primary/20"
                                    >
                                        <CalendarIcon
                                            class="w-4 h-4 text-primary"
                                        />
                                    </div>
                                    <div class="text-left">
                                        <h4
                                            class="text-sm font-black text-white uppercase tracking-tight"
                                        >
                                            {month.label}
                                        </h4>
                                        <div
                                            class="flex items-center gap-2 mt-0.5"
                                        >
                                            <Badge
                                                variant="outline"
                                                class="text-[9px] px-1.5 h-4 bg-zinc-800 border-zinc-700 font-bold uppercase"
                                            >
                                                {month.trades.length} TRADES
                                            </Badge>
                                            <div class="flex gap-2">
                                                {#each month.pnlEntries as entry}
                                                    <div
                                                        class="flex items-baseline gap-0.5"
                                                    >
                                                        <span
                                                            class="text-[8px] text-muted-foreground uppercase"
                                                            >{entry.curr}</span
                                                        >
                                                        <span
                                                            class="text-[9px] font-bold {entry.val >=
                                                            0
                                                                ? 'text-emerald-500'
                                                                : 'text-red-500'}"
                                                            >{formatNumber(
                                                                entry.val,
                                                            )}</span
                                                        >
                                                    </div>
                                                {/each}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <ChevronDown
                                    class="w-4 h-4 text-zinc-500 transition-transform duration-300 {isMonthExpanded
                                        ? 'rotate-180'
                                        : ''}"
                                />
                            </button>

                            {#if isMonthExpanded}
                                <div
                                    class="px-4 pb-4 space-y-3 animate-in fade-in slide-in-from-top-2"
                                >
                                    <Separator class="bg-zinc-800/50 mb-3" />
                                    {#each month.weeks as week (week.key)}
                                        {@const isWeekExpanded =
                                            expandedWeeks.has(week.key)}
                                        <div
                                            class="rounded-lg border border-zinc-800/30 bg-zinc-950/30 overflow-hidden"
                                        >
                                            <!-- Week Header -->
                                            <button
                                                class="w-full flex items-center justify-between p-3 hover:bg-white/5 transition-colors border-none bg-transparent cursor-pointer {activeContext.key ===
                                                week.key
                                                    ? 'bg-primary/5 ring-1 ring-inset ring-primary/20'
                                                    : ''}"
                                                onclick={() =>
                                                    toggleWeek(week.key)}
                                            >
                                                <div
                                                    class="flex items-center gap-3"
                                                >
                                                    <div
                                                        class="p-1.5 rounded-lg bg-zinc-800 border border-zinc-700"
                                                    >
                                                        <Activity
                                                            class="w-3.5 h-3.5 text-zinc-400"
                                                        />
                                                    </div>
                                                    <div class="text-left">
                                                        <span
                                                            class="text-[10px] font-black text-zinc-200 uppercase tracking-wide"
                                                        >
                                                            {week.label}
                                                        </span>
                                                        <div
                                                            class="flex items-center gap-2"
                                                        >
                                                            <span
                                                                class="text-[8px] text-zinc-500 font-bold uppercase"
                                                            >
                                                                {week.trades
                                                                    .length} Operações
                                                            </span>
                                                            <div
                                                                class="flex gap-2"
                                                            >
                                                                {#each week.pnlEntries as entry}
                                                                    <div
                                                                        class="flex items-baseline gap-0.5"
                                                                    >
                                                                        <span
                                                                            class="text-[7px] text-muted-foreground uppercase"
                                                                            >{entry.curr}</span
                                                                        >
                                                                        <span
                                                                            class="text-[8px] font-bold {entry.val >=
                                                                            0
                                                                                ? 'text-emerald-500'
                                                                                : 'text-rose-500'}"
                                                                            >{formatNumber(
                                                                                entry.val,
                                                                            )}</span
                                                                        >
                                                                    </div>
                                                                {/each}
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                                <ChevronDown
                                                    class="w-3 h-3 text-zinc-600 transition-transform duration-300 {isWeekExpanded
                                                        ? 'rotate-180'
                                                        : ''}"
                                                />
                                            </button>

                                            {#if isWeekExpanded}
                                                <div
                                                    class="p-2 space-y-2 animate-in fade-in slide-in-from-top-1"
                                                >
                                                    {#each week.days as day (day.key)}
                                                        {@const isExpanded =
                                                            expandedDays[
                                                                day.date
                                                            ]}
                                                        <div
                                                            class="group rounded-lg border border-white/5 bg-zinc-900/40 overflow-hidden"
                                                        >
                                                            <button
                                                                class="w-full flex items-center justify-between p-2 hover:bg-white/5 transition-colors border-none bg-transparent cursor-pointer {activeContext.key ===
                                                                day.date
                                                                    ? 'bg-primary/5 ring-1 ring-inset ring-primary/20'
                                                                    : ''}"
                                                                onclick={() =>
                                                                    toggleDay(
                                                                        day.date,
                                                                    )}
                                                            >
                                                                <div
                                                                    class="flex items-center gap-3"
                                                                >
                                                                    <div
                                                                        class="flex flex-col items-center justify-center p-0.5 rounded-md bg-zinc-950 border border-white/5 min-w-[32px] h-8"
                                                                    >
                                                                        <span
                                                                            class="text-[7px] uppercase font-bold text-zinc-500 leading-none mb-0.5"
                                                                        >
                                                                            {new Date(
                                                                                day.date +
                                                                                    "T12:00:00",
                                                                            )
                                                                                .toLocaleDateString(
                                                                                    $locale ||
                                                                                        "pt-BR",
                                                                                    {
                                                                                        month: "short",
                                                                                    },
                                                                                )
                                                                                .replace(
                                                                                    ".",
                                                                                    "",
                                                                                )}
                                                                        </span>
                                                                        <span
                                                                            class="text-xs font-black text-zinc-200 leading-none"
                                                                        >
                                                                            {new Date(
                                                                                day.date +
                                                                                    "T12:00:00",
                                                                            ).getDate()}
                                                                        </span>
                                                                    </div>
                                                                    <div
                                                                        class="text-left"
                                                                    >
                                                                        <span
                                                                            class="text-[9px] font-bold text-zinc-400 uppercase tracking-tight"
                                                                        >
                                                                            {day.label}
                                                                        </span>
                                                                        <div
                                                                            class="flex items-center gap-2"
                                                                        >
                                                                            <span
                                                                                class="text-[8px] text-zinc-500 font-medium"
                                                                                >({day
                                                                                    .trades
                                                                                    .length}
                                                                                trades)</span
                                                                            >
                                                                            {#each day.pnlEntries as entry}
                                                                                <div
                                                                                    class="flex items-baseline gap-0.5"
                                                                                >
                                                                                    <span
                                                                                        class="text-[7px] text-muted-foreground uppercase"
                                                                                        >{entry.curr}</span
                                                                                    >
                                                                                    <span
                                                                                        class="text-[8px] font-bold {entry.val >=
                                                                                        0
                                                                                            ? 'text-emerald-500'
                                                                                            : 'text-rose-500'}"
                                                                                        >{formatNumber(
                                                                                            entry.val,
                                                                                        )}</span
                                                                                    >
                                                                                </div>
                                                                            {/each}
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                                <ChevronRight
                                                                    class="w-3 h-3 text-zinc-700 transition-transform duration-300 {isExpanded
                                                                        ? 'rotate-90'
                                                                        : ''}"
                                                                />
                                                            </button>

                                                            {#if isExpanded}
                                                                <div
                                                                    class="p-2 pt-0 animate-in zoom-in-95 duration-200"
                                                                >
                                                                    <Table.Root>
                                                                        <Table.Header
                                                                        >
                                                                            <Table.Row
                                                                                class="hover:bg-transparent border-white/5"
                                                                            >
                                                                                <Table.Head
                                                                                    class="h-8 text-[9px] font-black uppercase text-zinc-500"
                                                                                    >{$t(
                                                                                        "trades.table.asset",
                                                                                    )}</Table.Head
                                                                                >
                                                                                <Table.Head
                                                                                    class="h-8 text-[9px] font-black uppercase text-zinc-500"
                                                                                    >{$t(
                                                                                        "trades.table.direction",
                                                                                    )}</Table.Head
                                                                                >
                                                                                <Table.Head
                                                                                    class="h-8 text-[9px] font-black uppercase text-zinc-500"
                                                                                    >{$t(
                                                                                        "trades.table.entry",
                                                                                    )}</Table.Head
                                                                                >
                                                                                <Table.Head
                                                                                    class="h-8 text-[9px] font-black uppercase text-zinc-500"
                                                                                    >{$t(
                                                                                        "trades.table.exit",
                                                                                    )}</Table.Head
                                                                                >
                                                                                <Table.Head
                                                                                    class="h-8 text-[9px] font-black uppercase text-zinc-500 text-right"
                                                                                    >{$t(
                                                                                        "trades.table.pl",
                                                                                    )}</Table.Head
                                                                                >
                                                                                <Table.Head
                                                                                    class="h-8 text-[9px] font-black uppercase text-zinc-500 text-right"
                                                                                    >{$t(
                                                                                        "trades.table.actions",
                                                                                    )}</Table.Head
                                                                                >
                                                                            </Table.Row>
                                                                        </Table.Header>
                                                                        <Table.Body
                                                                        >
                                                                            {#each day.trades as trade (trade.id)}
                                                                                <Table.Row
                                                                                    class="group/row hover:bg-white/5 border-white/5 cursor-pointer {activeContext.key ===
                                                                                    trade.id
                                                                                        ? 'bg-primary/5 border-l-2 border-l-primary'
                                                                                        : ''}"
                                                                                    onclick={() =>
                                                                                        handleSelectTrade(
                                                                                            trade,
                                                                                        )}
                                                                                >
                                                                                    <Table.Cell
                                                                                        class="py-2"
                                                                                    >
                                                                                        <div
                                                                                            class="flex flex-col"
                                                                                        >
                                                                                            <span
                                                                                                class="font-bold text-xs"
                                                                                                >{trade.asset_symbol}</span
                                                                                            >
                                                                                            <span
                                                                                                class="text-[8px] text-zinc-500 uppercase tracking-tighter"
                                                                                            >
                                                                                                {settingsStore.strategies.find(
                                                                                                    (
                                                                                                        s,
                                                                                                    ) =>
                                                                                                        s.id ===
                                                                                                        trade.strategy_id,
                                                                                                )
                                                                                                    ?.name ||
                                                                                                    "-"}
                                                                                            </span>
                                                                                        </div>
                                                                                    </Table.Cell>
                                                                                    <Table.Cell
                                                                                        class="py-2"
                                                                                    >
                                                                                        <Badge
                                                                                            variant="outline"
                                                                                            class="text-[8px] font-black px-1.5 h-4 border-none {trade.direction ===
                                                                                            'Buy'
                                                                                                ? 'bg-emerald-500/10 text-emerald-500'
                                                                                                : 'bg-rose-500/10 text-rose-500'}"
                                                                                        >
                                                                                            {trade.direction ===
                                                                                            "Buy"
                                                                                                ? $t(
                                                                                                      "trades.table.buy",
                                                                                                  )
                                                                                                : $t(
                                                                                                      "trades.table.sell",
                                                                                                  )}
                                                                                        </Badge>
                                                                                    </Table.Cell>
                                                                                    <Table.Cell
                                                                                        class="py-2"
                                                                                    >
                                                                                        <div
                                                                                            class="flex flex-col"
                                                                                        >
                                                                                            <span
                                                                                                class="text-xs font-mono"
                                                                                                >{trade.entry_price}</span
                                                                                            >
                                                                                            <span
                                                                                                class="text-[8px] text-zinc-500"
                                                                                                >{format(
                                                                                                    new Date(
                                                                                                        trade.date,
                                                                                                    ),
                                                                                                    "HH:mm",
                                                                                                )}</span
                                                                                            >
                                                                                        </div>
                                                                                    </Table.Cell>
                                                                                    <Table.Cell
                                                                                        class="py-2"
                                                                                    >
                                                                                        <div
                                                                                            class="flex flex-col"
                                                                                        >
                                                                                            <span
                                                                                                class="text-xs font-mono"
                                                                                                >{trade.exit_price ||
                                                                                                    "-"}</span
                                                                                            >
                                                                                            <span
                                                                                                class="text-[8px] text-zinc-500"
                                                                                                >{trade.exit_date
                                                                                                    ? format(
                                                                                                          new Date(
                                                                                                              trade.exit_date,
                                                                                                          ),
                                                                                                          "HH:mm",
                                                                                                      )
                                                                                                    : "-"}</span
                                                                                            >
                                                                                        </div>
                                                                                    </Table.Cell>
                                                                                    <Table.Cell
                                                                                        class="py-2 text-right"
                                                                                    >
                                                                                        <span
                                                                                            class="text-xs font-black font-mono {trade.result >=
                                                                                            0
                                                                                                ? 'text-emerald-500'
                                                                                                : 'text-rose-500'}"
                                                                                        >
                                                                                            {formatCurrency(
                                                                                                trade.result,
                                                                                                settingsStore.accounts.find(
                                                                                                    (
                                                                                                        a,
                                                                                                    ) =>
                                                                                                        a.id ===
                                                                                                        trade.account_id,
                                                                                                )
                                                                                                    ?.currency ||
                                                                                                    "BRL",
                                                                                            )}
                                                                                        </span>
                                                                                    </Table.Cell>
                                                                                    <Table.Cell
                                                                                        class="py-2 text-right"
                                                                                    >
                                                                                        <div
                                                                                            class="flex justify-end gap-1 opacity-0 group-hover/row:opacity-100 transition-opacity"
                                                                                        >
                                                                                            <Button
                                                                                                variant="ghost"
                                                                                                size="icon"
                                                                                                class="h-6 w-6"
                                                                                                onclick={() =>
                                                                                                    handleView(
                                                                                                        trade,
                                                                                                    )}
                                                                                            >
                                                                                                <Eye
                                                                                                    class="h-3 w-3"
                                                                                                />
                                                                                            </Button>
                                                                                            <Button
                                                                                                variant="ghost"
                                                                                                size="icon"
                                                                                                class="h-6 w-6"
                                                                                                onclick={() =>
                                                                                                    handleEdit(
                                                                                                        trade,
                                                                                                    )}
                                                                                            >
                                                                                                <Pencil
                                                                                                    class="h-3 w-3"
                                                                                                />
                                                                                            </Button>
                                                                                            <Button
                                                                                                variant="ghost"
                                                                                                size="icon"
                                                                                                class="h-6 w-6 text-rose-500"
                                                                                                onclick={() =>
                                                                                                    requestDelete(
                                                                                                        trade.id,
                                                                                                    )}
                                                                                            >
                                                                                                <Trash2
                                                                                                    class="h-3 w-3"
                                                                                                />
                                                                                            </Button>
                                                                                        </div>
                                                                                    </Table.Cell>
                                                                                </Table.Row>
                                                                            {/each}
                                                                        </Table.Body>
                                                                    </Table.Root>
                                                                </div>
                                                            {/if}
                                                        </div>
                                                    {/each}
                                                </div>
                                            {/if}
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/each}
                </div>
            {/if}
        </div>

        <!-- Right Sidebar: Interactive Metrics & Charts -->
        <div class="lg:col-span-3 space-y-4 self-start sticky top-4">
            <div class="flex items-center justify-between px-2 h-9">
                <h3
                    class="text-sm font-bold uppercase tracking-widest text-muted-foreground/60 flex items-center gap-2"
                >
                    <Activity class="w-4 h-4" />
                    Resumo Lateral
                </h3>
                <div class="flex items-center gap-2">
                    <Badge
                        variant="outline"
                        class="text-[10px] font-black bg-white/5 border-white/10 uppercase tracking-tighter"
                    >
                        {activeContext.label}
                    </Badge>
                    {#if activeContext.type !== "global"}
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-6 w-6 text-muted-foreground hover:text-rose-500 transition-colors"
                            onclick={() =>
                                (activeContext = {
                                    type: "global",
                                    key: null,
                                    label: "Todo o Período",
                                })}
                        >
                            <X class="w-3 h-3" />
                        </Button>
                    {/if}
                </div>
            </div>

            {#if activeContext.type === "trade" && activeContext.data}
                <!-- Card: Trade Essential Info (Visible only when trade selected) -->
                <Card.Root
                    class="shadow-sm bg-card/80 border-white/5 overflow-hidden"
                >
                    <Card.Header class="pb-2">
                        <div class="flex items-center justify-between">
                            <Badge
                                variant="secondary"
                                class="bg-primary/10 text-primary border-none font-black"
                            >
                                {activeContext.data.asset_symbol}
                            </Badge>
                            <span
                                class="text-[10px] font-bold text-muted-foreground uppercase opacity-50"
                            >
                                {format(
                                    new Date(activeContext.data.date),
                                    "dd MMMM yyyy HH:mm",
                                    {
                                        locale:
                                            $locale === "en-US"
                                                ? undefined
                                                : undefined,
                                    },
                                )}
                            </span>
                        </div>
                    </Card.Header>
                    <Card.Content class="grid grid-cols-2 gap-4">
                        <div class="space-y-1">
                            <span
                                class="text-[9px] font-bold text-muted-foreground uppercase"
                                >Resultado</span
                            >
                            <div
                                class="text-xl font-black {activeContext.data
                                    .result >= 0
                                    ? 'text-emerald-500'
                                    : 'text-rose-500'}"
                            >
                                {formatCurrency(
                                    activeContext.data.result,
                                    settingsStore.accounts.find(
                                        (a) =>
                                            a.id ===
                                            activeContext.data.account_id,
                                    )?.currency || "BRL",
                                )}
                            </div>
                        </div>
                        <div class="space-y-1 text-right">
                            <span
                                class="text-[9px] font-bold text-muted-foreground uppercase"
                                >Tipo</span
                            >
                            <div
                                class="text-xs font-bold uppercase tracking-widest"
                            >
                                {activeContext.data.type || "N/A"}
                            </div>
                        </div>
                    </Card.Content>
                </Card.Root>
            {/if}

            <div
                class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2 gap-4"
            >
                <!-- Card: KPIs Dashboard -->
                <Card.Root
                    class="shadow-sm bg-card/80 border-white/5 flex flex-col justify-between"
                >
                    <Card.Header
                        class="p-4 pb-2 border-b border-white/5 flex flex-row items-center justify-between space-y-0"
                    >
                        <span
                            class="text-[10px] font-black uppercase tracking-tighter text-muted-foreground"
                            >Desempenho</span
                        >
                        <TrendingUp
                            class="w-3 h-3 text-emerald-500 opacity-50"
                        />
                    </Card.Header>
                    <Card.Content class="p-4 pt-4">
                        <div class="space-y-6">
                            <div class="grid grid-cols-2 gap-4">
                                <div class="space-y-0.5">
                                    <span
                                        class="text-[8px] font-black text-muted-foreground uppercase opacity-50"
                                        >Profit Factor</span
                                    >
                                    <div
                                        class="text-sm font-black tracking-tighter text-teal-400"
                                    >
                                        {activeContextStats.profitFactor}
                                    </div>
                                </div>
                                <div class="space-y-0.5 text-right">
                                    <span
                                        class="text-[8px] font-black text-rose-500 uppercase opacity-80"
                                        >Drawdown</span
                                    >
                                    <div
                                        class="text-[11px] font-black text-rose-500 tracking-tighter"
                                    >
                                        {formatCurrency(
                                            activeContextStats.maxDrawdown,
                                            activeContextStats.mainCurrency,
                                        ).replace(/[^\d.,+-]/g, "")}
                                    </div>
                                </div>
                            </div>

                            <Separator class="bg-white/5" />

                            <div class="grid grid-cols-2 gap-4">
                                <div class="space-y-0.5">
                                    <span
                                        class="text-[8px] font-black text-muted-foreground uppercase opacity-50"
                                        >Win Rate</span
                                    >
                                    <div
                                        class="text-sm font-black tracking-tighter"
                                    >
                                        {activeContextStats.winRate}%
                                    </div>
                                </div>
                                <div class="space-y-0.5 text-right">
                                    <span
                                        class="text-[8px] font-black text-muted-foreground uppercase opacity-50"
                                        >Risco:Retorno</span
                                    >
                                    <div
                                        class="text-[11px] font-black tracking-tighter {parseFloat(
                                            activeContextStats.riskReward,
                                        ) >= 1
                                            ? 'text-emerald-500'
                                            : ''}"
                                    >
                                        1:{activeContextStats.riskReward}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Card.Content>
                </Card.Root>

                <!-- Card: Outcome/Pie Chart -->
                <Card.Root
                    class="shadow-sm bg-card/80 border-white/5 overflow-hidden"
                >
                    <Card.Header
                        class="p-4 pb-0 border-b border-white/5 flex flex-row items-center justify-between space-y-0 bg-white/[0.02]"
                    >
                        <span
                            class="text-[10px] font-black uppercase tracking-tighter text-muted-foreground"
                            >Distribuição</span
                        >
                        <div class="text-[9px] font-bold opacity-30 uppercase">
                            {activeContextStats.total} Trades
                        </div>
                    </Card.Header>
                    <Card.Content class="p-2 pb-0">
                        <div class="h-[220px] w-full">
                            <TradeOutcomePieChart
                                trades={filteredTradesForChart}
                            />
                        </div>
                    </Card.Content>
                </Card.Root>
            </div>

            <!-- Card: Equity Curve with Drawdown -->
            <Card.Root class="shadow-sm bg-card/80 border-white/5">
                <Card.Header
                    class="p-4 py-3 border-b border-white/5 flex flex-row items-center justify-between space-y-0"
                >
                    <span
                        class="text-[10px] font-black uppercase tracking-tighter text-muted-foreground"
                        >Curva de Capital + Drawdown</span
                    >
                </Card.Header>
                <Card.Content class="p-2">
                    <div class="h-[260px] w-full">
                        <TradeEquityChart trades={filteredTradesForChart} />
                    </div>
                </Card.Content>
            </Card.Root>
        </div>
    </div>
</div>

<DeleteConfirmationModal
    bind:open={isDeleteOpen}
    onConfirm={confirmDelete}
    onCancel={() => (isDeleteOpen = false)}
    title={$t("trades.delete.title")}
    description={$t("trades.delete.confirmation")}
/>

<!-- View Modal -->
<Dialog.Root bind:open={isViewOpen}>
    <Dialog.Content
        class="max-w-4xl glass border-white/10 text-white overflow-hidden p-0"
    >
        <Dialog.Header class="p-6 pb-2 text-white">
            <Dialog.Title>{$t("trades.details.title")}</Dialog.Title>
        </Dialog.Header>
        {#if selectedTrade}
            <div class="px-6 pb-6 mt-2">
                <TradeDetailView trade={selectedTrade} />
            </div>
        {/if}
    </Dialog.Content>
</Dialog.Root>

<!-- Wizard Modal (New/Edit) -->
<Dialog.Root bind:open={isEditOpen}>
    <Dialog.Content
        class="max-w-4xl max-h-[95vh] h-[850px] overflow-hidden p-0 flex flex-col border-white/10 glass text-white"
    >
        {#key selectedTrade?.id}
            <NewTradeWizard
                trade={selectedTrade}
                close={() => (isEditOpen = false)}
                onsave={() => {
                    isEditOpen = false;
                    toast.success(
                        selectedTrade
                            ? "Operação atualizada!"
                            : "Operação criada!",
                    );
                }}
            />
        {/key}
    </Dialog.Content>
</Dialog.Root>
