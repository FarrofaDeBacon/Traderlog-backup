<script lang="ts">
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import {
        Calendar,
        Wallet,
        Percent,
        Info,
        CheckCircle2,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";
    import type { TaxDarf } from "$lib/stores/irpfStore.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import TradeDetailView from "./TradeDetailView.svelte";

    let { darf, appraisal, isComplementary } = $props<{
        darf: TaxDarf | null;
        appraisal: any | null;
        isComplementary: boolean;
    }>();

    function formatCurrency(value: number) {
        return new Intl.NumberFormat($locale || "pt-BR", {
            style: "currency",
            currency: "BRL",
        }).format(value);
    }

    const filteredTrades = $derived.by(() => {
        if (!appraisal?.trade_ids) return [];
        
        // Match trades from store by their normalized IDs
        const appraisalTradeIds = appraisal.trade_ids.map((id: string) => tradesStore.normalizeId(id));
        
        return tradesStore.trades.filter(t => {
            const normalizedId = tradesStore.normalizeId(t.id);
            return appraisalTradeIds.includes(normalizedId);
        }).sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());
    });
</script>

<div class="space-y-6">
    <!-- Status & Period -->
    <div class="flex items-center justify-between">
        <div class="space-y-1">
            <span class="text-[10px] text-muted-foreground uppercase font-black tracking-widest opacity-60">
                {$t("finance.darfDetails.period")}
            </span>
            <div class="flex items-center gap-2">
                <Calendar class="w-4 h-4 text-muted-foreground/70" />
                <span class="text-lg font-bold font-mono tracking-tighter">
                    {darf?.period ||
                        (appraisal
                            ? `${String(appraisal.period_month).padStart(2, '0')}/${appraisal.period_year}`
                            : "")}
                </span>
            </div>
        </div>
        <div class="flex flex-col items-end gap-2">
            {#if darf}
                {#if darf.status === "Paid"}
                    <Badge
                        variant="outline"
                        class="bg-green-500 text-white border-green-600 px-3 py-1 font-black uppercase text-[10px] shadow-sm shadow-green-500/10"
                    >
                        <CheckCircle2 class="w-3.5 h-3.5 mr-1.5" />
                        {$t("finance.darfDetails.statusPaid")}
                    </Badge>
                {:else}
                    <Badge
                        variant="outline"
                        class="bg-amber-500 text-white border-amber-600 px-3 py-1 font-black uppercase text-[10px] shadow-sm shadow-amber-500/10"
                    >
                        <Info class="w-3.5 h-3.5 mr-1.5" />
                        {$t("finance.darfDetails.statusPending")}
                    </Badge>
                {/if}
            {:else if appraisal}
                <Badge
                    variant="outline"
                    class="bg-blue-500/10 text-blue-500 border-blue-500/20 px-3 py-1 font-black uppercase text-[10px]"
                >
                    <Info class="w-3.5 h-3.5 mr-1.5" />
                    {appraisal.status === "Paid"
                        ? $t("finance.darfDetails.statusPaid")
                        : "Apuração Salva"}
                </Badge>
            {/if}
            <Badge
                variant="outline"
                class={isComplementary
                    ? "bg-indigo-500/10 text-indigo-400 border-indigo-500/20 px-2 py-0.5 text-[10px] font-bold uppercase tracking-widest"
                    : "bg-emerald-500/10 text-emerald-400 border-emerald-500/20 px-2 py-0.5 text-[10px] font-bold uppercase tracking-widest"}
            >
                {isComplementary
                    ? $t("finance.darfDetails.complementaryBadge")
                    : "Guia Principal"}
            </Badge>
        </div>
    </div>

    <Separator class="bg-border/30" />

    <!-- Codes Section -->
    <div class="grid grid-cols-2 gap-4">
        <div class="space-y-1">
            <span class="text-[10px] text-muted-foreground uppercase font-black tracking-widest opacity-60">
                {$t("finance.darfDetails.revenueCode")}
            </span>
            <div class="text-base font-black text-foreground font-mono">
                {darf?.revenue_code || appraisal?.revenue_code || "6015"}
            </div>
        </div>
        <div class="space-y-1">
            <span class="text-[10px] text-muted-foreground uppercase font-black tracking-widest opacity-60">
                {$t("finance.darfDetails.dueDate")}
            </span>
            <div class="text-base font-bold text-foreground font-mono">
                {darf?.due_date
                    ? new Date(darf.due_date).toLocaleDateString($locale || "pt-BR")
                    : "-"}
            </div>
        </div>
    </div>

    <!-- Calculation Details Section -->
    {#if appraisal}
        <div class="space-y-3">
            <h4 class="text-[10px] font-black text-muted-foreground uppercase tracking-widest flex items-center gap-2">
                <Info class="w-3.5 h-3.5" />
                {$t("finance.darfDetails.calculationInfo")}
            </h4>
            <div class="bg-muted/30 border border-border/40 rounded-xl p-4 grid grid-cols-2 gap-y-4 gap-x-6">
                <div class="space-y-1">
                    <span class="text-[10px] text-muted-foreground uppercase font-bold opacity-70">
                        {$t("finance.darfDetails.grossResult")}
                    </span>
                    <p class="text-sm font-black font-mono {appraisal.gross_profit >= 0 ? 'text-emerald-500' : 'text-rose-600'}">
                        {formatCurrency(appraisal.gross_profit)}
                    </p>
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] text-muted-foreground uppercase font-bold opacity-70">
                        {$t("finance.darfDetails.compensatedLosses")}
                    </span>
                    <p class="text-sm font-black font-mono text-amber-500">
                        {formatCurrency(appraisal.compensated_loss || 0)}
                    </p>
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] text-muted-foreground uppercase font-bold opacity-70">
                        {$t("finance.darfDetails.calculationBasis")}
                    </span>
                    <p class="text-sm font-black font-mono text-foreground">
                        {formatCurrency(appraisal.calculation_basis)}
                    </p>
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] text-muted-foreground uppercase font-bold opacity-70">
                        {$t("finance.darfDetails.taxRate")}
                    </span>
                    <p class="text-sm font-black font-mono text-foreground">
                        {appraisal.tax_rate}%
                    </p>
                </div>
            </div>
        </div>
    {/if}

    <!-- Values Breakdown -->
    <div class="bg-muted/40 rounded-xl p-4 border border-border/30 space-y-3">
        <div class="flex justify-between items-center text-sm">
            <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest">
                {$t("finance.darfDetails.principalValue")}
            </span>
            <span class="font-mono font-bold text-foreground">
                {formatCurrency(darf?.principal_value || appraisal?.total_payable || 0)}
            </span>
        </div>

        {#if darf && darf.fine > 0}
            <div class="flex justify-between items-center text-sm">
                <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest flex items-center gap-1.5">
                    {$t("finance.darfDetails.fine")}
                    <Percent class="w-2.5 h-2.5" />
                </span>
                <span class="font-mono font-bold text-amber-500">
                    +{formatCurrency(darf.fine)}
                </span>
            </div>
        {/if}

        {#if darf && darf.interest > 0}
            <div class="flex justify-between items-center text-sm">
                <span class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest flex items-center gap-1.5">
                    {$t("finance.darfDetails.interest")}
                    <Info class="w-2.5 h-2.5" />
                </span>
                <span class="font-mono font-bold text-amber-500">
                    +{formatCurrency(darf.interest)}
                </span>
            </div>
        {/if}

        <div class="pt-2 border-t border-border/20 flex justify-between items-center">
            <span class="text-xs font-black text-foreground uppercase tracking-tight">
                {$t("finance.darfDetails.totalPaid")}
            </span>
            <span class="text-xl font-black text-emerald-500 font-mono tracking-tighter">
                {formatCurrency(darf?.total_value || appraisal?.total_payable || 0)}
            </span>
        </div>
    </div>

    <!-- Operations Table -->
    {#if appraisal}
        <div class="space-y-3">
            <h4 class="text-[10px] font-black text-muted-foreground uppercase tracking-widest flex items-center gap-2">
                <Wallet class="w-3.5 h-3.5" />
                {$t("finance.darfDetails.operations")}
            </h4>
            
            <TradeDetailView 
                trades={filteredTrades} 
                currency="BRL" 
            />
            
            {#if filteredTrades.length === 0 && appraisal.trade_ids?.length > 0}
                <div class="p-4 bg-amber-500/5 border border-amber-500/20 rounded-lg flex items-center gap-2">
                    <Info class="w-4 h-4 text-amber-500" />
                    <span class="text-[10px] font-bold text-amber-500 uppercase">
                        Aviso: {appraisal.trade_ids.length} operações vinculadas, mas não encontradas no histórico local.
                    </span>
                </div>
            {/if}
        </div>
    {/if}

    <!-- Payment Info -->
    {#if darf && darf.status === "Paid"}
        <div class="flex items-center gap-3 p-3 bg-indigo-500/5 rounded-lg border border-indigo-500/10">
            <Wallet class="w-5 h-5 text-indigo-400" />
            <div class="space-y-0.5">
                <p class="text-[10px] text-muted-foreground uppercase font-black tracking-widest">
                    {$t("finance.darfDetails.paidAt")}
                </p>
                <p class="text-sm font-bold text-foreground/80">
                    {darf.payment_date
                        ? new Date(darf.payment_date).toLocaleDateString($locale || "pt-BR", {
                              day: "numeric",
                              month: "long",
                              year: "numeric",
                          })
                        : $t("finance.darfDetails.dateNotAvailable")}
                </p>
            </div>
        </div>
    {/if}
</div>
