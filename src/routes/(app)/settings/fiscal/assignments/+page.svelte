<script lang="ts">
    import { Tag, ExternalLink } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import { goto } from "$app/navigation";

    let assetTypes = $derived(settingsStore.assetTypes);
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">
                {$t("settings.fiscal.assignments.title")}
            </h3>
            {$t("settings.fiscal.assignments.description")}
        </div>
        <Button
            onclick={() => goto("/settings/asset-types")}
            variant="secondary"
        >
            <ExternalLink class="w-4 h-4 mr-2" />
            {$t("settings.fiscal.assignments.goToAssetTypes")}
        </Button>
    </div>

    <div
        class="border rounded-lg overflow-hidden opacity-80 pointer-events-none grayscale-[0.3]"
    >
        <table class="w-full text-sm">
            <thead class="bg-muted text-muted-foreground border-b">
                <tr>
                    <th class="text-left p-3 font-medium"
                        >{$t("settings.fiscal.assignments.table.market")}</th
                    >
                    <th class="text-left p-3 font-medium"
                        >{$t("settings.fiscal.assignments.table.assetType")}</th
                    >
                    <th class="text-left p-3 font-medium"
                        >{$t(
                            "settings.fiscal.assignments.table.fiscalProfile",
                        )}</th
                    >
                </tr>
            </thead>
            <tbody class="divide-y">
                {#each assetTypes as at}
                    <tr class="hover:bg-muted/20 transition-colors">
                        <td class="p-3 text-muted-foreground">
                            {settingsStore.markets.find(
                                (m) => m.id === at.market_id,
                            )?.name || at.market_id}
                        </td>
                        <td class="p-3 font-medium">
                            {at.name}
                            <span class="text-xs text-muted-foreground"
                                >({at.code})</span
                            >
                        </td>
                        <td class="p-3">
                            {#if at.tax_profile_id}
                                {settingsStore.taxProfiles.find(
                                    (p) => p.id === at.tax_profile_id,
                                )?.name ||
                                    $t(
                                        "settings.fiscal.assignments.table.unknownProfile",
                                    )}
                            {:else}
                                <span class="text-muted-foreground italic"
                                    >{$t(
                                        "settings.fiscal.assignments.table.defaultExempt",
                                    )}</span
                                >
                            {/if}
                        </td>
                    </tr>
                {:else}
                    <tr>
                        <td
                            colspan="3"
                            class="p-6 text-center text-muted-foreground"
                        >
                            {$t("settings.fiscal.assignments.table.empty")}
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>

    <div
        class="p-4 bg-primary/5 border border-primary/20 rounded-lg text-sm text-primary/80"
    >
        <strong>{$t("settings.fiscal.assignments.tip.title")}</strong>
        {$t("settings.fiscal.assignments.tip.text")}
    </div>
</div>
