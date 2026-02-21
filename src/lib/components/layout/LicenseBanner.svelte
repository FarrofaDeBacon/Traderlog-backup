<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { AlertTriangle, Info, Clock, ExternalLink } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";

    const status = $derived(settingsStore.licenseStatus);
    const daysLeft = $derived(settingsStore.trialDaysLeft);
</script>

{#if status === "trial"}
    <div class="bg-blue-600 px-4 py-2 text-white flex items-center justify-between shadow-lg">
        <div class="flex items-center gap-2">
            <Clock class="w-4 h-4" />
            <span class="text-sm font-medium">
                Período de Experiência: <strong>{daysLeft} dias restantes</strong>
            </span>
        </div>
        <div class="flex items-center gap-4">
             <span class="text-xs opacity-90 hidden sm:inline">Adquira sua licença para continuar após o período de teste.</span>
             <Button variant="secondary" size="sm" class="h-7 text-xs" href="/settings/license">
                Ativar Licença
             </Button>
        </div>
    </div>
{:else if status === "expired"}
    <div class="bg-destructive px-4 py-2 text-destructive-foreground flex items-center justify-between shadow-xl animate-pulse">
        <div class="flex items-center gap-2">
            <AlertTriangle class="w-4 h-4" />
            <span class="text-sm font-bold">
                SEU PERÍODO DE TESTE EXPIROU!
            </span>
        </div>
        <Button variant="secondary" size="sm" class="h-7 text-xs font-bold" href="/settings/license">
            OBTER LICENÇA AGORA
        </Button>
    </div>
{/if}
