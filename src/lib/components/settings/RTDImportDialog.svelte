<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import * as Table from "$lib/components/ui/table";
    import * as Select from "$lib/components/ui/select";
    import { rtdStore } from "$lib/stores/rtd.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import { RefreshCcw, FileSpreadsheet, Check } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { untrack } from "svelte";

    let { open = $bindable(false) } = $props();

    interface ImportItem {
        symbol: string;
        name: string;
        typeId: string;
        selected: boolean;
        exists: boolean;
    }

    let items = $state<ImportItem[]>([]);

    // Sync items with rtdStore.quotes - uses untrack to avoid circular loop
    $effect(() => {
        if (open) {
            const quotes = rtdStore.quotes;
            const currentAssets = settingsStore.assets;

            untrack(() => {
                const currentSymbols = currentAssets.map((a) => a.symbol);
                const rtdSymbols = Object.keys(quotes);

                items = rtdSymbols.map((sym) => {
                    const exists = currentSymbols.includes(sym);
                    const existingItem = items.find((i) => i.symbol === sym);

                    return {
                        symbol: sym,
                        name: existingItem?.name || sym,
                        typeId: existingItem?.typeId || detectType(sym),
                        selected: existingItem?.selected ?? !exists,
                        exists,
                    };
                });
            });
        } else if (items.length > 0) {
            untrack(() => {
                items = [];
            });
        }
    });

    function detectType(symbol: string): string {
        const sym = symbol.toUpperCase();
        const typeMatches = (pattern: string) =>
            settingsStore.assetTypes.find((t) =>
                t.name.toLowerCase().includes(pattern.toLowerCase()),
            );

        // Brazilian Futures patterns (WIND24, WDOG24, etc)
        if (/^(WIN|WDO|WSP|DI1|BGI|CCM|IND|DOL)/i.test(sym)) {
            return (
                typeMatches("Futuro")?.id ||
                typeMatches("Indice")?.id ||
                settingsStore.assetTypes[0]?.id ||
                ""
            );
        }

        // Brazilian Stocks patterns (PETR4, VALE3, etc)
        if (/^[A-Z]{4}\d/i.test(sym)) {
            return (
                typeMatches("Aç")?.id ||
                typeMatches("Stock")?.id ||
                settingsStore.assetTypes[1]?.id ||
                ""
            );
        }

        return settingsStore.assetTypes[0]?.id || "";
    }

    let allSelected = $derived(
        items.filter((i) => !i.exists).length > 0 &&
            items.filter((i) => !i.exists).every((i) => i.selected),
    );
    let selectedCount = $derived(
        items.filter((i) => i.selected && !i.exists).length,
    );

    function toggleAll() {
        const target = !allSelected;
        items = items.map((i) => (i.exists ? i : { ...i, selected: target }));
    }

    async function handleImport() {
        const toImport = items.filter((i) => i.selected && !i.exists);
        if (toImport.length === 0) return;

        console.log(`[RTDImportDialog] Importing ${toImport.length} assets...`);

        for (const item of toImport) {
            // Determine point value based on symbol
            let pointValue = 1.0;
            if (item.symbol.startsWith("WDO")) pointValue = 10.0;
            else if (item.symbol.startsWith("WIN")) pointValue = 0.2;
            else if (item.symbol.startsWith("DOL")) pointValue = 50.0;
            else if (item.symbol.startsWith("IND")) pointValue = 1.0;

            await settingsStore.addAsset(
                {
                    symbol: item.symbol.toUpperCase(),
                    name:
                        item.name === item.symbol
                            ? `${item.symbol} (RTD)`
                            : item.name,
                    asset_type_id: item.typeId,
                    point_value: pointValue,
                    default_fee_id: "",
                    is_root: false,
                },
                false, // Don't save individually
            );
        }

        // Save all at once before refreshing
        await settingsStore.saveAssets();
        await settingsStore.loadData(true); // Refresh to ensure sync (silent mode)

        toast.success($t("settings.assets.importDialog.success", { values: { count: toImport.length } }));
        open = false;
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="max-w-[95vw] h-[90vh] flex flex-col">
        <Dialog.Header>
            <Dialog.Title
                >{$t("settings.assets.importDialog.title")}</Dialog.Title
            >
            <Dialog.Description>
                {$t("settings.assets.importDialog.description")}
            </Dialog.Description>
        </Dialog.Header>

        <div class="flex-1 overflow-auto py-4">
            {#if items.length === 0}
                <div
                    class="flex flex-col items-center justify-center py-12 text-center space-y-4"
                >
                    <div
                        class="p-4 bg-muted rounded-full text-muted-foreground"
                    >
                        <RefreshCcw class="w-8 h-8 animate-spin-slow" />
                    </div>
                    <div class="max-w-xs space-y-2">
                        <p class="font-medium">
                            {$t("settings.assets.importDialog.noAssets")}
                        </p>
                        <p class="text-sm text-muted-foreground">
                            {$t(
                                "settings.assets.importDialog.excelRequirement",
                            )}
                        </p>
                    </div>
                </div>
            {:else}
                <Table.Root>
                    <Table.Header>
                        <Table.Row>
                            <Table.Head class="w-12">
                                <Checkbox
                                    checked={allSelected}
                                    onCheckedChange={toggleAll}
                                    aria-label="Select all"
                                />
                            </Table.Head>
                            <Table.Head
                                >{$t(
                                    "settings.assets.importDialog.symbol",
                                )}</Table.Head
                            >
                            <Table.Head
                                >{$t(
                                    "settings.assets.importDialog.suggestedName",
                                )}</Table.Head
                            >
                            <Table.Head
                                >{$t(
                                    "settings.assets.importDialog.type",
                                )}</Table.Head
                            >
                            <Table.Head class="w-24 text-right"></Table.Head>
                        </Table.Row>
                    </Table.Header>
                    <Table.Body>
                        {#each items as item}
                            <Table.Row class={item.exists ? "opacity-50" : ""}>
                                <Table.Cell>
                                    <Checkbox
                                        bind:checked={item.selected}
                                        disabled={item.exists}
                                    />
                                </Table.Cell>
                                <Table.Cell class="font-mono font-bold"
                                    >{item.symbol}</Table.Cell
                                >
                                <Table.Cell>
                                    <input
                                        type="text"
                                        bind:value={item.name}
                                        class="w-full bg-transparent border-none focus:ring-0 text-sm p-0"
                                        placeholder={item.symbol}
                                        disabled={item.exists}
                                    />
                                </Table.Cell>
                                <Table.Cell>
                                    <Select.Root
                                        type="single"
                                        bind:value={item.typeId}
                                        disabled={item.exists}
                                    >
                                        <Select.Trigger
                                            class="h-8 border-none bg-transparent hover:bg-muted/50 transition-colors"
                                        >
                                            {settingsStore.getAssetTypeCode(
                                                item.typeId,
                                            )}
                                        </Select.Trigger>
                                        <Select.Content>
                                            {#each settingsStore.assetTypes as type}
                                                <Select.Item value={type.id}
                                                    >{type.name}</Select.Item
                                                >
                                            {/each}
                                        </Select.Content>
                                    </Select.Root>
                                </Table.Cell>
                                <Table.Cell class="text-right">
                                    {#if item.exists}
                                        <span
                                            class="text-xs text-muted-foreground flex items-center justify-end"
                                        >
                                            <Check class="w-3 h-3 mr-1" />
                                            {$t("settings.assets.importDialog.alreadyRegistered")}
                                        </span>
                                    {/if}
                                </Table.Cell>
                            </Table.Row>
                        {/each}
                    </Table.Body>
                </Table.Root>
            {/if}
        </div>

        <Dialog.Footer class="gap-2 sm:gap-0">
            <Button variant="ghost" onclick={() => (open = false)}>
                {$t("general.cancel")}
            </Button>
            <Button
                disabled={selectedCount === 0}
                onclick={handleImport}
                class="min-w-[200px] h-10 font-bold uppercase tracking-wider"
            >
                <FileSpreadsheet class="w-4 h-4 mr-2" />
                {$t("settings.assets.importDialog.importSelected", {
                    values: { count: selectedCount },
                })}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

<style>
    :global(.animate-spin-slow) {
        animation: spin 3s linear infinite;
    }

    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }
</style>
