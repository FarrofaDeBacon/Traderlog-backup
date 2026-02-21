<script lang="ts">
    import EChart from "$lib/components/ui/echart.svelte";
    import * as echarts from "echarts";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { locale } from "svelte-i18n";
    import { TrendingUp } from "lucide-svelte";

    let { accountId = $bindable("all"), currency = $bindable("BRL") } = $props<{
        accountId?: string | "all";
        currency?: string;
    }>();

    // Data processing logic
    let processedData = $derived.by(() => {
        // Explicitly access props for reactivity tracking
        const currentAccountId = accountId;
        const currentCurrency = currency;

        console.log("[AccountEvolutionChart] Reacting to:", {
            currentAccountId,
            currentCurrency,
        });

        let transactions = settingsStore.cashTransactions;
        console.log(
            "[AccountEvolutionChart] Total transactions available:",
            transactions.length,
        );

        // Filter by account if specified
        if (currentAccountId && currentAccountId !== "all") {
            const idToMatch = String(currentAccountId).trim();
            transactions = transactions.filter(
                (tx) => String(tx.account_id).trim() === idToMatch,
            );
            console.log(
                "[AccountEvolutionChart] Filtered by account:",
                transactions.length,
            );
        }

        // Sort by date ascending and ensure date is valid
        const sortedTxs = [...transactions]
            .filter((tx) => tx.date && typeof tx.date === "string")
            .sort((a, b) => a.date.localeCompare(b.date));

        const labels: string[] = [];
        const data: number[] = [];
        let runningTotal = 0;

        // Group by date to show daily points
        const dailyTotals: Record<string, number> = {};

        sortedTxs.forEach((tx) => {
            const date = tx.date;
            let amount = tx.amount;

            // Convert to the target currency
            const acc = settingsStore.accounts.find(
                (a) => String(a.id) === String(tx.account_id),
            );
            if (acc && acc.currency !== currentCurrency) {
                const srcCurrencyObj = settingsStore.currencies.find(
                    (c) => c.code === acc.currency,
                );
                const destCurrencyObj = settingsStore.currencies.find(
                    (c) => c.code === currentCurrency,
                );
                if (srcCurrencyObj && destCurrencyObj) {
                    amount =
                        (amount * srcCurrencyObj.exchange_rate) /
                        destCurrencyObj.exchange_rate;
                }
            }

            dailyTotals[date] = (dailyTotals[date] || 0) + amount;
        });

        const sortedDates = Object.keys(dailyTotals).sort();

        sortedDates.forEach((date) => {
            runningTotal += dailyTotals[date];
            labels.push(date);
            data.push(parseFloat(runningTotal.toFixed(2)));
        });

        return { labels, data };
    });

    let chartOptions = $derived.by(() => {
        const { labels, data } = processedData;
        const currentCurrency = currency;

        const safeFormat = (val: number, compact = false) => {
            try {
                return new Intl.NumberFormat($locale || "pt-BR", {
                    style: "currency",
                    currency: currentCurrency,
                    notation: compact ? "compact" : "standard",
                }).format(val);
            } catch (e) {
                return `${currentCurrency} ${val.toLocaleString($locale || "pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
            }
        };

        const parseDate = (val: any) => {
            if (!val) return null;
            if (val instanceof Date) return val;

            // Try parsing directly first
            let d = new Date(val);
            if (!isNaN(d.getTime())) return d;

            // Handle YYYY-MM-DD
            if (typeof val === "string" && val.includes("-")) {
                d = new Date(val + "T12:00:00");
                if (!isNaN(d.getTime())) return d;
            }

            return null;
        };

        return {
            backgroundColor: "transparent",
            tooltip: {
                trigger: "axis",
                backgroundColor: "rgba(10, 10, 10, 0.9)",
                borderColor: "#27272a",
                borderWidth: 1,
                textStyle: { color: "#fff", fontSize: 12 },
                formatter: (params: any) => {
                    const p = params[0];
                    const d = parseDate(p.name);
                    const dateStr = d
                        ? d.toLocaleDateString($locale || "pt-BR", {
                              day: "2-digit",
                              month: "short",
                              year: "numeric",
                          })
                        : p.name;
                    const val = safeFormat(p.value);

                    return `
                        <div class="flex flex-col gap-1 p-1">
                            <div class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">${dateStr}</div>
                            <div class="text-sm font-black text-white">${val}</div>
                        </div>
                    `;
                },
            },
            grid: {
                left: "2%",
                right: "2%",
                bottom: "5%",
                top: "10%",
                containLabel: true,
            },
            xAxis: {
                type: "category",
                boundaryGap: false,
                data: labels,
                axisLine: { lineStyle: { color: "#27272a" } },
                axisLabel: {
                    color: "#71717a",
                    fontSize: 10,
                    formatter: (value: string) => {
                        const d = parseDate(value);
                        return d
                            ? d.toLocaleDateString($locale || "pt-BR", {
                                  day: "2-digit",
                                  month: "short",
                              })
                            : value;
                    },
                },
                axisTick: { show: false },
            },
            yAxis: {
                type: "value",
                splitLine: {
                    lineStyle: {
                        color: "rgba(39, 39, 42, 0.5)",
                        type: "dashed",
                    },
                },
                axisLabel: {
                    color: "#71717a",
                    fontSize: 10,
                    formatter: (value: number) => {
                        return safeFormat(value, true);
                    },
                },
            },
            series: [
                {
                    name: "Patrimônio",
                    type: "line",
                    smooth: true,
                    symbol: "circle",
                    symbolSize: 6,
                    showSymbol: false,
                    lineStyle: { width: 3, color: "#10b981" }, // Emerald-500
                    itemStyle: { color: "#10b981" },
                    areaStyle: {
                        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                            { offset: 0, color: "rgba(16, 185, 129, 0.2)" },
                            { offset: 1, color: "rgba(16, 185, 129, 0.0)" },
                        ]),
                    },
                    data: data,
                    animation: true,
                    animationDuration: 500,
                    emphasis: {
                        lineStyle: { width: 4 },
                    },
                },
            ],
        };
    });
</script>

<div class="h-[320px] w-full">
    {#if processedData.data.length > 0}
        <EChart options={chartOptions} />
    {:else}
        <div
            class="h-full w-full flex flex-col items-center justify-center text-zinc-500 gap-2 border border-dashed border-zinc-800 rounded-xl bg-zinc-950/20"
        >
            <div
                class="w-10 h-10 rounded-full bg-zinc-900 flex items-center justify-center"
            >
                <TrendingUp class="w-5 h-5 opacity-20" />
            </div>
            <p class="text-xs font-medium uppercase tracking-widest opacity-50">
                Sem dados para o período
            </p>
        </div>
    {/if}
</div>
