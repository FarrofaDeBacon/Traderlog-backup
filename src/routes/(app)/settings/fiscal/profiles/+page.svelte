<script lang="ts">
    import { Plus, Pencil, Trash2, ArrowRightLeft } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Separator } from "$lib/components/ui/separator";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";
    import ProfileEditor from "../ProfileEditor.svelte";

    // Profile Editor State
    let isProfileEditorOpen = $state(false);
    let editingProfileId = $state<string | null>(null);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    function openNewProfile() {
        editingProfileId = null;
        isProfileEditorOpen = true;
    }

    function openEditProfile(id: string) {
        editingProfileId = id;
        isProfileEditorOpen = true;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteTaxProfile(deleteId);

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
                {$t("settings.fiscal.profiles.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.fiscal.profiles.description")}
            </p>
        </div>
        <Button onclick={openNewProfile}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.fiscal.profiles.new")}
        </Button>
    </div>

    <div class="grid gap-4 md:grid-cols-2">
        {#each settingsStore.taxProfiles as profile}
            <div
                class="flex flex-col p-5 rounded-lg border bg-card hover:border-primary/50 transition-all shadow-sm"
            >
                <div class="flex justify-between items-start mb-4">
                    <div>
                        <h4 class="font-bold text-lg">{profile.name}</h4>
                        {#if profile.description}
                            <p class="text-sm text-muted-foreground">
                                {profile.description}
                            </p>
                        {/if}
                    </div>
                    <div class="flex gap-1">
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-8 w-8"
                            onclick={() => openEditProfile(profile.id)}
                        >
                            <Pencil class="w-4 h-4" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-8 w-8 text-destructive"
                            onclick={() => requestDelete(profile.id)}
                        >
                            <Trash2 class="w-4 h-4" />
                        </Button>
                    </div>
                </div>

                <Separator class="my-2" />

                <!-- Linked Rules Summary -->
                <div class="space-y-2 mt-2">
                    <h5
                        class="text-xs font-semibold uppercase text-muted-foreground mb-2"
                    >
                        {$t("settings.fiscal.profiles.linkedRules")}
                    </h5>
                    {#each settingsStore.getEntriesForProfile(profile.id) as entry}
                        {#if settingsStore.taxRules.find((r) => r.id === entry.tax_rule_id)}
                            <div
                                class="flex justify-between items-center text-sm p-2 bg-muted/50 rounded border"
                            >
                                <div class="flex items-center gap-2">
                                    <span
                                        class="w-2 h-2 rounded-full bg-blue-500"
                                    ></span>
                                    <span
                                        >{settingsStore.modalities.find(
                                            (m) => m.id === entry.modality_id,
                                        )?.name || "M?"}</span
                                    >
                                </div>
                                <div class="font-medium">
                                    {settingsStore.taxRules.find(
                                        (r) => r.id === entry.tax_rule_id,
                                    )?.name}
                                </div>
                            </div>
                        {/if}
                    {:else}
                        <div class="text-xs text-muted-foreground italic p-2">
                            {$t("settings.fiscal.profiles.noLinkedRules")}
                        </div>
                    {/each}
                </div>
            </div>
        {:else}
            <div
                class="col-span-full flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg text-muted-foreground h-[200px]"
            >
                <ArrowRightLeft class="w-8 h-8 mb-2 opacity-20" />
                <span>{$t("settings.fiscal.profiles.empty")}</span>
                <Button variant="link" onclick={openNewProfile}
                    >{$t("settings.fiscal.profiles.create")}</Button
                >
            </div>
        {/each}
    </div>
</div>

<ProfileEditor
    bind:open={isProfileEditorOpen}
    bind:profileId={editingProfileId}
/>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />
