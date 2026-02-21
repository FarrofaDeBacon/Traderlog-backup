<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { Textarea } from "$lib/components/ui/textarea";
    import * as Select from "$lib/components/ui/select";
    import { t } from "svelte-i18n";
    import type { EmotionalState } from "$lib/stores/settings.svelte";

    let { initialData, onSave, onCancel } = $props<{
        initialData?: EmotionalState;
        onSave: (data: Omit<EmotionalState, "id">) => void;
        onCancel: () => void;
    }>();

    let formData = $state<Omit<EmotionalState, "id">>({
        name: initialData?.name ?? "",
        impact: initialData?.impact ?? "Negative",
        description: initialData?.description ?? "",
        potential_impact: initialData?.potential_impact ?? "",
        weight: initialData?.weight ?? 5.0,
    });

    const impactOptions = $derived([
        {
            value: "Positive",
            label: $t("settings.emotionalStates.form.impactOptions.Positive"),
        },
        {
            value: "Negative",
            label: $t("settings.emotionalStates.form.impactOptions.Negative"),
        },
        {
            value: "Neutral",
            label: $t("settings.emotionalStates.form.impactOptions.Neutral"),
        },
    ]);

    function save() {
        onSave(formData);
    }
</script>

<div class="space-y-4 py-4">
    <div class="grid grid-cols-3 gap-4">
        <div class="space-y-2">
            <Label>{$t("settings.emotionalStates.form.name")}</Label>
            <Input
                bind:value={formData.name}
                placeholder={$t(
                    "settings.emotionalStates.form.namePlaceholder",
                )}
            />
        </div>
        <div class="space-y-2">
            <Label>{$t("settings.emotionalStates.form.impact")}</Label>
            <Select.Root type="single" bind:value={formData.impact}>
                <Select.Trigger>
                    {impactOptions.find((o) => o.value === formData.impact)
                        ?.label ?? formData.impact}
                </Select.Trigger>
                <Select.Content>
                    {#each impactOptions as option}
                        <Select.Item value={option.value}
                            >{option.label}</Select.Item
                        >
                    {/each}
                </Select.Content>
            </Select.Root>
        </div>
        <div class="space-y-2">
            <Label>{$t("settings.emotionalStates.form.weight")}</Label>
            <Input
                type="number"
                step="0.1"
                min="0"
                max="10"
                bind:value={formData.weight}
                placeholder="0 - 10"
            />
        </div>
    </div>

    <div class="space-y-2">
        <Label>{$t("settings.emotionalStates.form.description")}</Label>
        <Textarea
            bind:value={formData.description}
            placeholder={$t(
                "settings.emotionalStates.form.descriptionPlaceholder",
            )}
        />
    </div>

    <div class="space-y-2">
        <Label>{$t("settings.emotionalStates.form.potentialImpact")}</Label>
        <Textarea
            bind:value={formData.potential_impact}
            placeholder={$t(
                "settings.emotionalStates.form.potentialImpactPlaceholder",
            )}
        />
    </div>

    <div class="flex justify-end gap-2 pt-4 border-t">
        <Button variant="outline" onclick={onCancel}
            >{$t("general.cancel")}</Button
        >
        <Button onclick={save}>{$t("general.save")}</Button>
    </div>
</div>
