<script lang="ts">
    import { Plus, Pencil, Trash2, Scale } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { settingsStore, type TaxRule } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let formData = $state<Omit<TaxRule, "id">>({
        name: "",
        tax_rate: 0,
        withholding_rate: 0,
        exemption_threshold: 0,
        basis: "NetProfit",
        cumulative_losses: true,
    });

    // --- RULES LOGIC ---

    function resetForm() {
        formData = {
            name: "",
            tax_rate: 15,
            withholding_rate: 0.005,
            exemption_threshold: 0,
            basis: "NetProfit",
            cumulative_losses: true,
        };
        editingId = null;
    }

    function openNew() {
        resetForm();
        isDialogOpen = true;
    }

    function openEdit(rule: TaxRule) {
        editingId = rule.id;
        formData = { ...rule };
        isDialogOpen = true;
    }

    let isSubmittingRule = $state(false);

    async function saveRule() {
        if (!formData.name) {
            toast.error($t("settings.fiscal.rules.form.nameRequired"));
            return;
        }

        if (isSubmittingRule) return;
        isSubmittingRule = true;

        try {
            if (editingId) {
                await settingsStore.updateTaxRule(editingId, formData);
            } else {
                await settingsStore.addTaxRule(formData);
            }
            // FECHAR modal
            isDialogOpen = false;
            toast.success($t("settings.fiscal.rules.form.successSave"));
        } catch (e) {
            console.error("Erro ao salvar regra:", e);
            toast.error($t("settings.fiscal.rules.form.errorSave"));
        } finally {
            isSubmittingRule = false;
        }
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteTaxRule(deleteId);

            if (!result.success) {
                toast.error(result.error || $t("general.error"));
            } else {
                toast.success($t("general.deleteSuccess"));
            }
            deleteId = null;
        }
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">
                {$t("settings.fiscal.rules.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.fiscal.rules.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.fiscal.rules.new")}
        </Button>
    </div>

    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {#each settingsStore.taxRules as rule}
            <div
                class="flex flex-col p-4 rounded-lg border bg-card hover:border-primary/50 transition-all shadow-sm group relative overflow-hidden"
            >
                <!-- Header -->
                <div class="flex items-start justify-between mb-4">
                    <div class="flex items-center gap-3">
                        <div class="p-2 bg-primary/10 rounded-lg text-primary">
                            <Scale class="w-5 h-5" />
                        </div>
                        <div>
                            <h4 class="font-bold text-base">
                                {rule.name}
                            </h4>
                            <span
                                class="text-[10px] uppercase text-muted-foreground tracking-wider"
                            >
                                {rule.basis === "NetProfit"
                                    ? $t(
                                          "settings.fiscal.rules.form.basisOptions.netProfit",
                                      )
                                    : $t(
                                          "settings.fiscal.rules.form.basisOptions.saleAmount",
                                      )}
                            </span>
                        </div>
                    </div>
                </div>

                <!-- Stats -->
                <div class="space-y-2 mb-4 flex-1">
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-muted-foreground"
                            >{$t("settings.fiscal.rules.form.rate")}:</span
                        >
                        <span class="font-mono font-bold">{rule.tax_rate}%</span
                        >
                    </div>
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-muted-foreground"
                            >{$t(
                                "settings.fiscal.rules.form.withholding",
                            )}:</span
                        >
                        <span class="font-mono">{rule.withholding_rate}%</span>
                    </div>
                    {#if rule.exemption_threshold > 0}
                        <div
                            class="flex justify-between items-center text-sm text-green-500"
                        >
                            <span
                                >{$t(
                                    "settings.fiscal.rules.form.exemption",
                                )}:</span
                            >
                            <span class="font-mono"
                                >R$ {rule.exemption_threshold.toLocaleString()}</span
                            >
                        </div>
                    {/if}
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-muted-foreground"
                            >{$t(
                                "settings.fiscal.rules.form.cumulative",
                            )}:</span
                        >
                        <span
                            class={rule.cumulative_losses
                                ? "text-green-500"
                                : "text-red-500"}
                        >
                            {rule.cumulative_losses
                                ? $t("general.yes")
                                : $t("general.no")}
                        </span>
                    </div>
                </div>

                <!-- Actions -->
                <div
                    class="flex items-center justify-end gap-2 pt-2 border-t mt-auto"
                >
                    <Button
                        variant="ghost"
                        size="sm"
                        class="h-8"
                        onclick={() => openEdit(rule)}
                    >
                        <Pencil class="w-3.5 h-3.5 mr-1" />
                        {$t("general.edit")}
                    </Button>
                    <Button
                        variant="ghost"
                        size="sm"
                        class="h-8 text-destructive hover:text-destructive"
                        onclick={() => requestDelete(rule.id)}
                    >
                        <Trash2 class="w-3.5 h-3.5 mr-1" />
                        {$t("general.delete")}
                    </Button>
                </div>
            </div>
        {:else}
            <div
                class="col-span-full flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg text-muted-foreground h-[200px]"
            >
                <Scale class="w-8 h-8 mb-2 opacity-20" />
                <span>{$t("settings.fiscal.rules.empty")}</span>
                <Button variant="link" onclick={openNew}
                    >{$t("settings.fiscal.rules.new")}</Button
                >
            </div>
        {/each}
    </div>
</div>

<!-- Rule Dialog -->
<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[500px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingId
                    ? $t("settings.fiscal.rules.form.titleEdit")
                    : $t("settings.fiscal.rules.form.titleNew")}</Dialog.Title
            >
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            <div class="space-y-2">
                <Label>{$t("settings.fiscal.rules.form.name")}</Label>
                <Input
                    bind:value={formData.name}
                    placeholder={$t(
                        "settings.fiscal.rules.form.namePlaceholder",
                    )}
                />
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="space-y-2">
                    <Label>{$t("settings.fiscal.rules.form.rate")}</Label>
                    <div class="relative">
                        <Input
                            type="number"
                            step="0.1"
                            bind:value={formData.tax_rate}
                        />
                        <span
                            class="absolute right-3 top-2.5 text-xs text-muted-foreground"
                            >%</span
                        >
                    </div>
                    <p class="text-[10px] text-muted-foreground">
                        {$t("settings.fiscal.rules.form.rateHint")}
                    </p>
                </div>
                <div class="space-y-2">
                    <Label>{$t("settings.fiscal.rules.form.withholding")}</Label
                    >
                    <div class="relative">
                        <Input
                            type="number"
                            step="0.001"
                            bind:value={formData.withholding_rate}
                        />
                        <span
                            class="absolute right-3 top-2.5 text-xs text-muted-foreground"
                            >%</span
                        >
                    </div>
                    <p class="text-[10px] text-muted-foreground">
                        {$t("settings.fiscal.rules.form.withholdingHint")}
                    </p>
                </div>
            </div>

            <div class="space-y-2">
                <Label>{$t("settings.fiscal.rules.form.basis")}</Label>
                <Select.Root type="single" bind:value={formData.basis}>
                    <Select.Trigger>
                        {formData.basis === "NetProfit"
                            ? $t(
                                  "settings.fiscal.rules.form.basisOptions.netProfit",
                              )
                            : $t(
                                  "settings.fiscal.rules.form.basisOptions.saleAmount",
                              )}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="NetProfit"
                            >{$t(
                                "settings.fiscal.rules.form.basisOptions.netProfit",
                            )}</Select.Item
                        >
                        <Select.Item value="GrossProfit"
                            >{$t(
                                "settings.fiscal.rules.form.basisOptions.saleAmount",
                            )}</Select.Item
                        >
                    </Select.Content>
                </Select.Root>
                <p class="text-[10px] text-muted-foreground">
                    {$t("settings.fiscal.rules.form.basisHint")}
                </p>
            </div>

            <div class="space-y-2">
                <Label>{$t("settings.fiscal.rules.form.exemption")}</Label>
                <div class="relative">
                    <span
                        class="absolute left-3 top-2.5 text-xs text-muted-foreground"
                        >R$</span
                    >
                    <Input
                        type="number"
                        step="1000"
                        class="pl-8"
                        bind:value={formData.exemption_threshold}
                    />
                </div>
                <p class="text-[10px] text-muted-foreground">
                    {$t("settings.fiscal.rules.form.exemptionHint")}
                </p>
            </div>

            <div
                class="flex items-center justify-between p-3 border rounded bg-muted/20"
            >
                <div class="space-y-0.5">
                    <Label class="text-base"
                        >{$t("settings.fiscal.rules.form.cumulative")}</Label
                    >
                    <p class="text-[10px] text-muted-foreground">
                        {$t("settings.fiscal.rules.form.cumulativeHint")}
                    </p>
                </div>
                <Switch bind:checked={formData.cumulative_losses} />
            </div>
        </div>

        <Dialog.Footer>
            <Button onclick={saveRule} disabled={isSubmittingRule}>
                {#if isSubmittingRule}
                    <span class="loading loading-spinner loading-xs"></span>
                    {$t("general.saving")}
                {:else}
                    {$t("settings.fiscal.rules.form.save")}
                {/if}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />
