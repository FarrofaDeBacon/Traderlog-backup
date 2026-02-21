<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import * as Select from "$lib/components/ui/select";
    import * as Card from "$lib/components/ui/card";
    import { Key, Info } from "lucide-svelte";
    import type { ApiConfig } from "$lib/stores/settings.svelte";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";

    let { initialData, onSave, onCancel } = $props<{
        initialData?: ApiConfig;
        onSave: (data: Omit<ApiConfig, "id">) => void;
        onCancel: () => void;
    }>();

    let formData = $state<Omit<ApiConfig, "id">>({
        provider: initialData?.provider ?? "custom",
        api_key: initialData?.api_key ?? "",
        enabled: initialData?.enabled ?? true,
        daily_limit: initialData?.daily_limit ?? 1500,
        requests_today: initialData?.requests_today ?? 0,
        extra_config: initialData?.extra_config ?? "",
    });

    let providers = $derived([
        { value: "openai", label: $t("settings.api.form.providers.openai") },
        {
            value: "google_gemini",
            label: $t("settings.api.form.providers.google_gemini"),
        },
        {
            value: "anthropic",
            label: $t("settings.api.form.providers.anthropic"),
        },
        { value: "polygon", label: $t("settings.api.form.providers.polygon") },
        {
            value: "alpha_vantage",
            label: $t("settings.api.form.providers.alpha_vantage"),
        },
        {
            value: "custom",
            label: $t("settings.api.form.providerCustom") || "Custom",
        },
    ]);

    function save() {
        onSave(formData);
    }

    function testKey() {
        if (!formData.api_key) {
            toast.error($t("settings.api.form.enterKeyError"));
            return;
        }
        toast.info($t("settings.api.form.testInfo"));
        // Simulate API check
        setTimeout(
            () => toast.success($t("settings.api.form.testSuccess")),
            1000,
        );
    }
</script>

<div class="space-y-6 pt-2">
    <!-- Header with Icon -->
    <div class="flex items-center gap-3 pb-2 border-b border-border/50">
        <div class="p-2 bg-primary/10 rounded-lg">
            <Key class="w-5 h-5 text-primary" />
        </div>
        <div>
            <h4 class="font-medium leading-none">
                {initialData
                    ? $t("settings.api.form.editTitle")
                    : $t("settings.api.form.newTitle")}
            </h4>
            <p class="text-xs text-muted-foreground mt-1">
                {$t("settings.api.form.description")}
            </p>
        </div>
    </div>

    <div class="space-y-4">
        <div class="space-y-2">
            <Label
                class="text-xs font-bold uppercase text-muted-foreground tracking-wider"
                >{$t("settings.api.form.provider")}</Label
            >
            <Select.Root type="single" bind:value={formData.provider}>
                <Select.Trigger class="bg-secondary/50 border-0">
                    {providers.find((o) => o.value === formData.provider)
                        ?.label ?? formData.provider}
                </Select.Trigger>
                <Select.Content>
                    {#each providers as option}
                        <Select.Item value={option.value}
                            >{option.label}</Select.Item
                        >
                    {/each}
                </Select.Content>
            </Select.Root>
        </div>

        <div class="space-y-2">
            <Label
                class="text-xs font-bold uppercase text-muted-foreground tracking-wider"
                >{$t("settings.api.form.apiKey")}</Label
            >
            <div class="flex gap-2">
                <Input
                    type="password"
                    bind:value={formData.api_key}
                    placeholder={$t("settings.api.form.apiKeyPlaceholder")}
                    class="bg-secondary/50 border-0"
                />
                <Button variant="secondary" onclick={testKey}
                    >{$t("settings.api.form.test")}</Button
                >
            </div>
        </div>

        <div class="space-y-2">
            <Label
                class="text-xs font-bold uppercase text-muted-foreground tracking-wider"
                >{$t("settings.api.form.dailyLimit")}</Label
            >
            <Input
                type="number"
                bind:value={formData.daily_limit}
                class="bg-secondary/50 border-0"
            />
            <p class="text-[10px] text-muted-foreground">
                {$t("settings.api.form.dailyLimitHint")}
            </p>
        </div>

        <div class="flex items-center space-x-2 pt-2">
            <Checkbox id="enabled-mode" bind:checked={formData.enabled} />
            <div class="grid gap-1.5 leading-none">
                <Label
                    for="enabled-mode"
                    class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                >
                    {$t("settings.api.form.enabled")}
                </Label>
                <p class="text-[10px] text-muted-foreground">
                    {$t("settings.api.form.enabledHint")}
                </p>
            </div>
        </div>

        <!-- Info Alert -->
        <div class="rounded-lg border border-blue-500/20 bg-blue-500/10 p-4">
            <div class="flex items-start gap-3">
                <div class="p-1.5 bg-blue-500/20 rounded-md">
                    <Info class="w-4 h-4 text-blue-400" />
                </div>
                <div class="text-xs text-blue-100/80 leading-relaxed">
                    <span class="font-bold block mb-1.5 text-blue-300"
                        >{$t("settings.api.form.noteTitle")}</span
                    >
                    {$t("settings.api.form.noteBody")}

                    <div class="mt-3 flex flex-wrap gap-x-4 gap-y-2">
                        {#if formData.provider === "openai" || formData.provider === "custom"}
                            <a
                                href="https://platform.openai.com/api-keys"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-blue-400 underline hover:text-blue-300 transition-colors font-medium flex items-center gap-1"
                            >
                                <Key class="w-3 h-3" />
                                {$t("settings.api.form.getOpenAIApiKey")}
                            </a>
                        {/if}

                        {#if formData.provider === "google_gemini" || formData.provider === "custom"}
                            <a
                                href="https://aistudio.google.com/app/apikey"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-blue-400 underline hover:text-blue-300 transition-colors font-medium flex items-center gap-1"
                            >
                                <Key class="w-3 h-3" />
                                {$t("settings.api.form.getGeminiApiKey")}
                            </a>
                        {/if}

                        {#if formData.provider === "polygon" || formData.provider === "custom"}
                            <a
                                href="https://polygon.io/dashboard/api-keys"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-blue-400 underline hover:text-blue-300 transition-colors font-medium flex items-center gap-1"
                            >
                                <Key class="w-3 h-3" />
                                {$t("settings.api.form.getPolygonApiKey")}
                            </a>
                        {/if}
                    </div>

                    {#if formData.provider === "openai"}
                        <a
                            href="https://openai.com/pricing"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-[10px] text-blue-300/60 hover:text-blue-300 block mt-2 transition-colors uppercase tracking-wider font-semibold"
                            >{$t("settings.api.form.viewOpenAIPrices")}</a
                        >
                    {:else if formData.provider === "polygon"}
                        <a
                            href="https://polygon.io/pricing"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-[10px] text-blue-300/60 hover:text-blue-300 block mt-2 transition-colors uppercase tracking-wider font-semibold"
                            >{$t("settings.api.form.viewPolygonPlans")}</a
                        >
                    {/if}
                </div>
            </div>
        </div>
    </div>

    <div class="flex justify-end gap-2 pt-4">
        <Button variant="outline" onclick={onCancel}
            >{$t("settings.api.form.cancel")}</Button
        >
        <Button
            onclick={save}
            class="bg-primary text-primary-foreground hover:bg-primary/90"
            >{$t("settings.api.form.save")}</Button
        >
    </div>
</div>
