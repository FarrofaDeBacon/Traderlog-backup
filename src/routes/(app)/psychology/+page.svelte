<script lang="ts">
    import { t, locale } from "svelte-i18n";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import * as Card from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import * as Select from "$lib/components/ui/select";
    import {
        Brain,
        TrendingUp,
        TrendingDown,
        CheckCircle2,
        AlertTriangle,
        Calendar,
        Eye,
        ChevronDown,
        Trash2,
        Info,
    } from "lucide-svelte";

    import EChart from "$lib/components/ui/echart.svelte";
    import { Button } from "$lib/components/ui/button";
    import DailyCheckinDialog from "$lib/components/psychology/DailyCheckinDialog.svelte";
    import DateFilter from "$lib/components/filters/DateFilter.svelte";
    import { toast } from "svelte-sonner";
    import * as Dialog from "$lib/components/ui/dialog";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { untrack } from "svelte";
    import type { EChartsOption } from "echarts";
    import Skeleton from "$lib/components/ui/skeleton.svelte";
    import { getLocalDatePart } from "$lib/utils";
    import type { Trade } from "$lib/types";

    let showCheckinDialog = $state(false);
    let showDeleteConfirm = $state(false);
    let entryToDelete = $state<string | null>(null);
    let showDayModal = $state(false);
    let selectedDayData = $state<any>(null);
    let selectedCurrency = $state<string | null>(null);
    let showInsightModal = $state(false);
    let insightData = $state<{
        label: string;
        avgWeight: number;
        equivalentState: any;
        items: any[];
    } | null>(null);

    function openDayDetails(day: any, currency: string | null = null) {
        selectedDayData = day;
        selectedCurrency = currency;
        showDayModal = true;
    }

    let isDeleteWithClosureOpen = $state(false);
    let associatedClosureIds: string[] = [];

    function requestDeleteJournal(id: string) {
        entryToDelete = id;
        const entry = settingsStore.journalEntries.find((j) => j.id === id);
        if (entry) {
            // Check if there are daily closures associated with this date
            const dateStr = entry.date.split("T")[0];
            const closures = settingsStore.cashTransactions.filter(
                (t) =>
                    t.id.includes("daily_closure_") &&
                    getLocalDatePart(t.date) === dateStr,
            );

            if (closures.length > 0) {
                associatedClosureIds = closures.map((c) => c.id);
                isDeleteWithClosureOpen = true;
                return;
            }
        }
        showDeleteConfirm = true;
    }

    async function confirmDeleteJournalWithClosure(deleteClosures: boolean) {
        if (entryToDelete) {
            const result =
                await settingsStore.removeJournalEntry(entryToDelete);
            if (result && result.success === false) {
                toast.error("Erro ao excluir o registro: " + result.error);
            } else {
                if (deleteClosures && associatedClosureIds.length > 0) {
                    let hasError = false;
                    for (const cid of associatedClosureIds) {
                        const cres =
                            await settingsStore.removeCashTransaction(cid);
                        if (!cres.success) hasError = true;
                    }
                    if (hasError) {
                        toast.error(
                            "Registro psicológico removido, mas houve erro ao remover o(s) fechamento(s) financeiro(s).",
                        );
                    } else {
                        toast.success(
                            $t("general.deleteSuccess") ||
                                "Removido com sucesso",
                        );
                    }
                } else {
                    toast.success(
                        $t("general.deleteSuccess") ||
                            "Registro removido com sucesso",
                    );
                }
            }
            entryToDelete = null;
            associatedClosureIds = [];
            isDeleteWithClosureOpen = false;
        }
    }

    async function confirmDeleteJournal() {
        if (entryToDelete) {
            const result =
                await settingsStore.removeJournalEntry(entryToDelete);
            if (result && result.success === false) {
                toast.error("Erro ao excluir: " + result.error);
            } else {
                toast.success(
                    $t("general.deleteSuccess") ||
                        "Registro removido com sucesso",
                );
            }
            entryToDelete = null;
        }
    }

    function openInsight(data: any) {
        // Collect all trades and journals from levels (Month/Week/Day)
        const items: any[] = [];
        const processLevel = (lvl: any) => {
            if (lvl.trades) items.push(...lvl.trades);
            if (lvl.journalEntries) items.push(...lvl.journalEntries);
            if (lvl.weeks) {
                const weeksList =
                    lvl.weeks instanceof Map
                        ? Array.from(lvl.weeks.values())
                        : Object.values(lvl.weeks || {});
                for (const w of weeksList) processLevel(w);
            }
            if (lvl.days) {
                for (const d of lvl.days) processLevel(d);
            }
        };
        processLevel(data);

        insightData = {
            label: data.label || "Detalhes do Cálculo",
            avgWeight: data.avgWeight,
            equivalentState: data.equivalentState,
            items,
        };
        showInsightModal = true;
    }

    // Filters
    let timeFilter = $state("all");
    let startDate = $state("");
    let endDate = $state("");
    let itemsLimit = $state("10"); // "10", "20", "50", "all"
    let groupBy = $state("day"); // "day", "week", "month"

    // Expansion state
    let expandedMonths = $state(new Set<string>());
    let expandedWeeks = $state(new Set<string>());
    let expandedDays = $state(new Set<string>());

    function toggleMonth(key: string) {
        console.log(`[PsychologyHub] Toggling month: ${key}`);
        if (expandedMonths.has(key)) {
            expandedMonths.delete(key);
        } else {
            expandedMonths.add(key);
        }
        expandedMonths = new Set(expandedMonths);
    }

    function toggleWeek(key: string) {
        console.log(`[PsychologyHub] Toggling week: ${key}`);
        if (expandedWeeks.has(key)) {
            expandedWeeks.delete(key);
        } else {
            expandedWeeks.add(key);
        }
        expandedWeeks = new Set(expandedWeeks);
    }

    function toggleDay(date: string) {
        console.log(`[PsychologyHub] Toggling day: ${date}`);
        if (expandedDays.has(date)) {
            expandedDays.delete(date);
        } else {
            expandedDays.add(date);
        }
        expandedDays = new Set(expandedDays);
    }

    // --- Derived Data ---

    // Optimized filter limits
    const filterLimits = $derived.by(() => {
        const now = new Date();
        now.setHours(0, 0, 0, 0);
        const todayStr = now.toISOString().split("T")[0];
        const yesterdayStr = new Date(now.getTime() - 86400000)
            .toISOString()
            .split("T")[0];
        const thisMonthStr = now.toISOString().slice(0, 7); // YYYY-MM

        return {
            today: todayStr,
            yesterday: yesterdayStr,
            thisMonth: thisMonthStr,
            customStart: startDate ? new Date(startDate) : null,
            customEnd: endDate ? new Date(endDate) : null,
        };
    });

    function isDateInRange(dateStr: string) {
        if (timeFilter === "all") return true;
        const onlyDate = dateStr.split("T")[0];

        if (timeFilter === "today") return onlyDate === filterLimits.today;
        if (timeFilter === "yesterday")
            return onlyDate === filterLimits.yesterday;
        if (timeFilter === "this_month")
            return onlyDate.startsWith(filterLimits.thisMonth);
        if (
            timeFilter === "custom" &&
            filterLimits.customStart &&
            filterLimits.customEnd
        ) {
            const d = new Date(onlyDate + "T12:00:00");
            return d >= filterLimits.customStart && d <= filterLimits.customEnd;
        }
        return true;
    }

    const filteredTrades = $derived(
        tradesStore.trades.filter((t) => isDateInRange(t.date)),
    );
    const filteredJournal = $derived(
        settingsStore.journalEntries.filter((j) => isDateInRange(j.date)),
    );
    const emotionalStates = $derived(settingsStore.emotionalStates);

    function getAdjustedWeight(
        stateId: string,
        intensity: number,
        statesMap: Map<string, any>,
    ) {
        const state = statesMap.get(stateId);
        if (!state) return 5;
        // Impact-based weight normalization
        if (state.impact === "Positive") return 5 + intensity / 2; // 5 to 10
        if (state.impact === "Negative") return 5 - intensity / 2; // 0 to 5
        return 5;
    }

    function calculateAverageEmotion(
        tradesList: any[],
        journalList: any[],
        statesMap: Map<string, any>,
    ) {
        let globalScoreSum = 0;
        let itemCount = 0;

        for (const trade of tradesList) {
            const entryState = trade.entry_emotional_state_id
                ? statesMap.get(trade.entry_emotional_state_id)
                : null;
            const exitState = trade.exit_emotional_state_id
                ? statesMap.get(trade.exit_emotional_state_id)
                : null;

            let tradeScore = 0;
            let count = 0;

            if (entryState) {
                tradeScore += entryState.weight ?? 5;
                count++;
            }
            if (exitState) {
                tradeScore += exitState.weight ?? 5;
                count++;
            }

            if (count > 0) {
                const finalTradeScore = tradeScore / count;
                globalScoreSum += finalTradeScore;
                itemCount++;
            }
        }

        for (const j of journalList) {
            if (j.emotional_state_id) {
                const state = statesMap.get(j.emotional_state_id);
                if (state) {
                    const weight = state.weight ?? 5;
                    globalScoreSum += weight;
                    itemCount++;
                }
            }
        }

        const avgWeight = itemCount > 0 ? globalScoreSum / itemCount : 5;

        // Encontrar o estado equivalente mais próximo
        let bestState = null;
        let minDiff = Infinity;
        const statesList =
            statesMap instanceof Map
                ? Array.from(statesMap.values())
                : Object.values(statesMap || {});
        for (const state of statesList) {
            const diff = Math.abs((state.weight || 0) - avgWeight);
            if (diff < minDiff) {
                minDiff = diff;
                bestState = state;
            }
        }

        return { avgWeight, equivalentState: bestState };
    }

    function getWeekKey(date: Date) {
        const d = new Date(date);
        const day = d.getDay();
        const diff = d.getDate() - day + (day === 0 ? -6 : 1);
        const monday = new Date(d.setDate(diff));
        return monday.toISOString().split("T")[0];
    }

    const hierarchicalPsychologyData = $derived.by(() => {
        if (settingsStore.isLoadingData) return [];
        const t0 = performance.now();
        const monthsMap = new Map<string, any>();

        const statesMap = new Map<string, any>();
        for (const s of emotionalStates) statesMap.set(s.id, s);

        // Group trades and journals by date first
        const dayMap = new Map<
            string,
            { trades: any[]; journalEntries: any[] }
        >();

        for (const trade of filteredTrades) {
            const dateStr = trade.date.split("T")[0];
            if (!dayMap.has(dateStr))
                dayMap.set(dateStr, { trades: [], journalEntries: [] });
            dayMap.get(dateStr)!.trades.push(trade);
        }

        for (const journal of filteredJournal) {
            const dateStr = journal.date.split("T")[0];
            if (!dayMap.has(dateStr))
                dayMap.set(dateStr, { trades: [], journalEntries: [] });
            dayMap.get(dateStr)!.journalEntries.push(journal);
        }

        // Build the hierarchy: Month -> Week -> Day
        for (const [dateStr, data] of dayMap.entries()) {
            const date = new Date(dateStr + "T12:00:00");
            const monthKey = dateStr.slice(0, 7); // YYYY-MM
            const weekKey = getWeekKey(date);

            if (!monthsMap.has(monthKey)) {
                monthsMap.set(monthKey, {
                    key: monthKey,
                    label: date.toLocaleDateString($locale || "pt-BR", {
                        month: "long",
                        year: "numeric",
                    }),
                    weeks: new Map<string, any>(),
                    totalPnlByCurrency: {} as Record<string, number>,
                    trades: [],
                    journalEntries: [],
                });
            }

            const month = monthsMap.get(monthKey);
            if (!month.weeks.has(weekKey)) {
                month.weeks.set(weekKey, {
                    key: weekKey,
                    label: `Semana de ${new Date(weekKey).toLocaleDateString($locale || "pt-BR", { day: "numeric", month: "short" })}`,
                    days: [],
                    totalPnlByCurrency: {} as Record<string, number>,
                    trades: [],
                    journalEntries: [],
                });
            }

            const week = month.weeks.get(weekKey);
            const dayEmotion = calculateAverageEmotion(
                data.trades,
                data.journalEntries,
                statesMap,
            );
            const dayPnl: Record<string, number> = {};
            for (const trade of data.trades) {
                const acc = settingsStore.accounts.find(
                    (a) => a.id === trade.account_id,
                );
                const curr = acc?.currency || "BRL";
                dayPnl[curr] = (dayPnl[curr] || 0) + trade.result;

                // Propagate to week and month
                week.totalPnlByCurrency[curr] =
                    (week.totalPnlByCurrency[curr] || 0) + trade.result;
                month.totalPnlByCurrency[curr] =
                    (month.totalPnlByCurrency[curr] || 0) + trade.result;
            }

            const dayObj = {
                key: dateStr,
                date: dateStr,
                label: date.toLocaleDateString($locale || "pt-BR", {
                    weekday: "long",
                    day: "numeric",
                }),
                trades: data.trades,
                journalEntries: data.journalEntries,
                totalPnlByCurrency: dayPnl,
                ...dayEmotion,
            };

            week.days.push(dayObj);
            week.trades.push(...data.trades);
            week.journalEntries.push(...data.journalEntries);

            month.trades.push(...data.trades);
            month.journalEntries.push(...data.journalEntries);
        }

        // Convert Maps to sorted arrays and calculate emotions for higher levels
        try {
            const result = Array.from(monthsMap?.values() || [])
                .sort((a: any, b: any) =>
                    (b.key || "").localeCompare(a.key || ""),
                )
                .map((month: any) => {
                    const monthEmotion = calculateAverageEmotion(
                        month.trades || [],
                        month.journalEntries || [],
                        statesMap,
                    );
                    const weeks = Array.from(
                        (month.weeks as Map<string, any>)?.values() || [],
                    )
                        .sort((a: any, b: any) =>
                            (b.key || "").localeCompare(a.key || ""),
                        )
                        .map((week: any) => {
                            const weekEmotion = calculateAverageEmotion(
                                week.trades || [],
                                week.journalEntries || [],
                                statesMap,
                            );
                            const days = (week.days || []).sort(
                                (a: any, b: any) =>
                                    (b.date || "").localeCompare(a.date || ""),
                            );
                            return { ...week, ...weekEmotion, days };
                        });
                    return { ...month, ...monthEmotion, weeks };
                });

            console.log(
                `[PsychologyHub] hierarchicalPsychologyData calculated in ${performance.now() - t0}ms`,
            );
            return result;
        } catch (err) {
            console.error("[PsychologyHub] Error processing results:", err);
            return [];
        }
    });

    // Auto-expand first items (Month and Week)
    $effect(() => {
        if (hierarchicalPsychologyData.length > 0) {
            untrack(() => {
                if (expandedMonths.size === 0) {
                    const firstMonth = hierarchicalPsychologyData[0];
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

    async function syncWeights() {
        try {
            await invoke("seed_emotional_states");
            await settingsStore.loadData();
            toast.success("Pesos sincronizados com sucesso!");
        } catch (e) {
            toast.error("Erro ao sincronizar pesos.");
        }
    }

    // Chart Logic (optimized)
    const correlationChartOption = $derived.by(() => {
        const t0 = performance.now();
        const trades = filteredTrades;
        const journals = filteredJournal;

        if (trades.length === 0 && journals.length === 0) return {};

        // Use Maps for O(1) lookup
        const journalMap = new Map<string, any>();
        for (const j of journals) {
            journalMap.set(j.date.split("T")[0], j);
        }

        const tradeMap = new Map<string, any[]>();
        for (const t of trades) {
            const date = t.date.split("T")[0];
            if (!tradeMap.has(date)) tradeMap.set(date, []);
            tradeMap.get(date)!.push(t);
        }

        const tradeDates = trades.map((t) => t.date.split("T")[0]);
        const journalDates = journals.map((j) => j.date.split("T")[0]);
        const allDates = [...new Set([...tradeDates, ...journalDates])].sort();

        const data = allDates.map((date) => {
            const dailyTrades = tradeMap.get(date) || [];
            const pnl = dailyTrades.reduce((acc, t) => acc + t.result, 0);
            const journal = journalMap.get(date);
            const intensity = journal ? journal.intensity : null;
            const emotionId = journal?.emotional_state_id;
            const emotionName = emotionId
                ? emotionalStates.find((e) => e.id === emotionId)?.name
                : "";
            return { date, pnl, intensity, emotionName };
        });

        console.log(
            `[PsychologyHub] correlationChartOption optimized calculation in ${performance.now() - t0}ms`,
        );

        return {
            backgroundColor: "transparent",
            tooltip: { trigger: "axis", axisPointer: { type: "cross" } },
            xAxis: {
                type: "category",
                data: data.map((d) => d.date),
                axisLabel: {
                    color: "#888",
                    fontSize: 10,
                    formatter: function (value: string) {
                        try {
                            const date = new Date(value + "T12:00:00");
                            return date
                                .toLocaleDateString($locale || "pt-BR", {
                                    day: "2-digit",
                                    month: "short",
                                })
                                .replace(".", "");
                        } catch (e) {
                            return value;
                        }
                    },
                },
            },
            yAxis: [
                {
                    type: "value",
                    name: $t("general.result"),
                    axisLabel: { color: "#888" },
                },
                {
                    type: "value",
                    name:
                        $t("general.intensityLabel") ||
                        $t("general.intensity_short") ||
                        "Intensidade",
                    min: 0,
                    max: 10,
                    axisLabel: { color: "#888" },
                },
            ],
            series: [
                {
                    name: $t("general.result"),
                    type: "bar",
                    data: data.map((d) => d.pnl),
                    itemStyle: {
                        color: (p: any) =>
                            p.value >= 0 ? "#10b981" : "#f43f5e",
                    },
                },
                {
                    name:
                        $t("general.intensityLabel") ||
                        $t("general.intensity_short") ||
                        "Intensidade",
                    type: "line",
                    yAxisIndex: 1,
                    data: data.map((d) => d.intensity),
                    smooth: true,
                    lineStyle: { color: "#3b82f6" },
                },
            ],
        } as EChartsOption;
    });

    // --- KPI & Summary Logic (Restored from Backup) ---
    const statsByEmotion = $derived.by(() => {
        if (settingsStore.isLoadingData) return [];
        const stats = new Map<
            string,
            {
                id: string;
                name: string;
                color: string;
                count: number;
                wins: number;
                losses: number;
                totalResult: number;
                impact: string;
            }
        >();

        const statesMap = new Map<string, any>();
        for (const s of emotionalStates || []) statesMap.set(s.id, s);

        for (const state of emotionalStates || []) {
            let stateColor = "#71717a"; // Neutral/Zinc-500 fallback
            if (state.impact === "Positive")
                stateColor = "#10b981"; // Emerald-500
            else if (state.impact === "Negative") stateColor = "#ef4444"; // Red-500

            stats.set(state.id, {
                id: state.id,
                name: state.name,
                color: stateColor,
                count: 0,
                wins: 0,
                losses: 0,
                totalResult: 0,
                impact: state.impact || "Neutral",
            });
        }

        // Ensure unknown is always there
        stats.set("unknown", {
            id: "unknown",
            name: "Não informado",
            color: "#71717a",
            count: 0,
            wins: 0,
            losses: 0,
            totalResult: 0,
            impact: "Neutral",
        });

        for (const trade of filteredTrades || []) {
            const stateId = trade.entry_emotional_state_id || "unknown";
            const current = stats.get(stateId);
            if (current) {
                current.count++;
                current.totalResult += Number(trade.result) || 0;
                if (trade.result > 0) current.wins++;
                else if (trade.result < 0) current.losses++;
            }
        }

        return Array.from(stats.values()).filter(
            (s) => s.count > 0 || s.id !== "unknown",
        );
    });

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
        filteredTrades.filter(
            (t) =>
                t.entry_emotional_state_id &&
                emotionalStates.find((e) => e.id === t.entry_emotional_state_id)
                    ?.impact === "Negative",
        ),
    );

    const tiltResult = $derived(
        tiltTrades.reduce((acc, t) => acc + t.result, 0),
    );

    const winRateChartOption = $derived.by(() => {
        const data = statsByEmotion.map((s) => ({
            name: s.name,
            value:
                s.count > 0
                    ? parseFloat(((s.wins / s.count) * 100).toFixed(1))
                    : 0,
            color:
                s.impact === "Positive"
                    ? "#10b981"
                    : s.impact === "Negative"
                      ? "#f43f5e"
                      : "#64748b",
        }));

        return {
            backgroundColor: "transparent",
            tooltip: { trigger: "axis", formatter: "{b}: {c}%" },
            xAxis: {
                type: "category",
                data: data.map((d) => d.name),
                axisLabel: { color: "#888", fontSize: 10 },
            },
            yAxis: {
                type: "value",
                max: 100,
                axisLabel: {
                    formatter: "{value}%",
                    color: "#888",
                    fontSize: 10,
                },
            },
            series: [
                {
                    data: data.map((d) => d.value),
                    type: "bar",
                    barWidth: "40%",
                    itemStyle: {
                        borderRadius: [4, 4, 0, 0],
                        color: (p: any) => data[p.dataIndex].color,
                    },
                },
            ],
        } as EChartsOption;
    });

    const resultByEmotionChartOption = $derived.by(() => {
        const data = statsByEmotion.map((s) => ({
            name: s.name,
            value: parseFloat(s.totalResult.toFixed(2)),
        }));

        return {
            backgroundColor: "transparent",
            tooltip: {
                trigger: "axis",
                formatter: (params: any) =>
                    `${params[0].name}: R$ ${params[0].value.toLocaleString($locale || "pt-BR")}`,
            },
            xAxis: {
                type: "category",
                data: data.map((d) => d.name),
                axisLabel: { color: "#888", fontSize: 10 },
            },
            yAxis: {
                type: "value",
                axisLabel: { color: "#888", fontSize: 10 },
            },
            series: [
                {
                    data: data.map((d) => d.value),
                    type: "bar",
                    barWidth: "40%",
                    itemStyle: {
                        borderRadius: [4, 4, 0, 0],
                        color: (p: any) =>
                            p.value >= 0 ? "#10b981" : "#f43f5e",
                    },
                },
            ],
        } as EChartsOption;
    });

    function formatCurrency(val: number, currency: string = "BRL") {
        try {
            return new Intl.NumberFormat($locale || "pt-BR", {
                style: "currency",
                currency: currency,
            }).format(val);
        } catch (e) {
            return `${currency} ${val.toLocaleString($locale || "pt-BR", { minimumFractionDigits: 2 })}`;
        }
    }

    const isLoading = $derived(
        tradesStore.isLoading || settingsStore.isLoadingData,
    );
</script>

<DailyCheckinDialog bind:open={showCheckinDialog} />

<DeleteConfirmationModal
    bind:open={showDeleteConfirm}
    onConfirm={confirmDeleteJournal}
    title={$t("general.confirmDelete") || "Confirmar Exclusão"}
    description="Você tem certeza que deseja excluir este registro diário psicológico? Esta ação não pode ser desfeita."
/>

<Dialog.Root bind:open={isDeleteWithClosureOpen}>
    <Dialog.Content
        class="max-w-[425px] w-full bg-popover/90 border-border backdrop-blur-xl shadow-2xl"
    >
        <Dialog.Header>
            <Dialog.Title class="text-foreground">
                {$t("general.confirmDelete") || "Excluir Registro Psicológico"}
            </Dialog.Title>
            <Dialog.Description class="text-muted-foreground">
                Este registro psicológico possui um <strong
                    class="text-foreground">Fechamento Diário Financeiro</strong
                > associado. Você deseja excluir apenas o registro psicológico ou
                ambos?
            </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer class="flex flex-col sm:flex-row gap-2 mt-4">
            <Button
                variant="outline"
                onclick={() => (isDeleteWithClosureOpen = false)}
                >{$t("general.cancel")}</Button
            >
            <Button
                variant="secondary"
                onclick={() => confirmDeleteJournalWithClosure(false)}
            >
                Só Psicológico
            </Button>
            <Button
                variant="destructive"
                onclick={() => confirmDeleteJournalWithClosure(true)}
            >
                Excluir Ambos
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

<div class="space-y-6 animate-in fade-in duration-500">
    <div class="flex-1 flex flex-col space-y-8 p-4 md:p-8">
        <div
            class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4"
        >
            <div class="space-y-1">
                <h2 class="text-3xl font-bold text-foreground tracking-tight">
                    {$t("psychology.title")}
                </h2>
                <p class="text-muted-foreground">
                    {$t("psychology.description")}
                </p>
            </div>
        </div>

        <div
            class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4"
        >
            <div class="flex items-center gap-2">
                <Button variant="outline" size="sm" onclick={syncWeights}>
                    <TrendingUp class="w-3 h-3 mr-1" />
                    {$t("psychology.syncWeights")}
                </Button>
                <Button onclick={() => (showCheckinDialog = true)} size="sm">
                    <Brain class="w-4 h-4 mr-2" />
                    {$t("psychology.checkin.button")}
                </Button>
            </div>
        </div>

        <Separator class="bg-border/20" />

        <!-- KPI Cards (Padronização Estilo Financeiro) -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {#if isLoading}
                {#each Array(4) as _}
                    <Skeleton class="h-32 rounded-xl" />
                {/each}
            {:else}
                <!-- Melhor Mindset -->
                <div
                    class="card-glass border-l-4 border-l-emerald-500 overflow-hidden"
                >
                    <div class="flex items-start justify-between py-1.5 px-3">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                        >
                            {$t("psychology.kpi.bestMindset")}
                        </span>
                        <Brain class="h-3 w-3 text-emerald-500" />
                    </div>
                    <div class="py-1 px-3 pb-2">
                        <div
                            class="text-base font-mono font-bold text-emerald-500 uppercase tracking-tight leading-none"
                        >
                            {bestMindset?.name || "-"}
                        </div>
                        <p
                            class="text-[10px] text-muted-foreground mt-1 underline decoration-emerald-500/30 underline-offset-2 decoration-dotted"
                        >
                            {$t("psychology.kpi.consolidated")}:
                            <span class="font-mono font-bold"
                                >{formatCurrency(
                                    bestMindset?.totalResult || 0,
                                )}</span
                            >
                        </p>
                    </div>
                </div>

                <!-- Pior Mindset -->
                <div
                    class="card-glass border-l-4 border-l-rose-500 overflow-hidden"
                >
                    <div class="flex items-start justify-between py-1.5 px-3">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                        >
                            {$t("psychology.kpi.worstMindset")}
                        </span>
                        <AlertTriangle class="h-3 w-3 text-rose-500" />
                    </div>
                    <div class="py-1 px-3 pb-2">
                        <div
                            class="text-base font-mono font-bold text-rose-500 uppercase tracking-tight leading-none"
                        >
                            {worstMindset?.name || "-"}
                        </div>
                        <p
                            class="text-[10px] text-muted-foreground mt-1 underline decoration-rose-500/30 underline-offset-2 decoration-dotted"
                        >
                            {$t("psychology.kpi.accumulatedLoss")}:
                            <span class="font-mono font-bold"
                                >{formatCurrency(
                                    worstMindset?.totalResult || 0,
                                )}</span
                            >
                        </p>
                    </div>
                </div>

                <!-- Custo do Tilt -->
                <div
                    class="card-glass border-l-4 border-l-orange-500 overflow-hidden"
                >
                    <div class="flex items-start justify-between py-1.5 px-3">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                        >
                            {$t("psychology.kpi.tiltCost")}
                        </span>
                        <TrendingDown class="h-3 w-3 text-orange-500" />
                    </div>
                    <div class="py-1 px-3 pb-2">
                        <div
                            class="text-base font-mono font-bold text-orange-500 tracking-tight leading-none"
                        >
                            {formatCurrency(tiltResult)}
                        </div>
                        <p
                            class="text-[10px] text-muted-foreground mt-1 whitespace-nowrap opacity-70"
                        >
                            {$t("psychology.kpi.tiltDescription", {
                                values: { count: tiltTrades.length },
                            })}
                        </p>
                    </div>
                </div>

                <!-- Registros -->
                <div
                    class="card-glass border-l-4 border-l-blue-500 overflow-hidden"
                >
                    <div class="flex items-start justify-between py-1.5 px-3">
                        <span
                            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                        >
                            {$t("psychology.kpi.records")}
                        </span>
                        <CheckCircle2 class="h-3 w-3 text-blue-500" />
                    </div>
                    <div class="py-1 px-3 pb-2">
                        <div
                            class="text-base font-mono font-bold text-blue-500 tracking-tight leading-none"
                        >
                            {filteredJournal.length}
                        </div>
                        <p
                            class="text-[10px] text-muted-foreground mt-1 opacity-70"
                        >
                            {$t("psychology.kpi.recordsDescription")}
                        </p>
                    </div>
                </div>
            {/if}
        </div>

        <!-- Charts Section (3 Columns - Restored) -->
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
            {#if isLoading}
                {#each Array(3) as _}
                    <Card.Root
                        class="bg-card/50 backdrop-blur-md border-border/50 h-[300px]"
                    >
                        <Card.Content class="p-6">
                            <Skeleton class="w-full h-full" />
                        </Card.Content>
                    </Card.Root>
                {/each}
            {:else}
                <Card.Root class="card-glass h-full">
                    <Card.Header class="pb-1">
                        <Card.Title
                            class="text-[10px] font-black uppercase tracking-widest text-muted-foreground/70"
                            >{$t("psychology.charts.pnlByEmotion")}</Card.Title
                        >
                    </Card.Header>
                    <Card.Content class="h-[250px] p-2">
                        <EChart options={resultByEmotionChartOption} />
                    </Card.Content>
                </Card.Root>

                <Card.Root class="card-glass h-full">
                    <Card.Header class="pb-1">
                        <Card.Title
                            class="text-[10px] font-black uppercase tracking-widest text-muted-foreground/70"
                            >{$t("psychology.charts.winRate")}</Card.Title
                        >
                    </Card.Header>
                    <Card.Content class="h-[250px] p-2">
                        <EChart options={winRateChartOption} />
                    </Card.Content>
                </Card.Root>

                <Card.Root class="card-glass h-full">
                    <Card.Header class="pb-1">
                        <Card.Title
                            class="text-[10px] font-black uppercase tracking-widest text-muted-foreground/70"
                            >{$t("psychology.charts.correlation")}</Card.Title
                        >
                    </Card.Header>
                    <Card.Content class="h-[250px] p-2">
                        <EChart options={correlationChartOption} />
                    </Card.Content>
                </Card.Root>
            {/if}
        </div>

        <!-- Detailed Analysis & Filters -->
        <div
            class="flex flex-col sm:flex-row gap-4 justify-between items-end card-glass p-4"
        >
            <div class="space-y-1">
                <h3
                    class="text-sm font-black uppercase tracking-widest text-muted-foreground"
                >
                    {$t("psychology.analysis.title")}
                </h3>
                <p
                    class="text-[10px] text-muted-foreground/60 font-bold uppercase tracking-tighter"
                >
                    {$t("psychology.analysis.hierarchy")}
                </p>
            </div>

            <div class="flex items-center gap-2">
                <DateFilter
                    bind:value={timeFilter}
                    bind:startDate
                    bind:endDate
                />
                <Select.Root type="single" bind:value={itemsLimit}>
                    <Select.Trigger
                        class="w-[120px] h-9 bg-background/50 border-border/50 text-[10px] font-bold uppercase"
                    >
                        {itemsLimit === "all"
                            ? $t("general.all")
                            : `${itemsLimit} items`}
                    </Select.Trigger>
                    <Select.Content class="bg-popover border-border">
                        <Select.Item
                            value="10"
                            class="text-[10px] font-bold uppercase"
                            >10 Itens</Select.Item
                        >
                        <Select.Item
                            value="25"
                            class="text-[10px] font-bold uppercase"
                            >25 Itens</Select.Item
                        >
                        <Select.Item
                            value="50"
                            class="text-[10px] font-bold uppercase"
                            >50 Itens</Select.Item
                        >
                        <Select.Item
                            value="all"
                            class="text-[10px] font-bold uppercase"
                            >TUDO</Select.Item
                        >
                    </Select.Content>
                </Select.Root>
            </div>
        </div>

        <!-- Main Split Content -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 pb-20">
            <!-- Left: Hierarchical Analysis -->
            <div class="lg:col-span-8 space-y-4">
                {#if hierarchicalPsychologyData.length === 0}
                    <div
                        class="h-40 flex items-center justify-center text-muted-foreground border-2 border-dashed rounded-xl border-border/50 uppercase text-[10px] font-black"
                    >
                        {$t("general.noData")}
                    </div>
                {:else}
                    <div class="space-y-4">
                        {#each hierarchicalPsychologyData as month}
                            {@const isMonthExpanded = expandedMonths.has(
                                month.key,
                            )}
                            <div
                                class="card-glass border border-border/50 overflow-hidden"
                            >
                                <!-- Month Header -->
                                <button
                                    class="w-full flex items-center justify-between p-3 rounded-xl card-glass border-primary/20 hover:bg-primary/15 transition-colors sticky top-0 z-10 backdrop-blur-md"
                                    onclick={() => toggleMonth(month.key)}
                                >
                                    <div
                                        class="flex items-center gap-4 animate-in fade-in slide-in-from-bottom-2"
                                    >
                                        <div
                                            class="p-2 rounded-lg bg-primary/20"
                                        >
                                            <Calendar
                                                class="w-4 h-4 text-primary"
                                            />
                                        </div>
                                        <div class="text-left">
                                            <h4
                                                class="text-sm font-black text-foreground uppercase tracking-tight"
                                            >
                                                {month.label}
                                            </h4>
                                            <div
                                                class="flex items-center gap-2 mt-0.5"
                                            >
                                                <Badge
                                                    variant="outline"
                                                    class="text-[9px] px-1.5 h-4 bg-muted border-border font-bold uppercase"
                                                >
                                                    {month.weeks.length}
                                                    {$t("general.weeks_upper")}
                                                </Badge>
                                                <div
                                                    class="flex gap-1.5 opacity-60"
                                                >
                                                    {#each Object.entries(month.totalPnlByCurrency) as [curr, total]}
                                                        <span
                                                            class="text-[9px] font-bold {(total as number) >=
                                                            0
                                                                ? 'text-emerald-500'
                                                                : 'text-rose-500'}"
                                                        >
                                                            {formatCurrency(
                                                                total as number,
                                                                curr,
                                                            )}
                                                        </span>
                                                    {/each}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-6">
                                        <div class="flex items-center gap-2">
                                            <div
                                                class="flex flex-col items-end mr-2"
                                            >
                                                <div
                                                    class="flex items-baseline gap-1.5"
                                                >
                                                    <span
                                                        class="text-xs font-black text-foreground"
                                                    >
                                                        {month.avgWeight.toFixed(
                                                            1,
                                                        )}
                                                    </span>
                                                    <span
                                                        class="text-[7px] font-bold text-muted-foreground uppercase tracking-tighter"
                                                        >SCORE</span
                                                    >
                                                </div>
                                                <Badge
                                                    class="text-[7px] h-3.5 px-1 font-black uppercase {month
                                                        .equivalentState
                                                        .impact === 'Positive'
                                                        ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                                                        : 'bg-rose-500/10 text-red-400 border-rose-500/20'}"
                                                    variant="outline"
                                                >
                                                    {month.equivalentState.name}
                                                </Badge>
                                            </div>
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="h-7 w-7 hover:bg-accent/10"
                                                onclick={(e) => {
                                                    e.stopPropagation();
                                                    openInsight(month);
                                                }}
                                            >
                                                <Eye
                                                    class="w-3.5 h-3.5 text-muted-foreground hover:text-foreground"
                                                />
                                            </Button>
                                        </div>
                                        <ChevronDown
                                            class="w-4 h-4 text-muted-foreground transition-transform duration-300 {isMonthExpanded
                                                ? 'rotate-180'
                                                : ''}"
                                        />
                                    </div>
                                </button>

                                {#if isMonthExpanded}
                                    <div
                                        class="pl-4 pb-4 space-y-3 border-l-2 border-border/30 ml-6 animate-in fade-in slide-in-from-top-2"
                                    >
                                        <Separator class="bg-border/20 mb-3" />
                                        {#each month.weeks as week}
                                            {@const isWeekExpanded =
                                                expandedWeeks.has(week.key)}
                                            <div
                                                class="card-glass border border-border/40 overflow-hidden"
                                            >
                                                <button
                                                    class="w-full flex items-center justify-between p-3 hover:bg-primary/10 transition-colors"
                                                    onclick={() =>
                                                        toggleWeek(week.key)}
                                                >
                                                    <div
                                                        class="flex items-center gap-3 animate-in fade-in slide-in-from-bottom-2"
                                                    >
                                                        <TrendingUp
                                                            class="w-3.5 h-3.5 text-emerald-400"
                                                        />
                                                        <div class="text-left">
                                                            <h5
                                                                class="text-xs font-black text-foreground/90 uppercase tracking-tight"
                                                            >
                                                                {week.label}
                                                            </h5>
                                                            <div
                                                                class="flex gap-1.5 opacity-60 mt-0.5"
                                                            >
                                                                {#each Object.entries(week.totalPnlByCurrency) as [curr, total]}
                                                                    <span
                                                                        class="text-[9px] font-bold {(total as number) >=
                                                                        0
                                                                            ? 'text-emerald-500'
                                                                            : 'text-rose-500'}"
                                                                    >
                                                                        {formatCurrency(
                                                                            total as number,
                                                                            curr,
                                                                        )}
                                                                    </span>
                                                                {/each}
                                                            </div>
                                                        </div>
                                                    </div>
                                                    <div
                                                        class="flex items-center gap-4"
                                                    >
                                                        <div
                                                            class="flex items-center gap-2"
                                                        >
                                                            <div
                                                                class="flex flex-col items-end mr-1"
                                                            >
                                                                <div
                                                                    class="flex items-baseline gap-1"
                                                                >
                                                                    <span
                                                                        class="text-[10px] font-black text-foreground"
                                                                    >
                                                                        {week.avgWeight.toFixed(
                                                                            1,
                                                                        )}
                                                                    </span>
                                                                    <span
                                                                        class="text-[6px] font-bold text-muted-foreground uppercase tracking-tighter"
                                                                        >SCORE</span
                                                                    >
                                                                </div>
                                                                <Badge
                                                                    class="text-[6px] h-3 px-1 font-black uppercase {week
                                                                        .equivalentState
                                                                        .impact ===
                                                                    'Positive'
                                                                        ? 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20'
                                                                        : 'bg-rose-500/10 text-rose-500 border-rose-500/20'}"
                                                                    variant="outline"
                                                                >
                                                                    {week
                                                                        .equivalentState
                                                                        .name}
                                                                </Badge>
                                                            </div>
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-6 w-6 hover:bg-accent/10"
                                                                onclick={(
                                                                    e,
                                                                ) => {
                                                                    e.stopPropagation();
                                                                    openInsight(
                                                                        week,
                                                                    );
                                                                }}
                                                            >
                                                                <Eye
                                                                    class="w-3 h-3 text-muted-foreground hover:text-foreground"
                                                                />
                                                            </Button>
                                                        </div>
                                                        <ChevronDown
                                                            class="w-3.5 h-3.5 text-muted-foreground transition-transform {isWeekExpanded
                                                                ? 'rotate-180'
                                                                : ''}"
                                                        />
                                                    </div>
                                                </button>

                                                {#if isWeekExpanded}
                                                    <div
                                                        class="pl-4 pb-2 space-y-2 border-l-2 border-border/20 ml-6 animate-in fade-in slide-in-from-top-1"
                                                    >
                                                        {#each week.days as day}
                                                            {@const isDayExpanded =
                                                                expandedDays.has(
                                                                    day.date,
                                                                )}
                                                            <div
                                                                class="card-glass border border-border/50 overflow-hidden"
                                                            >
                                                                <button
                                                                    class="w-full flex items-center justify-between p-2 sm:p-3 hover:bg-primary/10 transition-colors text-left"
                                                                    onclick={() =>
                                                                        toggleDay(
                                                                            day.date,
                                                                        )}
                                                                >
                                                                    <!-- LHS: Date Badge & Summary -->
                                                                    <div
                                                                        class="flex items-center gap-4"
                                                                    >
                                                                        <div
                                                                            class="flex flex-col items-center justify-center bg-muted/80 rounded-lg h-9 w-9 border border-border/50 shadow-inner"
                                                                        >
                                                                            <span
                                                                                class="text-[10px] font-black leading-none text-muted-foreground/80"
                                                                                >{new Date(
                                                                                    day.date +
                                                                                        "T12:00:00",
                                                                                )
                                                                                    .toLocaleString(
                                                                                        $locale ||
                                                                                            "pt-BR",
                                                                                        {
                                                                                            weekday:
                                                                                                "short",
                                                                                        },
                                                                                    )
                                                                                    .toUpperCase()}</span
                                                                            >
                                                                            <span
                                                                                class="text-sm font-black leading-none mt-0.5 text-foreground"
                                                                                >{new Date(
                                                                                    day.date +
                                                                                        "T12:00:00",
                                                                                ).getDate()}</span
                                                                            >
                                                                        </div>
                                                                        <div
                                                                            class="flex flex-col items-start px-1"
                                                                        >
                                                                            <span
                                                                                class="text-xs font-bold text-foreground"
                                                                                >{$t(
                                                                                    "general.dayClosure",
                                                                                )}</span
                                                                            >
                                                                            <div
                                                                                class="flex items-center gap-2"
                                                                            >
                                                                                <span
                                                                                    class="text-[9px] font-medium text-muted-foreground uppercase tracking-widest whitespace-nowrap"
                                                                                    >{day
                                                                                        .trades
                                                                                        .length}
                                                                                    trades</span
                                                                                >
                                                                                <span
                                                                                    class="w-1 h-1 rounded-full bg-border"

                                                                                ></span>
                                                                                <div
                                                                                    class="flex gap-1.5 opacity-60"
                                                                                >
                                                                                    {#each Object.entries(day.totalPnlByCurrency) as [curr, total]}
                                                                                        <span
                                                                                            class="text-[9px] font-bold {(total as number) >=
                                                                                            0
                                                                                                ? 'text-emerald-500'
                                                                                                : 'text-rose-500'}"
                                                                                        >
                                                                                            {formatCurrency(
                                                                                                total as number,
                                                                                                curr,
                                                                                            )}
                                                                                        </span>
                                                                                    {/each}
                                                                                </div>
                                                                            </div>
                                                                        </div>
                                                                    </div>

                                                                    <!-- RHS: PnL & Actions -->
                                                                    <div
                                                                        class="flex items-center gap-6"
                                                                    >
                                                                        <div
                                                                            class="flex items-center gap-4 animate-in fade-in slide-in-from-bottom-2"
                                                                        >
                                                                            <div
                                                                                class="flex items-center gap-2 mr-2"
                                                                            >
                                                                                {#if day.equivalentState}
                                                                                    <div
                                                                                        class="flex flex-col items-end"
                                                                                    >
                                                                                        <div
                                                                                            class="flex items-baseline gap-1.5"
                                                                                        >
                                                                                            <span
                                                                                                class="text-xs font-black text-foreground"
                                                                                            >
                                                                                                {day.avgWeight.toFixed(
                                                                                                    1,
                                                                                                )}
                                                                                            </span>
                                                                                            <span
                                                                                                class="text-[7px] font-bold text-muted-foreground uppercase tracking-tighter"
                                                                                                >SCORE</span
                                                                                            >
                                                                                        </div>
                                                                                        <Badge
                                                                                            class="text-[7px] h-3.5 px-1 font-black uppercase {day
                                                                                                .equivalentState
                                                                                                .impact ===
                                                                                            'Positive'
                                                                                                ? 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20'
                                                                                                : 'bg-rose-500/10 text-rose-500 border-rose-500/20'}"
                                                                                            variant="outline"
                                                                                        >
                                                                                            {day
                                                                                                .equivalentState
                                                                                                .name}
                                                                                        </Badge>
                                                                                    </div>
                                                                                {/if}
                                                                                <Button
                                                                                    variant="ghost"
                                                                                    size="icon"
                                                                                    class="h-7 w-7 hover:bg-white/10"
                                                                                    onclick={(
                                                                                        e,
                                                                                    ) => {
                                                                                        e.stopPropagation();
                                                                                        openInsight(
                                                                                            day,
                                                                                        );
                                                                                    }}
                                                                                >
                                                                                    <Eye
                                                                                        class="w-3.5 h-3.5 text-muted-foreground"
                                                                                    />
                                                                                </Button>
                                                                            </div>
                                                                            <div
                                                                                class="w-6 h-6 rounded-full bg-muted/50 flex items-center justify-center transition-transform {isDayExpanded
                                                                                    ? 'rotate-180'
                                                                                    : ''}"
                                                                            >
                                                                                <ChevronDown
                                                                                    class="w-3 h-3 text-muted-foreground"
                                                                                />
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </button>

                                                                {#if isDayExpanded}
                                                                    <div
                                                                        class="px-4 pb-4 animate-in fade-in slide-in-from-top-2"
                                                                    >
                                                                        <div
                                                                            class="rounded-lg border border-border/40 overflow-hidden bg-background/40 mt-1"
                                                                        >
                                                                            <table
                                                                                class="w-full text-left"
                                                                            >
                                                                                <thead
                                                                                    class="bg-muted/50 h-7 border-b border-border/20"
                                                                                >
                                                                                    <tr
                                                                                    >
                                                                                        <th
                                                                                            class="px-3 text-[8px] font-black text-muted-foreground uppercase"
                                                                                            >{$t(
                                                                                                "general.asset",
                                                                                            )}</th
                                                                                        >
                                                                                        <th
                                                                                            class="px-3 text-[8px] font-black text-muted-foreground uppercase"
                                                                                            >{$t(
                                                                                                "psychology.analysis.emotionalCalc",
                                                                                            )}</th
                                                                                        >
                                                                                        <th
                                                                                            class="px-3 text-[8px] font-black text-muted-foreground uppercase text-right"
                                                                                            >{$t(
                                                                                                "psychology.analysis.totalWeight",
                                                                                            )}</th
                                                                                        >
                                                                                    </tr>
                                                                                </thead>
                                                                                <tbody
                                                                                >
                                                                                    {#each day.trades as trade}
                                                                                        <tr
                                                                                            class="h-8 border-b border-border/10 last:border-0 hover:bg-primary/10 transition-colors"
                                                                                        >
                                                                                            <td
                                                                                                class="px-3 text-[10px] font-bold text-foreground uppercase"
                                                                                                >{trade.asset_symbol}</td
                                                                                            >
                                                                                            <td
                                                                                                class="px-3 py-1.5"
                                                                                            >
                                                                                                {#if trade.entry_emotional_state_id}
                                                                                                    {@const st =
                                                                                                        emotionalStates.find(
                                                                                                            (
                                                                                                                s,
                                                                                                            ) =>
                                                                                                                s.id ===
                                                                                                                trade.entry_emotional_state_id,
                                                                                                        )}
                                                                                                    <div
                                                                                                        class="flex items-center gap-2"
                                                                                                    >
                                                                                                        <span
                                                                                                            class="text-[9px] font-bold text-muted-foreground uppercase"
                                                                                                            >{st?.name}</span
                                                                                                        >
                                                                                                        <span
                                                                                                            class="text-[8px] text-muted-foreground/60 font-medium lowercase italic"
                                                                                                        >
                                                                                                            ({st?.weight}
                                                                                                            {$t(
                                                                                                                "general.weight",
                                                                                                            )}
                                                                                                            x
                                                                                                            {(
                                                                                                                trade.intensity ||
                                                                                                                5
                                                                                                            ).toFixed(
                                                                                                                1,
                                                                                                            )}
                                                                                                            {$t(
                                                                                                                "general.intensity_short",
                                                                                                            )}
                                                                                                        </span>
                                                                                                    </div>
                                                                                                {/if}
                                                                                            </td>
                                                                                            <td
                                                                                                class="px-3 text-[10px] font-black text-right text-foreground"
                                                                                            >
                                                                                                {#if trade.entry_emotional_state_id}
                                                                                                    {@const st =
                                                                                                        emotionalStates.find(
                                                                                                            (
                                                                                                                s,
                                                                                                            ) =>
                                                                                                                s.id ===
                                                                                                                trade.entry_emotional_state_id,
                                                                                                        )}
                                                                                                    {(
                                                                                                        (st?.weight ||
                                                                                                            5) *
                                                                                                        (trade.intensity ||
                                                                                                            5)
                                                                                                    ).toFixed(
                                                                                                        1,
                                                                                                    )}
                                                                                                {/if}
                                                                                            </td>
                                                                                        </tr>
                                                                                    {/each}
                                                                                </tbody>
                                                                            </table>
                                                                        </div>
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

            <!-- Right: Journal List -->
            <div class="lg:col-span-4 space-y-4">
                <h3
                    class="text-sm font-black uppercase tracking-widest text-muted-foreground/60"
                >
                    {$t("psychology.journal.title")}
                </h3>
                <div
                    class="rounded-xl border border-border/40 bg-card/20 backdrop-blur-sm overflow-hidden"
                >
                    <div class="max-h-[70vh] overflow-y-auto">
                        <table class="w-full text-left">
                            <thead
                                class="bg-muted/80 border-b border-border/40 sticky top-0 backdrop-blur-md"
                            >
                                <tr class="h-8">
                                    <th
                                        class="px-3 text-[9px] font-black text-muted-foreground uppercase"
                                        >{$t("general.date")}</th
                                    >
                                    <th
                                        class="px-3 text-[9px] font-black text-muted-foreground uppercase w-32"
                                        >{$t(
                                            "psychology.journal.entryStateShort",
                                        )}</th
                                    >
                                    <th
                                        class="px-3 text-[9px] font-black text-muted-foreground uppercase w-32"
                                        >{$t(
                                            "psychology.journal.exitStateShort",
                                        )}</th
                                    >
                                    <th
                                        class="px-3 text-[9px] font-black text-muted-foreground uppercase text-right"
                                        >{$t(
                                            "psychology.journal.intensityShort",
                                        )}</th
                                    >
                                    <th
                                        class="px-3 text-[9px] font-black text-muted-foreground uppercase text-right"
                                        >{$t("psychology.journal.score")}</th
                                    >
                                    <th
                                        class="px-3 text-[9px] font-black text-muted-foreground uppercase text-right"
                                        >{$t("general.actions")}</th
                                    >
                                </tr>
                            </thead>
                            <tbody>
                                {#each filteredJournal
                                    .slice()
                                    .sort( (a, b) => b.date.localeCompare(a.date), ) as entry}
                                    {@const em = emotionalStates.find(
                                        (s) =>
                                            s.id === entry.emotional_state_id,
                                    )}
                                    {@const itemScore = em?.weight ?? 5}
                                    {@const itemIntensity =
                                        entry.intensity || 5}
                                    <tr
                                        class="h-10 border-b border-border/10 hover:bg-primary/10 transition-colors group"
                                    >
                                        <td class="px-3">
                                            <div class="flex flex-col">
                                                <span
                                                    class="text-[10px] font-black text-foreground uppercase"
                                                >
                                                    {new Date(
                                                        entry.date +
                                                            "T12:00:00",
                                                    ).toLocaleDateString(
                                                        $locale || "pt-BR",
                                                        {
                                                            day: "2-digit",
                                                            month: "2-digit",
                                                        },
                                                    )}
                                                </span>
                                                <span
                                                    class="text-[8px] text-muted-foreground/50 font-medium"
                                                >
                                                    {new Date(
                                                        entry.date +
                                                            "T12:00:00",
                                                    ).toLocaleDateString(
                                                        $locale || "pt-BR",
                                                        { weekday: "short" },
                                                    )}
                                                </span>
                                            </div>
                                        </td>
                                        <td class="px-3">
                                            {#if em}
                                                <div
                                                    class="flex items-center gap-1.5"
                                                >
                                                    <span
                                                        class="text-[9px] font-black uppercase {em.impact ===
                                                        'Positive'
                                                            ? 'text-emerald-400'
                                                            : 'text-red-400'}"
                                                        >{em.name}</span
                                                    >
                                                    <span
                                                        class="text-[8px] text-muted-foreground font-medium"
                                                        >({em.weight ||
                                                            5})</span
                                                    >
                                                </div>
                                            {:else}
                                                <span
                                                    class="text-[8px] text-muted-foreground/30 italic font-bold"
                                                    >- -</span
                                                >
                                            {/if}
                                        </td>
                                        <td class="px-3">
                                            <span
                                                class="text-[8px] text-muted-foreground/20 font-bold"
                                                >N/A</span
                                            >
                                        </td>
                                        <td
                                            class="px-3 text-[10px] font-medium text-right text-muted-foreground/60"
                                        >
                                            {itemIntensity.toFixed(1)}
                                        </td>
                                        <td
                                            class="px-3 text-[10px] font-black text-right text-foreground"
                                        >
                                            {itemScore.toFixed(1)}
                                        </td>
                                        <td class="px-3 text-right">
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity"
                                                onclick={() =>
                                                    requestDeleteJournal(
                                                        entry.id,
                                                    )}
                                            >
                                                <Trash2
                                                    class="w-3 h-3 text-red-400"
                                                />
                                            </Button>
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<Dialog.Root bind:open={showInsightModal}>
    <Dialog.Content
        class="max-w-2xl bg-popover/90 border-border text-foreground p-0 overflow-hidden backdrop-blur-md"
    >
        {#if insightData}
            <div class="p-6 border-b border-border/40 bg-muted/30">
                <div class="flex items-center justify-between">
                    <div>
                        <h2
                            class="text-xl font-black uppercase tracking-tighter"
                        >
                            {$t("psychology.insight.title")}
                        </h2>
                        <p
                            class="text-xs font-bold text-muted-foreground/60 uppercase mt-1"
                        >
                            {insightData.label}
                        </p>
                    </div>
                    <div class="text-right">
                        <div class="flex items-baseline justify-end gap-1.5">
                            <span class="text-2xl font-black text-foreground"
                                >{insightData.avgWeight.toFixed(1)}</span
                            >
                            <span
                                class="text-[10px] font-bold text-muted-foreground uppercase"
                                >SCORE</span
                            >
                        </div>
                        <Badge
                            class="text-[9px] font-black uppercase {insightData
                                .equivalentState.impact === 'Positive'
                                ? 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20'
                                : 'bg-rose-500/10 text-rose-500 border-rose-500/20'}"
                            variant="outline"
                        >
                            {insightData.equivalentState.name}
                        </Badge>
                    </div>
                </div>
            </div>

            <div class="p-4 max-h-[60vh] overflow-y-auto custom-scrollbar">
                <table class="w-full text-left">
                    <thead class="bg-card/80 sticky top-0 backdrop-blur-md">
                        <tr class="h-8 border-b border-border/40">
                            <th
                                class="px-3 text-[9px] font-black text-muted-foreground uppercase"
                                >{$t("general.origin")}</th
                            >
                            <th
                                class="px-3 text-[9px] font-black text-muted-foreground uppercase w-32"
                                >{$t("psychology.journal.entryStateShort")}</th
                            >
                            <th
                                class="px-3 text-[9px] font-black text-muted-foreground uppercase w-32"
                                >{$t("psychology.journal.exitStateShort")}</th
                            >
                            <th
                                class="px-3 text-[9px] font-black text-muted-foreground uppercase text-right"
                                >{$t("psychology.journal.intensityShort")}</th
                            >
                            <th
                                class="px-3 text-[9px] font-black text-muted-foreground uppercase text-right"
                                >{$t("psychology.insight.opScore")}</th
                            >
                        </tr>
                    </thead>
                    <tbody>
                        {#each insightData.items as item}
                            {@const isTrade = !!item.asset_symbol}
                            {@const entrySt = emotionalStates.find(
                                (s) =>
                                    s.id ===
                                    (isTrade
                                        ? item.entry_emotional_state_id
                                        : item.emotional_state_id),
                            )}
                            {@const exitSt = isTrade
                                ? emotionalStates.find(
                                      (s) =>
                                          s.id === item.exit_emotional_state_id,
                                  )
                                : null}
                            {@const itemIntensity = item.intensity || 5}

                            {#if entrySt || exitSt}
                                {@const entryWeight = entrySt?.weight ?? null}
                                {@const exitWeight = exitSt?.weight ?? null}
                                {@const weights = [
                                    entryWeight,
                                    exitWeight,
                                ].filter((w) => w !== null)}
                                {@const itemScore =
                                    weights.length > 0
                                        ? weights.reduce((a, b) => a + b, 0) /
                                          weights.length
                                        : 5}

                                <tr
                                    class="h-10 border-b border-border/10 hover:bg-primary/10 transition-colors"
                                >
                                    <td class="px-3">
                                        <div class="flex flex-col">
                                            <span
                                                class="text-[10px] font-bold text-foreground uppercase"
                                            >
                                                {item.asset_symbol ||
                                                    $t(
                                                        "general.day",
                                                    ).toUpperCase()}
                                            </span>
                                            <span
                                                class="text-[8px] text-muted-foreground/60 font-medium"
                                            >
                                                {new Date(
                                                    item.date,
                                                ).toLocaleDateString(
                                                    $locale || "pt-BR",
                                                    {
                                                        day: "2-digit",
                                                        month: "2-digit",
                                                    },
                                                )}
                                            </span>
                                        </div>
                                    </td>
                                    <td class="px-3">
                                        {#if entrySt}
                                            <div
                                                class="flex items-center gap-1.5"
                                            >
                                                <span
                                                    class="text-[9px] font-black uppercase {entrySt.impact ===
                                                    'Positive'
                                                        ? 'text-emerald-400'
                                                        : 'text-red-400'}"
                                                    >{entrySt.name}</span
                                                >
                                                <span
                                                    class="text-[8px] text-muted-foreground font-medium"
                                                    >({entrySt.weight ||
                                                        5})</span
                                                >
                                            </div>
                                        {:else}
                                            <span
                                                class="text-[8px] text-muted-foreground/30 italic font-bold"
                                                >- -</span
                                            >
                                        {/if}
                                    </td>
                                    <td class="px-3">
                                        {#if exitSt}
                                            <div
                                                class="flex items-center gap-1.5"
                                            >
                                                <span
                                                    class="text-[9px] font-black uppercase {exitSt.impact ===
                                                    'Positive'
                                                        ? 'text-emerald-400'
                                                        : 'text-red-400'}"
                                                    >{exitSt.name}</span
                                                >
                                                <span
                                                    class="text-[8px] text-muted-foreground font-medium"
                                                    >({exitSt.weight ||
                                                        5})</span
                                                >
                                            </div>
                                        {:else if isTrade}
                                            <span
                                                class="text-[8px] text-muted-foreground/30 italic font-bold"
                                                >- -</span
                                            >
                                        {:else}
                                            <span
                                                class="text-[8px] text-muted-foreground/20 font-bold"
                                                >N/A</span
                                            >
                                        {/if}
                                    </td>
                                    <td
                                        class="px-3 text-[10px] font-medium text-right text-muted-foreground/60"
                                    >
                                        {itemIntensity.toFixed(1)}
                                    </td>
                                    <td
                                        class="px-3 text-[10px] font-black text-right text-foreground"
                                    >
                                        {itemScore.toFixed(1)}
                                    </td>
                                </tr>
                            {/if}
                        {/each}
                    </tbody>
                </table>
            </div>

            <div class="p-6 bg-card/40 border-t border-border/40">
                <div class="grid grid-cols-2 gap-8">
                    <div>
                        <h4
                            class="text-[10px] font-black text-muted-foreground/60 uppercase tracking-widest mb-2"
                        >
                            {$t("psychology.insight.formulaTitle")}
                        </h4>
                        <div class="flex flex-col gap-1.5">
                            <div class="flex flex-col">
                                <span
                                    class="text-[8px] font-black text-muted-foreground/40 uppercase mb-0.5"
                                    >Nota da Operação</span
                                >
                                <code
                                    class="text-[9px] text-muted-foreground/70 font-bold"
                                >
                                    (Peso Entrada + Peso Saída) / 2
                                </code>
                            </div>
                            <div class="flex flex-col">
                                <span
                                    class="text-[8px] font-black text-muted-foreground/40 uppercase mb-0.5"
                                    >Score Final</span
                                >
                                <code
                                    class="text-[9px] text-muted-foreground/40 font-bold"
                                >
                                    Σ (Notas das Operações) / Total de Itens
                                </code>
                            </div>
                        </div>
                    </div>
                    <div class="space-y-2">
                        <div
                            class="flex justify-between items-center text-[10px]"
                        >
                            <span
                                class="text-muted-foreground/50 font-bold uppercase"
                                >Soma dos Scores</span
                            >
                            <span class="text-foreground font-black">
                                {insightData.items
                                    .reduce((acc, item) => {
                                        const isTr = !!item.asset_symbol;
                                        const eSt = emotionalStates.find(
                                            (s) =>
                                                s.id ===
                                                (isTr
                                                    ? item.entry_emotional_state_id
                                                    : item.emotional_state_id),
                                        );
                                        const xSt = isTr
                                            ? emotionalStates.find(
                                                  (s) =>
                                                      s.id ===
                                                      item.exit_emotional_state_id,
                                              )
                                            : null;

                                        const eW = eSt?.weight ?? null;
                                        const xW = xSt?.weight ?? null;
                                        const weights = [eW, xW].filter(
                                            (w) => w !== null,
                                        );
                                        const iScore =
                                            weights.length > 0
                                                ? weights.reduce(
                                                      (a, b) => a + b,
                                                      0,
                                                  ) / weights.length
                                                : 5;

                                        return acc + iScore;
                                    }, 0)
                                    .toFixed(1)}
                            </span>
                        </div>
                        <div
                            class="flex justify-between items-center text-[10px]"
                        >
                            <span
                                class="text-muted-foreground/50 font-bold uppercase"
                                >Quantidade de Itens</span
                            >
                            <span class="text-foreground font-black">
                                {insightData.items.length}
                            </span>
                        </div>
                        <Separator class="bg-border/40" />
                        <div class="flex justify-between items-center">
                            <span
                                class="text-[11px] font-black text-primary uppercase"
                                >Média Final</span
                            >
                            <span class="text-lg font-black text-foreground"
                                >{insightData.avgWeight.toFixed(1)}</span
                            >
                        </div>
                    </div>
                </div>
            </div>
        {/if}
    </Dialog.Content>
</Dialog.Root>

<DailyCheckinDialog bind:open={showCheckinDialog} />

<Dialog.Root bind:open={showDayModal}>
    <Dialog.Content
        class="max-w-2xl bg-popover/90 border-border text-foreground p-0 overflow-hidden backdrop-blur-md"
    >
        {#if selectedDayData}
            <div class="p-6 border-b border-border/40 bg-muted/30">
                <div class="flex items-center justify-between">
                    <div>
                        <h2
                            class="text-xl font-black uppercase tracking-tighter"
                        >
                            Detalhes Psicológicos
                        </h2>
                        <p
                            class="text-xs font-bold text-muted-foreground/60 uppercase mt-1"
                        >
                            {new Date(
                                selectedDayData.date + "T12:00:00",
                            ).toLocaleDateString($locale || "pt-BR", {
                                weekday: "long",
                                day: "numeric",
                                month: "long",
                            })}
                        </p>
                    </div>
                    <div class="flex-1 overflow-y-auto min-h-0">
                        <div class="p-6 space-y-8">
                            <!-- Day Breakdown -->
                            <div class="space-y-4">
                                <h4
                                    class="text-xs font-black text-muted-foreground/60 uppercase tracking-widest flex items-center gap-2"
                                >
                                    <TrendingUp class="w-3 h-3" />
                                    Operações do Dia
                                    {#if selectedCurrency}
                                        <Badge
                                            variant="outline"
                                            class="text-[9px] font-black uppercase text-primary border-primary/20 bg-primary/5"
                                        >
                                            Filtro: {selectedCurrency}
                                        </Badge>
                                    {/if}
                                </h4>

                                {#each selectedDayData.trades.filter((t: any) => !selectedCurrency || settingsStore.accounts.find((a) => a.id === t.account_id)?.currency === selectedCurrency) as trade}
                                    <div
                                        class="p-4 rounded-xl border border-border/40 bg-muted/20 hover:bg-accent/10 transition-colors"
                                    >
                                        <div
                                            class="flex items-center justify-between mb-3"
                                        >
                                            <div
                                                class="flex items-center gap-3"
                                            >
                                                <Badge
                                                    class="h-5 font-black uppercase bg-primary/10 text-primary border-primary/20"
                                                    >{trade.asset_symbol}</Badge
                                                >
                                                <span
                                                    class="text-[10px] font-bold text-muted-foreground/50"
                                                >
                                                    {new Date(
                                                        trade.date,
                                                    ).toLocaleTimeString([], {
                                                        hour: "2-digit",
                                                        minute: "2-digit",
                                                    })}
                                                </span>
                                            </div>
                                            <span
                                                class="text-sm font-black {trade.result >=
                                                0
                                                    ? 'text-emerald-400'
                                                    : 'text-red-400'}"
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
                                        </div>

                                        <div
                                            class="flex items-center justify-between"
                                        >
                                            <div class="flex flex-col gap-1">
                                                <span
                                                    class="text-[9px] font-black text-muted-foreground/60 uppercase tracking-tighter"
                                                    >Entrada</span
                                                >
                                                {#if trade.entry_emotional_state_id}
                                                    {@const st =
                                                        emotionalStates.find(
                                                            (s) =>
                                                                s.id ===
                                                                trade.entry_emotional_state_id,
                                                        )}
                                                    <span
                                                        class="text-[10px] font-black uppercase {st?.impact ===
                                                        'Positive'
                                                            ? 'text-emerald-400'
                                                            : 'text-red-400'}"
                                                    >
                                                        {st?.name}
                                                    </span>
                                                {/if}
                                            </div>
                                            {#if trade.intensity}
                                                <div
                                                    class="flex flex-col gap-1"
                                                >
                                                    <span
                                                        class="text-[9px] font-black text-muted-foreground/60 uppercase tracking-tighter"
                                                        >Intensidade</span
                                                    >
                                                    <div
                                                        class="flex items-center gap-1 w-20"
                                                    >
                                                        <div
                                                            class="flex-1 h-1 bg-muted rounded-full overflow-hidden"
                                                        >
                                                            <div
                                                                class="h-full bg-primary"
                                                                style="width: {trade.intensity *
                                                                    10}%"
                                                            ></div>
                                                        </div>
                                                    </div>
                                                </div>
                                            {/if}
                                        </div>
                                        {#if trade.notes}
                                            <div
                                                class="mt-3 p-2 rounded bg-muted/40 border border-border/20"
                                            >
                                                <p
                                                    class="text-[10px] text-muted-foreground/60 italic font-medium leading-relaxed"
                                                >
                                                    "{trade.notes}"
                                                </p>
                                            </div>
                                        {/if}
                                    </div>
                                {/each}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div
                class="p-6 space-y-6 max-h-[60vh] overflow-y-auto custom-scrollbar"
            >
                <!-- Aggregate Stats -->
                <div class="grid grid-cols-2 gap-4">
                    <div
                        class="p-4 rounded-xl border border-border/40 bg-muted/20"
                    >
                        <p
                            class="text-[10px] font-black text-muted-foreground/60 uppercase mb-2"
                        >
                            Estado Dominante
                        </p>
                        {#if selectedDayData.equivalentState}
                            <div class="flex items-center gap-3">
                                <Badge
                                    class="text-xs px-3 h-6 font-black uppercase {selectedDayData
                                        .equivalentState.impact === 'Positive'
                                        ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                                        : 'bg-rose-500/10 text-red-400 border-rose-500/20'}"
                                >
                                    {selectedDayData.equivalentState.name}
                                </Badge>
                                <span
                                    class="text-[10px] font-bold text-muted-foreground/50"
                                    >Peso: {selectedDayData.avgWeight.toFixed(
                                        1,
                                    )}</span
                                >
                            </div>
                        {/if}
                    </div>
                    <div
                        class="p-4 rounded-xl border border-border/40 bg-muted/20"
                    >
                        <p
                            class="text-[10px] font-black text-muted-foreground/60 uppercase mb-2"
                        >
                            Total de Operações
                        </p>
                        <p class="text-xl font-black text-foreground">
                            {selectedDayData.trades.length}
                        </p>
                    </div>
                </div>

                <!-- Trades List with Detailed States -->
                <div class="space-y-3">
                    <h3
                        class="text-[11px] font-black uppercase tracking-widest text-muted-foreground/60"
                    >
                        Breakdown de Operações
                    </h3>
                    <div class="space-y-2">
                        {#each selectedDayData.trades as trade}
                            <div
                                class="p-4 rounded-xl border border-border/40 bg-muted/20 hover:bg-accent/10 transition-colors"
                            >
                                <div
                                    class="flex items-center justify-between mb-3"
                                >
                                    <div class="flex items-center gap-3">
                                        <Badge
                                            class="h-5 font-black uppercase bg-primary/10 text-primary border-primary/20"
                                            >{trade.asset_symbol}</Badge
                                        >
                                        <span
                                            class="text-[10px] font-bold text-muted-foreground/50"
                                        >
                                            {new Date(
                                                trade.date,
                                            ).toLocaleTimeString([], {
                                                hour: "2-digit",
                                                minute: "2-digit",
                                            })}
                                        </span>
                                    </div>
                                    <span
                                        class="text-sm font-black {trade.result >=
                                        0
                                            ? 'text-emerald-400'
                                            : 'text-red-400'}"
                                    >
                                        {formatCurrency(
                                            trade.result,
                                            settingsStore.accounts.find(
                                                (a) =>
                                                    a.id === trade.account_id,
                                            )?.currency || "BRL",
                                        )}
                                    </span>
                                </div>

                                <div class="flex items-center justify-between">
                                    <div class="flex flex-col gap-1">
                                        <span
                                            class="text-[9px] font-black text-muted-foreground/60 uppercase tracking-tighter"
                                            >Entrada</span
                                        >
                                        {#if trade.entry_emotional_state_id}
                                            {@const st = emotionalStates.find(
                                                (s) =>
                                                    s.id ===
                                                    trade.entry_emotional_state_id,
                                            )}
                                            <span
                                                class="text-[10px] font-black uppercase {st?.impact ===
                                                'Positive'
                                                    ? 'text-emerald-400'
                                                    : 'text-red-400'}"
                                            >
                                                {st?.name}
                                            </span>
                                        {/if}
                                    </div>
                                    {#if trade.intensity}
                                        <div class="flex flex-col gap-1">
                                            <span
                                                class="text-[9px] font-black text-muted-foreground/60 uppercase tracking-tighter"
                                                >Intensidade</span
                                            >
                                            <div
                                                class="flex items-center gap-1 w-20"
                                            >
                                                <div
                                                    class="flex-1 h-1 bg-muted rounded-full overflow-hidden"
                                                >
                                                    <div
                                                        class="h-full bg-primary"
                                                        style="width: {trade.intensity *
                                                            10}%"
                                                    ></div>
                                                </div>
                                            </div>
                                        </div>
                                    {/if}
                                </div>
                                {#if trade.notes}
                                    <div
                                        class="mt-3 p-2 rounded bg-muted/40 border border-border/20"
                                    >
                                        <p
                                            class="text-[10px] text-muted-foreground/60 italic font-medium leading-relaxed"
                                        >
                                            "{trade.notes}"
                                        </p>
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                </div>
            </div>

            <div
                class="p-4 border-t border-border/40 bg-muted/30 flex justify-end"
            >
                <Button
                    variant="outline"
                    class="h-9 border-border/40 bg-muted/40 hover:bg-accent/10 font-bold uppercase text-[10px]"
                    onclick={() => (showDayModal = false)}
                >
                    Fechar Detalhes
                </Button>
            </div>
        {/if}
    </Dialog.Content>
</Dialog.Root>
