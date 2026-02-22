<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Coins,
        Search,
        ArrowRightLeft,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import { settingsStore, type Currency } from "$lib/stores/settings.svelte";

    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";
    import { cn } from "$lib/utils";

    let isDialogOpen = $state(false);
    let isSyncing = $state(false);
    let editingId = $state<string | null>(null);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let filteredItems = $derived(
        [...settingsStore.currencies].sort((a, b) =>
            a.code.localeCompare(b.code),
        ),
    );

    let formData = $state<Omit<Currency, "id">>({
        code: "",
        symbol: "",
        name: "",
        exchange_rate: 1.0,
    });

    function resetForm() {
        formData = { code: "", symbol: "", name: "", exchange_rate: 1.0 };
        editingId = null;
    }

    function openNew() {
        resetForm();
        isDialogOpen = true;
    }

    function openEdit(currency: Currency) {
        editingId = currency.id;
        formData = { ...currency };
        isDialogOpen = true;
    }

    function saveCurrency() {
        if (editingId) {
            settingsStore.updateCurrency(editingId, formData);
        } else {
            settingsStore.addCurrency(formData);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteCurrency(deleteId);
            if (!result.success) {
                toast.error(result.error || $t("general.error"));
            } else {
                toast.success($t("general.deleteSuccess"));
            }
            deleteId = null;
        }
    }

    async function handleSync() {
        isSyncing = true;
        const result = await settingsStore.syncExchangeRates();
        if (result?.success) {
            toast.success(`Sincronizado! ${result.count} moedas atualizadas.`);
        } else {
            toast.error(result?.error || "Erro ao sincronizar moedas.");
        }
        isSyncing = false;
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between gap-4">
        <div></div>
        <div class="flex flex-wrap gap-2">
            <Button
                variant="outline"
                class="bg-zinc-900 border-zinc-800 hover:bg-zinc-800"
                onclick={handleSync}
                disabled={isSyncing}
            >
                <ArrowRightLeft
                    class={cn("w-4 h-4 mr-2", isSyncing && "animate-spin")}
                />
                {isSyncing ? "Sincronizando..." : "Sincronizar Câmbio"}
            </Button>
            <Button onclick={openNew}>
                <Plus class="w-4 h-4 mr-2" />
                {$t("settings.currencies.new")}
            </Button>
        </div>
    </div>

    <div class="grid gap-2">
        {#if filteredItems.length > 0}
            {#each filteredItems as currency}
                <div
                    class="flex items-center justify-between p-4 rounded-lg border bg-card hover:border-primary/50 transition-all group shadow-sm cursor-pointer"
                    onclick={() => openEdit(currency)}
                    onkeydown={(e) => e.key === "Enter" && openEdit(currency)}
                    tabindex={0}
                    role="button"
                >
                    <div class="flex items-center gap-4">
                        <div class="p-2.5 bg-muted rounded-xl shrink-0">
                            <Coins class="w-5 h-5 text-foreground/70" />
                        </div>
                        <div>
                            <div class="flex items-center gap-2">
                                <h4 class="font-bold text-base">
                                    {currency.code}
                                </h4>
                                <span
                                    class="text-[10px] font-bold bg-muted px-1.5 py-0.5 rounded text-muted-foreground"
                                    >{currency.symbol}</span
                                >
                            </div>
                            <p class="text-sm text-muted-foreground">
                                {currency.name}
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
                                openEdit(currency);
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
                                requestDelete(currency.id);
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
                <span class="text-sm">{$t("settings.currencies.empty")}</span>
            </div>
        {/if}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingId
                    ? $t("settings.currencies.edit")
                    : $t("settings.currencies.new")}</Dialog.Title
            >
            <Dialog.Description>
                {$t("settings.currencies.description")}
            </Dialog.Description>
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <Label for="code" class="text-right"
                    >{$t("settings.currencies.form.code")}</Label
                >
                <Input
                    id="code"
                    bind:value={formData.code}
                    class="col-span-3 uppercase"
                    placeholder={$t("settings.currencies.form.codePlaceholder")}
                    maxlength={3}
                />
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label for="symbol" class="text-right"
                    >{$t("settings.currencies.form.symbol")}</Label
                >
                <Input
                    id="symbol"
                    bind:value={formData.symbol}
                    class="col-span-3"
                    placeholder={$t(
                        "settings.currencies.form.symbolPlaceholder",
                    )}
                />
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label for="name" class="text-right"
                    >{$t("settings.currencies.form.name")}</Label
                >
                <Input
                    id="name"
                    bind:value={formData.name}
                    class="col-span-3"
                    placeholder={$t("settings.currencies.form.namePlaceholder")}
                />
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <Label for="rate" class="text-right">Câmbio (BRL)</Label>
                <Input
                    id="rate"
                    type="number"
                    step="0.0001"
                    bind:value={formData.exchange_rate}
                    class="col-span-3"
                    placeholder="Ex: 5.40"
                />
            </div>
        </div>

        <Dialog.Footer>
            <Button type="submit" onclick={saveCurrency}
                >{$t("settings.currencies.form.save")}</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
