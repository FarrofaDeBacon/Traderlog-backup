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
    import { FileSpreadsheet } from "lucide-svelte";

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
        default_fee_id: "",
        tax_profile_id: "",
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
                color: "text-purple-500",
                bg: "bg-purple-500/10",
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
        // Or just let them be created dynamically. Let's use the defined assetTypes to drive the order.
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
        if (typeId === "unknown") return "Outros";
        const type = settingsStore.assetTypes.find((t) => t.id === typeId);
        return type ? `${type.code} - ${type.name}` : "Desconhecido";
    }

    function openNew() {
        editingId = null;
        formData = {
            symbol: "",
            name: "",
            asset_type_id: "",
            point_value: 1,
            default_fee_id: "",
        };
        isDialogOpen = true;
    }

    function openEdit(item: Asset) {
        editingId = item.id;
        formData = { ...item };
        isDialogOpen = true;
    }

    function save() {
        if (editingId) {
            settingsStore.updateAsset(editingId, formData);
        } else {
            settingsStore.addAsset(formData);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    import { toast } from "svelte-sonner";

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
                                <div class="p-2.5 bg-muted rounded-xl shrink-0">
                                    <PieChart
                                        class="w-5 h-5 text-foreground/70"
                                    />
                                </div>
                                <div class="min-w-[150px]">
                                    <div class="flex items-center gap-2">
                                        <h4 class="font-bold text-base">
                                            {asset.symbol}
                                        </h4>
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
                                    <p class="text-sm text-muted-foreground">
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
        {:else}
            <!-- Empty State when no groups found (meaning filtered list is empty) -->
            <div
                class="flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg border-muted-foreground/25 text-muted-foreground h-[150px]"
            >
                <Search class="w-8 h-8 mb-2 opacity-20" />
                <span class="text-sm">{$t("settings.assets.empty")}</span>
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
                        <Select.Trigger
                            >{settingsStore.getAssetTypeCode(
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
                <Label class="text-right">Perfil de Corretagem</Label>
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.default_fee_id}
                    >
                        <Select.Trigger>
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
                <Label class="text-right">Perfil Fiscal</Label>
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.tax_profile_id}
                    >
                        <Select.Trigger>
                            {settingsStore.taxProfiles.find(
                                (p) => p.id === formData.tax_profile_id,
                            )?.name || "Usar padrão do Tipo"}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value=""
                                >Usar padrão do Tipo</Select.Item
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
