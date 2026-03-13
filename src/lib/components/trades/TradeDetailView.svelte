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
        Maximize2,
    } from "lucide-svelte";
    import ImageCarousel from "$lib/components/ui/ImageCarousel.svelte";

    const { trade } = $props<{ trade: any }>();
    let selectedImageIndex = $state<number | null>(null);

    const asset = $derived(
        settingsStore.assets.find((a) => {
            const sym = a.symbol.toUpperCase();
            const tradeSym = trade.asset_symbol.toUpperCase();
            // Match exactly or handle futures suffixes (e.g. WIN vs WINN24)
            return sym === tradeSym || 
                   (sym === "WIN" && tradeSym.startsWith("WIN")) ||
                   (sym === "WDO" && tradeSym.startsWith("WDO"));
        }),
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
    <!-- Status & Period (Header) -->
    <div class="flex items-center justify-between">
        <div class="space-y-1">
            <span class="text-[10px] text-muted-foreground uppercase font-black tracking-widest opacity-60">
                {$t("trades.table.asset")} & {$t("trades.table.date")}
            </span>
            <div class="flex items-center gap-3">
                <div class={`p-1.5 rounded-lg ${trade.direction?.toLowerCase() === "buy" ? "bg-green-500/10 text-green-500" : "bg-rose-500/10 text-rose-500"}`}>
                    {#if trade.direction?.toLowerCase() === "buy"}
                        <TrendingUp class="w-4 h-4" />
                    {:else}
                        <TrendingDown class="w-4 h-4" />
                    {/if}
                </div>
                <div class="flex items-baseline gap-2">
                    <span class="text-xl font-black tracking-tight">{trade.asset_symbol}</span>
                    <span class="text-xs font-bold text-muted-foreground font-mono opacity-50">/</span>
                    <span class="text-sm font-bold text-muted-foreground font-mono">
                        {trade.date?.split('T')[0] || trade.entry_date?.split('T')[0]}
                    </span>
                </div>
            </div>
        </div>
        <div class="flex flex-col items-end gap-1.5">
            <div class={`text-2xl font-black font-mono tracking-tighter ${(trade.result || 0) >= 0 ? "text-green-500" : "text-rose-500"}`}>
                {formatCurrency(trade.result || 0, account?.currency || "BRL", $locale || "pt-BR")}
            </div>
            <div class="flex gap-2">
                <Badge variant="outline" class="uppercase px-2 h-4 text-[9px] font-bold border-muted-foreground/20 text-muted-foreground tracking-widest">
                    {trade.direction?.toLowerCase() === "buy" ? $t("trades.table.buy") : $t("trades.table.sell")}
                </Badge>
                {#if trade.status === "closed"}
                    <Badge variant="outline" class="bg-blue-500/10 text-blue-400 border-blue-500/20 px-2 h-4 text-[9px] font-black uppercase tracking-widest">
                        FECHADA
                    </Badge>
                {/if}
            </div>
        </div>
    </div>

    <Separator class="bg-border/30" />

    <!-- Execution & Account Info (Grid) -->
    <div class="space-y-4">
        <h4 class="text-[10px] font-black text-muted-foreground uppercase tracking-widest flex items-center gap-2">
            <Target class="w-3.5 h-3.5" />
            {$t("trades.details.execution")} & {$t("trades.details.basic_info")}
        </h4>
        
        <div class="bg-muted/30 border border-border/40 rounded-xl p-4 grid grid-cols-3 gap-y-5 gap-x-4">
            <div class="space-y-1">
                <span class="text-[9px] text-muted-foreground uppercase font-bold tracking-wider opacity-70">
                    {$t("trades.details.entry_price")}
                </span>
                <p class="text-sm font-black font-mono text-foreground">
                    {trade.entry_price?.toLocaleString($locale || "pt-BR")}
                </p>
            </div>
            <div class="space-y-1">
                <span class="text-[9px] text-muted-foreground uppercase font-bold tracking-wider opacity-70">
                    {$t("trades.details.exit_price")}
                </span>
                <p class="text-sm font-black font-mono text-foreground">
                    {trade.exit_price?.toLocaleString($locale || "pt-BR") || "-"}
                </p>
            </div>
            <div class="space-y-1">
                <span class="text-[9px] text-muted-foreground uppercase font-bold tracking-wider opacity-70">
                    {$t("trades.table.quantity")}
                </span>
                <p class="text-sm font-black font-mono text-foreground">
                    {trade.quantity}
                </p>
            </div>
            <div class="space-y-1">
                <span class="text-[9px] text-muted-foreground uppercase font-bold tracking-wider opacity-70">
                    {$t("trades.details.fees")}
                </span>
                <p class="text-sm font-black font-mono text-rose-400">
                    {formatCurrency(trade.fee_total || 0, account?.currency || "BRL", $locale || "pt-BR")}
                </p>
            </div>
            <div class="space-y-1">
                <span class="text-[9px] text-muted-foreground uppercase font-bold tracking-wider opacity-70">
                    CONTA
                </span>
                <p class="text-sm font-black truncate text-foreground/80 lowercase first-letter:uppercase">
                    {account?.nickname || "-"}
                </p>
            </div>
            <div class="space-y-1">
                <span class="text-[9px] text-muted-foreground uppercase font-bold tracking-wider opacity-70">
                    ESTRATÉGIA
                </span>
                <p class="text-[11px] font-bold text-blue-400 truncate uppercase mt-0.5">
                    {strategy?.name || "N/A"}
                </p>
            </div>
        </div>
    </div>

    <!-- Psychology Section -->
    <div class="space-y-3">
        <h4 class="text-[10px] font-black text-muted-foreground uppercase tracking-widest flex items-center gap-2">
            <Brain class="w-3.5 h-3.5" />
            {$t("trades.details.psychology")}
        </h4>
        <div class="grid grid-cols-2 gap-3">
            <div class="flex items-center justify-between p-3 bg-muted/20 border border-border/30 rounded-lg">
                <span class="text-[10px] text-muted-foreground uppercase font-bold opacity-70">Entrada</span>
                <Badge class={`px-2 h-5 text-[10px] font-bold ${entryEmotion?.impact === "Positive" ? "bg-green-500/10 text-green-500 border-green-500/20" : "bg-rose-500/10 text-rose-500 border-rose-500/20"}`} variant="outline">
                    {entryEmotion?.name || "-"}
                </Badge>
            </div>
            <div class="flex items-center justify-between p-3 bg-muted/20 border border-border/30 rounded-lg">
                <span class="text-[10px] text-muted-foreground uppercase font-bold opacity-70">Saída</span>
                <Badge class={`px-2 h-5 text-[10px] font-bold ${exitEmotion?.impact === "Positive" ? "bg-green-500/10 text-green-500 border-green-500/20" : "bg-rose-500/10 text-rose-500 border-rose-500/20"}`} variant="outline">
                    {exitEmotion?.name || "-"}
                </Badge>
            </div>
        </div>
    </div>

    <!-- Partial Exits -->
    {#if trade.partial_exits?.length > 0}
        <div class="space-y-3">
            <div class="flex items-center justify-between">
                <h4 class="text-[10px] font-black text-muted-foreground uppercase tracking-widest flex items-center gap-2">
                    <ArrowRightLeft class="w-3.5 h-3.5" />
                    {$t("trades.details.partial_exits")}
                </h4>
                <Badge variant="outline" class="text-[8px] font-bold border-blue-500/20 text-blue-400 tracking-wider bg-blue-500/5 h-4">
                    MÉDIO DINÂMICO
                </Badge>
            </div>
            <div class="border border-border/30 rounded-xl overflow-hidden bg-muted/10">
                <table class="w-full text-xs">
                    <thead>
                        <tr class="text-left text-muted-foreground/60 border-b border-border/20 bg-muted/20">
                            <th class="py-2.5 px-4 font-black uppercase text-[9px] tracking-wider">TIPO</th>
                            <th class="py-2.5 px-4 font-black uppercase text-[9px] tracking-wider border-l border-border/5">DATA</th>
                            <th class="py-2.5 px-4 font-black text-right uppercase text-[9px] tracking-wider border-l border-border/5">PREÇO</th>
                            <th class="py-2.5 px-4 font-black text-right uppercase text-[9px] tracking-wider text-blue-400 border-l border-border/5">MÉDIO</th>
                            <th class="py-2.5 px-4 font-black text-right uppercase text-[9px] tracking-wider border-l border-border/5">QTY</th>
                            <th class="py-2.5 px-4 font-black text-right uppercase text-[9px] tracking-wider border-l border-border/5">RESULTADO</th>
                            <th class="py-2.5 px-4 font-black uppercase text-[9px] tracking-wider border-l border-border/5">OBS</th>
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
                                    let points = 0;

                                    if (isAddition) {
                                        const totalCost = (currentAvg * currentQty) + (partial.price * qty);
                                        currentQty += qty;
                                        currentAvg = totalCost / currentQty;
                                        displayAvg = currentAvg;
                                        result = 0;
                                        points = 0;
                                    } else {
                                        displayAvg = currentAvg;
                                        points = isBuy ? (partial.price - currentAvg) : (currentAvg - partial.price);
                                        result = points * qty * pointValue;
                                        currentQty -= qty;
                                    }

                                    return {
                                        ...partial,
                                        isAddition,
                                        avgAtMoment: displayAvg,
                                        calculatedResult: result,
                                        calculatedPoints: points
                                    };
                                });
                        })() as partial}
                            <tr class="hover:bg-muted/10 transition-colors">
                                <td class="py-2 px-4 border-r border-border/5">
                                    <Badge variant="outline" class={`h-4 px-1.5 text-[8px] font-black uppercase tracking-tighter ${partial.isAddition ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20" : "bg-blue-500/10 text-blue-400 border-blue-500/20"}`}>
                                        {partial.isAddition ? "ADIÇÃO" : "SAÍDA"}
                                    </Badge>
                                </td>
                                <td class="py-2 px-4 font-mono text-[10px] text-muted-foreground/60 border-r border-border/5">
                                    {partial.date ? new Date(partial.date).toLocaleString($locale || 'pt-BR', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' }) : "-"}
                                </td>
                                <td class="py-2 px-4 text-right font-mono text-foreground font-bold border-r border-border/5">
                                    {partial.price?.toLocaleString($locale || "pt-BR")}
                                </td>
                                <td class="py-2 px-4 text-right font-mono text-blue-400 font-bold bg-blue-400/5 border-r border-border/5">
                                    {partial.avgAtMoment?.toLocaleString($locale || "pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                </td>
                                <td class="py-2 px-4 text-right font-mono font-bold text-foreground border-r border-border/5">
                                    {Math.abs(partial.quantity)}
                                </td>
                                <td class={`py-2 px-4 text-right font-mono font-black border-r border-border/5 ${(partial.calculatedResult || 0) >= 0 ? "text-green-500" : "text-rose-500"}`}>
                                    {#if partial.isAddition}
                                        <span class="text-muted-foreground/30 italic text-[9px]">Aporte</span>
                                    {:else}
                                        <div class="flex flex-col items-end">
                                            <span>{formatCurrency(partial.calculatedResult || 0, account?.currency || "BRL", $locale || "pt-BR")}</span>
                                            <span class="text-[8px] opacity-40 -mt-1 font-bold">
                                                {partial.calculatedPoints >= 0 ? "+" : ""}{partial.calculatedPoints.toLocaleString($locale || "pt-BR", { maximumFractionDigits: 2 })} pts
                                            </span>
                                        </div>
                                    {/if}
                                </td>
                                <td class="py-2 px-4 text-[10px] text-muted-foreground italic border-l border-border/5">
                                    {partial.notes || partial.obs || "-"}
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    {/if}

    <!-- Notes & Media -->
    <div class="space-y-4">
        <h4 class="text-[10px] font-black text-muted-foreground uppercase tracking-widest flex items-center gap-2">
            <FileText class="w-3.5 h-3.5" />
            ANOTAÇÕES & MÍDIA
        </h4>
        
        <div class="space-y-4 bg-muted/10 p-4 rounded-xl border border-border/30">
            {#if trade.notes}
                <p class="text-sm text-foreground/80 leading-relaxed whitespace-pre-wrap font-medium border-l-2 border-primary/30 pl-3 italic">
                    {trade.notes}
                </p>
            {/if}

            {#if trade.images && trade.images.length > 0}
                <div class="grid grid-cols-4 gap-2">
                    {#each trade.images as img, i}
                        <button
                            class="aspect-video bg-muted/30 rounded-lg overflow-hidden border border-border/20 group relative cursor-pointer shadow-sm hover:border-primary/40 transition-all"
                            onclick={() => (selectedImageIndex = i)}
                        >
                            <img
                                src={img}
                                alt="Trade Snapshot"
                                class="w-full h-full object-cover group-hover:scale-110 transition-transform duration-500"
                            />
                            <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                                <Maximize2 class="w-4 h-4 text-white" />
                            </div>
                        </button>
                    {/each}
                </div>
            {:else if !trade.notes}
                <div class="flex flex-col items-center justify-center py-6 text-muted-foreground/30">
                    <Camera class="w-8 h-8 mb-2 opacity-20" />
                    <p class="text-[9px] font-black uppercase tracking-widest">
                        Sem anotações ou mídia
                    </p>
                </div>
            {/if}
        </div>
    </div>
</div>

<ImageCarousel images={trade.images} bind:index={selectedImageIndex} />

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 3px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(var(--border), 0.3);
        border-radius: 10px;
    }
</style>
