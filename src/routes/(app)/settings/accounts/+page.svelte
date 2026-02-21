<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Wallet,
        Building2,
        Hash,
        Upload,
        Search,
    } from "lucide-svelte";
    import * as Card from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";
    import { Badge } from "$lib/components/ui/badge";
    import { settingsStore, type Account } from "$lib/stores/settings.svelte";
    import { t, locale } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import { formatCurrency } from "$lib/utils";

    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";

    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);
    let searchTerm = $state("");

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);
    let isProcessing = $state(false);

    let filteredItems = $derived(
        settingsStore.accounts
            .filter(
                (item) =>
                    item.nickname
                        .toLowerCase()
                        .includes(searchTerm.toLowerCase()) ||
                    item.broker
                        .toLowerCase()
                        .includes(searchTerm.toLowerCase()),
            )
            .sort((a, b) => a.nickname.localeCompare(b.nickname)),
    );

    let groupedAccounts = $derived(
        filteredItems.reduce(
            (acc, account) => {
                const type = account.account_type || "Outros";
                if (!acc[type]) acc[type] = [];
                acc[type].push(account);
                return acc;
            },
            {} as Record<string, Account[]>,
        ),
    );

    const typeOrder = ["Real", "Prop", "Demo"];
    const typeLabels: Record<string, string> = {
        Real: "Contas Reais",
        Prop: "Mesas Proprietárias",
        Demo: "Simuladores / Demo",
    };

    // Form Stats
    let formData = $state<Omit<Account, "id">>({
        nickname: "",
        account_type: "Real",
        broker: "",
        account_number: "",
        currency: "BRL", // Default
        balance: 0,
        custom_logo: null,
    });

    function resetForm() {
        formData = {
            nickname: "",
            account_type: "Real",
            broker: "",
            account_number: "",
            currency: settingsStore.currencies[0]?.code || "BRL",
            balance: 0,
            custom_logo: null,
        };
        editingId = null;
    }

    function openNew() {
        resetForm();
        isDialogOpen = true;
    }

    function openEdit(account: Account) {
        editingId = account.id;
        formData = { ...account }; // Clone
        isDialogOpen = true;
    }

    async function saveAccount() {
        if (isProcessing) return;
        isProcessing = true;
        try {
            if (editingId) {
                settingsStore.updateAccount(editingId, formData);
            } else {
                settingsStore.addAccount(formData);
            }
            isDialogOpen = false;
        } finally {
            isProcessing = false;
        }
    }

    async function handleDeduplicate() {
        if (
            !confirm(
                $t("settings.accounts.confirmDeduplicate") ||
                    "Isso removerá contas duplicadas VAZIAS (sem trades). Deseja continuar?",
            )
        )
            return;
        isProcessing = true;
        try {
            await settingsStore.deduplicateAccounts();
            toast.success("Limpeza de duplicatas concluída.");
        } catch (e) {
            console.error(e);
            toast.error("Erro ao limpar duplicatas.");
        } finally {
            isProcessing = false;
        }
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId && !isProcessing) {
            isProcessing = true;
            try {
                const result = await settingsStore.deleteAccount(deleteId);
                if (result.success) {
                    toast.success($t("general.deleteSuccess"));
                    isDeleteOpen = false;
                    deleteId = null;
                } else {
                    toast.error(result.error || "Erro ao deletar conta");
                }
            } finally {
                isProcessing = false;
            }
        }
    }
</script>

<div class="space-y-8">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-2xl font-bold tracking-tight">
                {$t("settings.accounts.title")}
            </h3>
            <p class="text-muted-foreground">
                {$t("settings.accounts.description")}
            </p>
        </div>
    </div>

    <div class="flex items-center justify-between gap-4">
        <div class="relative flex-1 max-w-md">
            <Search
                class="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground"
            />
            <Input
                placeholder={$t("settings.accounts.searchPlaceholder")}
                bind:value={searchTerm}
                class="pl-8"
            />
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.accounts.new")}
        </Button>
        <Button
            variant="outline"
            onclick={handleDeduplicate}
            disabled={isProcessing}
        >
            <Trash2 class="w-4 h-4 mr-2" />
            Remover Duplicadas
        </Button>
    </div>

    {#if filteredItems.length === 0}
        <div
            class="flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg border-muted-foreground/25 hover:border-primary/50 text-muted-foreground hover:text-primary transition-all h-[200px]"
        >
            <p>{$t("settings.accounts.empty")}</p>
        </div>
    {:else}
        {#each typeOrder as type}
            {#if groupedAccounts[type] && groupedAccounts[type].length > 0}
                <div class="space-y-4">
                    <div class="flex items-center gap-2">
                        {#if type === "Real"}
                            <div class="p-1.5 bg-green-500/10 rounded">
                                <Building2 class="w-4 h-4 text-green-500" />
                            </div>
                        {:else if type === "Prop"}
                            <div class="p-1.5 bg-amber-500/10 rounded">
                                <Wallet class="w-4 h-4 text-amber-500" />
                            </div>
                        {:else}
                            <div class="p-1.5 bg-blue-500/10 rounded">
                                <Building2 class="w-4 h-4 text-blue-500" />
                            </div>
                        {/if}
                        <h4 class="text-lg font-semibold tracking-tight">
                            {$t(`settings.accounts.types.${type}`) || type}
                        </h4>
                    </div>

                    <div class="flex flex-col gap-3">
                        {#each groupedAccounts[type] as account}
                            <div
                                class="flex items-center justify-between p-4 rounded-lg border bg-card hover:border-primary/50 transition-all group shadow-sm cursor-pointer"
                                onclick={() => openEdit(account)}
                                onkeydown={(e) =>
                                    e.key === "Enter" && openEdit(account)}
                                tabindex="0"
                                role="button"
                            >
                                <!-- Left: Icon + Info -->
                                <div class="flex items-center gap-4 shrink-0">
                                    <div
                                        class="p-2.5 bg-muted rounded-xl shrink-0"
                                    >
                                        {#if account.account_type === "Real"}
                                            <Building2
                                                class="w-5 h-5 text-foreground/80"
                                            />
                                        {:else}
                                            <Wallet
                                                class="w-5 h-5 text-foreground/80"
                                            />
                                        {/if}
                                    </div>
                                    <div class="min-w-[150px]">
                                        <div class="flex items-center gap-2">
                                            <h4 class="font-bold text-base">
                                                {account.nickname}
                                            </h4>
                                            <Badge
                                                variant={account.account_type ===
                                                "Real"
                                                    ? "default"
                                                    : "secondary"}
                                                class="h-5 px-1.5 text-[10px] capitalize"
                                            >
                                                {account.account_type}
                                            </Badge>
                                        </div>
                                        <p
                                            class="text-sm text-muted-foreground"
                                        >
                                            {account.broker}
                                        </p>
                                    </div>
                                </div>

                                <!-- Middle: Balance -->
                                <div
                                    class="flex flex-col items-end gap-0.5 mr-auto ml-8"
                                >
                                    <span
                                        class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider"
                                        >SALDO</span
                                    >
                                    <span
                                        class="font-bold text-lg text-foreground tracking-tight"
                                    >
                                        {formatCurrency(
                                            account.balance,
                                            account.currency,
                                            $locale || "pt-BR",
                                        )}
                                    </span>
                                </div>

                                <!-- Right: Account # + Actions -->
                                <div class="flex items-center gap-6">
                                    {#if account.account_number}
                                        <div
                                            class="flex items-center gap-1.5 text-xs text-muted-foreground hidden md:flex"
                                        >
                                            <Hash class="w-3.5 h-3.5" />
                                            <span class="font-mono font-medium"
                                                >{account.account_number}</span
                                            >
                                        </div>
                                    {/if}

                                    <div
                                        class="flex items-center gap-1 opacity-100 md:opacity-0 md:group-hover:opacity-100 transition-opacity"
                                    >
                                        <!-- Edit button redundant if card is clickable, but kept for clarity/accessibility -->
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            onclick={(e) => {
                                                e.stopPropagation();
                                                openEdit(account);
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
                                                requestDelete(account.id);
                                            }}
                                        >
                                            <Trash2 class="w-4 h-4" />
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        {/each}

        <!-- Fallback for other types -->
        {#each Object.keys(groupedAccounts).filter((k) => !typeOrder.includes(k)) as type}
            <div class="space-y-4">
                <h4 class="text-lg font-semibold tracking-tight">{type}</h4>
                <Separator />
                <div class="flex flex-col gap-3">
                    {#each groupedAccounts[type] as account}
                        <Card.Root
                            class="cursor-pointer hover:border-primary/50 transition-colors"
                            onclick={() => openEdit(account)}
                        >
                            <Card.Header>
                                <Card.Title>{account.nickname}</Card.Title>
                            </Card.Header>
                        </Card.Root>
                    {/each}
                </div>
            </div>
        {/each}
    {/if}
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[500px]">
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2">
                <div class="p-2 bg-primary/10 rounded-lg">
                    <Wallet class="w-5 h-5 text-primary" />
                </div>
                {editingId
                    ? $t("settings.accounts.form.titleEdit")
                    : $t("settings.accounts.form.titleNew")}
            </Dialog.Title>
        </Dialog.Header>

        <div class="space-y-4 py-4">
            <!-- Icon Section -->
            <div class="space-y-2">
                <Label>{$t("settings.accounts.form.icon")}</Label>
                <div
                    class="border-2 border-dashed border-muted-foreground/25 rounded-lg p-4 flex flex-col items-center justify-center gap-2 hover:border-primary/50 transition-colors cursor-pointer bg-muted/5"
                >
                    <div class="p-2 bg-background rounded-full">
                        <Upload class="w-4 h-4 text-muted-foreground" />
                    </div>
                    <span class="text-sm font-medium"
                        >{$t("settings.accounts.form.addIcon")}</span
                    >
                    <span class="text-xs text-muted-foreground"
                        >{$t("settings.accounts.form.iconHint")}</span
                    >
                </div>
            </div>

            <!-- Apelido -->
            <div class="space-y-2">
                <Label for="nickname"
                    >{$t("settings.accounts.form.nickname")}</Label
                >
                <Input
                    id="nickname"
                    bind:value={formData.nickname}
                    placeholder={$t(
                        "settings.accounts.form.nicknamePlaceholder",
                    )}
                />
            </div>

            <div class="grid grid-cols-2 gap-4">
                <!-- Tipo -->
                <div class="space-y-2">
                    <Label for="type">{$t("settings.accounts.form.type")}</Label
                    >
                    <Select.Root
                        type="single"
                        bind:value={formData.account_type}
                    >
                        <Select.Trigger class="w-full">
                            {$t(
                                `settings.accounts.types.${formData.account_type}`,
                            ) || formData.account_type}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="Real"
                                >{$t(
                                    "settings.accounts.types.Real",
                                )}</Select.Item
                            >
                            <Select.Item value="Demo"
                                >{$t(
                                    "settings.accounts.types.Demo",
                                )}</Select.Item
                            >
                            <Select.Item value="Prop"
                                >{$t(
                                    "settings.accounts.types.Prop",
                                )}</Select.Item
                            >
                        </Select.Content>
                    </Select.Root>
                </div>

                <!-- Corretora -->
                <div class="space-y-2">
                    <Label for="broker"
                        >{$t("settings.accounts.form.broker")}</Label
                    >
                    <Input
                        id="broker"
                        bind:value={formData.broker}
                        placeholder={$t(
                            "settings.accounts.form.brokerPlaceholder",
                        )}
                    />
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <!-- Número -->
                <div class="space-y-2">
                    <Label for="acc_number"
                        >{$t("settings.accounts.form.number")}</Label
                    >
                    <Input
                        id="acc_number"
                        bind:value={formData.account_number}
                        placeholder={$t(
                            "settings.accounts.form.numberPlaceholder",
                        )}
                    />
                </div>

                <!-- Moeda -->
                <div class="space-y-2">
                    <Label for="currency"
                        >{$t("settings.accounts.form.currency")}</Label
                    >
                    <Select.Root type="single" bind:value={formData.currency}>
                        <Select.Trigger class="w-full">
                            {formData.currency}
                        </Select.Trigger>
                        <Select.Content>
                            {#each settingsStore.currencies as curr}
                                <Select.Item value={curr.code}>
                                    {curr.code} - {curr.name}
                                </Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <!-- Saldo -->
            <div class="space-y-2">
                <Label for="balance"
                    >{$t("settings.accounts.form.balance")}</Label
                >
                <div class="relative">
                    <span
                        class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground text-sm font-bold"
                    >
                        {settingsStore.getCurrencySymbol(formData.currency)}
                    </span>
                    <Input
                        type="number"
                        id="balance"
                        bind:value={formData.balance}
                        class="pl-8 font-mono"
                        step="0.01"
                        placeholder="0.00"
                    />
                </div>
            </div>
        </div>

        <Dialog.Footer>
            <Button variant="outline" onclick={() => (isDialogOpen = false)}
                >{$t("general.cancel")}</Button
            >
            <Button type="submit" onclick={saveAccount} disabled={isProcessing}>
                {isProcessing
                    ? "Salvando..."
                    : $t("settings.accounts.form.save")}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
