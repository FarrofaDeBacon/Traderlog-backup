<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import * as Select from "$lib/components/ui/select";
    import { t } from "svelte-i18n";
    import type { ChartType } from "$lib/stores/settings.svelte";

    let { initialData, onSave, onCancel } = $props<{
        initialData?: ChartType;
        onSave: (data: Omit<ChartType, "id">) => void;
        onCancel: () => void;
    }>();

    const data = $state.snapshot(initialData);
    let formData = $state<Omit<ChartType, "id">>({
        name: data?.name ?? "",
        base_type: data?.base_type ?? "TimeBased",
        parameter: data?.parameter ?? "",
    });

    let typeOptions = $derived([
        {
            value: "TimeBased",
            label: $t("settings.chartTypes.types.TimeBased"),
        },
        { value: "Renko", label: $t("settings.chartTypes.types.Renko") },
        { value: "Range", label: $t("settings.chartTypes.types.Range") },
    ]);

    function save() {
        onSave(formData);
    }
</script>

<div class="space-y-4 py-4">
    <div class="space-y-2">
        <Label>{$t("settings.chartTypes.name")}</Label>
        <Input
            bind:value={formData.name}
            placeholder={$t("settings.chartTypes.namePlaceholder")}
        />
    </div>

    <div class="grid grid-cols-2 gap-4">
        <div class="space-y-2">
            <Label>{$t("settings.chartTypes.methodology")}</Label>
            <Select.Root type="single" bind:value={formData.base_type}>
                <Select.Trigger>
                    {typeOptions.find((o) => o.value === formData.base_type)
                        ?.label ?? formData.base_type}
                </Select.Trigger>
                <Select.Content>
                    {#each typeOptions as option}
                        <Select.Item value={option.value}
                            >{option.label}</Select.Item
                        >
                    {/each}
                </Select.Content>
            </Select.Root>
        </div>
        <div class="space-y-2">
            <Label>{$t("settings.chartTypes.parameter")}</Label>
            <Input
                bind:value={formData.parameter}
                placeholder={$t(
                    "settings.chartTypes.parameterPlaceholder",
                )}
            />
        </div>
    </div>

    <div class="flex justify-end gap-2 pt-4 border-t">
        <Button variant="outline" onclick={onCancel}
            >{$t("general.cancel")}</Button
        >
        <Button onclick={save}>{$t("general.save")}</Button>
    </div>
</div>
