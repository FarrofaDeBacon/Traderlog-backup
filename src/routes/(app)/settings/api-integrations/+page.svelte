<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Plug,
        Link as LinkIcon,
        RefreshCw,
        Save,
        Search,
        Key,
        Globe,
        ExternalLink,
        ArrowRightLeft,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import * as Table from "$lib/components/ui/table";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Separator } from "$lib/components/ui/separator";
    import { settingsStore, type ApiConfig } from "$lib/stores/settings.svelte";
    import ApiConfigForm from "$lib/components/settings/ApiConfigForm.svelte";
    import ProfitRtdCard from "$lib/components/settings/ProfitRtdCard.svelte";
    import { Badge } from "$lib/components/ui/badge";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";
    import { cn } from "$lib/utils";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";

    let isDialogOpen = $state(false);
    let isSyncing = $state(false);
    let editingItem = $state<ApiConfig | undefined>(undefined);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    // Initial binding state
    let psychologyProvider = $state(settingsStore.psychologyApiId);
    let marketDataProvider = $state(settingsStore.marketDataApiId);

    // Calendar Source Config
    let calendarSource = $state("https://br.investing.com/economic-calendar/");

    function saveCalendarSource() {
        localStorage.setItem(
            "traderlog:settings:calendar_source",
            calendarSource,
        );
        toast.success($t("settings.api.integrations.serviceMapping.success"));
    }

    $effect(() => {
        const saved = localStorage.getItem(
            "traderlog:settings:calendar_source",
        );
        if (saved) calendarSource = saved;
    });

    // Derived lists for selection
    let aiProviders = $derived([
        {
            value: "mock",
            label: $t("settings.api.integrations.serviceMapping.mockLabel"),
        },
        ...settingsStore.apiConfigs
            .filter((c) =>
                ["openai", "google_gemini", "anthropic", "custom"].includes(
                    c.provider,
                ),
            )
            .map((c) => ({
                value: c.id,
                label: `${$t(`settings.api.form.providers.${c.provider}`) || c.provider} ${c.enabled ? "" : `(${$t("settings.api.integrations.keys.table.inactive")})`}`,
            })),
    ]);

    let marketProviders = $derived([
        {
            value: "mock",
            label: $t("settings.api.integrations.serviceMapping.mockDataLabel"),
        },
        ...settingsStore.apiConfigs
            .filter((c) =>
                ["polygon", "alpha_vantage", "custom"].includes(c.provider),
            )
            .map((c) => ({
                value: c.id,
                label: `${$t(`settings.api.form.providers.${c.provider}`) || c.provider} ${c.enabled ? "" : `(${$t("settings.api.integrations.keys.table.inactive")})`}`,
            })),
    ]);

    function openNew() {
        editingItem = undefined;
        isDialogOpen = true;
    }

    function openEdit(item: ApiConfig) {
        editingItem = item;
        isDialogOpen = true;
    }

    function save(data: Omit<ApiConfig, "id">) {
        if (editingItem) {
            settingsStore.updateApiConfig(editingItem.id, data);
        } else {
            settingsStore.addApiConfig(data);
        }
        isDialogOpen = false;
    }

    function remove(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.deleteApiConfig(deleteId);
            if (!result.success) {
                toast.error(result.error || $t("general.error"));
            } else {
                toast.success($t("general.deleteSuccess"));
            }
            deleteId = null;
        }
    }

    function saveServiceBindings() {
        settingsStore.saveServiceBindings(
            psychologyProvider,
            marketDataProvider,
        );
        toast.success($t("settings.api.integrations.serviceMapping.success"));
    }

    let searchTerm = $state("");

    // Filtered configs
    let filteredConfigs = $derived(
        settingsStore.apiConfigs
            .filter((c) =>
                c.provider.toLowerCase().includes(searchTerm.toLowerCase()),
            )
            .sort((a, b) => a.provider.localeCompare(b.provider)),
    );

    async function handleSync() {
        isSyncing = true;
        const result = await settingsStore.syncExchangeRates();
        if (result?.success) {
            toast.success(
                `Câmbio atualizado! ${result.count} moedas sincronizadas.`,
            );
        } else if (result) {
            toast.error(result.error || "Erro ao sincronizar moedas.");
        }
        isSyncing = false;
    }

    function saveCurrencyUrl() {
        settingsStore.saveUserProfile();
        toast.success("URL de Câmbio salva com sucesso!");
    }
</script>

<div class="space-y-8">
    <!-- Service Bindings (Keep as Card as it is separate from the List) -->
    <Card.Root class="border-l-4 border-l-primary bg-card/50">
        <Card.Header>
            <div class="flex items-center justify-between">
                <div>
                    <Card.Title class="flex items-center gap-2">
                        <LinkIcon class="w-5 h-5 text-primary" />
                        {$t("settings.api.integrations.serviceMapping.title")}
                    </Card.Title>
                    <Card.Description>
                        {$t(
                            "settings.api.integrations.serviceMapping.description",
                        )}
                    </Card.Description>
                </div>
            </div>
        </Card.Header>
        <Card.Content class="grid md:grid-cols-2 gap-6">
            <div class="space-y-2">
                <Label
                    >{$t(
                        "settings.api.integrations.serviceMapping.psychology",
                    )}</Label
                >
                <Select.Root type="single" bind:value={psychologyProvider}>
                    <Select.Trigger>
                        {aiProviders.find((o) => o.value === psychologyProvider)
                            ?.label ?? psychologyProvider}
                    </Select.Trigger>
                    <Select.Content>
                        {#each aiProviders as prov}
                            <Select.Item value={prov.value}
                                >{prov.label}</Select.Item
                            >
                        {/each}
                    </Select.Content>
                </Select.Root>
                <p class="text-xs text-muted-foreground">
                    {$t(
                        "settings.api.integrations.serviceMapping.psychologyHint",
                    )}
                </p>
            </div>

            <div class="space-y-2">
                <Label
                    >{$t(
                        "settings.api.integrations.serviceMapping.marketData",
                    )}</Label
                >
                <Select.Root type="single" bind:value={marketDataProvider}>
                    <Select.Trigger>
                        {marketProviders.find(
                            (o) => o.value === marketDataProvider,
                        )?.label ?? marketDataProvider}
                    </Select.Trigger>
                    <Select.Content>
                        {#each marketProviders as prov}
                            <Select.Item value={prov.value}
                                >{prov.label}</Select.Item
                            >
                        {/each}
                    </Select.Content>
                </Select.Root>
                <p class="text-xs text-muted-foreground">
                    {$t(
                        "settings.api.integrations.serviceMapping.marketDataHint",
                    )}
                </p>
            </div>
        </Card.Content>
        <Card.Footer class="justify-end border-t border-border/50 pt-4">
            <Button size="sm" onclick={saveServiceBindings}>
                <Save class="w-4 h-4 mr-2" />
                {$t("settings.api.integrations.serviceMapping.save")}
            </Button>
        </Card.Footer>
    </Card.Root>

    <!-- Profit RTD Connection -->
    <ProfitRtdCard />

    <!-- Currency API Config -->
    <Card.Root class="border-l-4 border-l-blue-500 bg-card/50">
        <Card.Header class="pb-3">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <ArrowRightLeft class="w-5 h-5 text-blue-500" />
                    <Card.Title class="text-base"
                        >Serviço de Câmbio (Moedas)</Card.Title
                    >
                </div>
                <Badge
                    variant="outline"
                    class="bg-blue-500/10 text-blue-500 border-blue-500/20"
                >
                    Público & Gratuito
                </Badge>
            </div>
            <Card.Description>
                Utilizado para converter seus saldos e resultados para BRL no
                Hub Financeiro. Usamos a <strong>AwesomeAPI</strong> por padrão.
            </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-4">
            <div class="space-y-2">
                <Label for="currency-url">Endereço da API (Endpoint)</Label>
                <div class="flex gap-2">
                    <Input
                        id="currency-url"
                        bind:value={settingsStore.userProfile.currency_api_url}
                        placeholder="https://..."
                        class="flex-1"
                    />
                    <Button variant="secondary" onclick={saveCurrencyUrl}>
                        <Save class="w-4 h-4 mr-2" />
                        Salvar URL
                    </Button>
                </div>
                <p class="text-xs text-muted-foreground">
                    Se o endereço mudar no futuro, você pode atualizá-lo aqui
                    sem mexer no código.
                </p>
            </div>
        </Card.Content>
        <Card.Footer class="justify-end border-t border-border/50 pt-4">
            <Button
                variant="outline"
                onclick={handleSync}
                disabled={isSyncing}
                class="hover:bg-blue-500/5 hover:text-blue-500 hover:border-blue-500/30"
            >
                <RefreshCw
                    class={cn("w-4 h-4 mr-2", isSyncing && "animate-spin")}
                />
                {isSyncing ? "Sincronizando..." : "Sincronizar Agora"}
            </Button>
        </Card.Footer>
    </Card.Root>

    <!-- Calendar Source Config -->
    <Card.Root>
        <Card.Header class="pb-3">
            <div class="flex items-center gap-2">
                <Globe class="w-5 h-5 text-blue-500" />
                <Card.Title class="text-base"
                    >{$t(
                        "settings.api.integrations.serviceMapping.calendarSource",
                    )}</Card.Title
                >
            </div>
            <Card.Description>
                {$t(
                    "settings.api.integrations.serviceMapping.calendarSourceHint",
                )}
            </Card.Description>
        </Card.Header>
        <Card.Content>
            <div class="flex gap-4">
                <div class="flex-1 relative">
                    <Input
                        bind:value={calendarSource}
                        placeholder="https://..."
                        class="pr-8"
                    />
                    <a
                        href={calendarSource}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="absolute right-3 top-2.5 text-muted-foreground hover:text-primary"
                    >
                        <ExternalLink class="w-4 h-4" />
                    </a>
                </div>
                <Button variant="secondary" onclick={saveCalendarSource}>
                    <Save class="w-4 h-4 mr-2" />
                    {$t(
                        "settings.api.integrations.serviceMapping.calendarSave",
                    )}
                </Button>
            </div>
        </Card.Content>
    </Card.Root>

    <div class="space-y-4">
        <div class="flex items-center justify-between">
            <div class="space-y-0.5">
                <h3
                    class="flex items-center gap-2 text-2xl font-bold tracking-tight"
                >
                    {$t("settings.api.integrations.keys.title")}
                </h3>
                <p class="text-muted-foreground">
                    {$t("settings.api.integrations.keys.description")}
                </p>
            </div>

            <div class="flex items-center gap-2">
                <div class="relative w-64">
                    <Search
                        class="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground"
                    />
                    <Input
                        placeholder={$t(
                            "settings.api.integrations.keys.searchPlaceholder",
                        )}
                        bind:value={searchTerm}
                        class="pl-8 bg-background"
                    />
                </div>
                <Button
                    onclick={openNew}
                    class="bg-primary hover:bg-primary/90 text-primary-foreground shadow-lg shadow-primary/20"
                >
                    <Plus class="w-4 h-4 mr-2" />
                    {$t("settings.api.integrations.keys.newItem")}
                </Button>
            </div>
        </div>

        <div
            class="rounded-md border bg-card text-card-foreground shadow-sm overflow-hidden"
        >
            <Table.Root>
                <Table.Header>
                    <Table.Row
                        class="hover:bg-transparent border-b border-border/50"
                    >
                        <Table.Head
                            class="w-[250px] uppercase text-xs font-bold tracking-wider text-muted-foreground/70"
                            >{$t(
                                "settings.api.integrations.keys.table.provider",
                            )}</Table.Head
                        >
                        <Table.Head
                            class="w-[100px] uppercase text-xs font-bold tracking-wider text-muted-foreground/70"
                            >{$t(
                                "settings.api.integrations.keys.table.status",
                            )}</Table.Head
                        >
                        <Table.Head
                            class="uppercase text-xs font-bold tracking-wider text-muted-foreground/70"
                            >{$t(
                                "settings.api.integrations.keys.table.usage",
                            )}</Table.Head
                        >
                        <Table.Head
                            class="uppercase text-xs font-bold tracking-wider text-muted-foreground/70"
                            >{$t(
                                "settings.api.integrations.keys.table.limit",
                            )}</Table.Head
                        >
                        <Table.Head
                            class="text-right w-[100px] uppercase text-xs font-bold tracking-wider text-muted-foreground/70"
                            >{$t(
                                "settings.api.integrations.keys.table.actions",
                            )}</Table.Head
                        >
                    </Table.Row>
                </Table.Header>
                <Table.Body>
                    {#each filteredConfigs as item}
                        <Table.Row
                            class="hover:bg-muted/50 border-b border-border/50 transition-colors"
                        >
                            <Table.Cell class="font-medium">
                                <div class="flex items-center gap-2">
                                    <!-- Optional Icon logic could go here -->
                                    {$t(
                                        `settings.api.form.providers.${item.provider}`,
                                    ) || item.provider}
                                </div>
                            </Table.Cell>
                            <Table.Cell>
                                {#if item.enabled}
                                    <Badge
                                        variant="outline"
                                        class="bg-primary/10 text-primary border-primary/20 hover:bg-primary/20 hover:text-primary transition-colors"
                                        >{$t(
                                            "settings.api.integrations.keys.table.active",
                                        )}</Badge
                                    >
                                {:else}
                                    <Badge
                                        variant="outline"
                                        class="text-muted-foreground bg-secondary/50 border-secondary"
                                        >{$t(
                                            "settings.api.integrations.keys.table.inactive",
                                        )}</Badge
                                    >
                                {/if}
                            </Table.Cell>
                            <Table.Cell class="text-muted-foreground">
                                {item.requests_today}
                            </Table.Cell>
                            <Table.Cell>
                                {item.daily_limit}
                            </Table.Cell>
                            <Table.Cell class="text-right">
                                <div
                                    class="flex items-center justify-end gap-1"
                                >
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        class="h-8 w-8 text-muted-foreground hover:text-foreground"
                                        onclick={() => openEdit(item)}
                                    >
                                        <Pencil class="w-4 h-4" />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        class="h-8 w-8 text-muted-foreground hover:text-destructive transition-colors"
                                        onclick={() => remove(item.id)}
                                    >
                                        <Trash2 class="w-4 h-4" />
                                    </Button>
                                </div>
                            </Table.Cell>
                        </Table.Row>
                    {:else}
                        <Table.Row>
                            <Table.Cell colspan={5} class="h-24 text-center">
                                {$t(
                                    "settings.api.integrations.keys.table.empty",
                                )}
                            </Table.Cell>
                        </Table.Row>
                    {/each}
                </Table.Body>
            </Table.Root>
        </div>
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content>
        <Dialog.Header>
            <Dialog.Title
                >{editingItem
                    ? $t("settings.api.form.editTitle")
                    : $t("settings.api.form.newTitle")}</Dialog.Title
            >
        </Dialog.Header>
        <ApiConfigForm
            initialData={editingItem}
            onSave={save}
            onCancel={() => (isDialogOpen = false)}
        />
    </Dialog.Content>
</Dialog.Root>
