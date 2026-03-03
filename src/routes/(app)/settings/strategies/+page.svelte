<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Target,
        X,
        UploadCloud,
        ImageIcon,
        Eye,
        Globe,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { t } from "svelte-i18n";
    import { Label } from "$lib/components/ui/label";
    import { Textarea } from "$lib/components/ui/textarea";
    import * as Card from "$lib/components/ui/card";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";
    import { Switch } from "$lib/components/ui/switch";
    import { settingsStore, type Strategy } from "$lib/stores/settings.svelte";
    import { Badge } from "$lib/components/ui/badge";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    // --- State ---
    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    // Initial Empty State
    const emptyForm = {
        name: "",
        description: "",
        market_ids: [] as string[],
        timeframes: [] as string[],
        asset_types: [] as string[],
        indicators: [] as string[],
        specific_assets: [] as string[],
        entry_criteria: "",
        exit_criteria: "",
        management_criteria: "",
        has_partial: false,
        partial_description: "",
        images: [] as { path: string; description: string }[],
    };

    let formData = $state<Omit<Strategy, "id">>({ ...emptyForm });

    // Temp inputs for adding tags
    let tempTimeframe = $state("");
    let tempIndicator = $state("");
    let tempAssetType = $state("");
    let tempSpecificAsset = $state("");

    // --- Actions ---
    function openNew() {
        formData = JSON.parse(JSON.stringify(emptyForm));
        editingId = null;
        isDialogOpen = true;
    }

    function openEdit(strategy: Strategy) {
        editingId = strategy.id;
        formData = JSON.parse(JSON.stringify(strategy));
        isDialogOpen = true;
    }

    function save() {
        if (editingId) {
            settingsStore.updateStrategy(editingId, formData);
        } else {
            settingsStore.addStrategy(formData);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteStrategy(deleteId);
            if (!result.success) {
                toast.error(result.error || $t("general.error"));
            } else {
                toast.success($t("general.deleteSuccess"));
            }
            deleteId = null;
        }
    }

    // --- Tag Helpers ---
    function addTag(
        field: "timeframes" | "indicators" | "asset_types" | "specific_assets",
        value: string,
    ) {
        if (!value.trim()) return;
        if (!formData[field].includes(value)) {
            formData[field] = [...formData[field], value];
        }
        // Clear appropriate temp input
        if (field === "timeframes") tempTimeframe = "";
        if (field === "indicators") tempIndicator = "";
        if (field === "asset_types") tempAssetType = "";
        if (field === "specific_assets") tempSpecificAsset = "";
    }

    function removeTag(
        field: "timeframes" | "indicators" | "asset_types" | "specific_assets",
        index: number,
    ) {
        formData[field] = formData[field].filter((_, i) => i !== index);
    }

    // --- Image Helper (Mock for now) ---
    function handleImageUpload(e: Event) {
        const input = e.target as HTMLInputElement;
        if (!input.files?.length) return;

        // In a real app, this would be a file upload to backend
        // Here we mock with base64 for local persistence in store
        Array.from(input.files).forEach((file) => {
            const reader = new FileReader();
            reader.onload = (e) => {
                if (e.target?.result) {
                    formData.images = [
                        ...formData.images,
                        { path: e.target.result as string, description: "" },
                    ];
                }
            };
            reader.readAsDataURL(file);
        });
        input.value = "";
    }

    function removeImage(index: number) {
        formData.images = formData.images.filter((_, i) => i !== index);
    }

    // Group Strategies
    let groupedStrategies = $derived.by(() => {
        const groups: Record<string, Strategy[]> = {};
        for (const item of settingsStore.strategies) {
            // Use first asset type as category, or "Geral"
            const category =
                item.asset_types.length > 0 ? item.asset_types[0] : "Geral";
            if (!groups[category]) {
                groups[category] = [];
            }
            groups[category].push(item);
        }
        return groups;
    });

    function getStrategyStyle(category: string) {
        // Use standard asset colors if possible, or generic
        if (category === "Geral")
            return {
                icon: Target,
                color: "text-muted-foreground",
                bg: "bg-muted",
            };
        // Reuse Asset logic roughly or just deterministic hashing
        return { icon: Target, color: "text-primary", bg: "bg-primary/10" };
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">
                {$t("settings.strategies.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.strategies.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.strategies.new")}
        </Button>
    </div>
    <Separator />

    <div class="space-y-8">
        {#each Object.entries(groupedStrategies) as [category, items]}
            {@const style = getStrategyStyle(category)}
            {@const Icon = style.icon}

            <div class="space-y-4">
                <!-- Rich Header -->
                <div class="flex items-center gap-2">
                    <div class={`p-1.5 rounded ${style.bg}`}>
                        <Icon class={`w-4 h-4 ${style.color}`} />
                    </div>
                    <h4 class="text-lg font-semibold tracking-tight uppercase">
                        {category}
                    </h4>
                </div>

                <div class="flex flex-col gap-3">
                    {#each items as strategy}
                        <div
                            class="flex items-center justify-between p-4 rounded-lg border bg-card hover:border-primary/50 transition-all group shadow-sm cursor-pointer"
                            onclick={() => openEdit(strategy)}
                            onkeydown={(e) =>
                                e.key === "Enter" && openEdit(strategy)}
                            tabindex="0"
                            role="button"
                        >
                            <!-- Left: Icon + Main Info -->
                            <div class="flex items-center gap-4">
                                <div class="p-2.5 bg-muted rounded-xl shrink-0">
                                    <Target
                                        class="w-5 h-5 text-foreground/70"
                                    />
                                </div>
                                <div class="min-w-[200px]">
                                    <h4 class="font-bold text-base">
                                        {strategy.name}
                                    </h4>
                                    {#if strategy.description}
                                        <p
                                            class="text-xs text-muted-foreground line-clamp-1 max-w-md"
                                        >
                                            {strategy.description}
                                        </p>
                                    {/if}
                                </div>
                            </div>

                            <!-- Middle: Chips -->
                            <div class="flex gap-2 mr-auto ml-4 flex-wrap">
                                {#each strategy.timeframes.slice(0, 3) as tf}
                                    <Badge
                                        variant="secondary"
                                        class="text-[10px] h-5 px-1.5 font-mono"
                                        >{tf}</Badge
                                    >
                                {/each}
                                {#each strategy.asset_types.slice(0, 2) as at}
                                    <Badge
                                        variant="outline"
                                        class="text-[10px] h-5 px-1.5"
                                        >{at}</Badge
                                    >
                                {/each}
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
                                        openEdit(strategy);
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
                                        requestDelete(strategy.id);
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

        {#if settingsStore.strategies.length === 0}
            <div
                class="flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg border-muted-foreground/25 text-muted-foreground h-[150px]"
            >
                <Target class="w-8 h-8 mb-2 opacity-20" />
                <span class="text-sm">{$t("settings.strategies.empty")}</span>
            </div>
        {/if}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[900px] h-[90vh] flex flex-col p-0 gap-0">
        <Dialog.Header class="px-6 py-4 border-b">
            <Dialog.Title
                >{editingId
                    ? $t("settings.strategies.edit")
                    : $t("settings.strategies.new")}</Dialog.Title
            >
            <Dialog.Description
                >{$t("settings.strategies.description")}</Dialog.Description
            >
        </Dialog.Header>

        <div class="flex-1 overflow-y-auto px-6 py-4 grid gap-6">
            <!-- Section 1: General Info -->
            <div class="grid grid-cols-3 gap-6">
                <div class="space-y-2">
                    <Label class="text-primary font-semibold"
                        >{$t("settings.strategies.form.name")}*</Label
                    >
                    <Input
                        bind:value={formData.name}
                        placeholder={$t(
                            "settings.strategies.form.namePlaceholder",
                        )}
                        class="text-lg font-medium"
                    />
                </div>

                <div class="col-span-2 space-y-2">
                    <Label>{$t("settings.strategies.form.description")}</Label>
                    <Input
                        bind:value={formData.description}
                        placeholder={$t(
                            "settings.strategies.form.descriptionPlaceholder",
                        )}
                    />
                </div>
            </div>

            <!-- Section 2: Context (Tags) -->
            <div class="p-4 rounded-lg bg-muted/40 border space-y-3">
                <h4
                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-2"
                >
                    <Target class="w-3 h-3" />
                    {$t("settings.strategies.form.operationalContext")}
                </h4>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <!-- Timeframes -->
                    <div class="space-y-1.5">
                        <Label class="text-xs"
                            >{$t("settings.strategies.form.timeframes")}</Label
                        >
                        <div class="flex gap-2">
                            <div class="flex-1">
                                <Select.Root
                                    type="single"
                                    bind:value={tempTimeframe}
                                >
                                    <Select.Trigger class="h-8 text-xs w-full">
                                        {tempTimeframe ||
                                            $t(
                                                "settings.strategies.form.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each settingsStore.timeframes as tf}
                                            <Select.Item
                                                value={tf.name}
                                                label={tf.name}
                                                >{tf.name}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <Button
                                size="icon"
                                variant="secondary"
                                class="h-8 w-8 shrink-0"
                                onclick={() =>
                                    addTag("timeframes", tempTimeframe)}
                                ><Plus class="w-3 h-3" /></Button
                            >
                        </div>
                        <div class="flex flex-wrap gap-1 min-h-[24px]">
                            {#each formData.timeframes as item, i}
                                <span
                                    class="bg-primary/10 text-primary text-[10px] px-1.5 py-0.5 rounded flex items-center gap-1"
                                >
                                    {item}
                                    <button
                                        onclick={() =>
                                            removeTag("timeframes", i)}
                                        class="hover:text-destructive"
                                        ><X class="w-3 h-3" /></button
                                    >
                                </span>
                            {/each}
                        </div>
                    </div>

                    <!-- Indicators -->
                    <div class="space-y-1.5">
                        <Label class="text-xs"
                            >{$t("settings.strategies.form.indicators")}</Label
                        >
                        <div class="flex gap-2">
                            <div class="flex-1">
                                <Select.Root
                                    type="single"
                                    bind:value={tempIndicator}
                                >
                                    <Select.Trigger class="h-8 text-xs w-full">
                                        {tempIndicator || "Selecionar"}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each settingsStore.indicators as ind}
                                            <Select.Item
                                                value={ind.name}
                                                label={ind.name}
                                                >{ind.name}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <Button
                                size="icon"
                                variant="secondary"
                                class="h-8 w-8 shrink-0"
                                onclick={() =>
                                    addTag("indicators", tempIndicator)}
                                ><Plus class="w-3 h-3" /></Button
                            >
                        </div>
                        <div class="flex flex-wrap gap-1 min-h-[24px]">
                            {#each formData.indicators as item, i}
                                <span
                                    class="bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 text-[10px] px-1.5 py-0.5 rounded flex items-center gap-1"
                                >
                                    {item}
                                    <button
                                        onclick={() =>
                                            removeTag("indicators", i)}
                                        class="hover:text-destructive"
                                        ><X class="w-3 h-3" /></button
                                    >
                                </span>
                            {/each}
                        </div>
                    </div>

                    <!-- Asset Types -->
                    <div class="space-y-1.5">
                        <Label class="text-xs"
                            >{$t("settings.strategies.form.assetTypes")}</Label
                        >
                        <div class="flex gap-2">
                            <div class="flex-1">
                                <Select.Root
                                    type="single"
                                    bind:value={tempAssetType}
                                >
                                    <Select.Trigger class="h-8 text-xs w-full">
                                        {tempAssetType || "Selecionar"}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each settingsStore.assetTypes as type}
                                            <Select.Item
                                                value={type.code}
                                                label={type.code}
                                                >{type.code}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <Button
                                size="icon"
                                variant="secondary"
                                class="h-8 w-8 shrink-0"
                                onclick={() =>
                                    addTag("asset_types", tempAssetType)}
                                ><Plus class="w-3 h-3" /></Button
                            >
                        </div>
                        <div class="flex flex-wrap gap-1 min-h-[24px]">
                            {#each formData.asset_types as item, i}
                                <span
                                    class="bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 text-[10px] px-1.5 py-0.5 rounded flex items-center gap-1"
                                >
                                    {item}
                                    <button
                                        onclick={() =>
                                            removeTag("asset_types", i)}
                                        class="hover:text-destructive"
                                        ><X class="w-3 h-3" /></button
                                    >
                                </span>
                            {/each}
                        </div>
                    </div>
                    <!-- Specific Assets -->
                    <div class="space-y-1.5">
                        <Label class="text-xs"
                            >{$t(
                                "settings.strategies.form.specificAssets",
                            )}</Label
                        >
                        <div class="flex gap-2">
                            <div class="flex-1">
                                <Select.Root
                                    type="single"
                                    bind:value={tempSpecificAsset}
                                >
                                    <Select.Trigger class="h-8 text-xs w-full">
                                        {tempSpecificAsset || "Selecionar"}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each settingsStore.assets as asset}
                                            <Select.Item
                                                value={asset.symbol}
                                                label={asset.symbol}
                                                >{asset.symbol}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <Button
                                size="icon"
                                variant="secondary"
                                class="h-8 w-8 shrink-0"
                                onclick={() =>
                                    addTag(
                                        "specific_assets",
                                        tempSpecificAsset,
                                    )}><Plus class="w-3 h-3" /></Button
                            >
                        </div>
                        <div class="flex flex-wrap gap-1 min-h-[24px]">
                            {#each formData.specific_assets as item, i}
                                <span
                                    class="bg-green-500/10 text-green-600 dark:text-green-400 text-[10px] px-1.5 py-0.5 rounded flex items-center gap-1"
                                >
                                    {item}
                                    <button
                                        onclick={() =>
                                            removeTag("specific_assets", i)}
                                        class="hover:text-destructive"
                                        ><X class="w-3 h-3" /></button
                                    >
                                </span>
                            {/each}
                        </div>
                    </div>
                </div>
            </div>

            <!-- Section 2.5: Markets -->
            <div class="p-4 rounded-lg bg-muted/40 border space-y-3">
                <h4
                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-2"
                >
                    <Globe class="w-3 h-3" />
                    {$t("settings.strategies.form.linkedMarkets")}
                </h4>
                <div class="flex flex-wrap gap-2">
                    {#each settingsStore.markets as market}
                        {@const isSelected = formData.market_ids.includes(
                            market.id,
                        )}
                        <button
                            type="button"
                            class="px-3 py-1.5 rounded-lg text-sm font-medium transition-all border {isSelected
                                ? 'bg-primary text-primary-foreground border-primary'
                                : 'bg-muted/30 text-muted-foreground border-transparent hover:bg-muted/50'}"
                            onclick={() => {
                                if (isSelected) {
                                    formData.market_ids =
                                        formData.market_ids.filter(
                                            (id) => id !== market.id,
                                        );
                                } else {
                                    formData.market_ids = [
                                        ...formData.market_ids,
                                        market.id,
                                    ];
                                }
                            }}
                        >
                            {market.code}
                            {#if market.trading_sessions?.length}
                                <span class="text-[10px] ml-1 opacity-70">
                                    ({market.trading_sessions[0]
                                        .start_time}-{market.trading_sessions[0]
                                        .end_time})
                                </span>
                            {/if}
                        </button>
                    {/each}
                </div>
                {#if formData.market_ids.length === 0}
                    <p class="text-xs text-muted-foreground/60">
                        {$t("settings.strategies.form.noMarketSelected")}
                    </p>
                {/if}
            </div>

            <!-- Section 3: Rules -->
            <div class="grid grid-cols-3 gap-6 min-h-[200px]">
                <div class="space-y-2 flex flex-col">
                    <Label
                        class="text-green-600 dark:text-green-400 font-bold text-xs uppercase"
                        >{$t("settings.strategies.form.entryTriggers")}</Label
                    >
                    <Textarea
                        bind:value={formData.entry_criteria}
                        class="flex-1 resize-none bg-muted/30 focus:bg-background transition-colors"
                        placeholder={$t(
                            "settings.strategies.form.entryPlaceholder",
                        )}
                    />
                </div>
                <div class="space-y-2 flex flex-col">
                    <Label
                        class="text-red-600 dark:text-red-400 font-bold text-xs uppercase"
                        >{$t("settings.strategies.form.stopExit")}</Label
                    >
                    <Textarea
                        bind:value={formData.exit_criteria}
                        class="flex-1 resize-none bg-muted/30 focus:bg-background transition-colors"
                        placeholder={$t(
                            "settings.strategies.form.stopPlaceholder",
                        )}
                    />
                </div>
                <div class="space-y-2 flex flex-col">
                    <Label
                        class="text-blue-600 dark:text-blue-400 font-bold text-xs uppercase"
                        >{$t("settings.strategies.form.management")}</Label
                    >
                    <Textarea
                        bind:value={formData.management_criteria}
                        class="flex-1 resize-none bg-muted/30 focus:bg-background transition-colors"
                        placeholder={$t(
                            "settings.strategies.form.managementPlaceholder",
                        )}
                    />
                </div>
            </div>

            <!-- Section 4: Partials & Images -->
            <div class="grid grid-cols-2 gap-6">
                <!-- Partials -->
                <div class="space-y-4 p-4 border rounded-lg bg-muted/20">
                    <div class="flex items-center gap-2">
                        <Switch
                            id="partials"
                            bind:checked={formData.has_partial}
                        />
                        <Label for="partials"
                            >{$t(
                                "settings.strategies.form.partials.switch",
                            )}</Label
                        >
                    </div>
                    {#if formData.has_partial}
                        <Textarea
                            bind:value={formData.partial_description}
                            placeholder={$t(
                                "settings.strategies.form.partials.placeholder",
                            )}
                            class="h-24 resize-none"
                        />
                    {/if}
                </div>

                <!-- Images -->
                <div class="space-y-2">
                    <div class="flex justify-between items-center">
                        <Label
                            >{$t(
                                "settings.strategies.form.visualExamples",
                            )}</Label
                        >
                        <Input
                            type="file"
                            accept="image/*"
                            class="hidden"
                            id="img-upload"
                            onchange={handleImageUpload}
                        />
                        <Button
                            variant="outline"
                            size="sm"
                            class="text-xs h-7"
                            onclick={() =>
                                document.getElementById("img-upload")?.click()}
                        >
                            <UploadCloud class="w-3 h-3 mr-2" />
                            {$t("settings.strategies.form.addImage")}
                        </Button>
                    </div>

                    <div class="grid grid-cols-3 gap-2">
                        {#each formData.images as img, i}
                            <div
                                class="relative group aspect-video bg-black rounded overflow-hidden border"
                            >
                                <img
                                    src={img.path}
                                    alt="Preview"
                                    class="w-full h-full object-cover"
                                />
                                <button
                                    class="absolute top-1 right-1 p-1 bg-red-500 text-white rounded opacity-0 group-hover:opacity-100 transition-opacity"
                                    onclick={() => removeImage(i)}
                                >
                                    <X class="w-3 h-3" />
                                </button>
                                <div class="absolute bottom-0 w-full">
                                    <input
                                        bind:value={img.description}
                                        class="w-full text-[10px] bg-black/80 text-white border-0 px-1 py-0.5 focus:outline-none"
                                        placeholder="Descrição..."
                                    />
                                </div>
                            </div>
                        {/each}
                        {#if formData.images.length === 0}
                            <div
                                class="col-span-3 flex items-center justify-center h-24 border border-dashed rounded text-muted-foreground text-xs"
                            >
                                {$t("settings.strategies.form.tips.noImages")}
                            </div>
                        {/if}
                    </div>
                </div>
            </div>
        </div>

        <Dialog.Footer class="px-6 py-4 border-t mt-auto">
            <Button variant="outline" onclick={() => (isDialogOpen = false)}
                >{$t("settings.strategies.form.cancel")}</Button
            >
            <Button onclick={save}>{$t("settings.strategies.form.save")}</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
