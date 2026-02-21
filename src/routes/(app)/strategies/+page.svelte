<script lang="ts">
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import StrategyPerformanceCard from "$lib/components/strategies/StrategyPerformanceCard.svelte";
    import { Input } from "$lib/components/ui/input";
    import { Search, Filter, Plus } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import * as Select from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";

    let searchTerm = $state("");
    let selectedType = $state("all");
    let currencyMode = $state<"original" | "main">("main");

    // Filter Logic
    let filteredStrategies = $derived(
        settingsStore.strategies.filter((s) => {
            const matchesSearch = s.name
                .toLowerCase()
                .includes(searchTerm.toLowerCase());

            const matchesType =
                selectedType === "all" ||
                s.asset_types.some((typeStr) => {
                    // Check if typeStr is the selected ID
                    if (typeStr === selectedType) return true;
                    // Check if the name of the type matching selectedType matches typeStr
                    const selectedTypeName = settingsStore.assetTypes.find(
                        (at) => at.id === selectedType,
                    )?.name;
                    if (selectedTypeName === typeStr) return true;
                    return false;
                });

            return matchesSearch && matchesType;
        }),
    );
</script>

<div class="space-y-6">
    <!-- Header with Search & actions -->
    <div
        class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4"
    >
        <div>
            <h2 class="text-3xl font-bold tracking-tight">
                {$t("strategy.list.title")}
            </h2>
            <p class="text-muted-foreground">
                {$t("strategy.list.description")}
            </p>
        </div>
        <div class="flex items-center gap-2 w-full md:w-auto">
            <div class="relative flex-1 md:w-64">
                <Search
                    class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground"
                />
                <Input
                    type="search"
                    placeholder={$t("strategy.list.searchPlaceholder")}
                    class="pl-9"
                    bind:value={searchTerm}
                />
            </div>
            <Button variant="outline" size="icon">
                <Filter class="w-4 h-4" />
            </Button>
            <div class="flex items-center bg-muted/30 rounded-lg p-1 mr-2">
                <Button
                    variant={currencyMode === "original"
                        ? "secondary"
                        : "ghost"}
                    size="sm"
                    class="h-7 px-2 text-[10px] uppercase font-bold"
                    onclick={() => (currencyMode = "original")}
                >
                    Original
                </Button>
                <Button
                    variant={currencyMode === "main" ? "secondary" : "ghost"}
                    size="sm"
                    class="h-7 px-2 text-[10px] uppercase font-bold"
                    onclick={() => (currencyMode = "main")}
                >
                    Principal
                </Button>
            </div>
            <Button href="/settings/strategies">
                <Plus class="w-4 h-4 mr-2" />
                {$t("strategy.list.new")}
            </Button>
        </div>
    </div>

    <Separator />

    <!-- Grid Layout -->
    {#if filteredStrategies.length > 0}
        <div
            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
        >
            {#each filteredStrategies as strategy (strategy.id)}
                <StrategyPerformanceCard
                    {strategy}
                    stats={tradesStore.getStrategyStats(
                        strategy.id,
                        settingsStore.accounts,
                        settingsStore.currencies,
                        settingsStore.userProfile,
                        currencyMode,
                    )}
                />
            {/each}
        </div>
    {:else}
        <div
            class="flex flex-col items-center justify-center p-12 border-2 border-dashed rounded-lg border-muted text-center h-[400px]"
        >
            <div class="p-4 bg-muted/50 rounded-full mb-4">
                <Search class="w-8 h-8 text-muted-foreground" />
            </div>
            <h3 class="text-lg font-semibold">
                {$t("strategy.list.notFound.title")}
            </h3>
            <p class="text-muted-foreground max-w-sm mt-2">
                {$t("strategy.list.notFound.description")}
            </p>
            <Button href="/settings/strategies" variant="link" class="mt-4">
                {$t("strategy.list.notFound.action")}
            </Button>
        </div>
    {/if}
</div>
