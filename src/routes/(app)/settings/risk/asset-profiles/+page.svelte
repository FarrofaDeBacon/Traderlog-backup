<script lang="ts">
    import { Plus, Pencil, Trash2, Search, Layers, Briefcase, FileText } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import { settingsStore, type AssetRiskProfile } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";
    import { Textarea } from "$lib/components/ui/textarea";

    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let formData = $state<Omit<AssetRiskProfile, "id">>({
        name: "",
        asset_id: "",
        default_stop_points: 0,
        min_contracts: 1,
        max_contracts: 1,
        notes: "",
    });

    let searchQuery = $state("");

    // Sort and filter profiles
    let filteredProfiles = $derived(
        [...settingsStore.assetRiskProfiles]
            .filter((p) => {
                const search = searchQuery.toLowerCase();
                const assetSymbol = settingsStore.assets.find(a => a.id === p.asset_id)?.symbol.toLowerCase() || "";
                return (
                    p.name.toLowerCase().includes(search) ||
                    assetSymbol.includes(search)
                );
            })
            .sort((a, b) => a.name.localeCompare(b.name))
    );

    // Get only root assets for the dropdown
    let rootAssets = $derived(
        settingsStore.assets.filter((a) => a.is_root).sort((a, b) => a.symbol.localeCompare(b.symbol))
    );

    function getAssetSymbol(assetId: string) {
        const asset = settingsStore.assets.find(a => a.id === assetId);
        return asset ? asset.symbol : $t("general.unknown");
    }

    function openNew() {
        editingId = null;
        formData = {
            name: "",
            asset_id: "",
            default_stop_points: 0,
            min_contracts: 1,
            max_contracts: 1,
            notes: "",
        };
        isDialogOpen = true;
    }

    function openEdit(item: AssetRiskProfile) {
        editingId = item.id || null;
        formData = { ...item };
        isDialogOpen = true;
    }

    function confirmDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    // Ações assíncronas
    async function handleDeleteConfirm() {
        if (!deleteId) return;
        const res = await settingsStore.deleteAssetRiskProfile(deleteId);
        if (res.success) {
            toast.success($t("general.saveSuccess") || "Perfil excluído com sucesso.");
        } else {
            toast.error(res.error || $t("general.error"));
        }
    }

    function save() {
        // Validações
        if (!formData.name.trim()) {
            toast.error($t("general.form.nameRequired") || "Nome é obrigatório");
            return;
        }
        if (!formData.asset_id) {
            toast.error($t("general.form.assetRequired") || "Ativo é obrigatório");
            return;
        }
        if (formData.default_stop_points < 0) {
            toast.error($t("general.form.invalidPoints") || "Stop Padrão não pode ser negativo");
            return;
        }
        if (formData.min_contracts > formData.max_contracts) {
            toast.error($t("general.form.invaildContracts") || "Mínimo de contratos não pode ser maior que o máximo");
            return;
        }

        if (editingId) {
            settingsStore.updateAssetRiskProfile(editingId, formData);
        } else {
            settingsStore.addAssetRiskProfile(formData);
        }

        isDialogOpen = false;
        toast.success($t("general.saveSuccess") || "Perfil salvo com sucesso.");
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div>
            <h2 class="text-2xl font-bold tracking-tight">
                {$t("settings.risk.assetProfiles.title") || "Perfis de Risco do Ativo"}
            </h2>
            <p class="text-muted-foreground">
                {$t("settings.risk.assetProfiles.description") || "Associe permissões, bloqueios e stop padrão para um ativo específico da plataforma."}
            </p>
        </div>
        <Button onclick={openNew} class="gap-2">
            <Plus class="w-4 h-4" />
            {$t("settings.risk.assetProfiles.new") || "Novo Perfil do Ativo"}
        </Button>
    </div>

    <!-- Barra de busca -->
    <div class="flex items-center gap-2 max-w-sm">
        <div class="relative flex-1">
            <Search
                class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground"
            />
            <Input
                type="search"
                placeholder={$t("settings.risk.assetProfiles.search") || "Buscar perfil..."}
                class="pl-8"
                bind:value={searchQuery}
            />
        </div>
    </div>

    <!-- Lista -->
    <div class="grid gap-4">
        {#if filteredProfiles.length === 0}
            <div
                class="flex flex-col items-center justify-center p-8 text-center border rounded-lg border-dashed bg-muted/20"
            >
                <Layers class="w-8 h-8 text-muted-foreground mb-4 opacity-50" />
                <p class="text-muted-foreground">
                    {$t("settings.risk.assetProfiles.empty") || "Nenhum Perfil de Risco de Ativo encontrado"}
                </p>
            </div>
        {:else}
            {#each filteredProfiles as profile}
                <div
                    class="flex items-center justify-between p-4 border rounded-lg hover:border-border/80 transition-colors"
                >
                    <div class="flex items-center gap-4">
                        <div
                            class="w-10 h-10 rounded-full flex items-center justify-center bg-blue-500/10 text-blue-500"
                        >
                            <Briefcase class="w-5 h-5" />
                        </div>
                        <div>
                            <p class="font-medium text-sm text-foreground">
                                {profile.name}
                            </p>
                            <p class="text-xs text-muted-foreground flex gap-2">
                                <span>{$t("settings.risk.assetProfiles.asset") || "Ativo"}: <strong>{getAssetSymbol(profile.asset_id)}</strong></span>
                                <span class="text-muted-foreground/30">•</span>
                                <span>{$t("settings.risk.assetProfiles.defaultStop") || "Stop Padrão"}: {profile.default_stop_points}</span>
                                <span class="text-muted-foreground/30">•</span>
                                <span>Mín/Máx Lotes: {profile.min_contracts} - {profile.max_contracts}</span>
                            </p>
                            {#if profile.notes}
                                <p class="text-xs text-muted-foreground/80 mt-1 italic flex items-center gap-1">
                                    <FileText class="w-3 h-3" /> {profile.notes}
                                </p>
                            {/if}
                        </div>
                    </div>
                    <div class="flex gap-2">
                        <Button
                            variant="ghost"
                            size="icon"
                            class="text-muted-foreground"
                            onclick={() => openEdit(profile)}
                        >
                            <Pencil class="w-4 h-4" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="text-destructive hover:bg-destructive/10"
                            onclick={() => confirmDelete(profile.id!)}
                        >
                            <Trash2 class="w-4 h-4" />
                        </Button>
                    </div>
                </div>
            {/each}
        {/if}
    </div>
</div>

<DeleteConfirmationModal
    bind:open={isDeleteOpen}
    onConfirm={handleDeleteConfirm}
    title={$t("settings.risk.assetProfiles.delete") || "Excluir Perfil"}
    description={$t("general.confirmDeleteDescription")}
/>

<!-- Modal Create/Edit -->
<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title>
                {editingId
                    ? $t("settings.risk.assetProfiles.edit") || "Editar Perfil"
                    : $t("settings.risk.assetProfiles.new") || "Novo Perfil do Ativo"}
            </Dialog.Title>
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">
                    {$t("settings.risk.assetProfiles.name") || "Nome"} <span class="text-destructive">*</span>
                </Label>
                <Input
                    bind:value={formData.name}
                    class="col-span-3"
                    placeholder="Ex: WDO Scalper, Indice Tendência"
                />
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">
                    {$t("settings.risk.assetProfiles.asset") || "Ativo Vinculado"} <span class="text-destructive">*</span>
                </Label>
                <div class="col-span-3">
                    <Select.Root
                        type="single"
                        bind:value={formData.asset_id}
                    >
                        <Select.Trigger class="w-full">
                            {getAssetSymbol(formData.asset_id) || "Selecione o ativo"}
                        </Select.Trigger>
                        <Select.Content>
                            {#each rootAssets as asset}
                                <Select.Item value={asset.id}>{asset.symbol} - {asset.name}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">
                    {$t("settings.risk.assetProfiles.defaultStop") || "Stop Padrão"}
                </Label>
                <Input
                    type="number"
                    step="0.01"
                    min="0"
                    bind:value={formData.default_stop_points}
                    class="col-span-3"
                />
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">
                    {$t("settings.risk.assetProfiles.minContracts") || "Contratos Mín."}
                </Label>
                <Input
                    type="number"
                    step="1"
                    min="1"
                    bind:value={formData.min_contracts}
                    class="col-span-3"
                />
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right">
                    {$t("settings.risk.assetProfiles.maxContracts") || "Contratos Máx."}
                </Label>
                <Input
                    type="number"
                    step="1"
                    min="1"
                    bind:value={formData.max_contracts}
                    class="col-span-3"
                />
            </div>

            <div class="grid grid-cols-4 items-start gap-4">
                <Label class="text-right mt-3">
                    {$t("settings.risk.assetProfiles.notes") || "Notas"}
                </Label>
                <Textarea
                    bind:value={formData.notes}
                    class="col-span-3"
                    placeholder="Observações opcionais"
                    rows={3}
                />
            </div>
        </div>

        <Dialog.Footer>
            <Button onclick={save}>{$t("general.save")}</Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
