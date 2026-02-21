<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { Textarea } from "$lib/components/ui/textarea";
    import * as Select from "$lib/components/ui/select";
    import { Plus, Trash2, Calculator } from "lucide-svelte";
    import { type Indicator } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";

    let { initialData, onSave, onCancel } = $props<{
        initialData?: Indicator;
        onSave: (data: Omit<Indicator, "id">) => void;
        onCancel: () => void;
    }>();

    let formData = $state<Omit<Indicator, "id">>({
        name: initialData?.name ?? "",
        category: initialData?.category ?? "Trend",
        plot_type: initialData?.plot_type ?? "Overlay",
        default_color: initialData?.default_color ?? "#00ff00",
        usage_description: initialData?.usage_description ?? "",
        parameters: initialData?.parameters ? [...initialData.parameters] : [],
    });

    // Presets definition
    const presets = $derived({
        custom: {
            label: $t("settings.indicators.presets.custom"),
            category: "Trend" as const,
            plot: "Overlay" as const,
            color: "#00ff00",
            params: [],
        },
        sma: {
            label: $t("settings.indicators.presets.sma"),
            category: "Trend" as const,
            plot: "Overlay" as const,
            color: "#ffb700",
            params: [
                { key: $t("settings.indicators.paramName"), value: "20" },
            ],
        },
        ema: {
            label: $t("settings.indicators.presets.ema"),
            category: "Trend" as const,
            plot: "Overlay" as const,
            color: "#00e5ff",
            params: [
                { key: $t("settings.indicators.paramName"), value: "9" },
            ],
        },
        rsi: {
            label: $t("settings.indicators.presets.rsi"),
            category: "Oscillator" as const,
            plot: "SubWindow" as const,
            color: "#d946ef",
            params: [
                { key: $t("settings.indicators.paramName"), value: "14" },
            ],
        },
        macd: {
            label: $t("settings.indicators.presets.macd"),
            category: "Oscillator" as const,
            plot: "SubWindow" as const,
            color: "#f43f5e",
            params: [
                {
                    key: $t("settings.indicators.params.Fast"),
                    value: "12",
                },
                {
                    key: $t("settings.indicators.params.Slow"),
                    value: "26",
                },
                {
                    key: $t("settings.indicators.params.Signal"),
                    value: "9",
                },
            ],
        },
        bb: {
            label: $t("settings.indicators.presets.bb"),
            category: "Trend" as const,
            plot: "Overlay" as const,
            color: "#8b5cf6",
            params: [
                { key: $t("settings.indicators.paramName"), value: "20" },
                {
                    key: $t("settings.indicators.params.Deviation"),
                    value: "2.0",
                },
            ],
        },
        vol: {
            label: $t("settings.indicators.presets.vol"),
            category: "Volume" as const,
            plot: "SubWindow" as const,
            color: "#10b981",
            params: [],
        },
        vwap: {
            label: $t("settings.indicators.presets.vwap"),
            category: "Trend" as const,
            plot: "Overlay" as const,
            color: "#f59e0b",
            params: [
                {
                    key: $t("settings.indicators.params.Anchorage"),
                    value: $t("settings.indicators.params.Daily"),
                },
            ],
        },
    });

    let selectedPreset = $state<string>("custom");

    function applyPreset(value: string) {
        selectedPreset = value;
        if (value === "custom") return;

        const p = presets[value as keyof typeof presets];
        if (p) {
            formData.name = p.label.split("(")[0].trim(); // Get standard name part
            formData.category = p.category;
            formData.plot_type = p.plot;
            formData.default_color = p.color;
            // Create deep copy of params to avoid connection to preset object
            formData.parameters = p.params.map((param) => ({ ...param }));
        }
    }

    let categoryOptions = $derived([
        {
            value: "Trend",
            label: $t("settings.indicators.categories.Trend"),
        },
        {
            value: "Oscillator",
            label: $t("settings.indicators.categories.Oscillator"),
        },
        {
            value: "Volume",
            label: $t("settings.indicators.categories.Volume"),
        },
        {
            value: "Other",
            label: $t("settings.indicators.categories.Other"),
        },
    ]);

    let plotOptions = $derived([
        {
            value: "Overlay",
            label: $t("settings.indicators.plotTypes.Overlay"),
        },
        {
            value: "SubWindow",
            label: $t("settings.indicators.plotTypes.SubWindow"),
        },
    ]);

    function addParam() {
        formData.parameters.push({ key: "", value: "" });
    }

    function removeParam(index: number) {
        formData.parameters = formData.parameters.filter((_, i) => i !== index);
    }

    function save() {
        onSave(formData);
    }
</script>

<div class="space-y-4 py-4">
    <!-- Preset Selection -->
    <div
        class="p-3 mb-2 rounded-lg border border-primary/20 bg-primary/5 space-y-2"
    >
        <Label class="text-primary font-bold flex items-center gap-2">
            <Calculator class="w-4 h-4" />
            {$t("settings.indicators.quickModel")}
        </Label>
        <Select.Root
            type="single"
            value={selectedPreset}
            onValueChange={applyPreset}
        >
            <Select.Trigger class="bg-background/80">
                {presets[selectedPreset as keyof typeof presets]?.label ??
                    $t("settings.indicators.selectModel")}
            </Select.Trigger>
            <Select.Content>
                {#each Object.entries(presets) as [key, preset]}
                    <Select.Item value={key}>{preset.label}</Select.Item>
                {/each}
            </Select.Content>
        </Select.Root>
        <p class="text-[10px] text-muted-foreground ml-1">
            {$t("settings.indicators.modelHint")}
        </p>
    </div>

    <div class="space-y-2">
        <Label>{$t("settings.indicators.name")}</Label>
        <Input
            bind:value={formData.name}
            placeholder={$t("settings.indicators.namePlaceholder")}
        />
    </div>

    <div class="grid grid-cols-2 gap-4">
        <div class="space-y-2">
            <Label>{$t("settings.indicators.category")}</Label>
            <Select.Root type="single" bind:value={formData.category}>
                <Select.Trigger>
                    {categoryOptions.find((o) => o.value === formData.category)
                        ?.label ?? formData.category}
                </Select.Trigger>
                <Select.Content>
                    {#each categoryOptions as option}
                        <Select.Item value={option.value}
                            >{option.label}</Select.Item
                        >
                    {/each}
                </Select.Content>
            </Select.Root>
        </div>
        <div class="space-y-2">
            <Label>{$t("settings.indicators.plotType")}</Label>
            <Select.Root type="single" bind:value={formData.plot_type}>
                <Select.Trigger>
                    {plotOptions.find((o) => o.value === formData.plot_type)
                        ?.label ?? formData.plot_type}
                </Select.Trigger>
                <Select.Content>
                    {#each plotOptions as option}
                        <Select.Item value={option.value}
                            >{option.label}</Select.Item
                        >
                    {/each}
                </Select.Content>
            </Select.Root>
        </div>
    </div>

    <div class="space-y-2">
        <Label>{$t("settings.indicators.defaultColor")}</Label>
        <div class="flex items-center gap-4 p-2 border rounded-md">
            <input
                type="color"
                bind:value={formData.default_color}
                class="h-10 w-16 bg-transparent border-0 rounded cursor-pointer"
            />
            <div
                class="flex-1 px-3 py-2 rounded text-xs font-bold text-center transition-colors"
                style="background-color: {formData.default_color}25; color: {formData.default_color}; border: 1px solid {formData.default_color}50;"
            >
                {$t("settings.indicators.previewColor")}
            </div>
        </div>
    </div>

    <div class="space-y-2">
        <Label>{$t("settings.indicators.usage")}</Label>
        <Textarea
            bind:value={formData.usage_description}
            placeholder={$t("settings.indicators.usagePlaceholder")}
            class="min-h-[80px]"
        />
    </div>

    <div class="space-y-2 border-t pt-4">
        <div class="flex items-center justify-between mb-2">
            <Label>{$t("settings.indicators.parameters")}</Label>
            <Button variant="ghost" size="sm" onclick={addParam}>
                <Plus class="w-4 h-4 mr-2" />
                {$t("settings.indicators.add")}
            </Button>
        </div>

        {#if formData.parameters.length === 0}
            <div
                class="p-4 border border-dashed rounded-lg text-center bg-muted/30"
            >
                <p class="text-sm text-muted-foreground">
                    {$t("settings.indicators.noParams")}
                </p>
                <Button variant="link" size="sm" onclick={addParam}
                    >{$t("settings.indicators.addFirst")}</Button
                >
            </div>
        {:else}
            <div class="space-y-2">
                {#each formData.parameters as param, i}
                    <div
                        class="flex gap-2 items-center animate-in slide-in-from-left-2 duration-200"
                    >
                        <div class="flex-1 space-y-1">
                            <span class="text-[10px] text-muted-foreground ml-1"
                                >{$t(
                                    "settings.indicators.paramNameLabel",
                                )}</span
                            >
                            <Input
                                bind:value={param.key}
                                placeholder={$t(
                                    "settings.indicators.paramNamePlaceholder",
                                )}
                                class="h-8 text-sm"
                            />
                        </div>
                        <div class="w-24 space-y-1">
                            <span class="text-[10px] text-muted-foreground ml-1"
                                >{$t(
                                    "settings.indicators.paramValue",
                                )}</span
                            >
                            <Input
                                bind:value={param.value}
                                placeholder="14"
                                class="h-8 text-sm"
                            />
                        </div>
                        <div class="pt-4">
                            <Button
                                variant="ghost"
                                size="icon"
                                class="h-8 w-8 text-muted-foreground hover:text-destructive"
                                onclick={() => removeParam(i)}
                            >
                                <Trash2 class="w-4 h-4" />
                            </Button>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <div class="flex justify-end gap-2 pt-4 border-t mt-2">
        <Button variant="outline" onclick={onCancel}
            >{$t("general.cancel")}</Button
        >
        <Button onclick={save}>{$t("general.save")}</Button>
    </div>
</div>
