<script lang="ts">
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
        CardDescription,
    } from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Button } from "$lib/components/ui/button";
    import { TrendingUp, Activity, Target, ArrowRight } from "lucide-svelte";
    import type { Strategy } from "$lib/stores/settings.svelte";
    import { goto } from "$app/navigation";
    import { t, locale } from "svelte-i18n";
    import { formatCurrency } from "$lib/utils";

    // Mock Stats Interface (until backend is ready)
    export interface StrategyStats {
        total_trades: number;
        win_rate: number;
        profit_factor: number;
        total_profit: number;
        average_profit: number;
        payoff: number;
        currency?: string;
    }

    // Props
    let {
        strategy,
        stats = {
            total_trades: 0,
            win_rate: 0,
            profit_factor: 0,
            total_profit: 0,
            average_profit: 0,
            payoff: 0,
        },
    }: { strategy: Strategy; stats?: StrategyStats } = $props();

    // Derived Visual States
    let statusColor = $derived(
        stats.total_profit >= 0 ? "text-emerald-500" : "text-rose-500",
    );
    let winRateColor = $derived(
        stats.win_rate >= 50
            ? "bg-emerald-500/10 text-emerald-500"
            : "bg-rose-500/10 text-rose-500",
    );
</script>

<Card
    class="group hover:border-primary/50 transition-all duration-300 bg-card/50 backdrop-blur-sm relative overflow-hidden flex flex-col h-full"
>
    <!-- Hover Glow -->
    <div
        class="absolute inset-0 bg-gradient-to-br from-primary/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
    ></div>

    <CardHeader class="pb-2 p-4 z-10">
        <div class="flex justify-between items-start gap-2">
            <div class="space-y-0.5 max-w-[65%]">
                <CardTitle
                    class="text-base font-bold tracking-tight group-hover:text-primary transition-colors truncate"
                    title={strategy.name}
                >
                    {strategy.name}
                </CardTitle>
                <div class="flex items-center gap-1.5">
                    <Badge
                        variant="outline"
                        class="text-[10px] font-normal border-white/10 h-4 px-1"
                    >
                        {stats.total_trades}
                        {$t("strategy.card.tradesSuffix")}
                    </Badge>
                    {#if strategy.specific_assets.length > 0}
                        <Badge
                            variant="secondary"
                            class="text-[9px] h-4 px-1 uppercase tracking-wide opacity-80"
                        >
                            {strategy.specific_assets[0]}
                        </Badge>
                    {/if}
                </div>
            </div>
            <div class={`text-right ${statusColor} shrink-0`}>
                <div class="text-base font-bold">
                    {formatCurrency(
                        stats.total_profit,
                        stats.currency || "BRL",
                    )}
                </div>
                <div
                    class="text-[9px] uppercase tracking-wider text-muted-foreground"
                >
                    {$t("strategy.card.totalResult")}
                </div>
            </div>
        </div>
    </CardHeader>

    <CardContent class="space-y-2 p-4 pt-0 z-10 flex-1 flex flex-col">
        <!-- Stats Grid -->
        <div class="grid grid-cols-2 gap-2">
            <!-- Win Rate -->
            <div
                class="space-y-0.5 p-1.5 rounded bg-muted/20 border border-border/50"
            >
                <div
                    class="text-[9px] uppercase tracking-wider text-muted-foreground flex items-center gap-1"
                >
                    <Target class="w-2.5 h-2.5" />
                    {$t("strategy.dashboard.stats.winRate")}
                </div>
                <div
                    class={`text-xs font-bold px-1 py-0.5 rounded w-fit ${winRateColor}`}
                >
                    {stats.win_rate.toFixed(1)}%
                </div>
            </div>

            <!-- Payoff -->
            <div
                class="space-y-0.5 p-1.5 rounded bg-muted/20 border border-border/50"
            >
                <div
                    class="text-[9px] uppercase tracking-wider text-muted-foreground flex items-center gap-1"
                >
                    <Activity class="w-2.5 h-2.5" />
                    {$t("strategy.dashboard.stats.payoff")}
                </div>
                <div class="text-xs font-bold text-foreground">
                    {stats.payoff.toFixed(2)}
                </div>
            </div>

            <!-- Profit Factor -->
            <div
                class="space-y-0.5 p-1.5 rounded bg-muted/20 border border-border/50 col-span-2 flex items-center justify-between"
            >
                <div
                    class="text-[9px] uppercase tracking-wider text-muted-foreground flex items-center gap-1"
                >
                    <TrendingUp class="w-2.5 h-2.5" />
                    {$t("strategy.dashboard.stats.profitFactor")}
                </div>
                <div
                    class={`text-xs font-bold ${stats.profit_factor > 1.2 ? "text-emerald-500" : stats.profit_factor >= 1 ? "text-yellow-500" : "text-rose-500"}`}
                >
                    {stats.profit_factor.toFixed(2)}
                </div>
            </div>
        </div>

        <div class="mt-auto pt-1">
            <Button
                variant="ghost"
                size="sm"
                class="w-full h-7 text-xs group-hover:bg-primary/10 group-hover:text-primary transition-colors justify-between px-2"
                onclick={() => goto(`/strategies/${strategy.id}`)}
            >
                {$t("strategy.card.viewAnalysis")}
                <ArrowRight
                    class="w-3 h-3 ml-1 group-hover:translate-x-1 transition-transform"
                />
            </Button>
        </div>
    </CardContent>
</Card>
