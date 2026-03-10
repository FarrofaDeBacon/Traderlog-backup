<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { t } from "svelte-i18n";
    import type { Tag } from "$lib/stores/settings.svelte";

    let { initialData, onSave, onCancel } = $props<{
        initialData?: Tag;
        onSave: (data: Omit<Tag, "id">) => void;
        onCancel: () => void;
    }>();

    const data = $state.snapshot(initialData);
    let formData = $state<Omit<Tag, "id">>({
        name: data?.name ?? "",
        color: data?.color ?? "#00ffff",
    });

    function save() {
        onSave(formData);
    }
</script>

<div class="space-y-4 py-4">
    <div class="space-y-2">
        <Label>{$t("settings.tags.form.name")}</Label>
        <Input
            bind:value={formData.name}
            placeholder={$t("settings.tags.form.namePlaceholder")}
        />
    </div>

    <div class="space-y-2">
        <Label>{$t("settings.tags.form.color")}</Label>
        <div class="flex items-center gap-4">
            <input
                type="color"
                bind:value={formData.color}
                class="h-10 w-20 bg-background border border-input rounded-md cursor-pointer"
            />
            <div
                class="px-4 py-1 rounded-full text-xs font-bold border"
                style="background-color: {formData.color}20; color: {formData.color}; border-color: {formData.color}50;"
            >
                {formData.name || $t("settings.tags.form.preview")}
            </div>
        </div>
    </div>

    <div class="flex justify-end gap-2 pt-4 border-t">
        <Button variant="outline" onclick={onCancel}
            >{$t("general.cancel")}</Button
        >
        <Button onclick={save}>{$t("general.save")}</Button>
    </div>
</div>
