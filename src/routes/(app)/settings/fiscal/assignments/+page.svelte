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
            <h3 class="text-lg font-medium">Atribuições (Ativos)</h3>
            <p class="text-sm text-muted-foreground">
                As atribuições de <strong>Perfil Fiscal</strong> e
                <strong>Perfil de Corretagem</strong>
                foram movidas para a tela de <strong>Tipos de Ativos</strong> para
                facilitar a gestão centralizada.
            </p>
        </div>
        <Button
            onclick={() => goto("/settings/asset-types")}
            variant="secondary"
        >
            <ExternalLink class="w-4 h-4 mr-2" />
            Ir para Tipos de Ativos
        </Button>
    </div>

    <div
        class="border rounded-lg overflow-hidden opacity-80 pointer-events-none grayscale-[0.3]"
    >
        <table class="w-full text-sm">
            <thead class="bg-muted text-muted-foreground border-b">
                <tr>
                    <th class="text-left p-3 font-medium">Mercado</th>
                    <th class="text-left p-3 font-medium">Tipo de Ativo</th>
                    <th class="text-left p-3 font-medium">Perfil Fiscal</th>
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
                                )?.name || "Perfil Desconhecido"}
                            {:else}
                                <span class="text-muted-foreground italic"
                                    >Padrão (Isento)</span
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
                            Nenhum tipo de ativo cadastrado.
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>

    <div
        class="p-4 bg-primary/5 border border-primary/20 rounded-lg text-sm text-primary/80"
    >
        <strong>Dica:</strong> Centralizamos essas configurações para que você possa
        definir o comportamento fiscal e as taxas de corretagem no mesmo lugar onde
        configura os multiplicadores de pontos.
    </div>
</div>
