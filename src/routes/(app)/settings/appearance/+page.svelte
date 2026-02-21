<script lang="ts">
    import { Separator } from "$lib/components/ui/separator";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import ModeToggle from "$lib/components/mode-toggle/ModeToggle.svelte";
    import { t } from "svelte-i18n";

    let compactMode = $state(false);
</script>

<div class="space-y-6">
    <div>
        <h3 class="text-lg font-medium">
            {$t("settings.appearance.title")}
        </h3>
        <p class="text-sm text-muted-foreground">
            {$t("settings.appearance.description")}
        </p>
    </div>
    <Separator />

    <div class="space-y-4">
        <div class="flex items-center justify-between rounded-lg border p-4">
            <div class="space-y-0.5">
                <Label class="text-base"
                    >{$t("settings.appearance.themeLabel")}</Label
                >
                <p class="text-sm text-muted-foreground">
                    {$t("settings.appearance.themeDescription")}
                </p>
            </div>
            <ModeToggle />
        </div>

        <div class="flex items-center justify-between rounded-lg border p-4">
            <div class="space-y-0.5">
                <Label for="compact-mode" class="text-base"
                    >{$t("settings.appearance.compactLabel")}</Label
                >
                <p class="text-sm text-muted-foreground">
                    {$t("settings.appearance.compactDescription")}
                </p>
            </div>
            <Switch id="compact-mode" bind:checked={compactMode} />
        </div>
    </div>
</div>
