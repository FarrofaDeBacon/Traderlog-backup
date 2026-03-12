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

<div class="space-y-6 max-h-[85vh] overflow-y-auto pr-2 custom-scrollbar">
    <!-- Header Summary (Restored large version) -->
    <div
        class="flex items-center justify-between bg-muted/30 p-5 rounded-xl border border-border/50 shadow-sm"
    >
        <div class="flex items-center gap-5">
            <div
                class={`p-3 rounded-full ${trade.direction === "buy" ? "bg-green-500/20 text-green-500" : "bg-rose-500/20 text-rose-500"}`}
            >
                {#if trade.direction === "buy"}
                    <TrendingUp class="w-7 h-7" />
                {:else}
                    <TrendingDown class="w-7 h-7" />
                {/if}
            </div>
            <div>
                <h3 class="text-2xl font-black text-foreground tracking-tight leading-none mb-1">
                    {trade.asset_symbol}
                </h3>
                <div class="flex items-center gap-2 text-sm text-muted-foreground font-medium">
                    <Badge variant="outline" class="uppercase px-2 h-5 text-[10px] font-bold border-muted-foreground/30">
                        {trade.direction === "buy" || trade.direction === "Buy" ? $t("trades.table.buy") : $t("trades.table.sell")}
                    </Badge>
                    <span class="opacity-30">•</span>
                    <span class="flex items-center gap-1.5">
                        <Calendar class="w-3.5 h-3.5 text-primary/60" />
                        {trade.date?.split('T')[0] || trade.entry_date?.split('T')[0]}
                    </span>
                </div>
            </div>
        </div>
        <div class="text-right">
            <div
                class={`text-3xl font-mono font-black tracking-tighter leading-none ${(trade.result || 0) >= 0 ? "text-green-500" : "text-rose-500"}`}
            >
                {formatCurrency(trade.result || 0, account?.currency || "BRL", $locale || "pt-BR")}
            </div>
            <div class="text-[10px] text-muted-foreground font-bold uppercase tracking-[0.1em] mt-1.5 opacity-70">
                {$t("trades.table.pl")}
            </div>
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Execution Data -->
        <Card.Root class="glass border-border/50 overflow-hidden shadow-sm">
            <Card.Header class="py-3 px-5 bg-muted/20 border-b border-border/10">
                <Card.Title class="text-[11px] font-bold flex items-center gap-2 text-primary/80 uppercase tracking-widest">
                    <Target class="w-4 h-4" />
                    {$t("trades.details.execution")}
                </Card.Title>
            </Card.Header>
            <Card.Content class="p-6">
                <div class="grid grid-cols-2 gap-x-8 gap-y-5">
                    <div class="space-y-1.5">
                        <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-tight opacity-70">{$t("trades.details.entry_price")}</span>
                        <div class="text-lg font-mono font-bold text-foreground">
                            {trade.entry_price?.toLocaleString($locale || "pt-BR")}
                        </div>
                    </div>
                    <div class="space-y-1.5">
                        <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-tight opacity-70">{$t("trades.details.exit_price")}</span>
                        <div class="text-lg font-mono font-bold text-foreground">
                            {trade.exit_price?.toLocaleString($locale || "pt-BR") || "-"}
                        </div>
                    </div>
                    <div class="space-y-1.5">
                        <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-tight opacity-70">{$t("trades.table.quantity")}</span>
                        <div class="text-lg font-mono font-bold text-foreground">{trade.quantity}</div>
                    </div>
                    <div class="space-y-1.5">
                        <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-tight opacity-70">{$t("trades.details.fees")}</span>
                        <div class="text-lg font-mono font-bold text-red-400">
                            {formatCurrency(trade.fee_total || 0, account?.currency || "BRL", $locale || "pt-BR")}
                        </div>
                    </div>
                </div>
            </Card.Content>
        </Card.Root>

        <!-- Basic Info & Psiquê -->
        <div class="space-y-6">
            <Card.Root class="glass border-border/50 overflow-hidden shadow-sm">
                <Card.Header class="py-3 px-5 bg-muted/20 border-b border-border/10">
                    <Card.Title class="text-[11px] font-bold flex items-center gap-2 text-primary/80 uppercase tracking-widest">
                        <Briefcase class="w-4 h-4" />
                        {$t("trades.details.basic_info")}
                    </Card.Title>
                </Card.Header>
                <Card.Content class="p-6 space-y-4">
                    <div class="flex justify-between items-center">
                        <span class="text-xs font-semibold text-muted-foreground uppercase opacity-70">Conta</span>
                        <span class="text-sm font-bold text-foreground">{account?.nickname || "-"}</span>
                    </div>
                    <Separator class="bg-border/5" />
                    <div class="flex justify-between items-center">
                        <span class="text-xs font-semibold text-muted-foreground uppercase opacity-70">Estratégia</span>
                        <Badge variant="secondary" class="bg-blue-500/10 text-blue-400 border-blue-500/20 font-bold px-3">
                            {strategy?.name || "N/A"}
                        </Badge>
                    </div>
                    <Separator class="bg-border/5" />
                    <div class="flex justify-between items-center">
                        <span class="text-xs font-semibold text-muted-foreground uppercase opacity-70">Modalidade</span>
                        <span class="text-sm font-bold text-foreground">{modality?.name || "-"}</span>
                    </div>
                </Card.Content>
            </Card.Root>

            <Card.Root class="glass border-border/50 overflow-hidden shadow-sm">
                <Card.Header class="py-3 px-5 bg-muted/20 border-b border-border/10">
                    <Card.Title class="text-[11px] font-bold flex items-center gap-2 text-primary/80 uppercase tracking-widest">
                        <Brain class="w-4 h-4" />
                        {$t("trades.details.psychology")}
                    </Card.Title>
                </Card.Header>
                <Card.Content class="p-6 flex flex-col gap-4">
                    <div class="flex justify-between items-center">
                        <span class="text-xs font-semibold text-muted-foreground uppercase tracking-tight opacity-70">Sentimento Entrada</span>
                        <Badge class={`px-3 py-0.5 font-bold ${entryEmotion?.impact === "Positive" ? "bg-green-500/20 text-green-500" : "bg-rose-500/20 text-rose-500"}`}>{entryEmotion?.name || "-"}</Badge>
                    </div>
                    <div class="flex justify-between items-center">
                        <span class="text-xs font-semibold text-muted-foreground uppercase tracking-tight opacity-70">Sentimento Saída</span>
                        <Badge class={`px-3 py-0.5 font-bold ${exitEmotion?.impact === "Positive" ? "bg-green-500/20 text-green-500" : "bg-rose-500/20 text-rose-500"}`}>{exitEmotion?.name || "-"}</Badge>
                    </div>
                </Card.Content>
            </Card.Root>
        </div>

        <!-- Partial Exits (Spacious & Detailed) -->
        {#if trade.partial_exits?.length > 0}
            <Card.Root class="glass border-border/50 md:col-span-2 shadow-sm overflow-hidden">
                <Card.Header class="py-3 px-5 bg-muted/20 border-b border-border/10 flex flex-row items-center justify-between">
                    <Card.Title class="text-[11px] font-bold flex items-center gap-2 text-primary/80 uppercase tracking-widest">
                        <ArrowRightLeft class="w-4 h-4" />
                        {$t("trades.details.partial_exits")}
                    </Card.Title>
                    <Badge variant="outline" class="text-[10px] font-bold border-blue-500/30 text-blue-400 tracking-widest bg-blue-500/5">
                        PREÇO MÉDIO MÓVEL
                    </Badge>
                </Card.Header>
                <Card.Content class="p-0">
                    <div class="overflow-x-auto">
                        <table class="w-full">
                            <thead>
                                <tr class="text-left text-muted-foreground bg-muted/10 border-b border-border/20">
                                    <th class="py-4 px-6 font-bold uppercase text-[10px] tracking-wider">{$t("trades.details.partial_date")}</th>
                                    <th class="py-4 px-6 font-bold text-right uppercase text-[10px] tracking-wider">{$t("trades.details.partial_price")}</th>
                                    <th class="py-4 px-6 font-bold text-right uppercase text-[10px] tracking-wider text-blue-400">MÉDIO NO MOMENTO</th>
                                    <th class="py-4 px-6 font-bold text-right uppercase text-[10px] tracking-wider">{$t("trades.details.partial_qty")}</th>
                                    <th class="py-4 px-6 font-bold text-right uppercase text-[10px] tracking-wider">{$t("trades.table.pl")}</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-border/10">
                                {#each (() => {
                                    let currentAvg = trade.entry_price || 0;
                                    let currentQty = trade.quantity || 0;
                                    const isBuy = trade.direction?.toLowerCase() === "buy";
                                    const pointValue = asset?.point_value || 1;
                                    
                                    return (trade.partial_exits || [])
                                        .map((p: any) => ({ ...p, 
                                            date: p.date || p.exit_date, 
                                            price: Number(p.price || p.exit_price || 0)
                                        }))
                                        .sort((a: any, b: any) => new Date(a.date).getTime() - new Date(b.date).getTime())
                                        .map((partial: any) => {
                                            const isAddition = partial.type === "addition";
                                            const qty = Math.abs(partial.quantity);
                                            let displayAvg = currentAvg;
                                            let result = 0;

                                            if (isAddition) {
                                                const totalCost = (currentAvg * currentQty) + (partial.price * qty);
                                                currentQty += qty;
                                                currentAvg = totalCost / currentQty;
                                                displayAvg = currentAvg;
                                                result = 0;
                                            } else {
                                                displayAvg = currentAvg;
                                                const points = isBuy ? (partial.price - currentAvg) : (currentAvg - partial.price);
                                                result = points * qty * pointValue;
                                                currentQty -= qty;
                                            }

                                            return {
                                                ...partial,
                                                isAddition,
                                                avgAtMoment: displayAvg,
                                                calculatedResult: result
                                            };
                                        });
                                })() as partial}
                                    <tr class="hover:bg-muted/10 transition-colors">
                                        <td class="py-4 px-6 text-xs font-mono text-muted-foreground">
                                            {partial.date ? new Date(partial.date).toLocaleString($locale || 'pt-BR', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' }) : "-"}
                                        </td>
                                        <td class="py-4 px-6 text-right font-mono text-foreground font-bold">
                                            {partial.price?.toLocaleString($locale || "pt-BR")}
                                        </td>
                                        <td class="py-4 px-6 text-right font-mono text-blue-400 font-bold bg-blue-400/5">
                                            {partial.avgAtMoment?.toLocaleString($locale || "pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                        </td>
                                        <td class="py-4 px-6 text-right font-mono font-black text-foreground">
                                            <Badge variant="outline" class="mr-2 font-mono h-5">{Math.abs(partial.quantity)}</Badge>
                                        </td>
                                        <td class={`py-4 px-6 text-right font-mono font-black ${(partial.calculatedResult || 0) >= 0 ? "text-green-500" : "text-rose-500"}`}>
                                            {#if partial.isAddition}
                                                <span class="text-muted-foreground/30 italic text-[10px]">Aporte</span>
                                            {:else}
                                                {formatCurrency(partial.calculatedResult || 0, account?.currency || "BRL", $locale || "pt-BR")}
                                            {/if}
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                </Card.Content>
            </Card.Root>
        {/if}

        <!-- Notes & Images -->
        <Card.Root class="glass border-border/50 md:col-span-2 shadow-sm overflow-hidden">
            <Card.Header class="py-3 px-5 bg-muted/20 border-b border-border/10">
                <Card.Title class="text-[11px] font-bold flex items-center gap-2 text-primary/80 uppercase tracking-widest">
                    <FileText class="w-4 h-4" />
                    ANOTAÇÕES E MÍDIA
                </Card.Title>
            </Card.Header>
            <Card.Content class="p-6 space-y-8">
                <div class="bg-muted/10 p-6 rounded-2xl border border-border/10 min-h-[120px] shadow-inner">
                    <p class="text-base text-foreground/90 leading-relaxed whitespace-pre-wrap font-medium italic">
                        {trade.notes || $t("trades.details.no_notes")}
                    </p>
                </div>

                {#if trade.images && trade.images.length > 0}
                    <div class="space-y-4">
                        <div class="flex items-center gap-2 text-[10px] font-bold uppercase tracking-widest text-muted-foreground opacity-60">
                            <Camera class="w-4 h-4" />
                            Capturas de Tela ({trade.images.length})
                        </div>
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                            {#each trade.images as img, i}
                                <button
                                    class="aspect-video bg-muted/30 rounded-xl overflow-hidden border border-border/20 group relative cursor-pointer shadow-md hover:border-primary/40 hover:shadow-lg transition-all"
                                    onclick={() => (selectedImageIndex = i)}
                                >
                                    <img
                                        src={img}
                                        alt="Trade Snapshot"
                                        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-700"
                                    />
                                    <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center backdrop-blur-[1px]">
                                        <div class="bg-white/20 p-3 rounded-full border border-white/30 transform translate-y-2 group-hover:translate-y-0 transition-transform">
                                            <Camera class="w-6 h-6 text-white" />
                                        </div>
                                    </div>
                                </button>
                            {/each}
                        </div>
                    </div>
                {:else}
                    <div class="flex flex-col items-center justify-center py-16 text-muted-foreground bg-muted/5 rounded-2xl border border-dashed border-border/20 opacity-60">
                        <Camera class="w-12 h-12 mb-4 opacity-10" />
                        <p class="text-[10px] font-bold uppercase tracking-[0.2em] opacity-40">
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
