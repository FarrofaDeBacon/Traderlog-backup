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

                isComplementary = darf.is_complementary;
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
        class="sm:max-w-[500px] bg-popover/80 backdrop-blur-md border-border text-foreground"
    >
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2 text-xl">
                <FileText class="w-5 h-5 text-amber-500" />
                {$t("finance.darfDetails.title")}
            </Dialog.Title>
            <Dialog.Description class="text-muted-foreground">
                {$t("finance.darfDetails.description")}
            </Dialog.Description>
        </Dialog.Header>

        {#if loading}
            <div class="py-12 flex flex-col items-center justify-center gap-4">
                <div
                    class="w-8 h-8 border-4 border-amber-500/20 border-t-amber-500 rounded-full animate-spin"
                ></div>
                <p class="text-sm text-muted-foreground">
                    {$t("finance.darfDetails.loading")}
                </p>
            </div>
        {:else if darf}
            <div class="space-y-6 pt-4">
                <!-- Status & Period -->
                <div class="flex items-center justify-between">
                    <div class="space-y-1">
                        <span
                            class="text-xs text-muted-foreground uppercase font-bold tracking-wider"
                            >{$t("finance.darfDetails.period")}</span
                        >
                        <div class="flex items-center gap-2">
                            <Calendar
                                class="w-4 h-4 text-muted-foreground/70"
                            />
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
                        <Badge
                            variant="outline"
                            class={darf.is_complementary
                                ? "bg-blue-500/10 text-blue-400 border-blue-500/20 px-2 py-0.5 text-[10px]"
                                : "bg-emerald-500/10 text-emerald-400 border-emerald-500/20 px-2 py-0.5 text-[10px]"}
                        >
                            {darf.is_complementary
                                ? $t("finance.darfDetails.complementaryBadge")
                                : "Guia Principal"}
                        </Badge>
                    </div>
                </div>

                <Separator class="bg-border/30" />

                <!-- Codes Section -->
                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-1">
                        <span
                            class="text-xs text-muted-foreground uppercase font-bold tracking-wider"
                            >{$t("finance.darfDetails.revenueCode")}</span
                        >
                        <div class="text-base font-semibold text-foreground">
                            {darf.revenue_code}
                        </div>
                    </div>
                    <div class="space-y-1">
                        <span
                            class="text-xs text-muted-foreground uppercase font-bold tracking-wider"
                            >{$t("finance.darfDetails.dueDate")}</span
                        >
                        <div class="text-base font-semibold text-foreground">
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
                            class="text-xs font-bold text-muted-foreground uppercase tracking-widest flex items-center gap-2"
                        >
                            <Info class="w-3.5 h-3.5" />
                            {$t("finance.darfDetails.calculationInfo")}
                        </h4>
                        <div
                            class="bg-muted/30 border border-border/40 rounded-xl p-4 grid grid-cols-2 gap-y-4 gap-x-6"
                        >
                            <div class="space-y-1">
                                <span
                                    class="text-[10px] text-muted-foreground uppercase"
                                    >{$t(
                                        "finance.darfDetails.grossResult",
                                    )}</span
                                >
                                <p
                                    class="text-sm font-semibold {appraisal.gross_profit >=
                                    0
                                        ? 'text-emerald-500 dark:text-emerald-400'
                                        : 'text-rose-500 dark:text-red-400'}"
                                >
                                    {formatCurrency(appraisal.gross_profit)}
                                </p>
                            </div>
                            <div class="space-y-1">
                                <span
                                    class="text-[10px] text-muted-foreground uppercase"
                                    >{$t(
                                        "finance.darfDetails.compensatedLosses",
                                    )}</span
                                >
                                <p
                                    class="text-sm font-semibold text-amber-600 dark:text-amber-500/80"
                                >
                                    {formatCurrency(
                                        appraisal.compensated_loss || 0,
                                    )}
                                </p>
                            </div>
                            <div class="space-y-1">
                                <span
                                    class="text-[10px] text-muted-foreground uppercase"
                                    >{$t(
                                        "finance.darfDetails.calculationBasis",
                                    )}</span
                                >
                                <p
                                    class="text-sm font-semibold text-foreground"
                                >
                                    {formatCurrency(
                                        appraisal.calculation_basis,
                                    )}
                                </p>
                            </div>
                            <div class="space-y-1">
                                <span
                                    class="text-[10px] text-muted-foreground uppercase"
                                    >{$t("finance.darfDetails.taxRate")}</span
                                >
                                <p
                                    class="text-sm font-semibold text-foreground"
                                >
                                    {appraisal.tax_rate}%
                                </p>
                            </div>
                        </div>
                    </div>
                {/if}

                <!-- Values Breakdown -->
                <div
                    class="bg-muted/40 rounded-xl p-4 border border-border/30 space-y-3"
                >
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-muted-foreground"
                            >{$t("finance.darfDetails.principalValue")}</span
                        >
                        <span class="font-mono text-foreground"
                            >{formatCurrency(darf.principal_value)}</span
                        >
                    </div>

                    {#if darf.fine > 0}
                        <div class="flex justify-between items-center text-sm">
                            <span
                                class="text-muted-foreground flex items-center gap-1.5"
                            >
                                {$t("finance.darfDetails.fine")}
                                <Percent class="w-3 h-3" />
                            </span>
                            <span
                                class="font-mono text-amber-600 dark:text-amber-500"
                                >+{formatCurrency(darf.fine)}</span
                            >
                        </div>
                    {/if}

                    {#if darf.interest > 0}
                        <div class="flex justify-between items-center text-sm">
                            <span
                                class="text-muted-foreground flex items-center gap-1.5"
                            >
                                {$t("finance.darfDetails.interest")}
                                <Info class="w-3 h-3" />
                            </span>
                            <span
                                class="font-mono text-amber-600 dark:text-amber-500"
                                >+{formatCurrency(darf.interest)}</span
                            >
                        </div>
                    {/if}

                    <div
                        class="pt-2 border-t border-border/20 flex justify-between items-center"
                    >
                        <span class="font-bold text-foreground"
                            >{$t("finance.darfDetails.totalPaid")}</span
                        >
                        <span
                            class="text-lg font-bold text-emerald-600 dark:text-green-400 font-mono"
                        >
                            {formatCurrency(darf.total_value)}
                        </span>
                    </div>
                </div>

                <!-- Payment Info -->
                <div
                    class="flex items-center gap-3 p-3 bg-muted/20 rounded-lg border border-border/30"
                >
                    <Wallet class="w-5 h-5 text-muted-foreground" />
                    <div class="space-y-0.5">
                        <p
                            class="text-[10px] text-muted-foreground uppercase font-bold"
                        >
                            {$t("finance.darfDetails.paidAt")}
                        </p>
                        <p class="text-xs text-foreground/80">
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
                <Info class="w-8 h-8 text-muted-foreground/30 mx-auto" />
                <p class="text-muted-foreground">
                    {$t("finance.darfDetails.errorMessage")}
                </p>
            </div>
        {/if}

        <Dialog.Footer
            class="mt-4 flex flex-col sm:flex-row gap-2 border-t border-border/20 pt-4"
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
                    class="border-border text-muted-foreground hover:bg-accent"
                >
                    {$t("finance.darfDetails.unpayCancel")}
                </Button>
            {:else}
                {#if darf && darf.status === "Paid"}
                    <Button
                        variant="destructive"
                        onclick={handleUnpay}
                        class="bg-red-600/10 hover:bg-red-600 text-red-600 dark:text-white hover:text-white border border-red-500/30 mr-auto group transition-all"
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
                    class="bg-muted hover:bg-accent text-foreground border-border"
                >
                    {$t("general.close")}
                </Button>
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
