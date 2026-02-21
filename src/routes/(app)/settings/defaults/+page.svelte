<script lang="ts">
    import { Separator } from "$lib/components/ui/separator";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { Switch } from "$lib/components/ui/switch";
    import * as Select from "$lib/components/ui/select";
    import { t } from "svelte-i18n";

    // Mock Data
    const strategies = [
        { value: "scalping", label: "Scalping" },
        { value: "daytrade", label: "Day Trade" },
        { value: "swing", label: "Swing Trade" },
    ];

    let selectedStrategy = $state("");
</script>

<div class="space-y-6">
    <div>
        <h3 class="text-lg font-medium">
            {$t("settings.defaults.title")}
        </h3>
        <p class="text-sm text-muted-foreground">
            {$t("settings.defaults.description")}
        </p>
    </div>
    <Separator />

    <div class="space-y-8 max-w-2xl">
        <div class="grid gap-4 md:grid-cols-2">
            <div class="space-y-2">
                <Label>{$t("settings.defaults.strategyLabel")}</Label>
                <Select.Root type="single" bind:value={selectedStrategy}>
                    <Select.Trigger class="w-full">
                        {strategies.find((s) => s.value === selectedStrategy)
                            ?.label ||
                            $t("settings.defaults.strategyPlaceholder")}
                    </Select.Trigger>
                    <Select.Content>
                        {#each strategies as strategy}
                            <Select.Item value={strategy.value}
                                >{strategy.label}</Select.Item
                            >
                        {/each}
                    </Select.Content>
                </Select.Root>
                <p class="text-[0.8rem] text-muted-foreground">
                    {$t("settings.defaults.strategyHint")}
                </p>
            </div>

            <div class="space-y-2">
                <Label>{$t("settings.defaults.riskLabel")}</Label>
                <Input type="number" placeholder="1.0" />
                <p class="text-[0.8rem] text-muted-foreground">
                    {$t("settings.defaults.riskHint")}
                </p>
            </div>
        </div>

        <div class="flex items-center space-x-2">
            <Switch id="auto-calc" />
            <Label for="auto-calc"
                >{$t("settings.defaults.autoCalcLabel")}</Label
            >
        </div>
    </div>
</div>
