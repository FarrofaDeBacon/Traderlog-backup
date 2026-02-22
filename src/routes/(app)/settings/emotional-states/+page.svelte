<script lang="ts">
    import { Plus, Pencil, Trash2, HeartPulse } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { Button } from "$lib/components/ui/button";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import {
        settingsStore,
        type EmotionalState,
    } from "$lib/stores/settings.svelte";
    import EmotionalStateForm from "$lib/components/settings/EmotionalStateForm.svelte";
    import { Badge } from "$lib/components/ui/badge";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingItem = $state<EmotionalState | undefined>(undefined);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    // Sort
    let filteredStates = $derived(
        [...settingsStore.emotionalStates].sort((a, b) =>
            a.name.localeCompare(b.name),
        ),
    );

    // Group by Impact
    let groupedStates = $derived.by(() => {
        const groups: Record<string, EmotionalState[]> = {
            Positive: [],
            Negative: [],
            Neutral: [],
        };

        for (const item of filteredStates) {
            if (groups[item.impact]) {
                groups[item.impact].push(item);
            } else {
                if (!groups["Neutral"]) groups["Neutral"] = []; // Fallback
                groups["Neutral"].push(item);
            }
        }
        return groups;
    });

    function getImpactLabel(impact: string) {
        return `settings.emotionalStates.form.impactOptions.${impact}`;
    }

    function getImpactColor(impact: string) {
        switch (impact) {
            case "Positive":
                return "text-green-500 bg-green-500/10 border-green-500/20";
            case "Negative":
                return "text-red-500 bg-red-500/10 border-red-500/20";
            default:
                return "text-muted-foreground bg-secondary border-secondary";
        }
    }

    function openNew() {
        editingItem = undefined;
        isDialogOpen = true;
    }

    function openEdit(item: EmotionalState) {
        editingItem = item;
        isDialogOpen = true;
    }

    function save(data: Omit<EmotionalState, "id">) {
        if (editingItem) {
            settingsStore.updateEmotionalState(editingItem.id, data);
        } else {
            settingsStore.addEmotionalState(data);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteEmotionalState(deleteId);
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
                {$t("settings.emotionalStates.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.emotionalStates.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.emotionalStates.new")}
        </Button>
    </div>
    <Separator />

    <div class="space-y-6">
        {#each Object.entries(groupedStates) as [impact, items]}
            {#if items.length > 0}
                <div class="space-y-2">
                    <div
                        class="flex items-center gap-2 text-sm font-semibold text-muted-foreground uppercase tracking-wider pl-1"
                    >
                        <span
                            class="w-2 h-2 rounded-full {impact === 'Positive'
                                ? 'bg-green-500'
                                : impact === 'Negative'
                                  ? 'bg-red-500'
                                  : 'bg-gray-500'}"
                        ></span>
                        {$t(getImpactLabel(impact))}
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
                                        class="p-2 bg-primary/10 rounded-lg group-hover:bg-primary/20 transition-colors"
                                    >
                                        <HeartPulse
                                            class="w-5 h-5 text-primary"
                                        />
                                    </div>
                                    <div>
                                        <div class="flex items-center gap-2">
                                            <span class="font-bold"
                                                >{item.name}</span
                                            >
                                            <Badge
                                                class="text-[10px] h-5 px-1.5 font-normal {getImpactColor(
                                                    item.impact,
                                                )}"
                                                variant="outline"
                                            >
                                                {$t(
                                                    getImpactLabel(item.impact),
                                                )}
                                            </Badge>
                                        </div>
                                        <p
                                            class="text-xs text-muted-foreground line-clamp-1"
                                        >
                                            {item.description}
                                        </p>
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
            {/if}
        {/each}
        {#if filteredStates.length === 0}
            <div
                class="flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg border-muted-foreground/25 text-muted-foreground h-[150px]"
            >
                <span class="text-sm">Nenhum estado emocional encontrado</span>
            </div>
        {/if}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[500px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingItem
                    ? $t("settings.emotionalStates.edit")
                    : $t("settings.emotionalStates.new")}</Dialog.Title
            >
        </Dialog.Header>
        <EmotionalStateForm
            initialData={editingItem}
            onSave={save}
            onCancel={() => (isDialogOpen = false)}
        />
    </Dialog.Content>
</Dialog.Root>
