<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { autoTradeService } from "$lib/services/autoTradeService.svelte";
    import { Zap, AlertCircle } from "lucide-svelte";
    import NewTradeWizard from "./NewTradeWizard.svelte";
    import { t, locale } from "svelte-i18n";

    let showWizard = $state(false);

    function handleConfirm() {
        showWizard = true;
    }

    function handleClose() {
        autoTradeService.clear();
        showWizard = false;
    }

    // Map autoTradeService.pendingTrade to the format NewTradeWizard expects
    const prefilledTrade = $derived.by(() => {
        if (!autoTradeService.pendingTrade) return null;
        
        return {
            id: autoTradeService.pendingTrade.existingTradeId,
            asset_symbol: autoTradeService.pendingTrade.symbol,
            entry_price: autoTradeService.pendingTrade.price,
            account_id: autoTradeService.pendingTrade.account_id,
            date: autoTradeService.pendingTrade.timestamp.toISOString(),
            _isAutoPartial: autoTradeService.pendingTrade.isPartial,
            _autoPrice: autoTradeService.pendingTrade.price,
            _autoType: autoTradeService.pendingTrade.type,
        };
    });
</script>

{#if autoTradeService.pendingTrade}
    <Dialog.Root
        open={autoTradeService.isDialogOpen}
        onOpenChange={(open) => !open && handleClose()}
    >
        <Dialog.Content class="max-w-md bg-zinc-950 border-zinc-800 text-white">
            {#if !showWizard}
                <div class="p-6 space-y-6">
                    <div class="flex items-center gap-4">
                        <div class="p-3 bg-amber-500/20 rounded-full">
                            <Zap class="w-6 h-6 text-amber-500" />
                        </div>
                        <div>
                            <h2 class="text-xl font-bold">
                                {autoTradeService.pendingTrade.type === 'partial_entry' 
                                    ? "Aumento de Posição" 
                                    : autoTradeService.pendingTrade.type === 'partial_exit'
                                        ? "Parcial Detectada"
                                        : $t("trades.wizard.auto_detection.title")}
                            </h2>
                            <p
                                class="text-xs text-muted-foreground uppercase tracking-widest"
                            >
                                {autoTradeService.pendingTrade.isPartial 
                                    ? "Atualização de Posição via RTD" 
                                    : $t("trades.wizard.auto_detection.source")}
                            </p>
                        </div>
                    </div>

                    <div
                        class="grid grid-cols-2 gap-4 bg-zinc-900/50 p-4 rounded-xl border border-zinc-800"
                    >
                        <div>
                            <p
                                class="text-[10px] font-bold text-muted-foreground uppercase"
                            >
                                {$t("trades.wizard.fields.asset")}
                            </p>
                            <p class="text-lg font-black">
                                {autoTradeService.pendingTrade.symbol}
                            </p>
                        </div>
                        <div>
                            <p
                                class="text-[10px] font-bold text-muted-foreground uppercase"
                            >
                                {$t("trades.wizard.fields.entry_price")}
                            </p>
                            <p class="text-lg font-black text-emerald-500">
                                {autoTradeService.pendingTrade.price.toLocaleString(
                                    $locale || "pt-BR",
                                    { minimumFractionDigits: 2 },
                                )}
                            </p>
                        </div>
                    </div>

                    <div class="flex gap-3 pt-4">
                        <Button
                            variant="outline"
                            class="flex-1 border-zinc-800"
                            onclick={handleClose}
                        >
                            {$t("trades.wizard.auto_detection.ignore")}
                        </Button>
                        <Button
                            class="flex-1 bg-emerald-600 hover:bg-emerald-700"
                            onclick={handleConfirm}
                        >
                            {autoTradeService.pendingTrade.isPartial 
                                ? "Atualizar Trade" 
                                : $t("trades.wizard.auto_detection.register")}
                        </Button>
                    </div>
                </div>
            {:else}
                <div class="w-full">
                    <NewTradeWizard
                        trade={prefilledTrade}
                        close={handleClose}
                        onsave={handleClose}
                    />
                </div>
            {/if}
        </Dialog.Content>
    </Dialog.Root>
{/if}
