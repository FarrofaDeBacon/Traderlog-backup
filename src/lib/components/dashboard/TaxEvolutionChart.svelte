<script lang="ts">
    import { onMount } from "svelte";
    import * as echarts from "echarts";
    import { formatCurrency } from "$lib/utils";

    interface TaxData {
        month: string;
        taxDue: number;
        taxPaid: number;
    }

    let { data = [] }: { data: TaxData[] } = $props();

    let chartContainer: HTMLDivElement;
    let chartInstance: echarts.ECharts;

    $effect(() => {
        if (chartInstance && data) {
            updateChart();
        }
    });

    onMount(() => {
        initChart();
        window.addEventListener("resize", handleResize);
        return () => {
            window.removeEventListener("resize", handleResize);
            chartInstance?.dispose();
        };
    });

    function handleResize() {
        chartInstance?.resize();
    }

    function initChart() {
        if (!chartContainer) return;
        chartInstance = echarts.init(chartContainer); // Removed "dark" explicitly to rely on option styling
        updateChart();
    }

    function updateChart() {
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
                            <span class="font-mono">${formatCurrency(item.value)}</span>
                        </div>`;
                    });
                    return tooltip;
                },
            },
            legend: {
                data: ["Imposto Devido", "Imposto Pago"],
                bottom: 0,
                textStyle: { color: "#a1a1aa" },
            },
            grid: {
                left: "3%",
                right: "4%",
                bottom: "10%",
                top: "10%",
                containLabel: true,
            },
            xAxis: {
                type: "category",
                data: data.map((d) => d.month),
                axisLine: { lineStyle: { color: "#3f3f46" } },
                axisLabel: { color: "#a1a1aa" },
            },
            yAxis: {
                type: "value",
                splitLine: { lineStyle: { color: "#27272a" } },
                axisLabel: {
                    color: "#a1a1aa",
                    formatter: (value: number) =>
                        new Intl.NumberFormat("pt-BR", {
                            notation: "compact",
                            compactDisplay: "short",
                            style: "currency",
                            currency: "BRL",
                        }).format(value),
                },
            },
            series: [
                {
                    name: "Imposto Devido",
                    type: "bar",
                    data: data.map((d) => d.taxDue),
                    itemStyle: { color: "#ef4444" }, // Red
                    barMaxWidth: 20,
                },
                {
                    name: "Imposto Pago",
                    type: "bar",
                    data: data.map((d) => d.taxPaid),
                    itemStyle: { color: "#22c55e" }, // Green
                    barMaxWidth: 20,
                },
            ],
        };
        chartInstance.setOption(option);
    }
</script>

<div bind:this={chartContainer} class="w-full h-full min-h-[300px]"></div>
