<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        BarChart3,
        Search,
        Clock,
        LayoutGrid,
        Layers,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import { settingsStore, type ChartType } from "$lib/stores/settings.svelte";
    import ChartTypeForm from "$lib/components/settings/ChartTypeForm.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingItem = $state<ChartType | undefined>(undefined);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let filteredItems = $derived(
        [...settingsStore.chartTypes].sort((a, b) =>
            a.name.localeCompare(b.name),
        ),
    );

    // Group indicators by category
    let groupedChartTypes = $derived.by(() => {
        const groups: Record<string, ChartType[]> = {};
        for (const item of filteredItems) {
            // Group by base_type or default "Geral"
            const key = item.base_type || "Outros";
            if (!groups[key]) {
                groups[key] = [];
            }
            groups[key].push(item);
        }
        return groups;
    });

    function getChartTypeStyle(baseType: string) {
        const t = baseType.toLowerCase();
        if (
            t.includes("time") ||
            t.includes("temporal") ||
            t.includes("min") ||
            t.includes("hora")
        )
            return {
                icon: Clock,
                color: "text-blue-500",
                bg: "bg-blue-500/10",
            };
        if (t.includes("renko") || t.includes("range") || t.includes("pontos"))
            return {
                icon: LayoutGrid,
                color: "text-orange-500",
                bg: "bg-orange-500/10",
            };

        return {
            icon: BarChart3,
            color: "text-muted-foreground",
            bg: "bg-muted",
        };
    }

    function openNew() {
        editingItem = undefined;
        isDialogOpen = true;
    }

    function openEdit(item: ChartType) {
        editingItem = item;
        isDialogOpen = true;
    }

    function save(data: Omit<ChartType, "id">) {
        if (editingItem) {
            settingsStore.updateChartType(editingItem.id, data);
        } else {
            settingsStore.addChartType(data);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteChartType(deleteId);
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
                {$t("settings.chartTypes.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.chartTypes.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.chartTypes.new")}
        </Button>
    </div>
    <Separator />

    <div class="space-y-8">
        {#each Object.entries(groupedChartTypes) as [baseType, items]}
            {@const style = getChartTypeStyle(baseType)}
            {@const Icon = style.icon}

            <div class="space-y-4">
                <!-- Rich Header -->
                <div class="flex items-center gap-2">
                    <div class={`p-1.5 rounded ${style.bg}`}>
                        <Icon class={`w-4 h-4 ${style.color}`} />
                    </div>
                    <h4 class="text-lg font-semibold tracking-tight uppercase">
                        {baseType}
                    </h4>
                </div>

                <div class="flex flex-col gap-3">
                    {#each items as item}
                        <div
                            class="flex items-center justify-between p-4 rounded-lg border bg-card hover:border-primary/50 transition-all group shadow-sm cursor-pointer"
                            onclick={() => openEdit(item)}
                            onkeydown={(e) =>
                                e.key === "Enter" && openEdit(item)}
                            tabindex="0"
                            role="button"
                        >
                            <div class="flex items-center gap-4">
                                <div class="p-2.5 bg-muted rounded-xl">
                                    <BarChart3
                                        class="w-5 h-5 text-foreground/70"
                                    />
                                </div>
                                <div>
                                    <h4 class="font-bold text-base">
                                        {item.name}
                                    </h4>
                                    <p class="text-sm text-muted-foreground">
                                        {item.base_type}
                                        {item.parameter
                                            ? `(${item.parameter})`
                                            : ""}
                                    </p>
                                </div>
                            </div>

                            <div
                                class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity"
                            >
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onclick={(e) => {
                                        e.stopPropagation();
                                        openEdit(item);
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
                <Search class="w-8 h-8 mb-2 opacity-20" />
                <span class="text-sm">{$t("settings.chartTypes.empty")}</span>
            </div>
        {/each}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[500px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingItem
                    ? $t("settings.chartTypes.edit")
                    : $t("settings.chartTypes.new")}</Dialog.Title
            >
        </Dialog.Header>
        <ChartTypeForm
            initialData={editingItem}
            onSave={save}
            onCancel={() => (isDialogOpen = false)}
        />
    </Dialog.Content>
</Dialog.Root>
