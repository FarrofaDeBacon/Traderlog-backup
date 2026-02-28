<script lang="ts">
    import { Plus, Pencil, Trash2, ShieldAlert } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import {
        settingsStore,
        type RiskProfile,
    } from "$lib/stores/settings.svelte";
    import RiskForm from "$lib/components/settings/RiskForm.svelte";
    import { Badge } from "$lib/components/ui/badge";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";

    import GrowthPlanDashboard from "$lib/components/risk/GrowthPlanDashboard.svelte";

    import RiskProfileDetails from "$lib/components/settings/RiskProfileDetails.svelte";

    let isDialogOpen = $state(false);
    let editingItem = $state<RiskProfile | undefined>(undefined);

    // Details Modal State
    let isDetailsOpen = $state(false);
    let viewingItem = $state<RiskProfile | undefined>(undefined);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    function openNew() {
        editingItem = undefined;
        isDialogOpen = true;
    }

    function openEdit(item: RiskProfile) {
        editingItem = item;
        isDialogOpen = true;
    }

    function openDetails(item: RiskProfile) {
        viewingItem = item;
        isDetailsOpen = true;
    }

    function save(data: Omit<RiskProfile, "id">) {
        if (editingItem) {
            settingsStore.updateRiskProfile(editingItem.id, data);
        } else {
            settingsStore.addRiskProfile(data);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteRiskProfile(deleteId);
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
                {$t("settings.risk.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.risk.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.risk.new")}
        </Button>
    </div>
    <GrowthPlanDashboard />
    <Separator />

    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {#each settingsStore.riskProfiles as profile}
            <Card.Root
                class="border-l-4 {profile.active
                    ? 'border-l-green-500 shadow-lg'
                    : 'border-l-primary'} hover:border-primary/50 transition-all cursor-pointer hover:shadow-md"
                onclick={() => openDetails(profile)}
                role="button"
                tabindex={0}
                onkeydown={(e) => e.key === "Enter" && openDetails(profile)}
            >
                <Card.Header class="pb-2">
                    <div class="flex justify-between items-start">
                        <div class="flex items-center gap-2">
                            <div class="p-2 bg-primary/10 rounded-lg">
                                <ShieldAlert class="w-5 h-5 text-primary" />
                            </div>
                            <div>
                                <div class="flex items-center gap-2">
                                    <Card.Title class="text-base"
                                        >{profile.name}</Card.Title
                                    >
                                    {#if profile.active}
                                        <Badge
                                            class="bg-green-500/20 text-green-500 hover:bg-green-500/30 border-green-500/50 text-[10px] py-0 h-4"
                                        >
                                            {$t(
                                                "settings.risk.profiles.active",
                                            )}
                                        </Badge>
                                    {/if}
                                </div>
                                <Card.Description>
                                    {#if profile.account_type_applicability === "All"}
                                        {$t("settings.risk.accountTypes.All")}
                                    {:else}
                                        {$t(
                                            `settings.risk.accountTypes.${profile.account_type_applicability}`,
                                        )}
                                    {/if}
                                </Card.Description>
                            </div>
                        </div>
                    </div>
                </Card.Header>
                <Card.Content class="text-sm space-y-2 pb-2">
                    <div class="flex justify-between items-center text-red-400">
                        <span>{$t("settings.risk.profiles.dailyLoss")}:</span>
                        <span class="font-bold"
                            >R$ {profile.max_daily_loss}</span
                        >
                    </div>
                    <div
                        class="flex justify-between items-center text-green-400"
                    >
                        <span>{$t("settings.risk.profiles.dailyTarget")}:</span>
                        <span class="font-bold">R$ {profile.daily_target}</span>
                    </div>
                    <Separator class="my-2" />
                    <div
                        class="flex justify-between items-center text-muted-foreground text-xs"
                    >
                        <span>Trades: {profile.max_trades_per_day}</span>
                        <span
                            >{$t("settings.risk.maxRiskPerTradeShort") ||
                                "Risco/Trade"}: {profile.max_risk_per_trade_percent}%</span
                        >
                    </div>
                    {#if profile.lock_on_loss}
                        <div class="flex justify-center pt-1">
                            <Badge variant="destructive" class="text-[10px] h-5"
                                >{$t("settings.risk.lockOnLossShort") ||
                                    "Bloqueia Plataforma"}</Badge
                            >
                        </div>
                    {/if}
                </Card.Content>
                <Card.Footer class="justify-between items-center pt-2">
                    <div>
                        {#if !profile.active}
                            <Button
                                variant="outline"
                                size="sm"
                                class="h-8 text-xs px-3"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    settingsStore.setActiveRiskProfile(
                                        profile.id,
                                    );
                                }}
                            >
                                {$t("settings.risk.profiles.activate")}
                            </Button>
                        {/if}
                    </div>
                    <div class="flex gap-2">
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-8 w-8 hover:bg-muted"
                            title={$t("settings.risk.edit")}
                            onclick={(e) => {
                                e.stopPropagation();
                                openEdit(profile);
                            }}
                        >
                            <Pencil class="w-4 h-4" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-8 w-8 text-destructive hover:text-destructive hover:bg-destructive/10"
                            title={$t("general.delete")}
                            onclick={(e) => {
                                e.stopPropagation();
                                requestDelete(profile.id);
                            }}
                        >
                            <Trash2 class="w-4 h-4" />
                        </Button>
                    </div></Card.Footer
                >
            </Card.Root>
        {/each}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<!-- Edit/New Modal -->
<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="max-w-2xl max-h-[90vh] overflow-y-auto">
        <Dialog.Header>
            <Dialog.Title>
                {editingItem
                    ? $t("settings.risk.edit")
                    : $t("settings.risk.new")}
            </Dialog.Title>
        </Dialog.Header>

        <RiskForm
            initialData={editingItem}
            onSave={save}
            onCancel={() => (isDialogOpen = false)}
        />
    </Dialog.Content>
</Dialog.Root>

<!-- Details Modal -->
<Dialog.Root bind:open={isDetailsOpen}>
    <Dialog.Content class="max-w-2xl max-h-[90vh] overflow-y-auto">
        {#if viewingItem}
            <RiskProfileDetails profile={viewingItem} />
            <div class="flex justify-end gap-2">
                <Button
                    variant="outline"
                    onclick={() => (isDetailsOpen = false)}
                    >{$t("general.back")}</Button
                >
                <Button
                    onclick={() => {
                        openEdit(viewingItem!);
                        isDetailsOpen = false;
                    }}
                >
                    <Pencil class="w-4 h-4 mr-2" />
                    {$t("settings.risk.edit")}
                </Button>
            </div>
        {/if}
    </Dialog.Content>
</Dialog.Root>
