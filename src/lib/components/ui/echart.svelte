<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { mode } from "mode-watcher";
    import * as echarts from "echarts";

    // Props
    let { options = {}, theme = null, class: className = "" } = $props();

    let chartContainer: HTMLDivElement;
    let chartInstance: echarts.ECharts | null = null;

    // Determine effective theme
    const effectiveTheme = $derived(theme || mode.current || "light");

    // Initialize/Update Chart when theme changes
    $effect(() => {
        if (!chartContainer || !effectiveTheme) return;

        // Dispose and re-init to apply new theme properly (ECharts requirement)
        if (chartInstance) chartInstance.dispose();
        chartInstance = echarts.init(chartContainer, effectiveTheme);
        chartInstance.setOption(options);
    });

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
