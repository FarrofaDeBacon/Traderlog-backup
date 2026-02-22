<script lang="ts">
    import { t } from "svelte-i18n";
    import EChart from "$lib/components/ui/echart.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { formatCurrency } from "$lib/utils";

    let { trades = [] } = $props();

    const chartOptions = $derived.by(() => {
        // Explicitly track trades for reactivity via snapshot
        const tradesData = $state.snapshot(trades);

        let gainCount = 0;
        let lossCount = 0;
        let beCount = 0;

        tradesData.forEach((t) => {
            const res = tradesStore.getConvertedTradeResult(
                t,
                settingsStore.accounts,
                settingsStore.currencies,
            );
            if (res > 0) gainCount++;
            else if (res < 0) lossCount++;
            else beCount++;
        });

        const gainLabel = $t("trades.wizard.charts.outcome.gain");
        const lossLabel = $t("trades.wizard.charts.outcome.loss");
        const beLabel = $t("trades.wizard.charts.outcome.be");
        const emptyLabel = $t("trades.wizard.charts.outcome.vazio");

        const data = [];
        if (gainCount > 0)
            data.push({
                value: gainCount,
                name: gainLabel,
                itemStyle: { color: "#10b981" },
            });
        if (lossCount > 0)
            data.push({
                value: lossCount,
                name: lossLabel,
                itemStyle: { color: "#f43f5e" },
            });
        if (beCount > 0)
            data.push({
                value: beCount,
                name: beLabel,
                itemStyle: { color: "#71717a" },
            });

        // Fallback for empty data
        const finalData =
            data.length > 0
                ? data
                : [
                      {
                          value: 1,
                          name: emptyLabel,
                          itemStyle: { color: "#27272a" },
                          label: { show: false },
                      },
                  ];

        return {
            backgroundColor: "transparent",
            tooltip: {
                trigger: "item",
                backgroundColor: "#18181b",
                borderColor: "#27272a",
                textStyle: { color: "#fff", fontSize: 12 },
                formatter: "{b}: <b>{c}</b> ({d}%)",
            },
            legend: {
                orient: "horizontal",
                bottom: 0,
                left: "center",
                itemWidth: 8,
                itemHeight: 8,
                textStyle: {
                    color: "#71717a",
                    fontSize: 10,
                    fontWeight: "bold",
                },
            },
            series: [
                {
                    type: "pie",
                    radius: ["40%", "60%"],
                    center: ["50%", "42%"],
                    avoidLabelOverlap: false,
                    itemStyle: {
                        borderRadius: 4,
                        borderColor: "#09090b",
                        borderWidth: 2,
                    },
                    label: {
                        show: true,
                        position: "center",
                        formatter: () =>
                            `{total|${tradesData.length}}\n{label|${$t("trades.wizard.charts.outcome.trades_label")}}`,
                        rich: {
                            total: {
                                fontSize: 10,
                                fontWeight: "black",
                                color: "#fff",
                            },
                            label: {
                                fontSize: 6,
                                fontWeight: "black",
                                color: "#52525b",
                                padding: [1, 0, 0, 0],
                            },
                        },
                    },
                    emphasis: {
                        label: {
                            show: true,
                        },
                    },
                    data: finalData,
                },
            ],
        };
    });
</script>

<div class="relative w-full h-full">
    {#if trades.length > 0}
        {#key trades}
            <EChart options={chartOptions} class="absolute inset-0" />
        {/key}
    {:else}
        <div
            class="absolute inset-0 flex items-center justify-center text-muted-foreground italic text-xs"
        >
            {$t("trades.wizard.charts.outcome.empty")}
        </div>
    {/if}
</div>
