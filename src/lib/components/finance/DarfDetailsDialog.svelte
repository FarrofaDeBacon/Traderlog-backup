<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import { irpfStore, type TaxDarf } from "$lib/stores/irpfStore.svelte";
    import {
        FileText,
        Calendar,
        Wallet,
        Percent,
        Info,
        CheckCircle2,
        Undo2,
    } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";
    import { toast } from "svelte-sonner";

    let { transactionId = "", darfId = "", open = $bindable(false) } = $props();

    let darf = $state<TaxDarf | null>(null);
    let appraisal = $state<any>(null);
    let loading = $state(false);
    let isComplementary = $state(false);

    $effect(() => {
        if (open && (transactionId || darfId)) {
            loadDarf();
        }
    });

    async function loadDarf() {
        loading = true;
        try {
            if (darfId) {
                console.log("[DARF DIALOG] Loading for darfId:", darfId);
                darf = await irpfStore.getDarfById(darfId);
            } else if (transactionId) {
                console.log(
                    "[DARF DIALOG] Loading for transactionId:",
                    transactionId,
                );
                darf = await irpfStore.getDarfByTransaction(transactionId);
            }

            if (darf) {
                console.log("[DARF DIALOG] DARF found:", darf);
                // Load appraisal
                if (darf.appraisal_id) {
                    appraisal = await irpfStore.getAppraisalById(
                        darf.appraisal_id,
                    );
                }

                // Check for complementary (search other darfs for same period and code)
                // We access the store's darf list (which should be loaded for the year)
                const otherDarfs = irpfStore.darfs.filter(
                    (d) =>
                        d.period === darf?.period &&
                        d.revenue_code === darf?.revenue_code &&
                        irpfStore.getId(d.id) !== irpfStore.getId(darf?.id),
                );
                isComplementary = otherDarfs.length > 0;
            } else {
                console.warn(
                    "[DARF DIALOG] No DARF found for transactionId:",
                    transactionId,
                );
            }
        } catch (e) {
            console.error("[DARF DIALOG] Error loading DARF/Appraisal:", e);
        } finally {
            loading = false;
        }
    }

    function formatCurrency(value: number) {
        return new Intl.NumberFormat($locale || "pt-BR", {
            style: "currency",
            currency: "BRL",
        }).format(value);
    }

    let showUnpayConfirm = $state(false);

    async function handleUnpay() {
        showUnpayConfirm = true;
    }

    async function confirmUnpay() {
        if (!darf) return;

        try {
            await irpfStore.unpayDarf(irpfStore.getId(darf.id));
            open = false; // Close dialog on success
            showUnpayConfirm = false;
        } catch (error) {
            console.error("[DARF DIALOG] Error undoing payment:", error);
            // Error already handled by store's toast
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content
        class="sm:max-w-[500px] bg-zinc-950 border-zinc-800 text-zinc-100"
    >
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2 text-xl">
                <FileText class="w-5 h-5 text-amber-500" />
                {$t("finance.darfDetails.title")}
            </Dialog.Title>
            <Dialog.Description class="text-zinc-400">
                {$t("finance.darfDetails.description")}
            </Dialog.Description>
        </Dialog.Header>

        {#if loading}
            <div class="py-12 flex flex-col items-center justify-center gap-4">
                <div
                    class="w-8 h-8 border-4 border-amber-500/20 border-t-amber-500 rounded-full animate-spin"
                ></div>
                <p class="text-sm text-zinc-500">
                    {$t("finance.darfDetails.loading")}
                </p>
            </div>
        {:else if darf}
            <div class="space-y-6 pt-4">
                <!-- Status & Period -->
                <div class="flex items-center justify-between">
                    <div class="space-y-1">
                        <span
                            class="text-xs text-zinc-500 uppercase font-bold tracking-wider"
                            >{$t("finance.darfDetails.period")}</span
                        >
                        <div class="flex items-center gap-2">
                            <Calendar class="w-4 h-4 text-zinc-400" />
                            <span class="text-lg font-medium"
                                >{darf.period}</span
                            >
                        </div>
                    </div>
                    <div class="flex flex-col items-end gap-2">
                        {#if darf.status === "Paid"}
                            <Badge
                                variant="outline"
                                class="bg-green-500/10 text-green-500 border-green-500/20 px-3 py-1"
                            >
                                <CheckCircle2 class="w-3.5 h-3.5 mr-1.5" />
                                {$t("finance.darfDetails.statusPaid")}
                            </Badge>
                        {:else}
                            <Badge
                                variant="outline"
                                class="bg-amber-500/10 text-amber-500 border-amber-500/20 px-3 py-1"
                            >
                                <Info class="w-3.5 h-3.5 mr-1.5" />
                                {$t("finance.darfDetails.statusPending")}
                            </Badge>
                        {/if}
                        {#if isComplementary}
                            <Badge
                                variant="outline"
                                class="bg-blue-500/10 text-blue-400 border-blue-500/20 px-2 py-0.5 text-[10px]"
                            >
                                {$t("finance.darfDetails.complementaryBadge")}
                            </Badge>
                        {/if}
                    </div>
                </div>

                <Separator class="bg-zinc-800" />

                <!-- Codes Section -->
                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-1">
                        <span
                            class="text-xs text-zinc-500 uppercase font-bold tracking-wider"
                            >{$t("finance.darfDetails.revenueCode")}</span
                        >
                        <div class="text-base font-semibold text-zinc-200">
                            {darf.revenue_code}
                        </div>
                    </div>
                    <div class="space-y-1">
                        <span
                            class="text-xs text-zinc-500 uppercase font-bold tracking-wider"
                            >{$t("finance.darfDetails.dueDate")}</span
                        >
                        <div class="text-base font-semibold text-zinc-200">
                            {darf.due_date
                                ? new Date(darf.due_date).toLocaleDateString(
                                      $locale || "pt-BR",
                                  )
                                : "-"}
                        </div>
                    </div>
                </div>

                <!-- NEW: Calculation Details Section -->
                {#if appraisal}
                    <div class="space-y-3">
                        <h4
                            class="text-xs font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2"
                        >
                            <Info class="w-3.5 h-3.5" />
                            {$t("finance.darfDetails.calculationInfo")}
                        </h4>
                        <div
                            class="bg-zinc-900/40 border border-zinc-800/60 rounded-xl p-4 grid grid-cols-2 gap-y-4 gap-x-6"
                        >
                            <div class="space-y-1">
                                <span
                                    class="text-[10px] text-zinc-500 uppercase"
                                    >{$t(
                                        "finance.darfDetails.grossResult",
                                    )}</span
                                >
                                <p
                                    class="text-sm font-semibold {appraisal.gross_profit >=
                                    0
                                        ? 'text-emerald-400'
                                        : 'text-red-400'}"
                                >
                                    {formatCurrency(appraisal.gross_profit)}
                                </p>
                            </div>
                            <div class="space-y-1">
                                <span
                                    class="text-[10px] text-zinc-500 uppercase"
                                    >{$t(
                                        "finance.darfDetails.compensatedLosses",
                                    )}</span
                                >
                                <p
                                    class="text-sm font-semibold text-amber-500/80"
                                >
                                    {formatCurrency(
                                        appraisal.compensated_loss || 0,
                                    )}
                                </p>
                            </div>
                            <div class="space-y-1">
                                <span
                                    class="text-[10px] text-zinc-500 uppercase"
                                    >{$t(
                                        "finance.darfDetails.calculationBasis",
                                    )}</span
                                >
                                <p class="text-sm font-semibold text-zinc-200">
                                    {formatCurrency(
                                        appraisal.calculation_basis,
                                    )}
                                </p>
                            </div>
                            <div class="space-y-1">
                                <span
                                    class="text-[10px] text-zinc-500 uppercase"
                                    >{$t("finance.darfDetails.taxRate")}</span
                                >
                                <p class="text-sm font-semibold text-zinc-200">
                                    {appraisal.tax_rate * 100}%
                                </p>
                            </div>
                        </div>
                    </div>
                {/if}

                <!-- Values Breakdown -->
                <div
                    class="bg-zinc-900/50 rounded-xl p-4 border border-zinc-800/50 space-y-3"
                >
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-zinc-400"
                            >{$t("finance.darfDetails.principalValue")}</span
                        >
                        <span class="font-mono text-zinc-200"
                            >{formatCurrency(darf.principal_value)}</span
                        >
                    </div>

                    {#if darf.fine > 0}
                        <div class="flex justify-between items-center text-sm">
                            <span
                                class="text-zinc-400 flex items-center gap-1.5"
                            >
                                {$t("finance.darfDetails.fine")}
                                <Percent class="w-3 h-3" />
                            </span>
                            <span class="font-mono text-amber-500"
                                >+{formatCurrency(darf.fine)}</span
                            >
                        </div>
                    {/if}

                    {#if darf.interest > 0}
                        <div class="flex justify-between items-center text-sm">
                            <span
                                class="text-zinc-400 flex items-center gap-1.5"
                            >
                                {$t("finance.darfDetails.interest")}
                                <Info class="w-3 h-3" />
                            </span>
                            <span class="font-mono text-amber-500"
                                >+{formatCurrency(darf.interest)}</span
                            >
                        </div>
                    {/if}

                    <div
                        class="pt-2 border-t border-zinc-800 flex justify-between items-center"
                    >
                        <span class="font-bold text-zinc-100"
                            >{$t("finance.darfDetails.totalPaid")}</span
                        >
                        <span
                            class="text-lg font-bold text-green-400 font-mono"
                        >
                            {formatCurrency(darf.total_value)}
                        </span>
                    </div>
                </div>

                <!-- Payment Info -->
                <div
                    class="flex items-center gap-3 p-3 bg-zinc-900/30 rounded-lg border border-zinc-800/30"
                >
                    <Wallet class="w-5 h-5 text-zinc-500" />
                    <div class="space-y-0.5">
                        <p
                            class="text-[10px] text-zinc-500 uppercase font-bold"
                        >
                            {$t("finance.darfDetails.paidAt")}
                        </p>
                        <p class="text-xs text-zinc-300">
                            {darf.payment_date
                                ? new Date(
                                      darf.payment_date,
                                  ).toLocaleDateString($locale || "pt-BR", {
                                      day: "numeric",
                                      month: "long",
                                      year: "numeric",
                                  })
                                : $t("finance.darfDetails.dateNotAvailable")}
                        </p>
                    </div>
                </div>
            </div>
        {:else}
            <div class="py-12 text-center space-y-2">
                <Info class="w-8 h-8 text-zinc-600 mx-auto" />
                <p class="text-zinc-500">
                    {$t("finance.darfDetails.errorMessage")}
                </p>
            </div>
        {/if}

        <Dialog.Footer
            class="mt-4 flex flex-col sm:flex-row gap-2 border-t border-zinc-800/50 pt-4"
        >
            {#if showUnpayConfirm}
                <div class="flex items-center gap-2 mr-auto">
                    <Undo2 class="w-4 h-4 text-red-500 animate-pulse" />
                    <span class="text-xs font-bold text-red-500 uppercase"
                        >{$t("finance.darfDetails.unpayTitle")}</span
                    >
                </div>
                <Button
                    variant="destructive"
                    onclick={confirmUnpay}
                    class="bg-red-600 hover:bg-red-700 text-white"
                >
                    {$t("finance.darfDetails.unpayConfirm")}
                </Button>
                <Button
                    variant="outline"
                    onclick={() => (showUnpayConfirm = false)}
                    class="border-zinc-700 text-zinc-400 hover:bg-zinc-800"
                >
                    {$t("finance.darfDetails.unpayCancel")}
                </Button>
            {:else}
                {#if darf && darf.status === "Paid"}
                    <Button
                        variant="destructive"
                        onclick={handleUnpay}
                        class="bg-red-600/20 hover:bg-red-600 text-white border border-red-500/30 mr-auto group transition-all"
                    >
                        <Undo2
                            class="w-4 h-4 mr-2 text-red-500 group-hover:text-white"
                        />
                        {$t("finance.darfDetails.unpayAction")}
                    </Button>
                {/if}
                <Button
                    variant="secondary"
                    onclick={() => (open = false)}
                    class="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 border-zinc-700"
                >
                    {$t("general.close")}
                </Button>
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
