<script lang="ts">
    import { Separator } from "$lib/components/ui/separator";
    import { Button } from "$lib/components/ui/button";
    import { t } from "svelte-i18n";
    // General settings can include app-wide language, update checks, etc.
    // For now, we'll keep it simple.
</script>

<div class="space-y-6">
    <div>
        <h3 class="text-lg font-medium">{$t("settings.general.title")}</h3>
        <p class="text-sm text-muted-foreground">
            {$t("settings.general.description")}
        </p>
    </div>
    <Separator />

    <div class="space-y-4">
        <div class="grid gap-1">
            <div class="font-medium">{$t("settings.general.version")}</div>
            <div class="text-sm text-muted-foreground">v0.1.0 (Alpha)</div>
        </div>
        <Button variant="outline" size="sm"
            >{$t("settings.general.checkUpdates")}</Button
        >
    </div>
</div>
