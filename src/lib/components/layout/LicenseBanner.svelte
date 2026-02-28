<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { AlertTriangle, Info, Clock, ExternalLink } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { t } from "svelte-i18n";

    const status = $derived(settingsStore.licenseStatus);
    const daysLeft = $derived(settingsStore.trialDaysLeft);
</script>

{#if status === "trial"}
    <div
        class="bg-blue-600 px-4 py-2 text-white flex items-center justify-between shadow-lg"
    >
        <div class="flex items-center gap-2">
            <Clock class="w-4 h-4" />
            <span class="text-sm font-medium">
                {$t("license.banner.trialActive", {
                    values: { days: daysLeft },
                })}
            </span>
        </div>
        <div class="flex items-center gap-4">
            <span class="text-xs opacity-90 hidden sm:inline"
                >{$t("license.banner.getLicensePrompt")}</span
            >
            <Button
                variant="secondary"
                size="sm"
                class="h-7 text-xs"
                href="/settings/license"
            >
                {$t("license.banner.activateNow")}
            </Button>
        </div>
    </div>
{:else if status === "expired"}
    <div
        class="bg-destructive px-4 py-2 text-destructive-foreground flex items-center justify-between shadow-xl animate-pulse"
    >
        <div class="flex items-center gap-2">
            <AlertTriangle class="w-4 h-4" />
            <span class="text-sm font-bold">
                {$t("license.banner.expired")}
            </span>
        </div>
        <Button
            variant="secondary"
            size="sm"
            class="h-7 text-xs font-bold"
            href="/settings/license"
        >
            {$t("license.banner.getLicenseNow")}
        </Button>
    </div>
{/if}
