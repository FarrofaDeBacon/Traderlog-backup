<script lang="ts">
    import EChart from "$lib/components/ui/echart.svelte";
    import * as echarts from "echarts";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { formatCurrency } from "$lib/utils";
    import { format } from "date-fns";

    let { trades = [] } = $props();

    const chartOptions = $derived.by(() => {
        const currentTrades = trades;
        const tradesData = $state.snapshot(currentTrades);

        // Sort trades by date
        const sortedTrades = [...tradesData].sort(
            (a, b) =>
                new Date(a.exit_date || a.date).getTime() -
                new Date(b.exit_date || b.date).getTime(),
        );

        let cumulativePnL = 0;
        let maxPeak = 0;
        const equityCurve = sortedTrades.map((t) => {
            const result = tradesStore.getConvertedTradeResult(
                t,
                settingsStore.accounts,
                settingsStore.currencies,
            );
            cumulativePnL += result;
            if (cumulativePnL > maxPeak) maxPeak = cumulativePnL;

            return {
                value: [
                    new Date(t.exit_date || t.date).getTime(),
                    cumulativePnL,
                ],
                drawdown: maxPeak - cumulativePnL,
                tradeId: t.id,
            };
        });

        const drawdownData = equityCurve.map((item) => ({
            value: [item.value[0], -item.drawdown], // Show as negative for visual clarity
        }));

        // Add a starting point at zero if there are trades
        if (equityCurve.length > 0) {
            const firstDate = new Date(
                sortedTrades[0].exit_date || sortedTrades[0].date,
            ).getTime();
            equityCurve.unshift({
                value: [firstDate - 3600000, 0],
                drawdown: 0,
                tradeId: "start",
            });
            drawdownData.unshift({
                value: [firstDate - 3600000, 0],
            });
        }

        return {
            backgroundColor: "transparent",
            grid: {
                top: "10%",
                left: "3%",
                right: "4%",
                bottom: "10%",
                containLabel: true,
            },
            tooltip: {
                trigger: "axis",
                backgroundColor: "#18181b",
                borderColor: "#27272a",
                textStyle: { color: "#fff", fontSize: 12 },
                formatter: (params: any) => {
                    const equity = params.find(
                        (p: any) => p.seriesName === "Capital",
                    )?.data;
                    const dd = params.find(
                        (p: any) => p.seriesName === "Drawdown",
                    )?.data;
                    if (!equity) return "";

                    const date = format(
                        new Date(equity.value[0]),
                        "dd/MM HH:mm",
                    );
                    const val = equity.value[1];
                    const color = val >= 0 ? "#10b981" : "#f43f5e";

                    let html = `
                        <div class="flex flex-col gap-1">
                            <span class="text-zinc-500 text-[10px] font-bold uppercase">${date}</span>
                            <div class="flex items-center justify-between gap-4">
                                <span class="text-[10px] text-zinc-400 uppercase">Capital</span>
                                <span class="text-xs font-black" style="color: ${color}">
                                    ${val >= 0 ? "+" : ""}${formatCurrency(val, settingsStore.userProfile?.main_currency || "BRL")}
                                </span>
                            </div>
                    `;

                    if (dd && dd.value[1] < 0) {
                        html += `
                            <div class="flex items-center justify-between gap-4 border-t border-white/5 pt-1 mt-1">
                                <span class="text-[10px] text-zinc-400 uppercase">Drawdown</span>
                                <span class="text-xs font-bold text-rose-500">
                                    ${formatCurrency(Math.abs(dd.value[1]), settingsStore.userProfile?.main_currency || "BRL")}
                                </span>
                            </div>
                        `;
                    }

                    html += `</div>`;
                    return html;
                },
            },
            xAxis: {
                type: "time",
                axisLine: { lineStyle: { color: "#27272a" } },
                axisLabel: {
                    color: "#71717a",
                    fontSize: 9,
                    formatter: (value: number) =>
                        format(new Date(value), "dd/MM"),
                },
                splitLine: { show: false },
            },
            yAxis: {
                type: "value",
                axisLine: { show: false },
                axisLabel: {
                    color: "#71717a",
                    fontSize: 9,
                    formatter: (value: number) =>
                        formatCurrency(
                            value,
                            settingsStore.userProfile?.main_currency || "BRL",
                        ).replace(/[^\d.,+-]/g, ""),
                },
                splitLine: { lineStyle: { color: "#27272a", type: "dashed" } },
            },
            series: [
                {
                    name: "Capital",
                    type: "line",
                    smooth: true,
                    showSymbol: true,
                    symbol: "circle",
                    symbolSize: 6,
                    z: 5,
                    animationDuration: 300,
                    data: equityCurve,
                    lineStyle: {
                        width: 3,
                        color: "#10b981",
                    },
                    areaStyle: {
                        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                            { offset: 0, color: "rgba(16, 185, 129, 0.2)" },
                            { offset: 1, color: "rgba(16, 185, 129, 0.0)" },
                        ]),
                    },
                    markLine: {
                        silent: true,
                        data: [{ yAxis: 0 }],
                        lineStyle: {
                            color: "#71717a",
                            type: "solid",
                            width: 1,
                            opacity: 0.8,
                        },
                        label: {
                            show: false,
                        },
                        symbol: ["none", "none"],
                    },
                },
                {
                    name: "Drawdown",
                    type: "line",
                    smooth: true,
                    showSymbol: false,
                    z: 2,
                    data: drawdownData,
                    lineStyle: { opacity: 0 },
                    areaStyle: {
                        color: "rgba(244, 63, 94, 0.23)",
                    },
                },
            ],
            animation: true,
        };
    });
</script>

<div class="relative w-full h-full min-h-[200px]">
    {#if trades.length > 0}
        {#key trades}
            <EChart options={chartOptions} class="absolute inset-0" />
        {/key}
    {:else}
        <div
            class="absolute inset-0 flex items-center justify-center text-muted-foreground italic text-xs"
        >
            Sem operações no período
        </div>
    {/if}
</div>
