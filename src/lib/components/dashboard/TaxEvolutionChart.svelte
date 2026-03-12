<script lang="ts">
    import { onMount, tick } from "svelte";
    import * as echarts from "echarts";
    import { formatCurrency } from "$lib/utils";
    import { t, locale } from "svelte-i18n";

    interface TaxData {
        month: string;
        taxDue: number;
        taxPaid: number;
    }

    let { data = [] }: { data: TaxData[] } = $props();

    let chartContainer: HTMLDivElement;
    let chartInstance = $state<echarts.ECharts>();
    let resizeObserver: ResizeObserver;

    $effect(() => {
        if (chartInstance && data) {
            updateChart();
        }
    });

    onMount(async () => {
        await tick();
        initChart();

        // Use ResizeObserver instead of window resize for better precision
        resizeObserver = new ResizeObserver(() => {
            if (chartInstance) {
                chartInstance.resize();
            }
        });

        if (chartContainer) {
            resizeObserver.observe(chartContainer);
        }

        // Handle common Svelte/ECharts race condition where container is not yet ready
        // despite being onMount + tick.
        const resizeTimeout = setTimeout(() => {
            if (chartInstance) chartInstance.resize();
        }, 300);

        return () => {
            clearTimeout(resizeTimeout);
            resizeObserver?.disconnect();
            chartInstance?.dispose();
        };
    });

    function initChart() {
        if (!chartContainer || chartInstance) return;

        // Ensure container has dimensions before init
        const rect = chartContainer.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) {
            // If dimensions aren't ready, wait a bit
            setTimeout(initChart, 100);
            return;
        }

        chartInstance = echarts.init(chartContainer);
        updateChart();
    }

    function updateChart() {
        if (!chartInstance) return;

        const taxDueLabel = $t("fiscal.irpf.table.toPay");
        const taxPaidLabel = $t("fiscal.irpf.table.paid");

        const option = {
            backgroundColor: "transparent",
            tooltip: {
                trigger: "axis",
                backgroundColor: "#1e1e2d",
                borderColor: "#3f3f46",
                textStyle: { color: "#e4e4e7" },
                formatter: function (params: any) {
                    let tooltip = `<div class="font-bold mb-1">${params[0].axisValue}</div>`;
                    params.forEach((item: any) => {
                        tooltip += `
                        <div class="flex justify-between items-center gap-4">
                            <span style="color:${item.color}">● ${item.seriesName}</span>
                            <span class="font-mono">${formatCurrency(item.value, "BRL", $locale || "pt-BR")}</span>
                        </div>`;
                    });
                    return tooltip;
                },
            },
            legend: {
                data: [taxDueLabel, taxPaidLabel],
                bottom: 0,
                textStyle: { color: "#a1a1aa" },
            },
            grid: {
                left: "3%",
                right: "4%",
                bottom: "15%", // Increased to fit labels
                top: "10%",
                containLabel: true,
            },
            xAxis: {
                type: "category",
                data: data.map((d) => d.month),
                axisLine: { lineStyle: { color: "#3f3f46" } },
                axisLabel: {
                    color: "#a1a1aa",
                    interval: 0, // Show all months
                },
            },
            yAxis: {
                type: "value",
                splitLine: { lineStyle: { color: "#27272a" } },
                axisLabel: {
                    color: "#a1a1aa",
                    formatter: (value: number) =>
                        new Intl.NumberFormat($locale || "pt-BR", {
                            notation: "compact",
                            compactDisplay: "short",
                            style: "currency",
                            currency: "BRL",
                        }).format(value),
                },
            },
            series: [
                {
                    name: taxDueLabel,
                    type: "bar",
                    data: data.map((d) => d.taxDue),
                    itemStyle: { color: "#ef4444" }, // Red
                    barMaxWidth: 20,
                    animationDuration: 1000,
                },
                {
                    name: taxPaidLabel,
                    type: "bar",
                    data: data.map((d) => d.taxPaid),
                    itemStyle: { color: "#22c55e" }, // Green
                    barMaxWidth: 20,
                    animationDuration: 1000,
                },
            ],
        };

        chartInstance.setOption(option, true);
    }
</script>

<div bind:this={chartContainer} class="w-full h-full min-h-[300px]"></div>
