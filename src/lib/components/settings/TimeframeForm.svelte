<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { t } from "svelte-i18n";
    import type { Timeframe } from "$lib/stores/settings.svelte";

    let { initialData, onSave, onCancel } = $props<{
        initialData?: Timeframe;
        onSave: (data: Omit<Timeframe, "id">) => void;
        onCancel: () => void;
    }>();

    const data = $state.snapshot(initialData);
    let formData = $state<Omit<Timeframe, "id">>({
        name: data?.name ?? "",
        value: data?.value ?? "",
    });

    function save() {
        onSave(formData);
    }
</script>

<div class="space-y-4 py-4">
    <div class="space-y-2">
        <Label>{$t("settings.timeframes.displayName")}</Label>
        <Input
            bind:value={formData.name}
            placeholder={$t("settings.timeframes.namePlaceholder")}
        />
    </div>

    <div class="space-y-2">
        <Label>{$t("settings.timeframes.value")}</Label>
        <Input
            bind:value={formData.value}
            placeholder={$t("settings.timeframes.valuePlaceholder")}
        />
    </div>

    <div class="flex justify-end gap-2 pt-4 border-t">
        <Button variant="outline" onclick={onCancel}
            >{$t("general.cancel")}</Button
        >
        <Button onclick={save}>{$t("general.save")}</Button>
    </div>
</div>
