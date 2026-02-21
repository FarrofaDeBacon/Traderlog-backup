<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Tag as TagIcon,
        Search,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import { settingsStore, type Tag } from "$lib/stores/settings.svelte";
    import TagForm from "$lib/components/settings/TagForm.svelte";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingItem = $state<Tag | undefined>(undefined);
    let searchTerm = $state("");

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let filteredItems = $derived(
        settingsStore.tags
            .filter((item) =>
                item.name.toLowerCase().includes(searchTerm.toLowerCase()),
            )
            .sort((a, b) => a.name.localeCompare(b.name)),
    );

    function openNew() {
        editingItem = undefined;
        isDialogOpen = true;
    }

    function openEdit(item: Tag) {
        editingItem = item;
        isDialogOpen = true;
    }

    function save(data: Omit<Tag, "id">) {
        if (editingItem) {
            settingsStore.updateTag(editingItem.id, data);
        } else {
            settingsStore.addTag(data);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteTag(deleteId);
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
            <h3 class="text-2xl font-bold tracking-tight">
                {$t("settings.tags.title")}
            </h3>
            <p class="text-muted-foreground">
                {$t("settings.tags.description")}
            </p>
        </div>
    </div>

    <div class="flex items-center justify-between gap-4">
        <div class="relative flex-1 max-w-md">
            <Search
                class="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground"
            />
            <Input
                placeholder={$t("settings.tags.searchPlaceholder")}
                bind:value={searchTerm}
                class="pl-8"
            />
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.tags.new")}
        </Button>
    </div>

    <div class="grid gap-2">
        {#if filteredItems.length > 0}
            {#each filteredItems as item}
                <div
                    class="flex items-center justify-between p-4 rounded-lg border bg-card hover:border-primary/50 transition-all group shadow-sm cursor-pointer"
                    onclick={() => openEdit(item)}
                    onkeydown={(e) => e.key === "Enter" && openEdit(item)}
                    tabindex="0"
                    role="button"
                >
                    <div class="flex items-center gap-4">
                        <!-- Color Circle acting as Icon -->
                        <div
                            class="w-9 h-9 rounded-full border-2 border-background shadow-sm flex items-center justify-center shrink-0"
                            style="background-color: {item.color}; opacity: 0.9;"
                        >
                            <TagIcon
                                class="w-4 h-4 text-white drop-shadow-md"
                            />
                        </div>
                        <div>
                            <h4 class="font-medium">{item.name}</h4>
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
                <span class="text-sm">{$t("settings.tags.empty")}</span>
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
                    ? $t("settings.tags.edit")
                    : $t("settings.tags.new")}</Dialog.Title
            >
        </Dialog.Header>
        <TagForm
            initialData={editingItem}
            onSave={save}
            onCancel={() => (isDialogOpen = false)}
        />
    </Dialog.Content>
</Dialog.Root>
