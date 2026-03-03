<script lang="ts">
    import { t, locale } from "svelte-i18n";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import * as Card from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import { formatCurrency } from "$lib/utils";
    import {
        TrendingUp,
        TrendingDown,
        Calendar,
        Clock,
        Target,
        Brain,
        Camera,
        FileText,
        Briefcase,
        ArrowRightLeft,
        DollarSign,
    } from "lucide-svelte";
    import ImageCarousel from "$lib/components/ui/ImageCarousel.svelte";

    const { trade } = $props<{ trade: any }>();
    let selectedImageIndex = $state<number | null>(null);

    const asset = $derived(
        settingsStore.assets.find((a) => a.symbol === trade.asset_symbol),
    );
    const strategy = $derived(
        settingsStore.strategies.find((s) => s.id === trade.strategy_id),
    );
    const account = $derived(
        settingsStore.accounts.find((a) => a.id === trade.account_id),
    );

    function getEmotionalState(id: string) {
        return settingsStore.emotionalStates.find((s) => s.id === id);
    }

    const entryEmotion = $derived(
        getEmotionalState(trade.entry_emotional_state_id),
    );
    const exitEmotion = $derived(
        getEmotionalState(trade.exit_emotional_state_id),
    );
    const modality = $derived(
        settingsStore.modalities.find((m) => m.id === trade.modality_id),
    );
</script>

<div class="space-y-6 max-h-[80vh] overflow-y-auto pr-2 custom-scrollbar">
    <!-- Header Summary -->
    <div
        class="flex items-center justify-between bg-muted/30 p-4 rounded-lg border border-border/50"
    >
        <div class="flex items-center gap-4">
            <div
                class={`p-3 rounded-full ${trade.direction === "buy" ? "bg-green-500/20 text-green-500" : "bg-red-500/20 text-red-500"}`}
            >
                {#if trade.direction === "buy"}
                    <TrendingUp class="w-6 h-6" />
                {:else}
                    <TrendingDown class="w-6 h-6" />
                {/if}
            </div>
            <div>
                <h3 class="text-xl font-bold text-foreground">
                    {trade.asset_symbol}
                </h3>
                <div
                    class="flex items-center gap-2 text-sm text-muted-foreground"
                >
                    <Badge variant="outline" class="uppercase">
                        {trade.direction === "buy" || trade.direction === "Buy"
                            ? $t("trades.table.buy")
                            : $t("trades.table.sell")}
                    </Badge>
                    <span>•</span>
                    <span class="flex items-center gap-1">
                        <Calendar class="w-3 h-3" />
                        {trade.date || trade.entry_date}
                    </span>
                </div>
            </div>
        </div>
        <div class="text-right">
            <div
                class={`text-2xl font-mono font-bold ${(trade.result || 0) >= 0 ? "text-green-500" : "text-red-500"}`}
            >
                {formatCurrency(
                    trade.result || 0,
                    account?.currency || "BRL",
                    $locale || "pt-BR",
                )}
            </div>
            <div class="text-xs text-muted-foreground uppercase tracking-wider">
                {$t("trades.table.pl")}
            </div>
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Execution Data -->
        <Card.Root class="glass border-border/50">
            <Card.Header class="pb-2">
                <Card.Title
                    class="text-sm font-medium flex items-center gap-2 text-primary"
                >
                    <Target class="w-4 h-4" />
                    {$t("trades.details.execution")}
                </Card.Title>
            </Card.Header>
            <Card.Content class="space-y-3">
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <span class="text-xs text-muted-foreground block"
                            >{$t("trades.details.entry_price")}</span
                        >
                        <span
                            class="font-mono font-bold text-foreground text-sm"
                            >{trade.entry_price?.toLocaleString(
                                $locale || "pt-BR",
                            )}</span
                        >
                    </div>
                    <div>
                        <span class="text-xs text-muted-foreground block"
                            >{$t("trades.details.exit_price")}</span
                        >
                        <span
                            class="font-mono font-bold text-foreground text-sm"
                            >{trade.exit_price?.toLocaleString(
                                $locale || "pt-BR",
                            ) || "-"}</span
                        >
                    </div>
                    <div>
                        <span class="text-xs text-muted-foreground block"
                            >{$t("trades.table.quantity")}</span
                        >
                        <span
                            class="text-foreground text-sm font-mono font-bold"
                            >{trade.quantity}</span
                        >
                    </div>
                    <div>
                        <span class="text-xs text-muted-foreground block"
                            >{$t("trades.details.fees")}</span
                        >
                        <span class="text-red-400 text-sm font-mono font-bold"
                            >{formatCurrency(
                                trade.fee_total || 0,
                                account?.currency || "BRL",
                                $locale || "pt-BR",
                            )}</span
                        >
                    </div>
                </div>
                <Separator class="bg-border/10" />
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <span class="text-xs text-muted-foreground block"
                            >{$t("trades.details.stop_loss")}</span
                        >
                        <span
                            class="font-mono font-bold text-red-500/80 text-sm"
                            >{trade.stop_loss || "-"}</span
                        >
                    </div>
                    <div>
                        <span class="text-xs text-muted-foreground block"
                            >{$t("trades.details.take_profit")}</span
                        >
                        <span
                            class="font-mono font-bold text-green-500/80 text-sm"
                            >{trade.take_profit || "-"}</span
                        >
                    </div>
                </div>
            </Card.Content>
        </Card.Root>

        <!-- Context & Account -->
        <Card.Root class="glass border-border/50">
            <Card.Header class="pb-2">
                <Card.Title
                    class="text-sm font-medium flex items-center gap-2 text-primary"
                >
                    <Briefcase class="w-4 h-4" />
                    {$t("trades.details.basic_info")}
                </Card.Title>
            </Card.Header>
            <Card.Content class="space-y-3">
                <div class="space-y-2">
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-muted-foreground"
                            >{$t("trades.filters.account")}</span
                        >
                        <span class="text-foreground font-medium"
                            >{account?.nickname || "-"}</span
                        >
                    </div>
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-muted-foreground"
                            >{$t("trades.table.asset_type")}</span
                        >
                        <span class="text-foreground">
                            {settingsStore.assetTypes.find(
                                (at) => at.id === trade.asset_type_id,
                            )?.name || "-"}
                        </span>
                    </div>
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-muted-foreground"
                            >{$t("trades.filters.strategy")}</span
                        >
                        <Badge
                            variant="secondary"
                            class="bg-blue-500/10 text-blue-400 border-blue-500/20"
                        >
                            {strategy?.name || $t("trades.wizard.summary.na")}
                        </Badge>
                    </div>
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-muted-foreground"
                            >{$t("trades.filters.modality")}</span
                        >
                        <span class="text-foreground"
                            >{modality?.name || "-"}</span
                        >
                    </div>
                </div>
            </Card.Content>
        </Card.Root>

        <!-- Psychology -->
        <Card.Root class="glass border-border/50 md:col-span-2">
            <Card.Header class="pb-2">
                <Card.Title
                    class="text-sm font-medium flex items-center gap-2 text-primary"
                >
                    <Brain class="w-4 h-4" />
                    {$t("trades.details.psychology")}
                </Card.Title>
            </Card.Header>
            <Card.Content>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div
                        class="space-y-2 p-3 rounded-lg bg-white/5 border border-white/5"
                    >
                        <span class="text-xs text-muted-foreground block"
                            >{$t("trades.details.emotional_entry")}</span
                        >
                        {#if entryEmotion}
                            <div class="flex items-center gap-2">
                                <Badge
                                    class={entryEmotion.impact === "Positive"
                                        ? "bg-green-500/20 text-green-500"
                                        : "bg-red-500/20 text-red-500"}
                                >
                                    {entryEmotion.name}
                                </Badge>
                                <span
                                    class="text-xs text-muted-foreground italic"
                                >
                                    "{entryEmotion.description}"
                                </span>
                            </div>
                        {:else}
                            {$t("trades.wizard.placeholders.no_states")}
                        {/if}
                    </div>

                    <div
                        class="space-y-2 p-3 rounded-lg bg-white/5 border border-white/5"
                    >
                        <span class="text-xs text-muted-foreground block"
                            >{$t("trades.details.emotional_exit")}</span
                        >
                        {#if exitEmotion}
                            <div class="flex items-center gap-2">
                                <Badge
                                    class={exitEmotion.impact === "Positive"
                                        ? "bg-green-500/20 text-green-500"
                                        : "bg-red-500/20 text-red-500"}
                                >
                                    {exitEmotion.name}
                                </Badge>
                                <span
                                    class="text-xs text-muted-foreground italic"
                                >
                                    "{exitEmotion.description}"
                                </span>
                            </div>
                        {:else}
                            <span class="text-sm text-muted-foreground"
                                >{$t(
                                    "trades.wizard.placeholders.no_states",
                                )}</span
                            >
                        {/if}
                    </div>
                </div>

                <div
                    class="mt-4 p-4 rounded-lg bg-muted/20 border border-border/20"
                >
                    <div
                        class="flex items-center gap-2 mb-2 text-xs font-semibold uppercase text-muted-foreground"
                    >
                        <ArrowRightLeft class="w-3 h-3" />
                        {$t("trades.wizard.adherence.title")}
                    </div>
                    <div class="flex items-center gap-2">
                        {#if trade.followed_plan}
                            <Badge class="bg-emerald-500 text-black"
                                >{$t("trades.wizard.summary.s")}</Badge
                            >
                            <span class="text-sm text-emerald-500/80"
                                >{$t("trades.wizard.adherence.yes_msg")}</span
                            >
                        {:else}
                            <Badge variant="destructive"
                                >{$t("trades.wizard.summary.n")}</Badge
                            >
                            <span class="text-sm text-red-500/80"
                                >{$t("trades.wizard.adherence.no_msg")}</span
                            >
                        {/if}
                    </div>
                </div>
            </Card.Content>
        </Card.Root>

        <!-- Notes & Images -->
        <Card.Root class="glass border-border/50 md:col-span-2">
            <Card.Header class="pb-2">
                <Card.Title
                    class="text-sm font-medium flex items-center gap-2 text-primary"
                >
                    <FileText class="w-4 h-4" />
                    {$t("trades.details.media")}
                </Card.Title>
            </Card.Header>
            <Card.Content class="space-y-4">
                <div
                    class="p-4 rounded-lg bg-muted/20 border border-dashed border-border/50"
                >
                    <p
                        class="text-sm text-foreground/90 leading-relaxed whitespace-pre-wrap"
                    >
                        {trade.notes || $t("trades.details.no_notes")}
                    </p>
                </div>

                {#if trade.images && trade.images.length > 0}
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
                        {#each trade.images as img, i}
                            <button
                                class="aspect-video bg-muted/50 rounded overflow-hidden border border-border/10 group relative cursor-pointer"
                                onclick={() => (selectedImageIndex = i)}
                            >
                                <img
                                    src={img}
                                    alt="Trade Screenshot"
                                    class="w-full h-full object-cover"
                                />
                                <div
                                    class="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
                                >
                                    <Camera class="w-5 h-5 text-foreground" />
                                </div>
                            </button>
                        {/each}
                    </div>
                {:else}
                    <div class="text-center py-6 text-muted-foreground">
                        <Camera class="w-8 h-8 mx-auto mb-2 opacity-20" />
                        <p class="text-xs uppercase tracking-widest">
                            {$t("trades.details.no_media")}
                        </p>
                    </div>
                {/if}
            </Card.Content>
        </Card.Root>
    </div>
</div>

<ImageCarousel images={trade.images} bind:index={selectedImageIndex} />

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: rgba(var(--muted), 0.05);
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(var(--muted-foreground), 0.2);
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: rgba(var(--muted-foreground), 0.3);
    }
</style>
