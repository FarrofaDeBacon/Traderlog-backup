<script lang="ts">
    import { t, locale } from "svelte-i18n";
    import { Card, CardContent } from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import { 
        calculateSquareOfNine, 
        getGannNaturalDates, 
        isNearGannVibration,
        getCycleDayCount,
        calculateAverageTimeBetweenTrades,
        formatDuration
    } from "$lib/utils/gann";
    import { Activity, Target, Clock, Zap, Timer } from "lucide-svelte";
    
    interface Props {
        lastPrice: number;
        trades: any[];
        referenceDate?: Date;
    }
    
    let { 
        lastPrice, 
        trades = [], 
        referenceDate = new Date(new Date().getFullYear(), 0, 1) 
    }: Props = $props();
    
    // Calculations
    const expansionLevels = $derived(calculateSquareOfNine(lastPrice, [45, 90, 180, 270, 360], 1));
    const retracementLevels = $derived(calculateSquareOfNine(lastPrice, [45, 90, 180, 270, 360], -1));
    const currentVibration = $derived(isNearGannVibration());
    const daysInCycle = $derived(getCycleDayCount(referenceDate));
    
    // Find Next Natural Date
    const nextNaturalDate = $derived.by(() => {
        const now = new Date();
        const dates = getGannNaturalDates();
        return dates.find(d => d.date > now) || dates[0];
    });

    const daysToNextCot = $derived(getCycleDayCount(new Date(), nextNaturalDate.date));

    function formatPrice(val: number) {
        return new Intl.NumberFormat($locale || "pt-BR", {
            style: "currency",
            currency: "BRL"
        }).format(val);
    }
</script>

<Card class="border shadow-md bg-card/60 backdrop-blur-xl overflow-hidden">
    <CardContent class="p-0">
        <!-- Header -->
        <div class="p-4 bg-muted/30 border-b flex justify-between items-center">
            <div>
                <h3 class="text-sm font-bold tracking-tight">{$t("dashboard.gann.title")}</h3>
                <p class="text-[10px] text-muted-foreground">{$t("dashboard.gann.subtitle")}</p>
            </div>
            <Activity class="w-4 h-4 text-emerald-500" />
        </div>

        <!-- KPIs Grid -->
        <div class="grid grid-cols-1 lg:grid-cols-3 divide-x divide-y lg:divide-y-0 text-center">
            <div class="p-4 flex flex-col gap-1">
                <div class="flex items-center gap-1.5 text-[10px] font-medium text-muted-foreground uppercase">
                    <Clock class="w-3 h-3" />
                    {$t("dashboard.gann.kpis.nextCot")}
                </div>
                <div class="text-sm font-bold flex items-baseline gap-1">
                    {daysToNextCot} <span class="text-[10px] font-normal text-muted-foreground lowercase">{$t("dashboard.gann.kpis.days")}</span>
                </div>
                <div class="text-[9px] text-emerald-500 font-medium">
                    {nextNaturalDate.label}
                </div>
            </div>

            <div class="p-4 flex flex-col gap-1">
                <div class="flex items-center gap-1.5 text-[10px] font-medium text-muted-foreground uppercase">
                    <Zap class="w-3 h-3" />
                    {$t("dashboard.gann.kpis.cycleDay")}
                </div>
                <div class="text-sm font-bold">
                    {daysInCycle} <span class="text-[10px] font-normal text-muted-foreground italic">{$t("dashboard.gann.kpis.outOfDays")}</span>
                </div>
                <div class="text-[9px] text-muted-foreground">
                    {$t("dashboard.gann.kpis.masterCycle")}
                </div>
            </div>

            <div class="p-4 flex flex-col gap-1">
                <div class="flex items-center gap-1.5 text-[10px] font-medium text-muted-foreground uppercase">
                    <Target class="w-3 h-3" />
                    {$t("dashboard.gann.kpis.vibration")}
                </div>
                <div class="text-sm font-bold">
                    {#if currentVibration.isNear}
                        <span class="text-rose-500">{$t("dashboard.gann.kpis.hot_window")}</span>
                    {:else}
                        <span class="text-muted-foreground">{$t("dashboard.gann.kpis.neutral")}</span>
                    {/if}
                </div>
                <div class="text-[9px] text-rose-400 font-medium">
                    {currentVibration.label}
                </div>
            </div>

        </div>

        <Separator />

        <!-- Levels Tables -->
        <div class="grid grid-cols-1 md:grid-cols-2 divide-x">
            <!-- Expansion -->
            <div class="p-4">
                <h4 class="text-[10px] font-black uppercase tracking-widest text-muted-foreground mb-3 flex items-center gap-2">
                    <div class="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                    {$t("dashboard.gann.sq9.expansion")}
                </h4>
                <div class="space-y-2">
                    {#each expansionLevels as lv}
                        <div class="flex justify-between items-center group">
                            <span class="text-[11px] font-mono text-muted-foreground group-hover:text-foreground transition-colors">
                                {$t("dashboard.gann.sq9.degrees", { values: { deg: lv.degrees } })}
                            </span>
                            <Badge variant="outline" class="h-5 text-[10px] font-mono bg-emerald-500/5 text-emerald-500 border-emerald-500/20">
                                {formatPrice(lv.price)}
                            </Badge>
                        </div>
                    {/each}
                </div>
            </div>

            <!-- Retracement -->
            <div class="p-4">
                <h4 class="text-[10px] font-black uppercase tracking-widest text-muted-foreground mb-3 flex items-center gap-2">
                    <div class="w-1.5 h-1.5 rounded-full bg-rose-500"></div>
                    {$t("dashboard.gann.sq9.retracement")}
                </h4>
                <div class="space-y-2">
                    {#each retracementLevels as lv}
                        <div class="flex justify-between items-center group">
                            <span class="text-[11px] font-mono text-muted-foreground group-hover:text-foreground transition-colors">
                                {$t("dashboard.gann.sq9.degrees", { values: { deg: lv.degrees } })}
                            </span>
                            <Badge variant="outline" class="h-5 text-[10px] font-mono bg-rose-500/5 text-rose-500 border-rose-500/20">
                                {formatPrice(lv.price)}
                            </Badge>
                        </div>
                    {/each}
                </div>
            </div>
        </div>
    </CardContent>
</Card>
