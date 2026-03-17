<!-- SIZING ENABLED ASSET FORM -->
<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Search,
        PieChart,
        CandlestickChart,
        Landmark,
        Bitcoin,
        Globe,
        Layers,
        Activity,
        Building2,
        FileSpreadsheet,
        Link,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";
    import { Badge } from "$lib/components/ui/badge";
    import { settingsStore, type Asset } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import RTDImportDialog from "$lib/components/settings/RTDImportDialog.svelte";
    import Skeleton from "$lib/components/ui/skeleton.svelte";
    import { toast } from "svelte-sonner";
    import { Checkbox } from "$lib/components/ui/checkbox";

    let isDialogOpen = $state(false);
    let isImportOpen = $state(false);
    let editingId = $state<string | null>(null);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let formData = $state<Omit<Asset, "id">>({
        symbol: "",
        name: "",
        asset_type_id: "",
        point_value: 1,
        contract_size: 1,
        default_fee_id: "",
        tax_profile_id: "",
        is_root: false,
        root_id: "none",
    });

    function getAssetTypeStyle(code: string = "") {
        const c = code.toUpperCase();
        if (c.includes("FUT"))
            return {
                icon: CandlestickChart,
                color: "text-blue-500",
                bg: "bg-blue-500/10",
            };
        if (c.includes("STK") || c.includes("AÇÃO") || c.includes("ACOES"))
            return {
                icon: Landmark,
                color: "text-green-500",
                bg: "bg-green-500/10",
            };
        if (c.includes("CRYPTO") || c.includes("CRIPTO") || c.includes("BTC"))
            return {
                icon: Bitcoin,
                color: "text-amber-500",
                bg: "bg-amber-500/10",
            };
        if (c.includes("FOREX") || c.includes("FX"))
            return {
                icon: Globe,
                color: "text-indigo-500",
                bg: "bg-indigo-500/10",
            };
        if (c.includes("FII"))
            return {
                icon: Building2,
                color: "text-orange-500",
                bg: "bg-orange-500/10",
            };
        return {
            icon: Layers,
            color: "text-muted-foreground",
            bg: "bg-muted",
        }; // Default
    }

    // Sort assets
    let filteredAssets = $derived(
        [...settingsStore.assets].sort((a, b) =>
            a.symbol.localeCompare(b.symbol),
        ),
    );

    // Group assets by type
    let groupedAssets = $derived.by(() => {
        const groups: Record<string, Asset[]> = {};

        // Initialize groups for all defined types to ensure order (optional, but good for consistency)
        for (const type of settingsStore.assetTypes) {
            const assetsInType = filteredAssets.filter(
                (a) =>
                    a.asset_type_id === type.id ||
                    a.asset_type_id.replace(/^asset_type:/, "") ===
                        type.id.replace(/^asset_type:/, ""),
            );
            if (assetsInType.length > 0) {
                groups[type.id] = assetsInType;
            }
        }

        // Handle assets with unknown types (if any)
        const unknownTypeAssets = filteredAssets.filter(
            (a) =>
                !settingsStore.assetTypes.find(
                    (t) =>
                        t.id === a.asset_type_id ||
                        t.id.replace(/^asset_type:/, "") ===
                            a.asset_type_id.replace(/^asset_type:/, ""),
                ),
        );
        if (unknownTypeAssets.length > 0) {
            groups["unknown"] = unknownTypeAssets;
        }

        return groups;
    });

    function getTypeName(typeId: string) {
        if (typeId === "unknown") return $t("settings.assets.groups.others");
        const type = settingsStore.assetTypes.find((t) => t.id === typeId);
        return type
            ? `${type.code} - ${type.name}`
            : $t("settings.assets.labels.unknown");
    }

    function openNew() {
        editingId = null;
        formData = {
            symbol: "",
            name: "",
            asset_type_id: "",
            point_value: 1,
            contract_size: 1,
            default_fee_id: "",
            tax_profile_id: "",
            is_root: false,
            root_id: "none",
        };
        isDialogOpen = true;
    }

    function openEdit(item: Asset) {
        editingId = item.id;
        formData = {
            symbol: item.symbol,
            name: item.name,
            asset_type_id: item.asset_type_id,
            point_value: item.point_value,
            contract_size: item.contract_size ?? 1,
            default_fee_id: item.default_fee_id,
            tax_profile_id: item.tax_profile_id || "",
            is_root: item.is_root || false,
            root_id: item.root_id || "none",
        };
        isDialogOpen = true;
    }

    async function save() {
        // Validações do Position Sizing
        if (formData.point_value <= 0) {
            toast.error("O valor do ponto deve ser maior que zero.");
            return;
        }

        const dataToSave = {
            ...formData,
            root_id: formData.root_id === "none" ? undefined : formData.root_id,
            contract_size: formData.contract_size,
        };

        if (editingId) {
            await settingsStore.updateAsset(editingId, dataToSave);
        } else {
            await settingsStore.addAsset(dataToSave);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            console.log("Confirming delete for:", deleteId);
            const result = await settingsStore.deleteAsset(deleteId);
            if (!result.success) {
                console.error("Delete failed:", result.error);
                toast.error(result.error || $t("general.error"));
            } else {
                toast.success($t("general.deleteSuccess"));
            }
            deleteId = null;
        }
    }

    let rootAssets = $derived(
        settingsStore.assets.filter((a) => a.is_root && a.id !== editingId),
    );

    function getRootSymbol(rootId: string | null) {
        if (!rootId) return "";
        const root = settingsStore.assets.find((a) => a.id === rootId);
        return root ? root.symbol : "";
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">
                {$t("settings.assets.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.assets.description")}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="outline" onclick={() => (isImportOpen = true)}>
                <FileSpreadsheet class="w-4 h-4 mr-2" />
                {$t("settings.assets.importRTD")}
            </Button>
            <Button onclick={openNew}>
                <Plus class="w-4 h-4 mr-2" />
                {$t("settings.assets.new")}
            </Button>
        </div>
    </div>
    <Separator />

    <RTDImportDialog bind:open={isImportOpen} />

    <!-- Grouped Clickable List Cards -->
    <div class="space-y-6">
        {#if settingsStore.isLoadingData && Object.keys(groupedAssets).length === 0}
            <div class="space-y-4">
                <Skeleton class="h-8 w-48" />
                <div class="grid gap-2">
                    {#each Array(5) as _}
                        <Skeleton class="h-20 rounded-lg" />
                    {/each}
                </div>
            </div>
        {:else if Object.keys(groupedAssets).length > 0}
            {#each Object.entries(groupedAssets) as [typeId, assets]}
                {@const typeCode = settingsStore.getAssetTypeCode(typeId)}
                {@const style = getAssetTypeStyle(typeCode)}
                {@const Icon = style.icon}

                <div class="space-y-4">
                    <!-- Rich Header -->
                    <div class="flex items-center gap-2">
                        <div class={`p-1.5 rounded ${style.bg}`}>
                            <Icon class={`w-4 h-4 ${style.color}`} />
                        </div>
                        <h4 class="text-lg font-semibold tracking-tight">
                            {getTypeName(typeId)}
                        </h4>
                    </div>

                    <div class="grid gap-2">
                        {#each assets as asset}
                            <div
                                class="flex items-center justify-between p-4 rounded-lg border bg-card text-card-foreground shadow-sm hover:border-primary/50 transition-all cursor-pointer group"
                                onclick={() => openEdit(asset)}
                                role="button"
                                tabindex="0"
                                onkeydown={(e) =>
                                    e.key === "Enter" && openEdit(asset)}
                            >
                                <!-- Left: Icon + Info -->
                                <div class="flex items-center gap-4 shrink-0">
                                    <div
                                        class="p-2.5 bg-muted rounded-xl shrink-0"
                                    >
                                        <PieChart
                                            class="w-5 h-5 text-foreground/70"
                                        />
                                    </div>
                                    <div class="min-w-[150px]">
                                        <div class="flex items-center gap-2">
                                            <h4 class="font-bold text-base">
                                                {asset.symbol}
                                            </h4>
                                            {#if asset.is_root}
                                                <Badge
                                                    variant="outline"
                                                    class="text-[10px] h-5 px-1.5 font-bold border-amber-500/50 text-amber-600 bg-amber-500/5"
                                                >
                                                    RAIZ
                                                </Badge>
                                            {/if}
                                            {#if asset.root_id}
                                                <div
                                                    class="flex items-center gap-1 text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded border border-border/50"
                                                >
                                                    <Link class="w-3 h-3" />
                                                    {getRootSymbol(
                                                        asset.root_id,
                                                    )}
                                                </div>
                                            {/if}
                                            {#if !settingsStore.assetTypes.find((t) => t.id === asset.asset_type_id || t.id.replace(/^asset_type:/, "") === asset.asset_type_id.replace(/^asset_type:/, ""))}
                                                <Badge
                                                    variant="destructive"
                                                    class="text-[10px] h-5 px-1.5 font-normal"
                                                >
                                                    {$t(
                                                        "settings.assets.labels.invalid",
                                                    )}
                                                </Badge>
                                            {:else}
                                                <Badge
                                                    variant="secondary"
                                                    class="text-[10px] h-5 px-1.5 font-normal border-primary/20"
                                                >
                                                    {settingsStore.getAssetTypeCode(
                                                        asset.asset_type_id,
                                                    )}
                                                </Badge>
                                            {/if}
                                        </div>
                                        <p
                                            class="text-sm text-muted-foreground"
                                        >
                                            {asset.name}
                                        </p>
                                    </div>
                                </div>

                                <!-- Middle: Value Point -->
                                <div
                                    class="flex flex-col items-center mr-auto ml-12"
                                >
                                    <span
                                        class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider"
                                        >{$t(
                                            "settings.assets.labels.pointValue",
                                        )}</span
                                    >
                                    <span
                                        class="font-bold text-lg text-foreground tracking-tight"
                                        >{asset.point_value}</span
                                    >
                                </div>

                                <!-- Right: Actions -->
                                <div
                                    class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
                                >
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        onclick={(e) => {
                                            e.stopPropagation(); // Prevent card click
                                            openEdit(asset);
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
                                            requestDelete(asset.id);
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
        {:else}
            <!-- Empty State when no groups found (meaning filtered list is empty) -->
            <div
                class="flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg border-muted-foreground/25 text-muted-foreground h-[150px]"
            >
                <Search class="w-8 h-8 mb-2 opacity-20" />
                <span class="text-sm">{$t("settings.assets.empty")}</span>
            </div>
        {/if}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[600px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingId
                    ? $t("settings.assets.edit")
                    : $t("settings.assets.new")}</Dialog.Title
            >
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assets.form.symbol")}</Label
                >
                <Input
                    bind:value={formData.symbol}
                    class="col-span-3 uppercase"
                    placeholder={$t("settings.assets.form.symbolPlaceholder")}
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assets.form.pointValue")}</Label
                >
                <Input
                    type="number"
                    step="0.0001"
                    bind:value={formData.point_value}
                    class="col-span-3"
                />
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right leading-tight">Tam. Contrato<br/><span class="text-[0.65rem] text-muted-foreground opacity-80">(Multiplicador)</span></Label>
                <Input
                    type="number"
                    step="0.01"
                    bind:value={formData.contract_size}
                    class="col-span-3"
                    placeholder="Ex: 1 (B3) ou 100000 (Forex)"
                />
            </div>


            <div class="grid grid-cols-4 items-center gap-4">
                <div class="col-start-2 col-span-3 flex items-center gap-2">
                    <Checkbox
                        id="is_root"
                        bind:checked={formData.is_root}
                        onCheckedChange={(v) => {
                            if (v) formData.root_id = "none";
                        }}
                    />
                    <Label for="is_root" class="cursor-pointer">
                        {$t("settings.assets.form.isRoot") || "Ativo Raiz"}
                    </Label>
                </div>
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assets.form.rootAsset") ||
                        "Vincular a Raiz"}</Label
                >
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        value={formData.root_id}
                        onValueChange={(v) => (formData.root_id = v)}
                        disabled={formData.is_root}
                    >
                        <Select.Trigger class="w-full">
                            {settingsStore.assets.find(
                                (a) => a.id === formData.root_id,
                            )?.symbol ||
                                (formData.root_id === "none"
                                    ? $t("general.none") || "Nenhum"
                                    : $t(
                                          "settings.assets.form.rootAssetPlaceholder",
                                      ) || "Selecione um ativo raiz")}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="none">{$t("general.none") || "Nenhum"}</Select.Item>
                            {#each rootAssets as root}
                                <Select.Item value={root.id}
                                    >{root.symbol}</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assets.form.name")}</Label
                >
                <Input
                    bind:value={formData.name}
                    class="col-span-3"
                    placeholder={$t("settings.assets.form.namePlaceholder")}
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assets.form.type")}</Label
                >
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.asset_type_id}
                    >
                        <Select.Trigger class="w-full">
                            {settingsStore.getAssetTypeCode(
                                formData.asset_type_id,
                            ) ||
                                $t(
                                    "settings.assets.form.typePlaceholder",
                                )}</Select.Trigger
                        >
                        <Select.Content>
                            {#each settingsStore.assetTypes as t}
                                <Select.Item value={t.id}>{t.code}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right leading-tight"
                    >{$t("settings.assets.form.brokerageProfile")}</Label
                >
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.default_fee_id}
                    >
                        <Select.Trigger class="w-full">
                            {settingsStore.fees.find(
                                (f) => f.id === formData.default_fee_id,
                            )?.name ||
                                $t(
                                    "settings.assets.form.specificFeePlaceholder",
                                )}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value=""
                                >{$t(
                                    "settings.assets.form.useTypeDefault",
                                )}</Select.Item
                            >
                            {#each settingsStore.fees as f}
                                <Select.Item value={f.id}>{f.name}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.assets.form.fiscalProfile")}</Label
                >
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.tax_profile_id}
                    >
                        <Select.Trigger class="w-full">
                            {settingsStore.taxProfiles.find(
                                (p) => p.id === formData.tax_profile_id,
                            )?.name ||
                                $t("settings.assets.form.useTypeDefault")}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value=""
                                >{$t(
                                    "settings.assets.form.useTypeDefault",
                                )}</Select.Item
                            >
                            {#each settingsStore.taxProfiles as p}
                                <Select.Item value={p.id}>{p.name}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>
        </div>

        <Dialog.Footer>
            <Button onclick={save}>{$t("settings.assets.form.save")}</Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
