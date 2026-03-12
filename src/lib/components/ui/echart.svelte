<script lang="ts">
    import { onMount, onDestroy, untrack } from "svelte";
    import { mode } from "mode-watcher";
    import * as echarts from "echarts";

    // Props
    let { options = {}, theme = null, class: className = "" } = $props();

    let chartContainer: HTMLDivElement;
    let chartInstance: echarts.ECharts | null = null;

    // Determine effective theme
    const effectiveTheme = $derived(theme || mode.current || "light");

    // Initialize Chart when theme or container changes
    $effect(() => {
        if (!chartContainer || !effectiveTheme) return;

        // Dispose and re-init to apply new theme properly
        untrack(() => {
            if (chartInstance && !chartInstance.isDisposed()) {
                chartInstance.dispose();
            }
            chartInstance = echarts.init(chartContainer, effectiveTheme);
            // After init, the second effect will handle loading the options
        });
    });

    // Update options when they change OR when chart instance is newly created
    $effect(() => {
        if (chartInstance && !chartInstance.isDisposed() && options) {
            chartInstance.setOption(options, {
                notMerge: true,
                lazyUpdate: true, 
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
