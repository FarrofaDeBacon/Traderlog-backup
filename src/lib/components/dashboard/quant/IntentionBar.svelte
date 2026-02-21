<script lang="ts">
    interface Props {
        buy: number;
        sell: number;
        label?: string;
    }

    let { buy, sell, label } = $props<Props>();

    let total = $derived(buy + sell || 1);
    let buyPercent = $derived((buy / total) * 100);
    let sellPercent = $derived((sell / total) * 100);
</script>

<div class="space-y-2">
    {#if label}
        <div class="flex justify-between text-xs font-medium">
            <span>{label}</span>
            <div class="space-x-2">
                <span class="text-green-500">{buyPercent.toFixed(1)}%</span>
                <span class="text-red-500">{sellPercent.toFixed(1)}%</span>
            </div>
        </div>
    {/if}
    <div class="h-4 w-full flex rounded-full overflow-hidden bg-muted/30">
        <div
            class="h-full bg-green-500/80 transition-all duration-500"
            style="width: {buyPercent}%"
        ></div>
        <div
            class="h-full bg-red-500/80 transition-all duration-500"
            style="width: {sellPercent}%"
        ></div>
    </div>
    <div
        class="flex justify-between text-[10px] text-muted-foreground uppercase tracking-wider"
    >
        <span>C: {buy.toLocaleString()}</span>
        <span>V: {sell.toLocaleString()}</span>
    </div>
</div>
