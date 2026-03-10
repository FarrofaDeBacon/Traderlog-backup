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
        BarChart2,
        Percent,
        Timer,
    } from "lucide-svelte";
    import { Badge } from "$lib/components/ui/badge";
    import HierarchicalList from "$lib/components/shared/HierarchicalList.svelte";
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
    import { calculateAverageTimeBetweenTrades, formatDuration } from "$lib/utils/gann";

    import { untrack } from "svelte";
    import { format } from "date-fns/format";

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
    }>({
        type: "global",
        key: null,
        label: $t("trades.messages.all_period") || "Todo o Período",
    });

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
        if (settingsStore.isLoadingData) return [];
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
                    label: $t("trades.dashboard.weekOf", {
                        values: {
                            date: new Date(
                                weekKey + "T12:00:00",
                            ).toLocaleDateString($locale || "pt-BR", {
                                day: "numeric",
                                month: "short",
                            }),
                        },
                    }).toUpperCase(),
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
        try {
            return Object.values(monthsMap)
                .sort((a, b) => (b.key || "").localeCompare(a.key || ""))
                .map((month) => {
                    const weeks = Object.values(month.weeks || {})
                        .sort((a: any, b: any) =>
                            (b.key || "").localeCompare(a.key || ""),
                        )
                        .map((week: any) => {
                            const days = Object.values(week.days || {})
                                .sort((a: any, b: any) =>
                                    (b.date || "").localeCompare(a.date || ""),
                                )
                                .map((day: any) => ({
                                    ...day,
                                    pnlEntries: Object.entries(
                                        day.totalPnlByCurrency || {},
                                    ).map(([curr, val]) => ({ curr, val })),
                                }));

                            return {
                                ...week,
                                days,
                                pnlEntries: Object.entries(
                                    week.totalPnlByCurrency || {},
                                ).map(([curr, val]) => ({ curr, val })),
                            };
                        });

                    return {
                        ...month,
                        weeks,
                        pnlEntries: Object.entries(
                            month.totalPnlByCurrency || {},
                        ).map(([curr, val]) => ({ curr, val })),
                    };
                });
        } catch (err) {
            console.error(
                "[TradesHub] Error processing results hierarchy:",
                err,
            );
            return [];
        }
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
            avgInterval: calculateAverageTimeBetweenTrades(filteredTrades),
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

    let deleteModalDescription = $state("");

    function requestDelete(trade: (typeof tradesStore.trades)[0]) {
        tradeToDelete = trade.id;

        // Check if this trade is part of any system-linked cash transaction (daily closure)
        const normalizedId =
            trade.id.split(":").pop()?.replace(/[⟨⟩`]/g, "") || trade.id;
        const linkedClosure = settingsStore.cashTransactions.find(
            (ct) =>
                ct.system_linked &&
                ct.trade_ids?.some(
                    (tid) =>
                        (tid.split(":").pop()?.replace(/[⟨⟩`]/g, "") || tid) ===
                        normalizedId,
                ),
        );

        if (linkedClosure) {
            deleteModalDescription =
                $t("trades.delete.confirmation_with_closure") ||
                "Esta operação faz parte de um Fechamento Diário. O extrato será recalculado automaticamente pós-exclusão. Tem certeza?";
        } else {
            deleteModalDescription =
                $t("trades.delete.confirmation") ||
                "Tem certeza que deseja excluir esta operação? Esta ação não pode ser desfeita.";
        }

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

                // Also trigger settings reload to ensure summary is perfectly synced
                settingsStore.loadData();
            } else {
                toast.error(
                    $t("trades.messages.delete_error") ||
                        "Erro ao excluir. Tente novamente.",
                );
            }
        }
        isDeleteOpen = false;
        tradeToDelete = null;
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

<div class="space-y-6 animate-in fade-in duration-500">
    <div class="flex-1 flex flex-col space-y-8 p-4 md:p-8">
        <div
            class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4"
        >
            <div class="space-y-1">
                <h2 class="text-3xl font-bold text-foreground tracking-tight">
                    {$t("trades.title")}
                </h2>
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
                <!-- Importar Trades button hidden for now -->
            </div>
        </div>

        <!-- KPI Row -->
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4 mb-2">
            <Card.Root
                class="card-glass border-l-2 {kpis.profitTotal >= 0
                    ? 'border-l-emerald-500'
                    : 'border-l-rose-500'} shadow-sm"
            >
                <Card.Content class="py-0.5 px-2">
                    <div class="flex items-center justify-between">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60 leading-none"
                        >
                            {$t("trades.quickStats.totalBalance")}
                        </span>
                        <Coins
                            class="w-3 h-3 {kpis.profitTotal >= 0
                                ? 'text-emerald-500'
                                : 'text-rose-500'}"
                        />
                    </div>
                    <div
                        class="mt-1 flex items-center justify-between gap-2 overflow-hidden"
                    >
                        <div class="flex flex-col">
                            {#each Object.entries(kpis.pnlByCurrency) as [curr, val]}
                                <div
                                    class="flex items-baseline gap-1 leading-none"
                                >
                                    <span
                                        class="text-[8px] font-black uppercase text-muted-foreground/40"
                                        >{curr}</span
                                    >
                                    <span
                                        class="text-sm font-mono font-bold tabular-nums tracking-tight {(val as number) >=
                                        0
                                            ? 'text-emerald-500'
                                            : 'text-rose-500'}"
                                    >
                                        {formatNumber(val as number)}
                                    </span>
                                </div>
                            {/each}
                        </div>
                        {#if Object.keys(kpis.pnlByCurrency).length > 1}
                            <div
                                class="text-right border-l border-border pl-2 leading-none shrink-0"
                            >
                                <span
                                    class="text-sm font-mono font-bold tabular-nums tracking-tight {kpis.consolidatedTotal >=
                                    0
                                        ? 'text-emerald-500'
                                        : 'text-rose-500'}"
                                >
                                    {formatCurrency(
                                        kpis.consolidatedTotal,
                                        kpis.mainCurrency,
                                        $locale || "pt-BR",
                                    )}
                                </span>
                            </div>
                        {/if}
                    </div>
                </Card.Content>
            </Card.Root>

            <Card.Root
                class="card-glass border-l-2 border-l-cyan-500 shadow-sm"
            >
                <Card.Content class="py-1 px-3">
                    <div class="flex items-center justify-between">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60 leading-none"
                        >
                            {$t("trades.quickStats.winRate")}
                        </span>
                        <Percent class="w-3 h-3 text-cyan-500" />
                    </div>
                    <div class="mt-1">
                        <div
                            class="text-sm font-mono font-bold text-foreground tabular-nums tracking-tight leading-none"
                        >
                            {kpis.winRate}%
                        </div>
                        <p
                            class="text-[9px] text-muted-foreground/50 leading-none mt-0.5"
                        >
                            {$t("trades.quickStats.winRateDesc", {
                                values: {
                                    winners: kpis.winners,
                                    total: kpis.total,
                                },
                            })}
                        </p>
                    </div>
                </Card.Content>
            </Card.Root>

            <Card.Root
                class="card-glass border-l-2 border-l-indigo-500 shadow-sm"
            >
                <Card.Content class="py-1 px-3">
                    <div class="flex items-center justify-between">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60 leading-none"
                        >
                            {$t("trades.quickStats.profitFactor")}
                        </span>
                        <TrendingUp class="w-3 h-3 text-indigo-500" />
                    </div>
                    <div class="mt-1">
                        <div
                            class="text-sm font-mono font-bold text-foreground tabular-nums tracking-tight leading-none"
                        >
                            {kpis.profitFactor}
                        </div>
                        <p
                            class="text-[9px] text-muted-foreground/50 leading-none mt-0.5"
                        >
                            {$t("trades.quickStats.profitFactorDesc")}
                        </p>
                    </div>
                </Card.Content>
            </Card.Root>

            <Card.Root
                class="card-glass border-l-2 border-l-amber-500 shadow-sm"
            >
                <Card.Content class="py-1 px-3">
                    <div class="flex items-center justify-between">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60 leading-none"
                        >
                            {$t("trades.quickStats.openTrades")}
                        </span>
                        <Clock class="w-3 h-3 text-amber-500" />
                    </div>
                    <div class="mt-1">
                        <div
                            class="text-sm font-mono font-bold text-foreground tabular-nums tracking-tight leading-none"
                        >
                            {kpis.openCount}
                        </div>
                        <p
                            class="text-[9px] text-muted-foreground/50 leading-none mt-0.5"
                        >
                            {$t("trades.quickStats.openTradesDesc")}
                        </p>
                    </div>
                </Card.Content>
            </Card.Root>

            <Card.Root
                class="card-glass border-l-2 border-l-fuchsia-500 shadow-sm"
            >
                <Card.Content class="py-1 px-3">
                    <div class="flex items-center justify-between">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60 leading-none"
                        >
                            {$t("trades.quickStats.avgInterval")}
                        </span>
                        <Timer class="w-3 h-3 text-fuchsia-500" />
                    </div>
                    <div class="mt-1">
                        <div
                            class="text-sm font-mono font-bold text-foreground tabular-nums tracking-tight leading-none"
                        >
                            {formatDuration(kpis.avgInterval)}
                        </div>
                        <p
                            class="text-[9px] text-muted-foreground/50 leading-none mt-0.5"
                        >
                            {$t("trades.quickStats.avgIntervalDesc") || "Intervalo médio entre operações consecutivas"}
                        </p>
                    </div>
                </Card.Content>
            </Card.Root>
        </div>

        {#if showFilters}
            <Card.Root class="card-glass shadow-xl overflow-visible">
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
                                    {$t("trades.table.status_closed") ||
                                        "Fechado"}
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
                                    <Select.Item value={curr}
                                        >{curr}</Select.Item
                                    >
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
                            class="p-6 rounded-full bg-muted border border-primary/10 mb-4 shadow-xl"
                        >
                            <BookOpen
                                class="w-12 h-12 text-muted-foreground/50"
                            />
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
                    <HierarchicalList
                        data={hierarchicalTradesData}
                        onMonthToggle={(key, expanded) => {
                            if (expanded) {
                                const month = hierarchicalTradesData.find(
                                    (m) => m.key === key,
                                );
                                if (month)
                                    activeContext = {
                                        type: "month",
                                        key,
                                        label: month.label,
                                    };
                            } else {
                                activeContext = {
                                    type: "global",
                                    key: null,
                                    label:
                                        $t("trades.messages.all_period") ||
                                        "Todo o Período",
                                };
                            }
                        }}
                        onWeekToggle={(key, expanded) => {
                            if (expanded) {
                                const week = hierarchicalTradesData
                                    .flatMap((m) => m.weeks)
                                    .find((w) => w.key === key);
                                if (week)
                                    activeContext = {
                                        type: "week",
                                        key,
                                        label: week.label,
                                    };
                            }
                        }}
                        onDayToggle={(key, expanded) => {
                            if (expanded) {
                                const day = hierarchicalTradesData
                                    .flatMap((m) => m.weeks)
                                    .flatMap((w) => w.days)
                                    .find((d) => d.key === key);
                                if (day)
                                    activeContext = {
                                        type: "day",
                                        key,
                                        label: day.label,
                                    };
                            }
                        }}
                    >
                        {#snippet monthBadges(month)}
                            <Badge
                                variant="outline"
                                class="text-[9px] px-1.5 h-4 bg-muted/50 border-border/50 font-bold uppercase"
                            >
                                {month.trades.length}
                                {$t("trades.messages.trades_count")}
                            </Badge>
                            <div class="flex gap-2">
                                {#each month.pnlEntries ?? [] as entry}
                                    <div class="flex items-baseline gap-0.5">
                                        <span
                                            class="text-[8px] text-muted-foreground uppercase"
                                            >{entry.curr}</span
                                        >
                                        <span
                                            class="text-[9px] font-mono font-bold {entry.val >=
                                            0
                                                ? 'text-emerald-500'
                                                : 'text-rose-500'}"
                                            >{formatNumber(entry.val)}</span
                                        >
                                    </div>
                                {/each}
                            </div>
                        {/snippet}
                        {#snippet weekBadges(week)}
                            <div class="flex gap-2">
                                {#each week.pnlEntries ?? [] as entry}
                                    <div class="flex items-baseline gap-0.5">
                                        <span
                                            class="text-[7px] text-muted-foreground uppercase"
                                            >{entry.curr}</span
                                        >
                                        <span
                                            class="text-[9px] font-mono font-bold {entry.val >=
                                            0
                                                ? 'text-emerald-500'
                                                : 'text-rose-500'}"
                                            >{formatNumber(entry.val)}</span
                                        >
                                    </div>
                                {/each}
                            </div>
                        {/snippet}
                        {#snippet dayBadges(day)}
                            <span
                                class="text-[8px] text-muted-foreground/60 font-medium"
                                >({day.trades.length}
                                {$t("trades.messages.trades_count")})</span
                            >
                            {#each day.pnlEntries ?? [] as entry}
                                <div class="flex items-baseline gap-0.5">
                                    <span
                                        class="text-[7px] text-muted-foreground uppercase"
                                        >{entry.curr}</span
                                    >
                                    <span
                                        class="text-[8px] font-mono font-bold {entry.val >=
                                        0
                                            ? 'text-emerald-500'
                                            : 'text-rose-500'}"
                                        >{formatNumber(entry.val)}</span
                                    >
                                </div>
                            {/each}
                        {/snippet}
                        {#snippet dayContent(day)}
                            <Table.Root>
                                <Table.Header>
                                    <Table.Row
                                        class="hover:bg-transparent border-border/10"
                                    >
                                        <Table.Head
                                            class="h-8 text-[9px] font-black uppercase text-muted-foreground"
                                            >{$t(
                                                "trades.table.asset",
                                            )}</Table.Head
                                        >
                                        <Table.Head
                                            class="h-8 text-[9px] font-black uppercase text-muted-foreground"
                                            >{$t(
                                                "trades.table.direction",
                                            )}</Table.Head
                                        >
                                        <Table.Head
                                            class="h-8 text-[9px] font-black uppercase text-muted-foreground"
                                            >{$t(
                                                "trades.table.entry",
                                            )}</Table.Head
                                        >
                                        <Table.Head
                                            class="h-8 text-[9px] font-black uppercase text-muted-foreground"
                                            >{$t(
                                                "trades.table.exit",
                                            )}</Table.Head
                                        >
                                        <Table.Head
                                            class="h-8 text-[9px] font-black uppercase text-muted-foreground text-right"
                                            >{$t("trades.table.pl")}</Table.Head
                                        >
                                        <Table.Head
                                            class="h-8 text-[9px] font-black uppercase text-muted-foreground text-right"
                                            >{$t(
                                                "trades.table.actions",
                                            )}</Table.Head
                                        >
                                    </Table.Row>
                                </Table.Header>
                                <Table.Body>
                                    {#each day.trades as trade (trade.id)}
                                        <Table.Row
                                            class="group/row hover:bg-primary/10 border-border/10 cursor-pointer"
                                            onclick={() =>
                                                handleSelectTrade(trade)}
                                        >
                                            <Table.Cell class="py-2">
                                                <div class="flex flex-col">
                                                    <span
                                                        class="font-bold text-xs"
                                                        >{trade.asset_symbol}</span
                                                    >
                                                    <span
                                                        class="text-[8px] text-muted-foreground uppercase tracking-tighter"
                                                    >
                                                        {settingsStore.strategies.find(
                                                            (s) =>
                                                                s.id ===
                                                                trade.strategy_id,
                                                        )?.name || "-"}
                                                    </span>
                                                </div>
                                            </Table.Cell>
                                            <Table.Cell class="py-2">
                                                <div
                                                    class="flex flex-col gap-1 items-start"
                                                >
                                                    <Badge
                                                        variant="secondary"
                                                        class="text-[9px] font-black uppercase h-5 {trade.exit_price ||
                                                        trade.exit_date
                                                            ? 'bg-emerald-500/10 text-emerald-500'
                                                            : 'bg-amber-500/10 text-amber-500'}"
                                                    >
                                                        {trade.exit_price ||
                                                        trade.exit_date
                                                            ? $t(
                                                                  "trades.table.status_closed",
                                                              ) || "Fechado"
                                                            : $t(
                                                                  "trades.table.status_open",
                                                              ) || "Aberto"}
                                                    </Badge>
                                                    <Badge
                                                        variant="outline"
                                                        class="text-[8px] font-black px-1.5 h-4 border-none {trade.direction ===
                                                        'Buy'
                                                            ? 'bg-blue-500/10 text-blue-500'
                                                            : 'bg-orange-500/10 text-orange-500'}"
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
                                                </div>
                                            </Table.Cell>
                                            <Table.Cell class="py-2">
                                                <div class="flex flex-col">
                                                    <span
                                                        class="text-xs font-mono font-bold tabular-nums"
                                                        >{trade.entry_price}</span
                                                    >
                                                    <span
                                                        class="text-[8px] text-muted-foreground/60"
                                                        >{format(
                                                            new Date(
                                                                trade.date,
                                                            ),
                                                            "HH:mm",
                                                        )}</span
                                                    >
                                                </div>
                                            </Table.Cell>
                                            <Table.Cell class="py-2">
                                                <div class="flex flex-col">
                                                    <span
                                                        class="text-xs font-mono font-bold tabular-nums"
                                                        >{trade.exit_price ||
                                                            "-"}</span
                                                    >
                                                    <span
                                                        class="text-[8px] text-muted-foreground/60"
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
                                            <Table.Cell class="py-2 text-right">
                                                <span
                                                    class="text-xs font-mono font-bold {trade.result >=
                                                    0
                                                        ? 'text-emerald-500'
                                                        : 'text-rose-500'}"
                                                >
                                                    {formatCurrency(
                                                        trade.result,
                                                        settingsStore.accounts.find(
                                                            (a) =>
                                                                a.id ===
                                                                trade.account_id,
                                                        )?.currency || "BRL",
                                                    )}
                                                </span>
                                            </Table.Cell>
                                            <Table.Cell class="py-2 text-right">
                                                <div
                                                    class="flex justify-end gap-1 opacity-0 group-hover/row:opacity-100 transition-opacity"
                                                >
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-6 w-6"
                                                        onclick={() =>
                                                            handleView(trade)}
                                                    >
                                                        <Eye class="h-3 w-3" />
                                                    </Button>
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-6 w-6"
                                                        onclick={() =>
                                                            handleEdit(trade)}
                                                    >
                                                        <Pencil
                                                            class="h-3 w-3"
                                                        />
                                                    </Button>
                                                    <button
                                                        class="p-1 px-2 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer border-none bg-transparent"
                                                        title={$t(
                                                            "trades.table.actions_delete",
                                                        ) || "Excluir"}
                                                        onclick={(e) => {
                                                            e.stopPropagation();
                                                            requestDelete(
                                                                trade,
                                                            );
                                                        }}
                                                    >
                                                        <Trash2
                                                            class="h-3 w-3"
                                                        />
                                                    </button>
                                                </div>
                                            </Table.Cell>
                                        </Table.Row>
                                    {/each}
                                </Table.Body>
                            </Table.Root>
                        {/snippet}
                    </HierarchicalList>
                {/if}
            </div>

            <!-- Right Sidebar: Interactive Metrics & Charts -->
            <div class="lg:col-span-3 space-y-4 self-start sticky top-4">
                <div class="flex items-center justify-between px-2 h-9">
                    <h3
                        class="text-sm font-bold uppercase tracking-widest text-muted-foreground/60 flex items-center gap-2"
                    >
                        <BarChart2 class="w-4 h-4" />
                        {$t("trades.sidebar.summary")}
                    </h3>
                    <div class="flex items-center gap-2">
                        <Badge
                            variant="outline"
                            class="text-[10px] font-black bg-muted border-border/40 uppercase tracking-tighter"
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
                                        label:
                                            $t("trades.messages.all_period") ||
                                            "Todo o Período",
                                    })}
                            >
                                <X class="w-3 h-3" />
                            </Button>
                        {/if}
                    </div>
                </div>

                {#if activeContext.type === "trade" && activeContext.data}
                    <!-- Card: Trade Essential Info (Visible only when trade selected) -->
                    <Card.Root class="card-glass shadow-sm overflow-hidden">
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
                                    class="text-xl font-black {activeContext
                                        .data.result >= 0
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
                    <Card.Root class="card-glass flex flex-col justify-between">
                        <Card.Header
                            class="p-4 pb-2 border-b border-border/10 flex flex-row items-center justify-between space-y-0"
                        >
                            <span
                                class="text-[10px] font-black uppercase tracking-tighter text-muted-foreground"
                                >{$t("trades.sidebar.performance")}</span
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
                                            class="text-sm font-mono font-bold tracking-tighter text-teal-400"
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
                                            class="text-[11px] font-mono font-bold text-rose-500 tracking-tighter"
                                        >
                                            {formatCurrency(
                                                activeContextStats.maxDrawdown,
                                                activeContextStats.mainCurrency,
                                                $locale || "pt-BR",
                                            ).replace(/[^\d.,+-]/g, "")}
                                        </div>
                                    </div>
                                </div>

                                <Separator class="bg-border/10" />

                                <div class="grid grid-cols-2 gap-4">
                                    <div class="space-y-0.5">
                                        <span
                                            class="text-[8px] font-black text-muted-foreground uppercase opacity-50"
                                            >Win Rate</span
                                        >
                                        <div
                                            class="text-sm font-mono font-bold tracking-tighter"
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
                                            class="text-[11px] font-mono font-bold tracking-tighter {parseFloat(
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
                    <Card.Root class="card-glass shadow-sm overflow-hidden">
                        <Card.Header
                            class="p-4 pb-0 border-b border-border/10 flex flex-row items-center justify-between space-y-0 bg-muted/5"
                        >
                            <span
                                class="text-[10px] font-black uppercase tracking-tighter text-muted-foreground"
                                >{$t("trades.sidebar.distribution")}</span
                            >
                            <div
                                class="text-[9px] font-mono font-bold opacity-30 uppercase"
                            >
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
                <Card.Root class="card-glass">
                    <Card.Header
                        class="p-4 py-3 border-b border-border/10 flex flex-row items-center justify-between space-y-0"
                    >
                        <span
                            class="text-[10px] font-black uppercase tracking-tighter text-muted-foreground"
                            >{$t("trades.sidebar.equityCurve")}</span
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
</div>

<DeleteConfirmationModal
    bind:open={isDeleteOpen}
    onConfirm={confirmDelete}
    onCancel={() => (isDeleteOpen = false)}
    title={$t("trades.delete.title")}
    description={deleteModalDescription}
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
                editTradeId={selectedTrade?.id}
                close={() => (isEditOpen = false)}
                onsave={() => {
                    isEditOpen = false;
                    // Sync DB state AFTER the dialog is closed (100ms after) to avoid triggering
                    // $effect in NewTradeWizard while it is still mounted
                    setTimeout(() => {
                        selectedTrade = null;
                        tradesStore.loadTrades();
                        // Also reload cash transactions so the statement updates after trade edit
                        settingsStore.loadCashTransactions();
                    }, 100);
                }}
            />
        {/key}
    </Dialog.Content>
</Dialog.Root>
