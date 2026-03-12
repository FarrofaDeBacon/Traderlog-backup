<script lang="ts">
    import { t, locale } from "svelte-i18n";
    import { quantEngine } from "$lib/services/quantAnalysis.svelte";
    import * as Card from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import {
        Activity,
        Landmark,
        TrendingUp,
        TrendingDown,
    } from "lucide-svelte";
    import IntentionBar from "./quant/IntentionBar.svelte";
    import { rtdStore } from "$lib/stores/rtd.svelte";

    let sentiment = $derived(quantEngine.sentiment);
    let win = $derived(rtdStore.winQuote);
    let wdo = $derived(rtdStore.wdoQuote);

    function formatPercent(val: number) {
        return val.toFixed(2) + "%";
    }

    function formatBRL(val: number) {
        return new Intl.NumberFormat($locale || "pt-BR", {
            style: "currency",
            currency: "BRL",
        }).format(val);
    }
</script>

<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
    <!-- WIN Signal -->
    <Card.Root class="border-l-2 border-l-blue-500 shadow-sm bg-card">
        <Card.Header
            class="flex flex-row items-center justify-between space-y-0 py-0.5 px-2"
        >
            <Card.Title
                class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                >{$t("dashboard.market_context.win")}</Card.Title
            >
            <Activity class="h-3 w-3 text-blue-500" />
        </Card.Header>
        <Card.Content class="py-0.5 px-2">
            <div
                class="text-base font-mono font-bold tracking-tight leading-none"
            >
                {win?.last || "---"}
            </div>
            <div class="mt-3">
                <IntentionBar
                    buy={rtdStore.winBook?.bid || 0}
                    sell={rtdStore.winBook?.ask || 0}
                />
            </div>
        </Card.Content>
    </Card.Root>

    <!-- WDO Signal -->
    <Card.Root class="border-l-2 border-l-emerald-500 shadow-sm bg-card">
        <Card.Header
            class="flex flex-row items-center justify-between space-y-0 py-0.5 px-2"
        >
            <Card.Title
                class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                >{$t("dashboard.market_context.wdo")}</Card.Title
            >
            <Activity class="h-3 w-3 text-emerald-500" />
        </Card.Header>
        <Card.Content class="py-0.5 px-2">
            <div
                class="text-base font-mono font-bold tracking-tight leading-none"
            >
                {wdo?.last || "---"}
            </div>
            <div class="mt-3">
                <IntentionBar
                    buy={rtdStore.wdoBook?.bid || 0}
                    sell={rtdStore.wdoBook?.ask || 0}
                />
            </div>
        </Card.Content>
    </Card.Root>

    <!-- SELIC -->
    <Card.Root class="border-l-2 border-l-indigo-500 shadow-sm bg-card">
        <Card.Header
            class="flex flex-row items-center justify-between space-y-0 py-0.5 px-2"
        >
            <Card.Title
                class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                >{$t("dashboard.market_context.selic")}</Card.Title
            >
            <Landmark class="h-3 w-3 text-indigo-500" />
        </Card.Header>
        <Card.Content class="py-0.5 px-2">
            <div
                class="text-base font-mono font-bold tracking-tight leading-none"
            >
                {formatPercent(quantEngine.selic)}
            </div>
            <p
                class="text-[9px] text-muted-foreground mt-1 uppercase font-medium"
            >
                {$t("dashboard.market_context.selic_desc")}
            </p>
        </Card.Content>
    </Card.Root>

    <!-- CDI (DI1) -->
    <Card.Root class="border-l-2 border-l-amber-500 shadow-sm bg-card">
        <Card.Header
            class="flex flex-row items-center justify-between space-y-0 py-0.5 px-2"
        >
            <Card.Title
                class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                >{$t("dashboard.market_context.di")}</Card.Title
            >
            <TrendingUp class="h-3 w-3 text-amber-500" />
        </Card.Header>
        <Card.Content class="py-0.5 px-2">
            <div
                class="text-base font-mono font-bold tracking-tight leading-none"
            >
                {formatPercent(quantEngine.cdi)}
            </div>
            <p
                class="text-[9px] text-muted-foreground mt-1 uppercase font-medium"
            >
                {$t("dashboard.market_context.di_desc")}
            </p>
        </Card.Content>
    </Card.Root>

    <!-- PTAX -->
    <Card.Root class="border-l-2 border-l-slate-400 shadow-sm bg-card">
        <Card.Header
            class="flex flex-row items-center justify-between space-y-0 py-0.5 px-2"
        >
            <Card.Title
                class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
                >{$t("dashboard.market_context.ptax")}</Card.Title
            >
            <Landmark class="h-3 w-3 text-muted-foreground" />
        </Card.Header>
        <Card.Content class="py-0.5 px-2">
            <div
                class="text-base font-mono font-bold tracking-tight leading-none"
            >
                {formatBRL(quantEngine.ptax)}
            </div>
            <p
                class="text-[9px] text-muted-foreground mt-1 uppercase font-medium"
            >
                {$t("dashboard.market_context.ptax_desc")}
            </p>
        </Card.Content>
    </Card.Root>
</div>div>
