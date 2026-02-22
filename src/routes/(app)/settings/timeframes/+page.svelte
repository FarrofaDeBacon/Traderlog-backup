<script lang="ts">
    import { Plus, Pencil, Trash2, Clock, Search } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import { settingsStore, type Timeframe } from "$lib/stores/settings.svelte";
    import TimeframeForm from "$lib/components/settings/TimeframeForm.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingItem = $state<Timeframe | undefined>(undefined);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let filteredItems = $derived(
        [...settingsStore.timeframes].sort((a, b) =>
            a.name.localeCompare(b.name),
        ), // Basic sort
    );

    function openNew() {
        editingItem = undefined;
        isDialogOpen = true;
    }

    function openEdit(item: Timeframe) {
        editingItem = item;
        isDialogOpen = true;
    }

    function save(data: Omit<Timeframe, "id">) {
        if (editingItem) {
            settingsStore.updateTimeframe(editingItem.id, data);
        } else {
            settingsStore.addTimeframe(data);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteTimeframe(deleteId);
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
    <div class="flex items-center justify-between gap-4">
        <div></div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.timeframes.new")}
        </Button>
    </div>

    <div class="grid gap-2">
        {#if filteredItems.length > 0}
            {#each filteredItems as item}
                <div
                    class="flex items-center justify-between p-4 rounded-lg border bg-card hover:border-primary/50 transition-all group shadow-sm cursor-pointer"
                    onclick={() => openEdit(item)}
                    onkeydown={(e) => e.key === "Enter" && openEdit(item)}
                    tabindex={0}
                    role="button"
                >
                    <div class="flex items-center gap-4">
                        <div class="p-2 bg-primary/10 rounded-lg">
                            <Clock class="w-5 h-5 text-primary" />
                        </div>
                        <div>
                            <h4 class="font-medium">{item.name}</h4>
                            <p class="text-sm text-muted-foreground">
                                {item.value}
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
                            class="text-destructive hover:text-destructive"
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
        {:else}
            <div
                class="flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg border-muted-foreground/25 text-muted-foreground h-[150px]"
            >
                <Search class="w-8 h-8 mb-2 opacity-20" />
                <span class="text-sm">{$t("settings.timeframes.empty")}</span>
            </div>
        {/if}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingItem
                    ? $t("settings.timeframes.edit")
                    : $t("settings.timeframes.new")}</Dialog.Title
            >
        </Dialog.Header>
        <TimeframeForm
            initialData={editingItem}
            onSave={save}
            onCancel={() => (isDialogOpen = false)}
        />
    </Dialog.Content>
</Dialog.Root>
