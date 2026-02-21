<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import * as echarts from "echarts";

    // Props
    let { options = {}, theme = null, class: className = "" } = $props();

    let chartContainer: HTMLDivElement;
    let chartInstance: echarts.ECharts | null = null;

    // Initialize Chart
    const initChart = () => {
        if (!chartContainer) return;

        // Dispose existing if any
        if (chartInstance) chartInstance.dispose();

        chartInstance = echarts.init(chartContainer, theme);
        chartInstance.setOption(options);

        // Resize observer setup is handled below
    };

    // Use snapshot for all options passed to ECharts
    let snapshottedOptions = $derived($state.snapshot(options));

    // Update options when they change
    $effect(() => {
        if (chartInstance && snapshottedOptions) {
            chartInstance.setOption(snapshottedOptions, {
                notMerge: true,
                lazyUpdate: false,
            });
        }
    });

    onMount(() => {
        initChart();

        // Responsive Resizing
        const resizeObserver = new ResizeObserver(() => {
            if (chartInstance && !chartInstance.isDisposed()) {
                chartInstance.resize();
            }
        });
        resizeObserver.observe(chartContainer);

        return () => {
            resizeObserver.disconnect();
            if (chartInstance && !chartInstance.isDisposed()) {
                chartInstance.dispose();
            }
        };
    });
</script>

<div bind:this={chartContainer} class="w-full h-full {className}"></div>
