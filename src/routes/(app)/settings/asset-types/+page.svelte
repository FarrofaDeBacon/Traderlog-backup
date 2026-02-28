<script lang="ts">
    import { Plus, Pencil, Trash2, Tag, Globe } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Card from "$lib/components/ui/card";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";
    import { Badge } from "$lib/components/ui/badge";
    import { settingsStore, type AssetType } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);
    let formData = $state<Omit<AssetType, "id">>({
        name: "",
        code: "",
        market_id: "",
        default_fee_id: "",
        tax_profile_id: "", // New field
        unit_label: "Pts",
        result_type: "points",
    });

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteAssetType(deleteId);
            if (!result.success) {
                toast.error(result.error || $t("general.error"));
            } else {
                toast.success($t("general.deleteSuccess"));
            }
            deleteId = null;
        }
    }

    function openNew() {
        editingId = null;
        formData = {
            name: "",
            code: "",
            market_id: "",
            default_fee_id: "",
            tax_profile_id: "",
            unit_label: "Pts",
            result_type: "points",
        };
        isDialogOpen = true;
    }

    function openEdit(item: AssetType) {
        editingId = item.id;
        formData = { ...item };
        isDialogOpen = true;
    }

    function save() {
        if (editingId) {
            settingsStore.updateAssetType(editingId, formData);
        } else {
            settingsStore.addAssetType(formData);
        }
        isDialogOpen = false;
    }

    // Group by Market
    let groupedTypes = $derived.by(() => {
        const groups: Record<string, AssetType[]> = {};
        for (const type of settingsStore.assetTypes) {
            const marketCode =
                settingsStore.getMarketCode(type.market_id) ||
                $t("settings.assets.groups.others");
            if (!groups[marketCode]) {
                groups[marketCode] = [];
            }
            groups[marketCode].push(type);
        }
        return groups;
    });

    function getMarketIcon(col: string) {
        // Simple visual hashing or default
        return Globe;
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">
                {$t("settings.assetTypes.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.assetTypes.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.assetTypes.new")}
        </Button>
    </div>
    <Separator />

    <div class="space-y-8">
        {#each Object.entries(groupedTypes) as [marketCode, types]}
            <div class="space-y-4">
                <!-- Rich Header -->
                <div class="flex items-center gap-2">
                    <div class="p-1.5 rounded bg-blue-500/10">
                        <Globe class="w-4 h-4 text-blue-500" />
                    </div>
                    <h4 class="text-lg font-semibold tracking-tight">
                        {marketCode}
                    </h4>
                </div>

                <div class="flex flex-col gap-3">
                    {#each types as type}
                        <div
                            class="flex items-center justify-between p-4 rounded-lg border bg-card text-card-foreground shadow-sm hover:border-primary/50 transition-all cursor-pointer group"
                            onclick={() => openEdit(type)}
                            role="button"
                            tabindex={0}
                            onkeydown={(e) =>
                                e.key === "Enter" && openEdit(type)}
                        >
                            <!-- Left: Icon + Info -->
                            <div class="flex items-center gap-4 shrink-0">
                                <div class="p-2.5 bg-muted rounded-xl shrink-0">
                                    <Tag class="w-5 h-5 text-foreground/70" />
                                </div>
                                <div class="min-w-[150px]">
                                    <div class="flex items-center gap-2">
                                        <h4 class="font-bold text-base">
                                            {type.code}
                                        </h4>
                                        <Badge
                                            variant="outline"
                                            class="text-[10px] h-5 px-1.5 font-normal"
                                        >
                                            {marketCode}
                                        </Badge>
                                    </div>
                                    <p class="text-sm text-muted-foreground">
                                        {type.name}
                                    </p>
                                </div>
                            </div>

                            <!-- Middle: Tax & Fee Profiles -->
                            <div
                                class="hidden xl:flex flex-col gap-1 min-w-[200px] border-l pl-4"
                            >
                                <div class="flex items-center gap-2">
                                    <span
                                        class="text-[10px] text-muted-foreground font-medium uppercase min-w-[80px]"
                                        >{$t(
                                            "settings.assetTypes.labels.fiscalProfile",
                                        )}:</span
                                    >
                                    <Badge
                                        variant="outline"
                                        class="text-[10px] py-0 border-primary/20 bg-primary/5"
                                    >
                                        {settingsStore.taxProfiles.find(
                                            (p) => p.id === type.tax_profile_id,
                                        )?.name ||
                                            $t(
                                                "settings.assetTypes.labels.default",
                                            )}
                                    </Badge>
                                </div>
                                <div class="flex items-center gap-2">
                                    <span
                                        class="text-[10px] text-muted-foreground font-medium uppercase min-w-[80px]"
                                        >{$t(
                                            "settings.assetTypes.labels.brokerageProfile",
                                        )}:</span
                                    >
                                    <Badge
                                        variant="outline"
                                        class="text-[10px] py-0 border-muted-foreground/20"
                                    >
                                        {settingsStore.fees.find(
                                            (f) => f.id === type.default_fee_id,
                                        )?.name ||
                                            $t(
                                                "settings.assetTypes.labels.none",
                                            )}
                                    </Badge>
                                </div>
                            </div>

                            <!-- Right: Actions -->
                            <div
                                class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-auto"
                            >
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onclick={(e) => {
                                        e.stopPropagation(); // Avoid triggering edit
                                        openEdit(type);
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
                                        requestDelete(type.id);
                                    }}
                                >
                                    <Trash2 class="w-4 h-4" />
                                </Button>
                            </div>
                        </div>
                    {/each}
                </div>
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
                    ? $t("settings.assetTypes.edit")
                    : $t("settings.assetTypes.new")}</Dialog.Title
            >
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assetTypes.form.name")}</Label
                >
                <Input
                    bind:value={formData.name}
                    class="col-span-3"
                    placeholder={$t("settings.assetTypes.form.namePlaceholder")}
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assetTypes.form.code")}</Label
                >
                <Input
                    bind:value={formData.code}
                    class="col-span-3 uppercase"
                    placeholder={$t("settings.assetTypes.form.codePlaceholder")}
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assetTypes.form.market")}</Label
                >
                <div class="col-span-3">
                    <Select.Root type="single" bind:value={formData.market_id}>
                        <Select.Trigger
                            >{settingsStore.getMarketCode(formData.market_id) ||
                                $t(
                                    "settings.assetTypes.form.marketPlaceholder",
                                )}</Select.Trigger
                        >
                        <Select.Content>
                            {#each settingsStore.markets as m}
                                <Select.Item value={m.id}>{m.code}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assetTypes.form.unitLabel")}</Label
                >
                <Input
                    bind:value={formData.unit_label}
                    class="col-span-3"
                    placeholder={$t(
                        "settings.assetTypes.form.unitLabelPlaceholder",
                    )}
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assetTypes.form.resultType")}</Label
                >
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.result_type}
                    >
                        <Select.Trigger>
                            {formData.result_type === "points"
                                ? $t(
                                      "settings.assetTypes.form.resultTypeOptions.points",
                                  )
                                : $t(
                                      "settings.assetTypes.form.resultTypeOptions.currency",
                                  )}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="points"
                                >{$t(
                                    "settings.assetTypes.form.resultTypeOptions.points",
                                )}</Select.Item
                            >
                            <Select.Item value="currency"
                                >{$t(
                                    "settings.assetTypes.form.resultTypeOptions.currency",
                                )}</Select.Item
                            >
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <Separator class="my-2" />

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assetTypes.labels.fiscalProfile")}</Label
                >
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.tax_profile_id}
                    >
                        <Select.Trigger>
                            {settingsStore.taxProfiles.find(
                                (p) => p.id === formData.tax_profile_id,
                            )?.name ||
                                $t("settings.assetTypes.labels.noneExempt")}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value=""
                                >{$t(
                                    "settings.assetTypes.labels.noneExempt",
                                )}</Select.Item
                            >
                            {#each settingsStore.taxProfiles as p}
                                <Select.Item value={p.id}>{p.name}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assetTypes.labels.brokerageProfile")}</Label
                >
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.default_fee_id}
                    >
                        <Select.Trigger>
                            {settingsStore.fees.find(
                                (f) => f.id === formData.default_fee_id,
                            )?.name ||
                                $t("settings.assetTypes.labels.noneDefault")}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value=""
                                >{$t(
                                    "settings.assetTypes.labels.noneDefault",
                                )}</Select.Item
                            >
                            {#each settingsStore.fees as f}
                                <Select.Item value={f.id}>{f.name}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>
        </div>

        <Dialog.Footer>
            <Button onclick={save}>{$t("settings.assetTypes.form.save")}</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
