<script lang="ts">
    import { Plus, Pencil, Trash2, Workflow, Search } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Textarea } from "$lib/components/ui/textarea";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import { settingsStore, type Modality } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);
    let formModality = $state<Omit<Modality, "id">>({
        name: "",
        description: "",
    });

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let filteredItems = $derived(
        [...settingsStore.modalities].sort((a, b) =>
            a.name.localeCompare(b.name),
        ),
    );

    function openNew() {
        editingId = null;
        formModality = { name: "", description: "" };
        isDialogOpen = true;
    }

    function openEdit(item: Modality) {
        editingId = item.id;
        formModality = { ...item };
        isDialogOpen = true;
    }

    function save() {
        if (editingId) {
            settingsStore.updateModality(editingId, formModality);
        } else {
            settingsStore.addModality(formModality);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteModality(deleteId);
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
            {$t("settings.modalities.new")}
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
                        <div class="p-2 bg-primary/10 rounded-lg shrink-0">
                            <Workflow class="w-5 h-5 text-primary" />
                        </div>
                        <div>
                            <h4 class="font-medium">{item.name}</h4>
                            {#if item.description}
                                <p
                                    class="text-sm text-muted-foreground line-clamp-1"
                                >
                                    {item.description}
                                </p>
                            {/if}
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
                <span class="text-sm">{$t("settings.modalities.empty")}</span>
            </div>
        {/if}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title>
                {editingId
                    ? $t("settings.modalities.edit")
                    : $t("settings.modalities.new")}
            </Dialog.Title>
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.modalities.form.name")}</Label
                >
                <Input
                    bind:value={formModality.name}
                    class="col-span-3"
                    placeholder={$t("settings.modalities.form.namePlaceholder")}
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.modalities.form.description")}</Label
                >
                <Textarea
                    bind:value={formModality.description}
                    class="col-span-3"
                    placeholder={$t(
                        "settings.modalities.form.descriptionPlaceholder",
                    )}
                />
            </div>
        </div>

        <Dialog.Footer>
            <Button onclick={save}>{$t("settings.modalities.form.save")}</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
