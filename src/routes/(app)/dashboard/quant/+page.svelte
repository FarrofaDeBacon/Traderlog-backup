<script lang="ts">
    import { rtdStore } from "$lib/stores/rtd.svelte";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import IntentionBar from "$lib/components/dashboard/quant/IntentionBar.svelte";
    import { Separator } from "$lib/components/ui/separator";
    import { Activity, Users, BarChart3, Clock } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let win = $derived(rtdStore.winQuote);
    let wdo = $derived(rtdStore.wdoQuote);
    let winBook = $derived(rtdStore.winBook);
    let wdoBook = $derived(rtdStore.wdoBook);

    let lastUpdateStr = $derived(
        rtdStore.lastUpdate
            ? rtdStore.lastUpdate.toLocaleTimeString()
            : "--:--:--",
    );
</script>

<div class="h-full flex-1 flex-col space-y-4 p-8 md:flex">
    <div class="flex items-center justify-between">
        <div>
            <h2 class="text-3xl font-bold tracking-tight">
                {$t("dashboard.quant.title")}
            </h2>
            <p class="text-muted-foreground">
                {$t("dashboard.quant.subtitle")}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Badge
                variant={rtdStore.lastUpdate ? "default" : "outline"}
                class="flex gap-1 items-center"
            >
                <Clock class="w-3 h-3" />
                {lastUpdateStr}
            </Badge>
        </div>
    </div>

    <Separator />

    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <!-- WIN Section -->
        <Card class="col-span-2">
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle class="text-sm font-medium"
                    >{$t("dashboard.quant.index")}</CardTitle
                >
                <Activity class="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold">{win?.last || "---"}</div>
                <div class="text-xs text-muted-foreground flex gap-2 mt-1">
                    <span class="text-green-500">M: {win?.high || "---"}</span>
                    <span class="text-red-500">m: {win?.low || "---"}</span>
                    <span>V: {win?.volume?.toLocaleString() || "---"}</span>
                </div>

                <div class="mt-6">
                    <IntentionBar
                        label={$t("dashboard.quant.intention")}
                        buy={winBook?.bid || 0}
                        sell={winBook?.ask || 0}
                    />
                </div>
            </CardContent>
        </Card>

        <!-- WDO Section -->
        <Card class="col-span-2">
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle class="text-sm font-medium"
                    >{$t("dashboard.quant.dollar")}</CardTitle
                >
                <Activity class="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold">{wdo?.last || "---"}</div>
                <div class="text-xs text-muted-foreground flex gap-2 mt-1">
                    <span class="text-green-500">M: {wdo?.high || "---"}</span>
                    <span class="text-red-500">m: {wdo?.low || "---"}</span>
                    <span>V: {wdo?.volume?.toLocaleString() || "---"}</span>
                </div>

                <div class="mt-6">
                    <IntentionBar
                        label={$t("dashboard.quant.intention")}
                        buy={wdoBook?.bid || 0}
                        sell={wdoBook?.ask || 0}
                    />
                </div>
            </CardContent>
        </Card>

        <!-- Player Flow (Ranking) -->
        <Card class="col-span-4">
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle
                    class="text-sm font-medium leading-none flex items-center gap-2"
                >
                    <Users class="h-4 w-4" />
                    {$t("dashboard.quant.ranking")}
                </CardTitle>
            </CardHeader>
            <CardContent>
                <div
                    class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4"
                >
                    {#each rtdStore.players.slice(0, 18) as player}
                        <div
                            class="flex flex-col border rounded-md p-2 bg-muted/10"
                        >
                            <span
                                class="text-[10px] font-bold truncate uppercase"
                                >{player.broker}</span
                            >
                            <span
                                class={player.saldo >= 0
                                    ? "text-green-500 font-mono text-xs"
                                    : "text-red-500 font-mono text-xs"}
                            >
                                {player.saldo.toLocaleString()}
                            </span>
                        </div>
                    {/each}
                    {#if rtdStore.players.length === 0}
                        <div
                            class="col-span-full h-20 flex items-center justify-center text-muted-foreground text-sm italic"
                        >
                            {$t("dashboard.quant.waitingData")}
                        </div>
                    {/if}
                </div>
            </CardContent>
        </Card>
    </div>
</div>
