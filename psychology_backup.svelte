<script lang="ts">
    import { t, locale } from "svelte-i18n";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import * as Select from "$lib/components/ui/select";
    import {
        Brain,
        TrendingUp,
        TrendingDown,
        AlertTriangle,
        CheckCircle2,
        Trash2,
    } from "lucide-svelte";
    import EChart from "$lib/components/ui/echart.svelte";
    import type { EChartsOption } from "echarts";
    import { Button } from "$lib/components/ui/button";
    import DailyCheckinDialog from "$lib/components/psychology/DailyCheckinDialog.svelte";
    import DateFilter from "$lib/components/filters/DateFilter.svelte";
    import { toast } from "svelte-sonner";
    import * as Dialog from "$lib/components/ui/dialog";

    let showCheckinDialog = $state(false);
    let showDeleteConfirm = $state(false);
    let entryToDelete = $state<string | null>(null);

    // Filters
    let timeFilter = $state("all");
    let startDate = $state("");
    let endDate = $state("");
    let itemsLimit = $state("10"); // "10", "20", "50", "all"

    // --- Derived Data ---

    // Filter Helper
    function isDateInRange(dateStr: string) {
        if (timeFilter === "all") return true;
        const d = new Date(dateStr);
        const now = new Date();
        now.setHours(0, 0, 0, 0);
        d.setHours(0, 0, 0, 0); // compare dates only

        if (timeFilter === "today") return d.getTime() === now.getTime();
        if (timeFilter === "yesterday")
            return d.getTime() === now.getTime() - 86400000;

        // Simple range check for custom
        if (timeFilter === "custom" && startDate && endDate) {
            return d >= new Date(startDate) && d <= new Date(endDate);
        }

        // For other presets, we can use a helper or simplify for now (implementing basic ones)
        // Expanding basics:
        if (timeFilter === "this_month") {
            return (
                d.getMonth() === now.getMonth() &&
                d.getFullYear() === now.getFullYear()
            );
        }

        return true;
    }

    // Filtered Lists
    const filteredTrades = $derived(
        settingsStore.trades.filter((t) => isDateInRange(t.date)),
    );
    const filteredJournal = $derived(
        settingsStore.journalEntries.filter((j) => isDateInRange(j.date)),
    );

    // Paginated Lists
    const paginatedTrades = $derived(
        itemsLimit === "all"
            ? filteredTrades
            : filteredTrades.slice(0, parseInt(itemsLimit)),
    );
    const paginatedJournal = $derived(
        itemsLimit === "all"
            ? filteredJournal
            : filteredJournal.slice().reverse().slice(0, parseInt(itemsLimit)),
    );

    // All trades with emotional data (filtered for charts?? usually yes)
    // For charts, let's keep using *ALL* trades or filtered?
    // Usually charts reflect the filter. Let's make charts use filteredTrades.
    const trades = $derived(filteredTrades);
    const emotionalStates = $derived(settingsStore.emotionalStates);

    function deleteJournalEntry(id: string) {
        entryToDelete = id;
        showDeleteConfirm = true;
    }

    function confirmDelete() {
        if (entryToDelete) {
            settingsStore.removeJournalEntry(entryToDelete);
            toast.success("Registro excluído.");
        }
        showDeleteConfirm = false;
        entryToDelete = null;
    }

    // Filter Logic (Timeframe could be added here)
    // For now, using all trades

    // Group Trades by Entry Emotion
    const statsByEmotion = $derived.by(() => {
        const stats = new Map<
            string,
            {
                id: string;
                name: string;
                count: number;
                wins: number;
                losses: number;
                totalResult: number;
                impact: "Positive" | "Negative" | "Neutral";
            }
        >();

        // Initialize with all known states (even if no trades)
        for (const state of emotionalStates) {
            stats.set(state.id, {
                id: state.id,
                name: state.name,
                count: 0,
                wins: 0,
                losses: 0,
                totalResult: 0,
                impact: state.impact,
            });
        }

        // Add 'Unknown' state
        stats.set("unknown", {
            id: "unknown",
            name: "Não Registrado",
            count: 0,
            wins: 0,
            losses: 0,
            totalResult: 0,
            impact: "Neutral",
        });

        for (const trade of trades) {
            const stateId = trade.entry_emotional_state_id || "unknown";
            const current = stats.get(stateId);

            if (current) {
                current.count++;
                current.totalResult += trade.result;
                if (trade.result > 0) current.wins++;
                else if (trade.result < 0) current.losses++;
            }
        }

        return Array.from(stats.values()).filter(
            (s) => s.count > 0 || s.id !== "unknown",
        );
    });

    // KPIs
    const bestMindset = $derived.by(() => {
        const sorted = [...statsByEmotion].sort(
            (a, b) => b.totalResult - a.totalResult,
        );
        return sorted[0];
    });

    const worstMindset = $derived.by(() => {
        const sorted = [...statsByEmotion].sort(
            (a, b) => a.totalResult - b.totalResult,
        );
        return sorted[0];
    });

    const tiltTrades = $derived(
        trades.filter(
            (t) =>
                t.entry_emotional_state_id &&
                emotionalStates.find((e) => e.id === t.entry_emotional_state_id)
                    ?.impact === "Negative",
        ),
    );

    const tiltResult = $derived(
        tiltTrades.reduce((acc, t) => acc + t.result, 0),
    );

    // --- NEW: Daily Mood vs Result Correlation ---
    const correlationChartOption = $derived.by(() => {
        // 1. Get all unique dates from trades and journal
        const tradeDates = trades.map((t) => t.date);
        const journalDates = settingsStore.journalEntries.map((j) => j.date);
        const allDates = [...new Set([...tradeDates, ...journalDates])].sort();

        const data = allDates.map((date) => {
            // Daily PnL
            const dailyTrades = trades.filter((t) => t.date === date);
            const pnl = dailyTrades.reduce((acc, t) => acc + t.result, 0);

            // Daily Intensity
            const journal = settingsStore.getJournalEntryByDate(date);
            const intensity = journal ? journal.intensity : null;
            const emotionName = journal
                ? emotionalStates.find(
                      (e) => e.id === journal.emotional_state_id,
                  )?.name
                : "";

            return { date, pnl, intensity, emotionName };
        });

        if (data.length === 0) return {};

        return {
            backgroundColor: "transparent",
            title: {
                text: "Correlação: Intensidade Emocional vs Resultado",
                left: "center",
                textStyle: { color: "#888", fontSize: 14 },
            },
            tooltip: {
                trigger: "axis",
                axisPointer: { type: "cross" },
                formatter: (params: any) => {
                    let res = `<b>${params[0].axisValue}</b><br/>`;
                    params.forEach((p: any) => {
                        if (p.seriesName === "Resultado") {
                            const val =
                                typeof p.value === "number" ? p.value : 0;
                            res += `${p.marker} Resultado: R$ ${val.toFixed(2)}<br/>`;
                        } else if (p.seriesName === "Intensidade") {
                            // Find extra data for emotion name
                            const dayData = data.find(
                                (d) => d.date === p.axisValue,
                            );
                            res += `${p.marker} Intensidade: ${p.value}/10 (${dayData?.emotionName || "N/A"})<br/>`;
                        }
                    });
                    return res;
                },
            },
            grid: { left: "3%", right: "3%", bottom: "3%", containLabel: true },
            xAxis: {
                type: "category",
                data: data.map((d) => d.date),
                axisLabel: { color: "#888" },
            },
            yAxis: [
                {
                    type: "value",
                    name: "Resultado (R$)",
                    splitLine: { show: false },
                    axisLabel: { color: "#888" },
                },
                {
                    type: "value",
                    name: "Intensidade (0-10)",
                    min: 0,
                    max: 10,
                    splitLine: {
                        show: true,
                        lineStyle: { type: "dashed", opacity: 0.3 },
                    },
                    axisLabel: { color: "#888" },
                },
            ],
            series: [
                {
                    name: "Resultado",
                    type: "bar",
                    data: data.map((d) => d.pnl),
                    itemStyle: {
                        color: (params: any) =>
                            params.value >= 0 ? "#22c55e" : "#ef4444",
                    },
                },
                {
                    name: "Intensidade",
                    type: "line",
                    yAxisIndex: 1,
                    data: data.map((d) => d.intensity),
                    smooth: true,
                    lineStyle: { width: 3, color: "#3b82f6" },
                    symbol: "circle",
                    symbolSize: 8,
                    itemStyle: { color: "#3b82f6" },
                },
            ],
        } as EChartsOption;
    });

    // Chart Configuration
    const getWinRateOption = (): EChartsOption => {
        const data = statsByEmotion.map((s) => ({
            name: s.name,
            value:
                s.count > 0
                    ? parseFloat(((s.wins / s.count) * 100).toFixed(1))
                    : 0,
            itemStyle: {
                color:
                    s.impact === "Positive"
                        ? "#10b981"
                        : s.impact === "Negative"
                          ? "#f43f5e"
                          : "#64748b",
            },
        }));

        return {
            backgroundColor: "transparent",
            title: {
                text: $t("psychology.charts.winRateTitle"),
                left: "center",
                textStyle: { fontSize: 14, color: "#888" },
            },
            tooltip: { trigger: "axis", formatter: "{b}: {c}%" },
            xAxis: {
                type: "category",
                data: data.map((d) => d.name),
                axisLabel: { color: "#888" },
            },
            yAxis: {
                type: "value",
                max: 100,
                axisLabel: { formatter: "{value}%", color: "#888" },
            },
            series: [
                {
                    data: data.map((d) => d.value),
                    type: "bar",
                    barWidth: "40%",
                    itemStyle: { borderRadius: [4, 4, 0, 0] },
                },
            ],
        };
    };

    const getResultOption = (): EChartsOption => {
        const data = statsByEmotion.map((s) => ({
            name: s.name,
            value: parseFloat(s.totalResult.toFixed(2)),
            itemStyle: {
                color: s.totalResult >= 0 ? "#10b981" : "#f43f5e",
            },
        }));

        return {
            backgroundColor: "transparent",
            title: {
                text: $t("psychology.charts.resultTitle"),
                left: "center",
                textStyle: { fontSize: 14, color: "#888" },
            },
            tooltip: {
                trigger: "axis",
                formatter: (params: any) =>
                    `${params[0].name}: R$ ${params[0].value.toLocaleString($locale || "pt-BR")}`,
            },
            xAxis: {
                type: "category",
                data: data.map((d) => d.name),
                axisLabel: { color: "#888" },
            },
            yAxis: { type: "value", axisLabel: { color: "#888" } },
            series: [
                {
                    data: data.map((d) => d.value),
                    type: "bar",
                    barWidth: "40%",
                    itemStyle: { borderRadius: [4, 4, 0, 0] },
                },
            ],
        };
    };

    function formatCurrency(val: number) {
        return new Intl.NumberFormat($locale || "pt-BR", {
            style: "currency",
            currency: "BRL",
        }).format(val);
    }
</script>

<div class="space-y-6 animate-in fade-in duration-500">
    <!-- Header -->
    <div
        class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4"
    >
        <div>
            <h2 class="text-3xl font-bold tracking-tight">
                {$t("psychology.title")}
            </h2>
            <p class="text-muted-foreground">{$t("psychology.description")}</p>
        </div>
        <!-- Checkin Button -->
        <Button onclick={() => (showCheckinDialog = true)}>
            <Brain class="w-4 h-4 mr-2" />
            {$t("psychology.checkin.button")}
        </Button>
    </div>

    <Separator />

    <!-- KPI Cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- Best Mindset -->
        <Card class="border-l-4 border-l-emerald-500 shadow-sm bg-card">
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle class="text-sm font-medium"
                    >{$t("psychology.kpi.bestMindset")}</CardTitle
                >
                <Brain class="h-4 w-4 text-emerald-500" />
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold text-emerald-600">
                    {bestMindset?.name || "-"}
                </div>
                <p class="text-xs text-emerald-600/80">
                    {formatCurrency(bestMindset?.totalResult || 0)} de lucro
                </p>
            </CardContent>
        </Card>

        <!-- Worst Mindset -->
        <Card class="border-l-4 border-l-rose-500 shadow-sm bg-card">
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle class="text-sm font-medium"
                    >{$t("psychology.kpi.worstMindset")}</CardTitle
                >
                <AlertTriangle class="h-4 w-4 text-rose-500" />
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold text-rose-600">
                    {worstMindset?.name || "-"}
                </div>
                <p class="text-xs text-rose-600/80">
                    {formatCurrency(worstMindset?.totalResult || 0)} de prejuízo
                </p>
            </CardContent>
        </Card>

        <!-- Tilt Factor -->
        <Card class="border-l-4 border-l-orange-500 shadow-sm bg-card">
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle class="text-sm font-medium"
                    >{$t("psychology.kpi.tiltCost")}</CardTitle
                >
                <TrendingDown class="h-4 w-4 text-orange-500" />
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold text-orange-600">
                    {formatCurrency(tiltResult)}
                </div>
                <p class="text-xs text-muted-foreground">
                    {($t as any)("psychology.kpi.tiltDescription", {
                        count: tiltTrades.length,
                    })}
                </p>
            </CardContent>
        </Card>

        <!-- Discipline Score (Mock) -->
        <Card class="border-l-4 border-l-blue-500 shadow-sm bg-card">
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle class="text-sm font-medium"
                    >{$t("psychology.kpi.unregistered")}</CardTitle
                >
                <CheckCircle2 class="h-4 w-4 text-blue-500" />
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold text-blue-600">
                    {statsByEmotion.find((s) => s.id === "unknown")?.count || 0}
                </div>
                <p class="text-xs text-muted-foreground">
                    {$t("psychology.kpi.unregisteredDescription")}
                </p>
            </CardContent>
        </Card>
    </div>

    <!-- Charts Section (3 Columns) -->
    <div
        class="grid grid-cols-1 lg:grid-cols-3 gap-6 lg:h-[350px] min-h-[350px]"
    >
        <Card class="shadow-sm bg-card flex flex-col">
            <!-- <CardHeader class="p-4 pb-0"><CardTitle class="text-sm">Resultado</CardTitle></CardHeader> -->
            <CardContent class="p-4 flex-1 relative">
                <EChart options={getResultOption()} class="absolute inset-0" />
            </CardContent>
        </Card>
        <Card class="shadow-sm bg-card flex flex-col">
            <!-- <CardHeader class="p-4 pb-0"><CardTitle class="text-sm">Taxa de Acerto</CardTitle></CardHeader> -->
            <CardContent class="p-4 flex-1 relative">
                <EChart options={getWinRateOption()} class="absolute inset-0" />
            </CardContent>
        </Card>
        <Card class="shadow-sm bg-card flex flex-col">
            <!-- <CardHeader class="p-4 pb-0"><CardTitle class="text-sm">Correlação</CardTitle></CardHeader> -->
            <CardContent class="p-4 flex-1 relative">
                <EChart
                    options={correlationChartOption}
                    class="absolute inset-0"
                />
            </CardContent>
        </Card>
    </div>

    <!-- Filter Controls (Below Charts) -->
    <div class="flex flex-col sm:flex-row gap-2 justify-end">
        <DateFilter bind:value={timeFilter} bind:startDate bind:endDate />

        <Select.Root type="single" bind:value={itemsLimit}>
            <Select.Trigger class="w-[120px]">
                {itemsLimit === "all"
                    ? $t("general.all")
                    : `${itemsLimit} ${$t("general.items")}`}
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="10">10 {$t("general.items")}</Select.Item>
                <Select.Item value="20">20 {$t("general.items")}</Select.Item>
                <Select.Item value="50">50 {$t("general.items")}</Select.Item>
                <Select.Item value="all">{$t("general.all")}</Select.Item>
            </Select.Content>
        </Select.Root>
    </div>

    <!-- Lists Section (2 Columns) -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- Last Trades List -->
        <div class="space-y-4">
            <h3 class="text-lg font-semibold tracking-tight">
                {$t("psychology.list.title")}
            </h3>
            <div class="rounded-md border bg-card/50">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b bg-muted/50 text-left">
                            <th
                                class="h-10 px-4 font-medium text-muted-foreground"
                                >{$t("psychology.list.date")}</th
                            >
                            <th
                                class="h-10 px-4 font-medium text-muted-foreground"
                                >{$t("psychology.list.asset")}</th
                            >
                            <th
                                class="h-10 px-4 font-medium text-muted-foreground"
                                >{$t("psychology.list.entryState")}</th
                            >
                            <th
                                class="h-10 px-4 font-medium text-muted-foreground text-right"
                                >{$t("psychology.list.result")}</th
                            >
                        </tr>
                    </thead>
                    <tbody>
                        {#each trades.slice(0, 10) as trade}
                            <tr
                                class="border-b last:border-0 hover:bg-muted/30 transition-colors"
                            >
                                <td class="p-4 align-middle whitespace-nowrap"
                                    >{trade.date}</td
                                >
                                <td class="p-4 align-middle font-bold"
                                    >{trade.asset_symbol}</td
                                >
                                <td class="p-4 align-middle">
                                    {#if trade.entry_emotional_state_id}
                                        {@const em = emotionalStates.find(
                                            (e) =>
                                                e.id ===
                                                trade.entry_emotional_state_id,
                                        )}
                                        <Badge
                                            variant={em?.impact === "Positive"
                                                ? "default"
                                                : em?.impact === "Negative"
                                                  ? "destructive"
                                                  : "secondary"}
                                        >
                                            {em?.name ||
                                                $t(
                                                    "psychology.list.unknownBadge",
                                                )}
                                        </Badge>
                                    {:else}
                                        <span
                                            class="text-muted-foreground text-xs"
                                            >-</span
                                        >
                                    {/if}
                                </td>
                                <td
                                    class="p-4 align-middle text-right font-bold {trade.result >=
                                    0
                                        ? 'text-emerald-500'
                                        : 'text-rose-500'}"
                                >
                                    {formatCurrency(trade.result)}
                                </td>
                            </tr>
                        {/each}
                        {#if trades.length === 0}
                            <tr>
                                <td
                                    colspan="4"
                                    class="h-24 text-center text-muted-foreground"
                                >
                                    {$t("psychology.list.noTrades")}
                                </td>
                            </tr>
                        {/if}
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Journal Entries List -->
        <div class="space-y-4">
            <h3 class="text-lg font-semibold tracking-tight">
                {$t("psychology.journalList.title")}
            </h3>
            <div class="rounded-md border bg-card/50">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b bg-muted/50 text-left">
                            <th
                                class="h-10 px-4 font-medium text-muted-foreground"
                                >{$t("psychology.journalList.date")}</th
                            >
                            <th
                                class="h-10 px-4 font-medium text-muted-foreground"
                                >{$t("psychology.journalList.emotion")}</th
                            >
                            <th
                                class="h-10 px-4 font-medium text-muted-foreground text-center"
                                >{$t("psychology.journalList.intensity")}</th
                            >
                            <th
                                class="h-10 px-4 font-medium text-muted-foreground"
                                >{$t("psychology.journalList.notes")}</th
                            >
                        </tr>
                    </thead>
                    <tbody>
                        {#each settingsStore.journalEntries
                            .slice()
                            .reverse()
                            .slice(0, 10) as entry}
                            <tr
                                class="border-b last:border-0 hover:bg-muted/30 transition-colors"
                            >
                                <td class="p-4 align-middle whitespace-nowrap"
                                    >{entry.date}</td
                                >
                                <td class="p-4 align-middle">
                                    {#if entry.emotional_state_id}
                                        {@const em = emotionalStates.find(
                                            (e) =>
                                                e.id ===
                                                entry.emotional_state_id,
                                        )}
                                        <Badge
                                            variant={em?.impact === "Positive"
                                                ? "default"
                                                : em?.impact === "Negative"
                                                  ? "destructive"
                                                  : "secondary"}
                                        >
                                            {em?.name || "-"}
                                        </Badge>
                                    {:else}
                                        <span
                                            class="text-muted-foreground text-xs"
                                            >-</span
                                        >
                                    {/if}
                                </td>
                                <td class="p-4 align-middle text-center">
                                    <div
                                        class="flex items-center justify-center gap-1"
                                    >
                                        <span class="font-bold"
                                            >{entry.intensity}</span
                                        >
                                        <span
                                            class="text-xs text-muted-foreground"
                                            >/10</span
                                        >
                                    </div>
                                    <!-- Mini bar visualization -->
                                    <div
                                        class="w-full h-1 bg-muted rounded-full mt-1 overflow-hidden"
                                    >
                                        {#if entry.emotional_state_id}
                                            {@const em = emotionalStates.find(
                                                (e) =>
                                                    e.id ===
                                                    entry.emotional_state_id,
                                            )}
                                            <div
                                                class="h-full {em?.impact ===
                                                'Positive'
                                                    ? 'bg-emerald-500'
                                                    : em?.impact === 'Negative'
                                                      ? 'bg-rose-500'
                                                      : 'bg-slate-500'}"
                                                style="width: {entry.intensity *
                                                    10}%"
                                            ></div>
                                        {:else}
                                            <div
                                                class="h-full bg-primary"
                                                style="width: {entry.intensity *
                                                    10}%"
                                            ></div>
                                        {/if}
                                    </div>
                                </td>
                                <td class="p-4 align-middle">
                                    <p
                                        class="truncate max-w-[200px] text-muted-foreground"
                                        title={entry.content}
                                    >
                                        {entry.content || "-"}
                                    </p>
                                </td>
                                <td class="p-4 align-middle text-right">
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        class="h-8 w-8 text-muted-foreground hover:text-red-500"
                                        onclick={() =>
                                            deleteJournalEntry(entry.id)}
                                    >
                                        <Trash2 class="w-4 h-4" />
                                    </Button>
                                </td>
                            </tr>
                        {/each}
                        {#if settingsStore.journalEntries.length === 0}
                            <tr>
                                <td
                                    colspan="4"
                                    class="h-24 text-center text-muted-foreground"
                                >
                                    {$t("psychology.journalList.noEntries")}
                                </td>
                            </tr>
                        {/if}
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</div>

<DailyCheckinDialog bind:open={showCheckinDialog} />

<Dialog.Root bind:open={showDeleteConfirm}>
    <Dialog.Content>
        <Dialog.Header>
            <Dialog.Title>{$t("psychology.deleteDialog.title")}</Dialog.Title>
            <Dialog.Description>
                {$t("psychology.deleteDialog.description")}
            </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer>
            <Button
                variant="outline"
                onclick={() => (showDeleteConfirm = false)}
                >{$t("psychology.deleteDialog.cancel")}</Button
            >
            <Button variant="destructive" onclick={confirmDelete}
                >{$t("psychology.deleteDialog.confirm")}</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
