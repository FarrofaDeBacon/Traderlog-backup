<script lang="ts">
    import { onMount } from "svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { TrendingUp, TrendingDown } from "lucide-svelte";

    // Filtering only currencies that have a rate and are not BRL
    let tickerItems = $derived(
        settingsStore.currencies
            .filter((c) => c.code !== "BRL" && c.exchange_rate > 0)
            .map((c) => ({
                pair: `${c.code}/BRL`,
                rate: c.exchange_rate,
                symbol: c.symbol,
            })),
    );

    // Default items if none exist to avoid empty bar
    const defaultItems = [
        { pair: "USD/BRL", rate: 5.42 },
        { pair: "EUR/BRL", rate: 5.85 },
        { pair: "GBP/BRL", rate: 6.92 },
        { pair: "USDT/BRL", rate: 5.43 },
        { pair: "BTC/BRL", rate: 564231.0 },
    ];

    let displayItems = $derived(
        tickerItems.length > 0 ? tickerItems : defaultItems,
    );

    onMount(() => {
        // Initial sync on mount
        settingsStore.syncExchangeRates();

        // Auto-refresh every 2 minutes (120000ms)
        const interval = setInterval(() => {
            console.log("[CurrencyTicker] Background Auto-Sync starting...");
            settingsStore.syncExchangeRates();
        }, 120000);

        return () => clearInterval(interval);
    });
</script>

<div
    class="w-full bg-zinc-950/80 backdrop-blur-md border-y border-zinc-800/50 py-1.5 overflow-hidden select-none"
>
    <div class="ticker-content flex items-center whitespace-nowrap">
        <!-- We repeat the content to create a seamless loop -->
        <div class="ticker-track flex items-center gap-12 px-6">
            {#each [...displayItems, ...displayItems, ...displayItems] as item}
                <div class="flex items-center gap-2">
                    <span
                        class="text-[10px] font-bold text-zinc-500 uppercase tracking-tighter"
                        >{item.pair}</span
                    >
                    <span
                        class="font-mono text-sm font-black text-zinc-100 italic"
                    >
                        {item.rate.toLocaleString("pt-BR", {
                            minimumFractionDigits: 2,
                            maximumFractionDigits: 4,
                        })}
                    </span>
                    {#if item.rate > 5}
                        <!-- Just a visual detail -->
                        <TrendingUp class="w-3 h-3 text-emerald-500/50" />
                    {:else}
                        <TrendingUp class="w-3 h-3 text-emerald-500/20" />
                    {/if}
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .ticker-track {
        display: flex;
        animation: scroll 60s linear infinite;
    }

    .ticker-track:hover {
        animation-play-state: paused;
    }

    @keyframes scroll {
        0% {
            transform: translateX(0);
        }
        100% {
            transform: translateX(-33.33%);
        }
    }

    /* Glossy effect */
    .ticker-track span {
        text-shadow: 0 0 10px rgba(255, 255, 255, 0.1);
    }
</style>
