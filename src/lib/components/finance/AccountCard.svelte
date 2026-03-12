<script lang="ts">
    import * as Card from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Wallet, Landmark, ArrowUpRight } from "lucide-svelte";
    import { settingsStore, type Account } from "$lib/stores/settings.svelte";
    import { t, locale } from "svelte-i18n";
    import { cn, formatCurrency } from "$lib/utils";

    let { account }: { account: Account } = $props();

    let currencyStr = $derived(account.currency);
    let currencyObj = $derived(
        settingsStore.currencies.find((c) => c.code === currencyStr),
    );
    let exchangeRate = $derived(currencyObj?.exchange_rate || 1);
    let balanceInBRL = $derived(account.balance * exchangeRate);

    const typeColors = {
        Real: "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
        Demo: "bg-blue-500/10 text-blue-500 border-blue-500/20",
        Prop: "bg-amber-500/10 text-amber-500 border-amber-500/20",
    };

    // Determine color based on balance (optional, currently static primary)
    let cardAccentColor = $derived(
        account.balance >= 0 ? "border-primary/20" : "border-rose-500/20",
    );

    // Sparkline data
    import EChart from "$lib/components/ui/echart.svelte";
    import * as echarts from "echarts";

    let sparklineData = $derived.by(() => {
        const transactions = settingsStore.cashTransactions
            .filter((tx) => tx.account_id === account.id)
            .sort((a, b) => a.date.localeCompare(b.date));

        let runningTotal = 0;
        const dailyPoints: number[] = [];
        const dailyTotals: Record<string, number> = {};

        transactions.forEach((tx) => {
            dailyTotals[tx.date] = (dailyTotals[tx.date] || 0) + tx.amount;
        });

        const sortedDates = Object.keys(dailyTotals).sort();
        sortedDates.forEach((date) => {
            runningTotal += dailyTotals[date];
            dailyPoints.push(parseFloat(runningTotal.toFixed(2)));
        });

        // Ensure we have at least 2 points for a line
        if (dailyPoints.length === 1) dailyPoints.unshift(0);
        return dailyPoints;
    });

    let chartOptions = $derived({
        backgroundColor: "transparent",
        grid: { left: 0, right: 0, top: 0, bottom: 0 },
        xAxis: { type: "category", show: false, boundaryGap: false },
        yAxis: { type: "value", show: false, min: "dataMin" },
        series: [
            {
                data: sparklineData,
                type: "line",
                smooth: true,
                symbol: "none",
                lineStyle: {
                    width: 2,
                    color: account.balance >= 0 ? "#10b981" : "#f43f5e",
                },
                areaStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        {
                            offset: 0,
                            color:
                                account.balance >= 0
                                    ? "rgba(16, 185, 129, 0.15)"
                                    : "rgba(244, 63, 94, 0.15)",
                        },
                        { offset: 1, color: "transparent" },
                    ]),
                },
            },
        ],
    });
</script>

<Card.Root
    class={cn(
        "relative overflow-hidden group transition-all duration-300 bg-card border shadow-sm hover:shadow-md",
        "min-h-[140px] flex flex-col justify-between",
    )}
>
    <Card.Header class="p-3 pb-0">
        <div class="flex items-start justify-between">
            <div class="flex items-center gap-2">
                <div
                    class="w-8 h-8 rounded-lg bg-muted/50 border border-border flex items-center justify-center group-hover:border-primary/40 transition-colors"
                >
                    {#if account.custom_logo}
                        <img
                            src={account.custom_logo}
                            alt={account.broker}
                            class="w-5 h-5 object-contain"
                        />
                    {:else}
                        <Landmark
                            class="w-4 h-4 text-muted-foreground group-hover:text-primary transition-colors"
                        />
                    {/if}
                </div>
                <div class="flex flex-col">
                    <Card.Title
                        class="text-xs font-bold tracking-tight leading-tight"
                        >{account.nickname}</Card.Title
                    >
                    <Card.Description
                        class="text-[9px] font-medium text-muted-foreground uppercase tracking-widest leading-none mt-0.5"
                        >{account.broker}</Card.Description
                    >
                </div>
            </div>
            <Badge
                variant="outline"
                class={cn(
                    "text-[7px] font-bold uppercase tracking-widest px-1.5 py-0 h-4 border shadow-none",
                    typeColors[
                        account.account_type as keyof typeof typeColors
                    ] || "bg-muted text-muted-foreground",
                )}
            >
                {account.account_type}
            </Badge>
        </div>
    </Card.Header>

    <Card.Content class="p-3 pt-1">
        <div class="flex items-end justify-between gap-2">
            <div class="flex-1 min-w-0">
                <span
                    class="text-[8px] font-bold text-muted-foreground uppercase tracking-widest mb-0.5 block"
                    >{$t("dashboard.account.balance")}</span
                >
                <div class="flex items-baseline gap-1">
                    <h2 class="text-lg font-bold tracking-tight">
                        {formatCurrency(
                            account.balance,
                            account.currency,
                            $locale || "pt-BR",
                        )}
                    </h2>
                </div>

                {#if account.currency !== "BRL"}
                    <div class="flex items-center gap-1 mt-0.5">
                        <span
                            class="text-[9px] font-medium text-muted-foreground italic"
                        >
                            ≈ {formatCurrency(
                                balanceInBRL,
                                "BRL",
                                $locale || "pt-BR",
                            )}
                        </span>
                    </div>
                {/if}
            </div>

            <div class="flex flex-col items-end gap-1">
                <!-- Sparkline Chart -->
                <div
                    class="w-12 h-6 shrink-0 relative overflow-hidden rounded-md bg-muted/20"
                >
                    {#if sparklineData.length > 0}
                        <EChart
                            options={chartOptions}
                            class="absolute inset-0 opacity-80"
                        />
                    {/if}
                </div>

                <div class="flex items-center gap-1.5">
                    <span
                        class="text-[8px] font-medium text-muted-foreground uppercase tracking-tighter"
                        >{account.account_number || $t("general.noData")}</span
                    >
                    <div class="w-1 h-1 rounded-full bg-border"></div>
                    <span class="text-[8px] font-bold text-primary"
                        >{account.currency}</span
                    >
                </div>
            </div>
        </div>
    </Card.Content>
</Card.Root>
