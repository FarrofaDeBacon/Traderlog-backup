<script lang="ts">
    import { page } from "$app/stores";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Card, CardContent } from "$lib/components/ui/card";
    import { Separator } from "$lib/components/ui/separator";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Tooltip from "$lib/components/ui/tooltip";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { Textarea } from "$lib/components/ui/textarea";
    import {
        ArrowLeft,
        TrendingUp,
        Activity,
        Clock,
        Target,
        Calendar,
        Maximize2,
        X,
        Globe,
        FileText,
        ImageIcon,
    } from "lucide-svelte";
    import * as Dialog from "$lib/components/ui/dialog";
    import { onMount, onDestroy } from "svelte";
    import EChart from "$lib/components/ui/echart.svelte";
    import * as echarts from "echarts";
    import { t, locale } from "svelte-i18n";
    import { mode } from "mode-watcher";

    // Theme-aware colors for ECharts
    const chartTheme = $derived(
        mode.current === "dark"
            ? {
                  text: "#94a3b8",
                  border: "#334155",
                  grid: "rgba(148, 163, 184, 0.1)",
                  tooltipBg: "rgba(15, 23, 42, 0.9)",
                  tooltipBorder: "#334155",
              }
            : {
                  text: "#64748b",
                  border: "#e2e8f0",
                  grid: "rgba(100, 116, 139, 0.1)",
                  tooltipBg: "rgba(255, 255, 255, 0.9)",
                  tooltipBorder: "#e2e8f0",
              },
    );

    const strategyId = $page.params.id;
    let strategy = $derived(
        settingsStore.strategies.find((s) => s.id === strategyId),
    );

    // Default Stats (Zeroed)
    // Filtered trades based on strategy and selected market
    const strategyTrades = $derived.by(() => {
        let trades = tradesStore.trades.filter(
            (t) => t.strategy_id === strategyId,
        );

        if (selectedMarketId) {
            // Find assets that belong to this market
            const marketAssets =
                settingsStore.getMarketAssets(selectedMarketId);
            const marketAssetSymbols = marketAssets.map((a: any) => a.symbol);
            trades = trades.filter((t: any) =>
                marketAssetSymbols.includes(t.asset_symbol),
            );
        }

        return trades.sort(
            (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
        );
    });

    // Derived Stats
    const stats = $derived.by(() => {
        const total = strategyTrades.length;
        if (total === 0) {
            return {
                netResult: 0,
                drawdown: 0,
                winRate: 0,
                profitFactor: 0,
                payoff: 0,
                mathExpectation: 0,
                maxDrawdown: 0,
                recoveryFactor: 0,
                planAdherence: 0,
                totalTrades: 0,
                winningTrades: 0,
                bestTrade: 0,
                worstTrade: 0,
                avgDurationWin: 0,
                avgDurationLoss: 0,
                avgTimeBetweenTrades: 0,
                equityCurve: { dates: [], data: [0] },
                drawdownCurve: { dates: [], data: [0] },
            };
        }

        let netResult = 0;
        let totalWin = 0;
        let totalLoss = 0;
        let wins = 0;
        let best = -Infinity;
        let worst = Infinity;
        let followedPlanCount = 0;
        let totalDurationWin = 0;
        let totalDurationLoss = 0;

        const equityData: number[] = [0];
        const drawdownData: number[] = [0];
        const dates: string[] = ["Start"];
        let peak = 0;
        let maxDD = 0;

        strategyTrades.forEach((t) => {
            const res = tradesStore.getConvertedTradeResult(
                t,
                settingsStore.accounts,
                settingsStore.currencies,
            );
            netResult += res;

            if (res > 0) {
                wins++;
                totalWin += res;
                if (res > best) best = res;
                // Calculate duration if available
                if (t.exit_date) {
                    const dur =
                        (new Date(t.exit_date).getTime() -
                            new Date(t.date).getTime()) /
                        60000;
                    totalDurationWin += dur;
                }
            } else {
                totalLoss += Math.abs(res);
                if (res < worst) worst = res;
                if (t.exit_date) {
                    const dur =
                        (new Date(t.exit_date).getTime() -
                            new Date(t.date).getTime()) /
                        60000;
                    totalDurationLoss += dur;
                }
            }

            if (t.followed_plan) followedPlanCount++;

            // Equity & Drawdown Curve
            equityData.push(Number(netResult.toFixed(2)));
            dates.push(new Date(t.date).toLocaleDateString($locale || "pt-BR"));

            if (netResult > peak) peak = netResult;
            const dd = peak - netResult;
            drawdownData.push(Number(-dd.toFixed(2)));
            if (dd > maxDD) maxDD = dd;
        });

        const winRate = (wins / total) * 100;
        const profitFactor =
            totalLoss === 0 ? (totalWin > 0 ? 99 : 0) : totalWin / totalLoss;
        const payoff =
            wins > 0 && total - wins > 0
                ? totalWin / wins / (totalLoss / (total - wins))
                : 0;
        const recoveryFactor = maxDD === 0 ? 0 : netResult / maxDD;

        // Time between trades
        let avgTimeBetween = 0;
        if (total > 1) {
            const first = new Date(strategyTrades[0].date).getTime();
            const last = new Date(strategyTrades[total - 1].date).getTime();
            avgTimeBetween = (last - first) / (total - 1) / (1000 * 3600 * 24);
        }

        return {
            netResult,
            drawdown: Number((peak - netResult).toFixed(2)),
            winRate,
            profitFactor,
            payoff,
            mathExpectation: netResult / total,
            maxDrawdown: maxDD,
            recoveryFactor,
            planAdherence: (followedPlanCount / total) * 100,
            totalTrades: total,
            winningTrades: wins,
            bestTrade: best === -Infinity ? 0 : best,
            worstTrade: worst === Infinity ? 0 : worst,
            avgDurationWin: wins > 0 ? Math.round(totalDurationWin / wins) : 0,
            avgDurationLoss:
                total - wins > 0
                    ? Math.round(totalDurationLoss / (total - wins))
                    : 0,
            avgTimeBetweenTrades: Number(avgTimeBetween.toFixed(1)),
            equityCurve: { dates, data: equityData },
            drawdownCurve: { dates, data: drawdownData },
        };
    });

    // Chart Options Generators
    const equityOptions = $derived({
        backgroundColor: "transparent",
        tooltip: {
            trigger: "axis",
            backgroundColor: "var(--popover)",
            borderColor: "var(--border)",
            textStyle: { color: "var(--foreground)" },
            formatter: (params: any) => {
                const p = params[0];
                return `<div class="font-bold mb-1">${p.name}</div>
                        <div class="text-emerald-500 font-black">${formatCurrency(p.value)}</div>`;
            },
        },
        grid: {
            left: "3%",
            right: "4%",
            bottom: "8%",
            top: "10%",
            containLabel: true,
        },
        xAxis: {
            type: "category",
            boundaryGap: false,
            data: stats.equityCurve.dates,
            axisLine: { lineStyle: { color: chartTheme.border } },
            axisLabel: { color: chartTheme.text, fontSize: 10 },
        },
        yAxis: {
            type: "value",
            splitLine: { lineStyle: { color: chartTheme.grid } },
            axisLabel: {
                color: chartTheme.text,
                fontSize: 10,
                formatter: (value: number) =>
                    new Intl.NumberFormat($locale || "pt-BR", {
                        notation: "compact",
                        style: "currency",
                        currency: "BRL",
                    }).format(value),
            },
        },
        series: [
            {
                name: "Capital",
                type: "line",
                step: "end", // Professional execution view
                symbol: "circle",
                symbolSize: 4,
                lineStyle: { width: 3, color: "#10b981" },
                areaStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: "rgba(16, 185, 129, 0.2)" },
                        { offset: 1, color: "rgba(16, 185, 129, 0.0)" },
                    ]),
                },
                data: stats.equityCurve.data,
            },
        ],
    });

    const drawdownOptions = $derived({
        backgroundColor: "transparent",
        tooltip: {
            trigger: "axis",
            backgroundColor: "var(--popover)",
            borderColor: "var(--border)",
            textStyle: { color: "var(--foreground)" },
            formatter: (params: any) => {
                const p = params[0];
                return `<div class="font-bold mb-1">${p.name}</div>
                        <div class="text-rose-500 font-black">${formatCurrency(p.value)}</div>`;
            },
        },
        grid: {
            left: "3%",
            right: "4%",
            bottom: "8%",
            top: "10%",
            containLabel: true,
        },
        xAxis: {
            type: "category",
            boundaryGap: false,
            data: stats.drawdownCurve.dates,
            show: false,
        },
        yAxis: {
            type: "value",
            inverse: true,
            splitLine: { lineStyle: { color: chartTheme.grid } },
            axisLabel: {
                color: chartTheme.text,
                fontSize: 10,
                formatter: (value: number) =>
                    new Intl.NumberFormat($locale || "pt-BR", {
                        notation: "compact",
                        style: "currency",
                        currency: "BRL",
                    }).format(value),
            },
        },
        series: [
            {
                name: "Drawdown",
                type: "line",
                step: "end",
                symbol: "none",
                lineStyle: { width: 2, color: "#f43f5e" }, // Rose-500
                areaStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: "rgba(244, 63, 94, 0.3)" },
                        { offset: 1, color: "rgba(244, 63, 94, 0.0)" },
                    ]),
                },
                data: stats.drawdownCurve.data,
            },
        ],
    });

    // Helper
    function formatCurrency(val: number) {
        return new Intl.NumberFormat($locale || "pt-BR", {
            style: "currency",
            currency: "BRL",
        }).format(val);
    }

    // Heatmap - Dynamic based on market trading hours
    let dayNames = $derived([
        $t("general.weekdays.sun"),
        $t("general.weekdays.mon"),
        $t("general.weekdays.tue"),
        $t("general.weekdays.wed"),
        $t("general.weekdays.thu"),
        $t("general.weekdays.fri"),
        $t("general.weekdays.sat"),
    ]);

    // Get available markets for this strategy
    const availableMarkets = $derived.by(() => {
        if (!strategy) return [];

        // If strategy has market_ids, use them
        if (strategy.market_ids?.length) {
            return settingsStore.markets.filter((m) =>
                strategy.market_ids.includes(m.id),
            );
        }

        // Fallback: infer from assets
        const assetSymbol = strategy.specific_assets[0];
        const asset = settingsStore.assets.find(
            (a) => a.symbol === assetSymbol,
        );
        if (!asset) return [];
        const assetType = settingsStore.assetTypes.find(
            (t) => t.id === asset.asset_type_id,
        );
        if (!assetType) return [];
        const market = settingsStore.markets.find(
            (m) => m.id === assetType.market_id,
        );
        return market ? [market] : [];
    });

    // Selected market for heatmap filter
    let selectedMarketId = $state<string | null>(null);

    // Initial load: logic to determine if we should auto-select a market or show "All"
    let initialized = $state(false);
    $effect(() => {
        if (!initialized && availableMarkets.length > 0) {
            // By default, if there are available markets, we can auto-select the first one
            // or keep it as null (All). Let's default to null to show "All" initially.
            // selectedMarketId = availableMarkets[0].id; // Removed auto-selection to show "All" by default
            initialized = true;
        }
    });

    const market = $derived(
        settingsStore.markets.find((m) => m.id === selectedMarketId) ?? null,
    );

    // Generate hour labels from trading sessions
    const hourLabels = $derived.by(() => {
        if (!market?.trading_sessions?.length) {
            // Default: 9h-18h
            return Array.from({ length: 10 }, (_, i) => `${9 + i}h`);
        }
        const hours: string[] = [];
        for (const session of market.trading_sessions) {
            const startH = parseInt(session.start_time.split(":")[0]);
            const endH = parseInt(session.end_time.split(":")[0]);
            for (let h = startH; h <= endH; h++) {
                const label = `${h}h`;
                if (!hours.includes(label)) hours.push(label);
            }
        }
        return hours.sort((a, b) => parseInt(a) - parseInt(b));
    });

    // Trading days for this market
    const days = $derived.by(() => {
        if (!market?.trading_days?.length) {
            return ["Seg", "Ter", "Qua", "Qui", "Sex"];
        }
        return market.trading_days.map((d) => dayNames[d]);
    });

    // Generate blocks grid (rows = days, cols = hours) with varied intensity
    const blocks = $derived.by(() => {
        const numDays = days.length;
        const numHours = hourLabels.length;
        const grid = Array(numDays)
            .fill(0)
            .map(() => Array(numHours).fill(0));

        strategyTrades.forEach((t) => {
            const date = new Date(t.date);
            const dayIdx = (date.getDay() + 6) % 7; // Map Sun(0)-Sat(6) to Mon(0)-Sun(6)? Let's check `market.trading_days`
            // Actually `market.trading_days` uses 0=Sun. But our heatmap rows are based on `days` array.
            // Let's find the matching day index in the `days` array.
            const dayName = dayNames[date.getDay()];
            const dayRow = days.indexOf(dayName);

            const hour = date.getHours();
            const hourCol = hourLabels.indexOf(`${hour}h`);

            if (dayRow !== -1 && hourCol !== -1) {
                const res = tradesStore.getConvertedTradeResult(
                    t,
                    settingsStore.accounts,
                    settingsStore.currencies,
                );
                grid[dayRow][hourCol] += res;
            }
        });

        return grid;
    });

    function getBlockColor(val: number) {
        if (val === 0) return "bg-muted/10 hover:bg-muted/20";
        if (val > 0) {
            return val > 1000
                ? "bg-emerald-600 hover:bg-emerald-500"
                : "bg-emerald-500/60 hover:bg-emerald-400/80";
        } else {
            return val < -1000
                ? "bg-rose-700 hover:bg-rose-600"
                : "bg-rose-500/60 hover:bg-rose-400/80";
        }
    }

    // Image viewer logic
    let selectedImageIndex = $state<number | null>(null);
</script>

{#if !strategy}
    <div class="flex flex-col items-center justify-center h-[50vh] gap-4">
        <h2 class="text-xl font-bold">{$t("strategy.notFound")}</h2>
        <Button href="/strategies" variant="outline"
            >{$t("general.back")}</Button
        >
    </div>
{:else}
    <div class="space-y-4 p-2 md:p-0 animate-in fade-in duration-500">
        <!-- Header -->
        <div class="flex items-center gap-4 mb-2">
            <Button
                variant="ghost"
                size="icon"
                href="/strategies"
                class="h-8 w-8"
            >
                <ArrowLeft class="w-4 h-4" />
            </Button>
            <h1 class="text-xl font-semibold tracking-tight">
                {strategy.name}
            </h1>
            <Badge variant="outline" class="font-mono text-xs">
                {strategy.specific_assets[0] || $t("strategy.dossier.multi")}
            </Badge>
        </div>

        <Tabs.Root value="dashboard" class="w-full">
            <Tabs.List class="grid w-full grid-cols-2 mb-4">
                <Tabs.Trigger value="dashboard"
                    >{$t("strategy.dashboard.tabs.dashboard")}</Tabs.Trigger
                >
                <Tabs.Trigger value="dossier"
                    >{$t("strategy.dashboard.tabs.dossier")}</Tabs.Trigger
                >
            </Tabs.List>

            <!-- TAB 1: DASHBOARD -->
            <Tabs.Content value="dashboard" class="space-y-4">
                <!-- Market Chips (Filtering) -->
                <div class="flex flex-wrap gap-2 mb-2 p-1">
                    <Button
                        variant={selectedMarketId === null
                            ? "default"
                            : "outline"}
                        size="sm"
                        class="h-7 px-3 text-[10px] font-bold tracking-wider uppercase transition-all"
                        onclick={() => (selectedMarketId = null)}
                    >
                        {$t("general.all")}
                    </Button>
                    {#each availableMarkets as m}
                        <Button
                            variant={selectedMarketId === m.id
                                ? "default"
                                : "outline"}
                            size="sm"
                            class="h-7 px-3 text-[10px] font-bold tracking-wider uppercase transition-all"
                            onclick={() => (selectedMarketId = m.id)}
                        >
                            {m.code}
                        </Button>
                    {/each}
                    {#if availableMarkets.length === 0}
                        <div class="text-[10px] text-muted-foreground italic">
                            {$t("strategy.dashboard.heatmap.noMarket")}
                        </div>
                    {/if}
                </div>

                <!-- Top Section: Charts & Main KPIs (6:1 Grid) -->
                <div class="flex flex-col xl:grid xl:grid-cols-7 gap-4">
                    <!-- LEFT SIDE: Charts & Heatmap (6 cols) -->
                    <div class="col-span-6 flex flex-col gap-4">
                        <div
                            class="grid grid-cols-1 md:grid-cols-2 gap-4 h-[380px]"
                        >
                            <!-- Equity Chart -->
                            <Card
                                class="flex flex-col border shadow-md bg-card/60 backdrop-blur-xl border-l-4 border-l-emerald-500"
                            >
                                <CardContent class="p-4 flex-1 flex flex-col">
                                    <div
                                        class="flex justify-between items-center mb-4"
                                    >
                                        <h3
                                            class="text-xs font-semibold text-muted-foreground uppercase tracking-wider"
                                        >
                                            {$t(
                                                "strategy.dashboard.charts.equityStrategy",
                                            )}
                                        </h3>
                                        <span
                                            class="text-lg font-bold {stats.netResult >=
                                            0
                                                ? 'text-emerald-500'
                                                : 'text-rose-500'}"
                                        >
                                            {formatCurrency(stats.netResult)}
                                        </span>
                                    </div>
                                    <div class="flex-1 w-full relative min-h-0">
                                        {#if strategyTrades.length > 0}
                                            {#key stats}
                                                <EChart
                                                    options={equityOptions}
                                                    class="absolute inset-0"
                                                />
                                            {/key}
                                        {:else}
                                            <div
                                                class="absolute inset-0 flex items-center justify-center text-muted-foreground italic text-xs"
                                            >
                                                {$t(
                                                    "strategy.dashboard.messages.noTrades",
                                                )}
                                            </div>
                                        {/if}
                                    </div>
                                </CardContent>
                            </Card>

                            <!-- Drawdown Chart -->
                            <Card
                                class="flex flex-col border shadow-md bg-card/60 backdrop-blur-xl border-l-4 border-l-rose-500"
                            >
                                <CardContent class="p-4 flex-1 flex flex-col">
                                    <div
                                        class="flex justify-between items-center mb-4"
                                    >
                                        <h3
                                            class="text-xs font-semibold text-muted-foreground uppercase tracking-wider"
                                        >
                                            {$t(
                                                "strategy.dashboard.charts.drawdown",
                                            )}
                                        </h3>
                                        <span
                                            class="text-lg font-bold text-rose-500"
                                        >
                                            {formatCurrency(stats.drawdown)}
                                        </span>
                                    </div>
                                    <div class="flex-1 w-full relative min-h-0">
                                        {#if strategyTrades.length > 0}
                                            {#key stats}
                                                <EChart
                                                    options={drawdownOptions}
                                                    class="absolute inset-0"
                                                />
                                            {/key}
                                        {:else}
                                            <div
                                                class="absolute inset-0 flex items-center justify-center text-muted-foreground italic text-xs"
                                            >
                                                {$t(
                                                    "strategy.dashboard.messages.noTrades",
                                                )}
                                            </div>
                                        {/if}
                                    </div>
                                </CardContent>
                            </Card>
                        </div>

                        <!-- Heatmap Area (integrated into left column) -->
                        <Card
                            class="border shadow-md bg-card/60 backdrop-blur-xl flex flex-col h-[280px]"
                        >
                            <CardContent class="p-4 flex-1 flex flex-col">
                                <div
                                    class="flex justify-between items-center mb-4 px-2"
                                >
                                    <h3
                                        class="text-xs font-bold text-muted-foreground uppercase tracking-widest"
                                    >
                                        {$t("strategy.dashboard.heatmap.title")}
                                    </h3>
                                    <div
                                        class="flex gap-4 text-[10px] font-medium"
                                    >
                                        <span class="flex items-center gap-1.5"
                                            ><div
                                                class="w-2.5 h-2.5 rounded-full bg-rose-700"
                                            ></div>
                                            {$t(
                                                "strategy.dashboard.heatmap.legend.strongLoss",
                                            )}</span
                                        >
                                        <span class="flex items-center gap-1.5"
                                            ><div
                                                class="w-2.5 h-2.5 rounded-full bg-emerald-700"
                                            ></div>
                                            {$t(
                                                "strategy.dashboard.heatmap.legend.strongProfit",
                                            )}</span
                                        >
                                    </div>
                                </div>
                                <div class="flex-1 w-full min-h-[180px]">
                                    <div
                                        class="grid gap-1.5 h-full"
                                        style="grid-template-columns: max-content repeat({hourLabels.length}, minmax(0, 1fr)); grid-template-rows: repeat({days.length}, 1fr);"
                                    >
                                        {#each days as day, i}
                                            <div
                                                class="flex items-center text-[10px] text-muted-foreground font-bold pr-3"
                                            >
                                                {day}
                                            </div>
                                            {#each blocks[i] as val, j}
                                                <Tooltip.Root>
                                                    <Tooltip.Trigger
                                                        class={`rounded-sm transition-all hover:scale-110 hover:z-10 w-full h-full shadow-sm ${getBlockColor(val)}`}
                                                    />
                                                    <Tooltip.Content>
                                                        <div
                                                            class="text-xs font-bold"
                                                        >
                                                            {day} - {hourLabels[
                                                                j
                                                            ]}
                                                        </div>
                                                        <div
                                                            class="text-[10px] text-muted-foreground"
                                                        >
                                                            {formatCurrency(
                                                                val,
                                                            )}
                                                        </div>
                                                    </Tooltip.Content>
                                                </Tooltip.Root>
                                            {/each}
                                        {/each}

                                        <!-- Hour Labels (Bottom Row) -->
                                        <div class="p-1"></div>
                                        <!-- Empty spacer for days column -->
                                        {#each hourLabels as hour}
                                            <div
                                                class="text-[8px] text-muted-foreground font-bold text-center pt-1 border-t border-muted/20"
                                            >
                                                {hour}
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    </div>

                    <!-- RIGHT SIDE: Integrated Sidebar (1 col) -->
                    <div class="col-span-1 flex flex-col gap-2 h-[676px]">
                        <!-- 1. Net Result -->
                        <Card
                            class="flex-1 h-[78px] shadow-md bg-card/60 backdrop-blur-xl border border-l-4 border-l-emerald-500"
                        >
                            <CardContent
                                class="p-2 flex flex-col justify-center items-center text-center h-full"
                            >
                                <div
                                    class="text-[9px] font-bold text-muted-foreground uppercase mb-1"
                                >
                                    {$t("strategy.dashboard.stats.netResult")}
                                </div>
                                <div
                                    class="text-base font-black {stats.netResult >=
                                    0
                                        ? 'text-emerald-500'
                                        : 'text-rose-500'}"
                                >
                                    {formatCurrency(stats.netResult)}
                                </div>
                            </CardContent>
                        </Card>
                        <!-- 2. Win Rate -->
                        <Card
                            class="flex-1 h-[78px] shadow-md bg-card/60 backdrop-blur-xl border border-l-4 border-l-blue-500"
                        >
                            <CardContent
                                class="p-2 flex flex-col justify-center items-center text-center h-full"
                            >
                                <div
                                    class="text-[9px] font-bold text-muted-foreground uppercase mb-1"
                                >
                                    {$t("strategy.dashboard.stats.winRate")}
                                </div>
                                <div class="text-base font-black text-blue-500">
                                    {stats.winRate.toFixed(1)}%
                                </div>
                            </CardContent>
                        </Card>
                        <!-- 3. Profit Factor -->
                        <Card
                            class="flex-1 h-[78px] shadow-md bg-card/60 backdrop-blur-xl border border-l-4 border-l-yellow-600"
                        >
                            <CardContent
                                class="p-2 flex flex-col justify-center items-center text-center h-full"
                            >
                                <div
                                    class="text-[9px] font-bold text-muted-foreground uppercase mb-1"
                                >
                                    {$t(
                                        "strategy.dashboard.stats.profitFactor",
                                    )}
                                </div>
                                <div
                                    class="text-base font-black text-yellow-600"
                                >
                                    {stats.profitFactor.toFixed(2)}
                                </div>
                            </CardContent>
                        </Card>
                        <!-- 4. Payoff -->
                        <Card
                            class="flex-1 h-[78px] shadow-md bg-card/60 backdrop-blur-xl border border-l-4 border-l-indigo-400"
                        >
                            <CardContent
                                class="p-2 flex flex-col justify-center items-center text-center h-full"
                            >
                                <div
                                    class="text-[9px] font-bold text-muted-foreground uppercase mb-1"
                                >
                                    {$t("strategy.dashboard.stats.payoff")}
                                </div>
                                <div
                                    class="text-base font-black text-indigo-400"
                                >
                                    {stats.payoff.toFixed(2)}
                                </div>
                            </CardContent>
                        </Card>
                        <!-- 5. Math Expectation -->
                        <Card
                            class="flex-1 h-[78px] shadow-md bg-card/60 backdrop-blur-xl border border-l-4 border-l-slate-400"
                        >
                            <CardContent
                                class="p-2 flex flex-col justify-center items-center text-center h-full"
                            >
                                <div
                                    class="text-[9px] font-bold text-muted-foreground uppercase mb-1"
                                >
                                    {$t(
                                        "strategy.dashboard.stats.mathExpectation",
                                    )}
                                </div>
                                <div class="text-sm font-black text-slate-300">
                                    {formatCurrency(stats.mathExpectation)}
                                </div>
                            </CardContent>
                        </Card>
                        <!-- 6. Max Drawdown -->
                        <Card
                            class="flex-1 h-[78px] shadow-md bg-card/60 backdrop-blur-xl border border-l-4 border-l-rose-500"
                        >
                            <CardContent
                                class="p-2 flex flex-col justify-center items-center text-center h-full"
                            >
                                <div
                                    class="text-[9px] font-bold text-muted-foreground uppercase mb-1"
                                >
                                    {$t("strategy.dashboard.stats.maxDrawdown")}
                                </div>
                                <div class="text-base font-black text-rose-500">
                                    {formatCurrency(stats.maxDrawdown)}
                                </div>
                            </CardContent>
                        </Card>
                        <!-- 7. Recovery Factor -->
                        <Card
                            class="flex-1 h-[78px] shadow-md bg-card/60 backdrop-blur-xl border border-l-4 border-l-cyan-500"
                        >
                            <CardContent
                                class="p-2 flex flex-col justify-center items-center text-center h-full"
                            >
                                <div
                                    class="text-[9px] font-bold text-muted-foreground uppercase mb-1"
                                >
                                    {$t(
                                        "strategy.dashboard.stats.recoveryFactor",
                                    )}
                                </div>
                                <div class="text-base font-black text-cyan-500">
                                    {stats.recoveryFactor.toFixed(2)}
                                </div>
                            </CardContent>
                        </Card>
                        <!-- 8. Plan Adherence -->
                        <Card
                            class="flex-1 h-[78px] shadow-md bg-card/60 backdrop-blur-xl border border-l-4 border-l-orange-500"
                        >
                            <CardContent
                                class="p-2 flex flex-col justify-center items-center text-center h-full"
                            >
                                <div
                                    class="text-[9px] font-bold text-muted-foreground uppercase mb-1"
                                >
                                    {$t(
                                        "strategy.dashboard.stats.planAdherence",
                                    )}
                                </div>
                                <div
                                    class="text-base font-black text-orange-500"
                                >
                                    {stats.planAdherence.toFixed(1)}%
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </div>

                <!-- Bottom Metrics Row -->
                <div
                    class="grid grid-cols-2 md:grid-cols-4 xl:grid-cols-7 gap-4"
                >
                    <Card
                        class="border shadow-sm bg-card/60 backdrop-blur-xl border-l-4 border-l-slate-400 h-[80px]"
                    >
                        <CardContent
                            class="p-0 flex flex-col justify-center items-center h-full"
                        >
                            <div
                                class="text-[9px] font-bold text-muted-foreground uppercase mb-0.5"
                            >
                                {$t("strategy.dashboard.stats.totalTrades")}
                            </div>
                            <div class="text-sm font-black">
                                {stats.totalTrades}
                            </div>
                        </CardContent>
                    </Card>
                    <Card
                        class="border shadow-sm bg-card/60 backdrop-blur-xl border-l-4 border-l-emerald-500 h-[80px]"
                    >
                        <CardContent
                            class="p-0 flex flex-col justify-center items-center h-full"
                        >
                            <div
                                class="text-[9px] font-bold text-muted-foreground uppercase mb-0.5"
                            >
                                {$t("strategy.dashboard.stats.winningTrades")}
                            </div>
                            <div class="text-sm font-black text-emerald-500">
                                {stats.winningTrades}
                            </div>
                        </CardContent>
                    </Card>
                    <Card
                        class="border shadow-sm bg-card/60 backdrop-blur-xl border-l-4 border-l-emerald-600 h-[80px]"
                    >
                        <CardContent
                            class="p-0 flex flex-col justify-center items-center h-full"
                        >
                            <div
                                class="text-[9px] font-bold text-muted-foreground uppercase mb-0.5"
                            >
                                {$t("strategy.dashboard.stats.bestTrade")}
                            </div>
                            <div class="text-sm font-black text-emerald-600">
                                {formatCurrency(stats.bestTrade)}
                            </div>
                        </CardContent>
                    </Card>
                    <Card
                        class="border shadow-sm bg-card/60 backdrop-blur-xl border-l-4 border-l-rose-500 h-[80px]"
                    >
                        <CardContent
                            class="p-0 flex flex-col justify-center items-center h-full"
                        >
                            <div
                                class="text-[9px] font-bold text-muted-foreground uppercase mb-0.5"
                            >
                                {$t("strategy.dashboard.stats.worstTrade")}
                            </div>
                            <div class="text-sm font-black text-rose-500">
                                {formatCurrency(stats.worstTrade)}
                            </div>
                        </CardContent>
                    </Card>
                    <Card
                        class="border shadow-sm bg-card/60 backdrop-blur-xl border-l-4 border-l-blue-400 h-[80px]"
                    >
                        <CardContent
                            class="p-0 flex flex-col justify-center items-center h-full"
                        >
                            <div
                                class="text-[9px] font-bold text-muted-foreground uppercase mb-0.5"
                            >
                                {$t("strategy.dashboard.stats.avgDurationWin")}
                            </div>
                            <div class="text-sm font-black text-emerald-500">
                                {stats.avgDurationWin}m
                            </div>
                        </CardContent>
                    </Card>
                    <Card
                        class="border shadow-sm bg-card/60 backdrop-blur-xl border-l-4 border-l-rose-400 h-[80px]"
                    >
                        <CardContent
                            class="p-0 flex flex-col justify-center items-center h-full"
                        >
                            <div
                                class="text-[9px] font-bold text-muted-foreground uppercase mb-0.5"
                            >
                                {$t("strategy.dashboard.stats.avgDurationLoss")}
                            </div>
                            <div class="text-sm font-black text-rose-500">
                                {stats.avgDurationLoss}m
                            </div>
                        </CardContent>
                    </Card>
                    <Card
                        class="border shadow-sm bg-card/60 backdrop-blur-xl border-l-4 border-l-indigo-400 h-[80px]"
                    >
                        <CardContent
                            class="p-0 flex flex-col justify-center items-center h-full"
                        >
                            <div
                                class="text-[9px] font-bold text-muted-foreground uppercase mb-0.5"
                            >
                                {$t(
                                    "strategy.dashboard.stats.avgTimeBetweenTrades",
                                )}
                            </div>
                            <div class="text-sm font-black text-blue-500">
                                {stats.avgTimeBetweenTrades}d
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </Tabs.Content>

            <!-- TAB 2: DOSSIÊ (Read-only Strategy Details) -->
            <Tabs.Content value="dossier" class="space-y-4">
                <!-- Operational Context -->
                <Card class="border-primary/20 bg-primary/5">
                    <CardContent class="p-4 space-y-3">
                        <div class="flex items-center gap-2 mb-2">
                            <Target class="w-4 h-4 text-primary" />
                            <h3
                                class="text-sm font-semibold tracking-tight uppercase"
                            >
                                {$t("strategy.dossier.context")}
                            </h3>
                        </div>

                        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <!-- Timeframes -->
                            <div class="space-y-1">
                                <Label
                                    class="text-[10px] text-muted-foreground uppercase font-bold"
                                    >{$t("strategy.dossier.timeframes")}</Label
                                >
                                <div class="flex flex-wrap gap-1.5">
                                    {#each strategy.timeframes as tf}
                                        <Badge
                                            variant="secondary"
                                            class="font-mono text-[10px] h-5"
                                            >{tf}</Badge
                                        >
                                    {:else}
                                        <span
                                            class="text-xs text-muted-foreground"
                                            >-</span
                                        >
                                    {/each}
                                </div>
                            </div>
                            <!-- Indicators -->
                            <div class="space-y-1">
                                <Label
                                    class="text-[10px] text-muted-foreground uppercase font-bold"
                                    >{$t("strategy.dossier.indicators")}</Label
                                >
                                <div class="flex flex-wrap gap-1.5">
                                    {#each strategy.indicators as ind}
                                        <Badge
                                            variant="outline"
                                            class="bg-background text-[10px] h-5"
                                            >{ind}</Badge
                                        >
                                    {:else}
                                        <span
                                            class="text-xs text-muted-foreground"
                                            >-</span
                                        >
                                    {/each}
                                </div>
                            </div>
                            <!-- Asset Types -->
                            <div class="space-y-1">
                                <Label
                                    class="text-[10px] text-muted-foreground uppercase font-bold"
                                    >{$t("strategy.dossier.assetTypes")}</Label
                                >
                                <div class="flex flex-wrap gap-1.5">
                                    {#each strategy.asset_types as at}
                                        <Badge
                                            variant="secondary"
                                            class="text-[10px] h-5">{at}</Badge
                                        >
                                    {:else}
                                        <span
                                            class="text-xs text-muted-foreground"
                                            >-</span
                                        >
                                    {/each}
                                </div>
                            </div>
                            <!-- Specific Assets -->
                            <div class="space-y-1">
                                <Label
                                    class="text-[10px] text-muted-foreground uppercase font-bold"
                                    >{$t(
                                        "strategy.dossier.specificAssets",
                                    )}</Label
                                >
                                <div class="flex flex-wrap gap-1.5">
                                    {#each strategy.specific_assets as sa}
                                        <Badge
                                            class="bg-emerald-500/10 text-emerald-600 hover:bg-emerald-500/20 text-[10px] h-5"
                                            >{sa}</Badge
                                        >
                                    {:else}
                                        <span
                                            class="text-xs text-muted-foreground"
                                            >{$t("strategy.dossier.all")}</span
                                        >
                                    {/each}
                                </div>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                <!-- Linked Markets -->
                <Card>
                    <CardContent class="p-4">
                        <div class="flex items-center gap-2 mb-2">
                            <Globe class="w-4 h-4 text-muted-foreground" />
                            <h3
                                class="text-sm font-semibold text-muted-foreground uppercase tracking-wider"
                            >
                                {$t("strategy.dossier.linkedMarkets")}
                            </h3>
                        </div>
                        <div class="flex flex-wrap gap-2">
                            {#each availableMarkets as m}
                                <div
                                    class="flex items-center gap-2 px-2 py-1 rounded-md bg-muted text-xs border"
                                >
                                    <span class="font-bold">{m.code}</span>
                                    {#if m.trading_sessions?.length}
                                        <span
                                            class="text-[10px] text-muted-foreground"
                                            >({m.trading_sessions[0].start_time}
                                            - {m.trading_sessions[0]
                                                .end_time})</span
                                        >
                                    {/if}
                                </div>
                            {:else}
                                <span class="text-xs text-muted-foreground"
                                    >{$t(
                                        "strategy.dossier.noLinkedMarket",
                                    )}</span
                                >
                            {/each}
                        </div>
                    </CardContent>
                </Card>

                <!-- Triggers & Logic -->
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <!-- Entry -->
                    <Card class="border-emerald-500/20 bg-emerald-500/5">
                        <CardContent class="p-4 space-y-2">
                            <h4
                                class="text-xs font-bold uppercase tracking-wider text-emerald-600 flex items-center gap-2"
                            >
                                <TrendingUp class="w-3 h-3" />
                                {$t("strategy.dossier.entryTriggers")}
                            </h4>
                            <div
                                class="bg-background/50 p-3 rounded-md border min-h-[100px] text-xs whitespace-pre-line"
                            >
                                {strategy.entry_criteria ||
                                    $t("strategy.dossier.noCriteria")}
                            </div>
                        </CardContent>
                    </Card>

                    <!-- Exit -->
                    <Card class="border-rose-500/20 bg-rose-500/5">
                        <CardContent class="p-4 space-y-2">
                            <h4
                                class="text-xs font-bold uppercase tracking-wider text-rose-600 flex items-center gap-2"
                            >
                                <Activity class="w-3 h-3" />
                                {$t("strategy.dossier.exit")}
                            </h4>
                            <div
                                class="bg-background/50 p-3 rounded-md border min-h-[100px] text-xs whitespace-pre-line"
                            >
                                {strategy.exit_criteria ||
                                    $t("strategy.dossier.noCriteria")}
                            </div>
                        </CardContent>
                    </Card>

                    <!-- Management -->
                    <Card class="border-blue-500/20 bg-blue-500/5">
                        <CardContent class="p-4 space-y-2">
                            <h4
                                class="text-xs font-bold uppercase tracking-wider text-blue-600 flex items-center gap-2"
                            >
                                <Clock class="w-3 h-3" />
                                {$t("strategy.dossier.management")}
                            </h4>
                            <div
                                class="bg-background/50 p-3 rounded-md border min-h-[100px] text-xs whitespace-pre-line"
                            >
                                {strategy.management_criteria ||
                                    $t("strategy.dossier.noCriteria")}
                            </div>
                        </CardContent>
                    </Card>
                </div>

                <!-- Partials -->
                {#if strategy.has_partial}
                    <div
                        class="p-4 rounded-lg border border-dashed border-amber-500/30 bg-amber-500/5"
                    >
                        <h4
                            class="text-xs font-bold uppercase tracking-wider text-amber-600 mb-2 flex items-center gap-2"
                        >
                            <Target class="w-3 h-3" />
                            {$t("strategy.dossier.partials")}
                        </h4>
                        <p class="text-xs whitespace-pre-line text-foreground">
                            {strategy.partial_description || "—"}
                        </p>
                    </div>
                {/if}

                <!-- Visual Examples -->
                <div class="space-y-3">
                    <h3 class="text-sm font-semibold tracking-tight">
                        {$t("strategy.dossier.visualExamples")}
                    </h3>
                    <Separator />
                    {#if strategy.images.length > 0}
                        <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
                            {#each strategy.images as img, i}
                                <button
                                    class="relative group aspect-video rounded-lg overflow-hidden border bg-muted"
                                    onclick={() => (selectedImageIndex = i)}
                                >
                                    <img
                                        src={img.path}
                                        alt="Exemplo"
                                        class="w-full h-full object-cover transition-transform group-hover:scale-105"
                                    />
                                    <div
                                        class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
                                    >
                                        <Maximize2 class="w-6 h-6 text-white" />
                                    </div>
                                </button>
                            {/each}
                        </div>
                    {:else}
                        <div
                            class="flex flex-col items-center justify-center p-6 border-2 border-dashed rounded-lg text-muted-foreground"
                        >
                            <ImageIcon class="w-6 h-6 mb-2 opacity-50" />
                            <span class="text-xs"
                                >{$t("strategy.dossier.noImages")}</span
                            >
                        </div>
                    {/if}
                </div>
            </Tabs.Content>
        </Tabs.Root>

        <!-- Gallery Overlay -->
        {#if selectedImageIndex !== null && strategy.images.length > 0}
            <div
                role="button"
                tabindex="0"
                class="fixed inset-0 z-[100] bg-black/95 backdrop-blur-sm flex flex-col items-center justify-center p-4 animate-in fade-in duration-200 border-0 cursor-default focus:outline-none"
                onclick={() => (selectedImageIndex = null)}
                onkeydown={(e) => {
                    if (e.key === "Escape") selectedImageIndex = null;
                }}
            >
                <button
                    class="absolute top-4 right-4 text-white/50 hover:text-white"
                    onclick={() => (selectedImageIndex = null)}
                >
                    <X class="w-8 h-8" />
                </button>
                <img
                    src={strategy.images[selectedImageIndex].path}
                    alt="Full view"
                    class="max-w-full max-h-[90vh] object-contain rounded-lg border border-white/10 shadow-2xl"
                />
                {#if strategy.images[selectedImageIndex].description}
                    <p
                        class="mt-4 text-white/80 text-center max-w-2xl bg-black/50 p-2 rounded"
                    >
                        {strategy.images[selectedImageIndex].description}
                    </p>
                {/if}
            </div>
        {/if}
    </div>
{/if}

<svelte:window
    onkeydown={(e) => {
        if (e.key === "Escape") selectedImageIndex = null;
    }}
/>
