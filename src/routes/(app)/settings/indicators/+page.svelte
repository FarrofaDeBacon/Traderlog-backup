<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Activity,
        TrendingUp,
        BarChart3,
        Layers,
        LineChart,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import { settingsStore, type Indicator } from "$lib/stores/settings.svelte";
    import IndicatorForm from "$lib/components/settings/IndicatorForm.svelte";
    import { Badge } from "$lib/components/ui/badge";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingItem = $state<Indicator | undefined>(undefined);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    function getCategoryStyle(category: string) {
        const c = category.toUpperCase();
        if (c.includes("OSCILLATOR") || c.includes("OSCILADOR"))
            return {
                icon: Activity,
                color: "text-purple-500",
                bg: "bg-purple-500/10",
            };
        if (c.includes("TREND") || c.includes("TENDENCIA"))
            return {
                icon: TrendingUp,
                color: "text-green-500",
                bg: "bg-green-500/10",
            };
        if (c.includes("VOLUME"))
            return {
                icon: BarChart3,
                color: "text-blue-500",
                bg: "bg-blue-500/10",
            };
        if (c.includes("AVERAGE") || c.includes("MEDIA"))
            return {
                icon: LineChart,
                color: "text-amber-500",
                bg: "bg-amber-500/10",
            };

        return { icon: Layers, color: "text-muted-foreground", bg: "bg-muted" };
    }

    // Sort indicators
    let filteredIndicators = $derived(
        [...settingsStore.indicators].sort((a, b) =>
            a.name.localeCompare(b.name),
        ),
    );

    // Group indicators by category
    let groupedIndicators = $derived.by(() => {
        const groups: Record<string, Indicator[]> = {};
        for (const item of filteredIndicators) {
            const category = item.category ?? "Outros";
            if (!groups[category]) {
                groups[category] = [];
            }
            groups[category].push(item);
        }
        // Sort keys to ensure consistent order (e.g. Trend, Oscillator...)
        return Object.keys(groups)
            .sort()
            .reduce(
                (acc, key) => {
                    acc[key] = groups[key];
                    return acc;
                },
                {} as Record<string, Indicator[]>,
            );
    });

    function openNew() {
        editingItem = undefined;
        isDialogOpen = true;
    }

    function openEdit(item: Indicator) {
        editingItem = item;
        isDialogOpen = true;
    }

    function save(data: Omit<Indicator, "id">) {
        if (editingItem) {
            settingsStore.updateIndicator(editingItem.id, data);
        } else {
            settingsStore.addIndicator(data);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteIndicator(deleteId);
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
                {$t("settings.indicators.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.indicators.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.indicators.new")}
        </Button>
    </div>
    <Separator />

    <div class="space-y-8">
        {#each Object.entries(groupedIndicators) as [category, items]}
            {@const style = getCategoryStyle(category)}
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

                <div class="grid gap-2">
                    {#each items as item}
                        <div
                            class="flex items-center justify-between p-3 rounded-lg border bg-card text-card-foreground shadow-sm hover:bg-accent hover:border-primary/30 transition-all cursor-pointer group"
                            onclick={() => openEdit(item)}
                            role="button"
                            tabindex="0"
                            onkeydown={(e) =>
                                e.key === "Enter" && openEdit(item)}
                        >
                            <div class="flex items-center gap-4">
                                <div
                                    class="w-10 h-10 rounded-lg flex items-center justify-center bg-muted/50 group-hover:bg-primary/10 transition-colors"
                                >
                                    <Activity
                                        class="w-5 h-5 transition-colors"
                                        style="color: {item.default_color}"
                                    />
                                </div>
                                <div>
                                    <div class="flex items-center gap-2">
                                        <span class="font-bold"
                                            >{item.name}</span
                                        >
                                        <Badge
                                            variant="outline"
                                            class="text-[10px] h-5 px-1.5 font-normal"
                                        >
                                            {item.plot_type}
                                        </Badge>
                                    </div>
                                    <div class="flex gap-1 mt-1">
                                        {#each item.parameters as param}
                                            <span
                                                class="text-[10px] text-muted-foreground bg-secondary/50 px-1.5 rounded"
                                            >
                                                {param.key}: {param.value}
                                            </span>
                                        {/each}
                                    </div>
                                </div>
                            </div>

                            <div
                                class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
                            >
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="h-8 w-8 text-destructive hover:text-destructive hover:bg-destructive/10"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        requestDelete(item.id);
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
                <span class="text-sm">{$t("settings.indicators.empty")}</span>
            </div>
        {/each}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[600px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingItem
                    ? $t("settings.indicators.edit")
                    : $t("settings.indicators.new")}</Dialog.Title
            >
        </Dialog.Header>
        <IndicatorForm
            initialData={editingItem}
            onSave={save}
            onCancel={() => (isDialogOpen = false)}
        />
    </Dialog.Content>
</Dialog.Root>
