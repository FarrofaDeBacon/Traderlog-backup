<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Receipt,
        Calculator,
        Building2,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Textarea } from "$lib/components/ui/textarea";
    import * as Card from "$lib/components/ui/card";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Separator } from "$lib/components/ui/separator";
    import {
        settingsStore,
        type FeeProfile,
    } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let formData = $state<Omit<FeeProfile, "id">>({
        name: "",
        broker: "",
        fixed_fee: 0,
        percentage_fee: 0,
        exchange_fee: 0,
        iss: 0,
        currency_spread: 0,
        withholding_tax: 0,
        income_tax_rate: 0,
        custom_items: [],
        tax_rule_id: undefined,
        notes: "",
    });

    // Unique Brokers from Accounts
    let uniqueBrokers = $derived(
        Array.from(new Set(settingsStore.accounts.map((a) => a.broker))).sort(),
    );

    // Group Fees by Broker
    let groupedFees = $derived.by(() => {
        const groups: Record<string, FeeProfile[]> = {};
        for (const fee of settingsStore.fees) {
            const broker = fee.broker || $t("general.all");
            if (!groups[broker]) {
                groups[broker] = [];
            }
            groups[broker].push(fee);
        }
        return groups;
    });

    function getBrokerStyle(name: string) {
        const n = name.toLowerCase();
        if (n.includes("binance"))
            return { color: "text-amber-500", bg: "bg-amber-500/10" };
        if (n.includes("xp"))
            return { color: "text-blue-500", bg: "bg-blue-500/10" };
        if (n.includes("profit") || n.includes("nelogica"))
            return { color: "text-green-500", bg: "bg-green-500/10" };
        return { color: "text-muted-foreground", bg: "bg-muted" };
    }

    // Simulation simulation
    let simValue = $state(10000);

    function resetForm() {
        formData = {
            name: "",
            broker: "",
            fixed_fee: 0,
            percentage_fee: 0,
            exchange_fee: 0,
            iss: 0,
            currency_spread: 0,
            withholding_tax: 0,
            income_tax_rate: 0,
            custom_items: [],
            tax_rule_id: undefined,
            notes: "",
        };
        editingId = null;
    }

    function openNew() {
        resetForm();
        isDialogOpen = true;
    }

    function openEdit(profile: FeeProfile) {
        editingId = profile.id;
        formData = { ...profile };
        isDialogOpen = true;
    }

    function save() {
        const payload = { ...formData };
        if (payload.tax_rule_id === "no_rule") {
            payload.tax_rule_id = undefined;
        }

        if (editingId) {
            settingsStore.updateFeeProfile(editingId, payload);
        } else {
            settingsStore.addFeeProfile(payload);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteFeeProfile(deleteId);
            if (!result.success) {
                toast.error(result.error || $t("general.error"));
            } else {
                toast.success($t("general.deleteSuccess"));
            }
            deleteId = null;
        }
    }

    // Calculation Helpers
    function calc(val: number, pct: number) {
        return (val * pct) / 100;
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">
                {$t("settings.fees.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.fees.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.fees.new")}
        </Button>
    </div>
    <Separator />

    <div class="space-y-8">
        {#each Object.entries(groupedFees) as [brokerName, profiles]}
            {@const style = getBrokerStyle(brokerName)}
            <div class="space-y-4">
                <!-- Rich Header -->
                <div class="flex items-center gap-2">
                    <div class={`p-1.5 rounded ${style.bg}`}>
                        <Building2 class={`w-4 h-4 ${style.color}`} />
                    </div>
                    <h4 class="text-lg font-semibold tracking-tight">
                        {brokerName}
                    </h4>
                </div>

                <div class="flex flex-col gap-3">
                    {#each profiles as profile}
                        <div
                            class="flex items-center justify-between p-4 rounded-lg border bg-card hover:border-primary/50 transition-all group shadow-sm cursor-pointer"
                            onclick={() => openEdit(profile)}
                            onkeydown={(e) =>
                                e.key === "Enter" && openEdit(profile)}
                            tabindex="0"
                            role="button"
                        >
                            <!-- Left: Icon + Info -->
                            <div class="flex items-center gap-4 shrink-0">
                                <div class="p-2.5 bg-muted rounded-xl shrink-0">
                                    <Receipt
                                        class="w-5 h-5 text-foreground/70"
                                    />
                                </div>
                                <div class="min-w-[150px]">
                                    <h4 class="font-bold text-base">
                                        {profile.name}
                                    </h4>
                                    <p class="text-sm text-muted-foreground">
                                        {profile.broker}
                                    </p>
                                </div>
                            </div>

                            <!-- Middle: Stats (Horizontal) -->
                            <div
                                class="flex items-center gap-6 mr-auto ml-8 text-sm"
                            >
                                {#if profile.fixed_fee > 0}
                                    <div class="flex flex-col items-start">
                                        <span
                                            class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider"
                                            >{$t(
                                                "settings.fees.labels.fixed",
                                            )}</span
                                        >
                                        <span class="font-mono font-medium"
                                            >{profile.fixed_fee}</span
                                        >
                                    </div>
                                {/if}
                                {#if profile.percentage_fee > 0}
                                    <div class="flex flex-col items-start">
                                        <span
                                            class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider"
                                            >{$t(
                                                "settings.fees.labels.brokerage",
                                            )}</span
                                        >
                                        <span class="font-mono font-medium"
                                            >{profile.percentage_fee}%</span
                                        >
                                    </div>
                                {/if}
                                {#if profile.exchange_fee > 0}
                                    <div class="flex flex-col items-start">
                                        <span
                                            class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider"
                                            >{$t(
                                                "settings.fees.labels.exchange",
                                            )}</span
                                        >
                                        <span class="font-mono font-medium"
                                            >{profile.exchange_fee}%</span
                                        >
                                    </div>
                                {/if}
                            </div>

                            <!-- Right: Actions -->
                            <div
                                class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
                            >
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        openEdit(profile);
                                    }}
                                >
                                    <Pencil class="w-4 h-4" />
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="text-destructive hover:text-destructive hover:bg-destructive/10"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        requestDelete(profile.id);
                                    }}
                                >
                                    <Trash2 class="w-4 h-4" />
                                </Button>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        {:else}
            <div
                class="flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg border-muted-foreground/25 text-muted-foreground h-[150px]"
            >
                <span class="text-sm">{$t("settings.fees.empty")}</span>
            </div>
        {/each}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[600px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingId
                    ? $t("settings.fees.form.titleEdit")
                    : $t("settings.fees.form.titleNew")}</Dialog.Title
            >
            <Dialog.Description
                >{$t("settings.fees.form.description")}</Dialog.Description
            >
        </Dialog.Header>

        <div class="grid gap-6 py-4">
            <div class="grid grid-cols-2 gap-4">
                <div class="space-y-2">
                    <Label>{$t("settings.fees.form.name")}</Label>
                    <Input
                        bind:value={formData.name}
                        placeholder={$t("settings.fees.form.namePlaceholder")}
                    />
                </div>
                <div class="space-y-2">
                    <Label>{$t("settings.fees.form.broker")}</Label>
                    <Select.Root type="single" bind:value={formData.broker}>
                        <Select.Trigger>
                            {formData.broker || $t("settings.fees.form.select")}
                        </Select.Trigger>
                        <Select.Content>
                            {#each uniqueBrokers as broker}
                                <Select.Item value={broker}
                                    >{broker}</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <Separator />

            <Separator />

            <div class="space-y-4">
                <div class="flex items-center gap-2">
                    <Calculator class="w-4 h-4" />
                    <h4 class="font-medium">
                        {$t("settings.fees.form.operationalCosts")}
                    </h4>
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-2">
                        <Label>{$t("settings.fees.form.fixedFee")}</Label>
                        <Input
                            type="number"
                            step="0.01"
                            bind:value={formData.fixed_fee}
                        />
                        <span class="text-[10px] text-muted-foreground"
                            >{$t("settings.fees.form.fixedFeeHint")}</span
                        >
                    </div>
                    <div class="space-y-2">
                        <Label>{$t("settings.fees.form.percentageFee")}</Label>
                        <Input
                            type="number"
                            step="0.001"
                            bind:value={formData.percentage_fee}
                        />
                        <span class="text-[10px] text-muted-foreground"
                            >{$t("settings.fees.form.percentageFeeHint")}</span
                        >
                    </div>
                    <div class="space-y-2">
                        <Label>{$t("settings.fees.form.exchangeFee")}</Label>
                        <Input
                            type="number"
                            step="0.001"
                            bind:value={formData.exchange_fee}
                        />
                        <span class="text-[10px] text-muted-foreground"
                            >{$t("settings.fees.form.exchangeFeeHint")}</span
                        >
                    </div>
                    <div class="space-y-2">
                        <Label>{$t("settings.fees.form.iss")}</Label>
                        <Input
                            type="number"
                            step="0.01"
                            bind:value={formData.iss}
                        />
                        <span class="text-[10px] text-muted-foreground"
                            >{$t("settings.fees.form.issHint")}</span
                        >
                    </div>
                    <div class="space-y-2">
                        <Label>{$t("settings.fees.form.currencySpread")}</Label>
                        <Input
                            type="number"
                            step="0.01"
                            bind:value={formData.currency_spread}
                        />
                        <span class="text-[10px] text-muted-foreground"
                            >{$t("settings.fees.form.currencySpreadHint")}</span
                        >
                    </div>
                </div>

                <!-- Custom Costs -->
                <div class="space-y-3 pt-2">
                    <div class="flex items-center justify-between">
                        <Label>{$t("settings.fees.form.customCosts")}</Label>
                        <Button
                            variant="outline"
                            size="sm"
                            class="h-7 text-xs"
                            onclick={() => {
                                formData.custom_items = [
                                    ...formData.custom_items,
                                    {
                                        id: crypto.randomUUID(),
                                        name: "",
                                        value: 0,
                                        type: "fixed",
                                        category: "cost",
                                    },
                                ];
                            }}
                        >
                            <Plus class="w-3 h-3 mr-1" />
                            {$t("settings.fees.form.addItem")}
                        </Button>
                    </div>

                    {#if formData.custom_items.filter((i) => i.category === "cost").length === 0}
                        <div
                            class="text-xs text-muted-foreground text-center py-2 border border-dashed rounded bg-muted/30"
                        >
                            {$t("settings.fees.form.noCustomItems")}
                        </div>
                    {/if}

                    {#each formData.custom_items as item, i}
                        {#if item.category === "cost"}
                            <div
                                class="flex items-start gap-2 p-2 rounded border bg-card"
                            >
                                <div class="grid grid-cols-12 gap-2 flex-1">
                                    <div class="col-span-6 space-y-1">
                                        <Label
                                            class="text-[10px] uppercase text-muted-foreground"
                                            >{$t("general.name")}</Label
                                        >
                                        <Input
                                            class="h-8 text-sm"
                                            bind:value={item.name}
                                            placeholder={$t(
                                                "settings.fees.form.itemNamePlaceholder",
                                            )}
                                        />
                                    </div>
                                    <div class="col-span-3 space-y-1">
                                        <Label
                                            class="text-[10px] uppercase text-muted-foreground"
                                            >{$t("general.type")}</Label
                                        >
                                        <Select.Root
                                            type="single"
                                            bind:value={item.type}
                                        >
                                            <Select.Trigger class="h-8 text-xs">
                                                {item.type === "fixed"
                                                    ? "$"
                                                    : "%"}
                                            </Select.Trigger>
                                            <Select.Content>
                                                <Select.Item value="fixed"
                                                    >{$t(
                                                        "settings.fees.form.fixed",
                                                    )}</Select.Item
                                                >
                                                <Select.Item value="percentage"
                                                    >{$t(
                                                        "settings.fees.form.percentage",
                                                    )}</Select.Item
                                                >
                                            </Select.Content>
                                        </Select.Root>
                                    </div>
                                    <div class="col-span-3 space-y-1">
                                        <Label
                                            class="text-[10px] uppercase text-muted-foreground"
                                            >{$t("general.value")}</Label
                                        >
                                        <Input
                                            type="number"
                                            step={item.type === "fixed"
                                                ? "0.01"
                                                : "0.001"}
                                            class="h-8 text-sm"
                                            bind:value={item.value}
                                        />
                                    </div>
                                </div>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="h-8 w-8 text-destructive hover:bg-destructive/10 mt-5"
                                    onclick={() => {
                                        formData.custom_items =
                                            formData.custom_items.filter(
                                                (_, idx) => idx !== i,
                                            );
                                    }}
                                >
                                    <Trash2 class="w-4 h-4" />
                                </Button>
                            </div>
                        {/if}
                    {/each}
                </div>

                <!-- Simulator -->
                <div class="rounded-md bg-muted p-4 space-y-2">
                    <div class="flex items-center justify-between">
                        <span
                            class="text-xs font-semibold uppercase text-muted-foreground"
                            >{$t("settings.fees.form.simulation", {
                                values: { value: simValue },
                            })}</span
                        >
                        <Input
                            type="number"
                            class="text-xs w-24 h-6"
                            bind:value={simValue}
                        />
                    </div>
                    <div class="text-sm space-y-1 font-mono">
                        <div class="flex justify-between">
                            <span>{$t("settings.fees.form.simFixed")}</span>
                            <span>{formData.fixed_fee.toFixed(2)}</span>
                        </div>
                        <div class="flex justify-between">
                            <span>{$t("settings.fees.form.simPercent")}</span>
                            <span
                                >{calc(
                                    simValue,
                                    formData.percentage_fee,
                                ).toFixed(2)}</span
                            >
                        </div>
                        <div class="flex justify-between">
                            <span>{$t("settings.fees.form.simExchange")}</span>
                            <span
                                >{calc(simValue, formData.exchange_fee).toFixed(
                                    2,
                                )}</span
                            >
                        </div>

                        <div
                            class="border-t pt-1 mt-1 flex justify-between font-bold text-primary"
                        >
                            <span>{$t("settings.fees.form.totalCosts")}</span>
                            <span>
                                {(
                                    formData.fixed_fee +
                                    calc(simValue, formData.percentage_fee) +
                                    calc(simValue, formData.exchange_fee) +
                                    calc(
                                        simValue + formData.fixed_fee,
                                        formData.iss,
                                    ) +
                                    calc(simValue, formData.currency_spread) +
                                    formData.custom_items
                                        .filter((i) => i.category === "cost")
                                        .reduce(
                                            (acc, item) =>
                                                acc +
                                                (item.type === "fixed"
                                                    ? item.value
                                                    : calc(
                                                          simValue,
                                                          item.value,
                                                      )),
                                            0,
                                        )
                                ).toFixed(2)}
                            </span>
                        </div>
                    </div>
                </div>

                <div class="space-y-2">
                    <Label>{$t("settings.fees.form.notes")}</Label>
                    <Textarea
                        bind:value={formData.notes}
                        placeholder={$t("settings.fees.form.notesPlaceholder")}
                    />
                </div>
            </div>

            <Dialog.Footer>
                <Button onclick={save}>{$t("settings.fees.form.save")}</Button>
            </Dialog.Footer>
        </div></Dialog.Content
    >
</Dialog.Root>
