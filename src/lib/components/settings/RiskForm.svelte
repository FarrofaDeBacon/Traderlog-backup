<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import {
        Shield,
        Target,
        Lock,
        AlertTriangle,
        TrendingUp,
        Plus,
        Trash2,
        Zap,
        Brain,
        Target as TargetIcon,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import type { RiskProfile, GrowthPhase } from "$lib/types";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Card from "$lib/components/ui/card";

    import { Badge } from "$lib/components/ui/badge";

    let { initialData, onSave, onCancel } = $props<{
        initialData?: RiskProfile;
        onSave: (data: Omit<RiskProfile, "id">) => void;
        onCancel: () => void;
    }>();

    // Presets Configuration
    const presets = $derived({
        conservative: {
            name: $t("settings.risk.form.presets.conservative"),
            max_daily_loss: 500,
            daily_target: 300,
            max_risk_per_trade_percent: 1.0,
            max_trades_per_day: 3,
            min_risk_reward: 2.0,
            lock_on_loss: true,
        },
        moderate: {
            name: $t("settings.risk.form.presets.moderate"),
            max_daily_loss: 1000,
            daily_target: 800,
            max_risk_per_trade_percent: 2.0,
            max_trades_per_day: 5,
            min_risk_reward: 1.5,
            lock_on_loss: true,
        },
        aggressive: {
            name: $t("settings.risk.form.presets.aggressive"),
            max_daily_loss: 2000,
            daily_target: 2000,
            max_risk_per_trade_percent: 5.0,
            max_trades_per_day: 10,
            min_risk_reward: 1.0,
            lock_on_loss: false,
        },
    });

    let selectedPreset = $state<string>("custom");

    let formData = $state<Omit<RiskProfile, "id">>({
        name: initialData?.name ?? "",
        max_daily_loss: initialData?.max_daily_loss ?? 0,
        daily_target: initialData?.daily_target ?? 0,
        max_risk_per_trade_percent:
            initialData?.max_risk_per_trade_percent ?? 1.0,
        max_trades_per_day: initialData?.max_trades_per_day ?? 5,
        min_risk_reward: initialData?.min_risk_reward ?? 1.5,
        lock_on_loss: initialData?.lock_on_loss ?? false,
        account_type_applicability:
            initialData?.account_type_applicability ?? "All",
        growth_plan_enabled: initialData?.growth_plan_enabled ?? false,
        current_phase_index: initialData?.current_phase_index ?? 0,
        growth_phases: initialData?.growth_phases ?? [],
        psychological_coupling_enabled:
            initialData?.psychological_coupling_enabled ?? false,
        outlier_regression_enabled:
            initialData?.outlier_regression_enabled ?? false,
        sniper_mode_enabled: initialData?.sniper_mode_enabled ?? false,
        sniper_mode_selectivity: initialData?.sniper_mode_selectivity ?? 3,
        psychological_lookback_count:
            initialData?.psychological_lookback_count ?? 10,
        outlier_lookback_count: initialData?.outlier_lookback_count ?? 20,
        psychological_threshold: initialData?.psychological_threshold ?? -2,
        lot_reduction_multiplier: initialData?.lot_reduction_multiplier ?? 0.5,
        psychological_search_strategy:
            initialData?.psychological_search_strategy ?? "Strict",
        account_ids: initialData?.account_ids ?? [],
        active: initialData?.active ?? false,
    });

    function applyPreset(key: string) {
        selectedPreset = key;
        if (key === "custom") return;

        const p = presets[key as keyof typeof presets];
        formData.name = p.name;
        formData.max_daily_loss = p.max_daily_loss;
        formData.daily_target = p.daily_target;
        formData.max_risk_per_trade_percent = p.max_risk_per_trade_percent;
        formData.max_trades_per_day = p.max_trades_per_day;
        formData.min_risk_reward = p.min_risk_reward;
        formData.lock_on_loss = p.lock_on_loss;

        // Auto-apply corresponding growth plan
        if (growthPresets[key as keyof typeof growthPresets]) {
            applyGrowthPreset(key);
            formData.growth_plan_enabled = true;
        }
    }

    $effect(() => {
        // Only update if initialData changes significantly or logic requires it.
        // For simple forms, often only init is enough, but keeping existing pattern.
        if (initialData) {
            let fd = { ...initialData };
            formData = {
                id: fd.id,
                name: fd.name,
                max_daily_loss: fd.max_daily_loss,
                daily_target: fd.daily_target,
                max_risk_per_trade_percent: fd.max_risk_per_trade_percent,
                max_trades_per_day: fd.max_trades_per_day,
                min_risk_reward: fd.min_risk_reward,
                lock_on_loss: fd.lock_on_loss,
                account_type_applicability: fd.account_type_applicability,
                growth_plan_enabled: fd.growth_plan_enabled ?? false,
                current_phase_index: fd.current_phase_index ?? 0,
                growth_phases: fd.growth_phases ? [...fd.growth_phases] : [],
                psychological_coupling_enabled:
                    fd.psychological_coupling_enabled ?? false,
                outlier_regression_enabled:
                    fd.outlier_regression_enabled ?? false,
                sniper_mode_enabled: fd.sniper_mode_enabled ?? false,
                sniper_mode_selectivity: fd.sniper_mode_selectivity ?? 3,
                psychological_lookback_count:
                    fd.psychological_lookback_count ?? 10,
                outlier_lookback_count: fd.outlier_lookback_count ?? 20,
                psychological_threshold: fd.psychological_threshold ?? -2,
                lot_reduction_multiplier: fd.lot_reduction_multiplier ?? 0.5,
                psychological_search_strategy:
                    fd.psychological_search_strategy ?? "Strict",
                account_ids: fd.account_ids ?? [],
                active: fd.active ?? false,
            };
            selectedPreset = "custom";
        }
    });

    const accountTypes = $derived([
        {
            value: "All",
            label: $t("settings.risk.accountTypes.All") || "Todas as Contas",
        },
        {
            value: "Prop",
            label: $t("settings.risk.accountTypes.Prop") || "Mesa Proprietária",
        },
        {
            value: "Real",
            label: $t("settings.risk.accountTypes.Real") || "Conta Real",
        },
        {
            value: "Demo",
            label: $t("settings.risk.accountTypes.Demo") || "Conta Demo",
        },
        {
            value: "Specific",
            label:
                $t("settings.risk.accountTypes.Specific") ||
                "Specific Accounts",
        },
    ]);

    import { settingsStore } from "$lib/stores/settings.svelte";

    // Growth Plan Presets
    const growthPresets = $derived({
        conservative: {
            name: $t("settings.risk.form.presets.conservative"),
            description: $t("settings.risk.growthPlan.enableDesc"),
            phases: Array.from({ length: 5 }, (_, i) => ({
                id: crypto.randomUUID(),
                name: `${$t("general.items")} ${i + 1}`,
                description: `Fase ${i + 1}`,
                max_lots: i + 1,
                max_daily_loss: (i + 1) * 200, // Grows 200 per level
                progression_rules: [
                    { condition: "days_positive" as const, value: 5 },
                ],
                regression_rules: [
                    {
                        condition: "drawdown_limit" as const,
                        value: (i + 1) * 100,
                    },
                ],
            })),
        },
        moderate: {
            name: $t("settings.risk.form.presets.moderate"),
            description: $t("settings.risk.growthPlan.enableDesc"),
            phases: Array.from({ length: 8 }, (_, i) => ({
                id: crypto.randomUUID(),
                name: `${$t("general.items")} ${i + 1}`,
                description: `Fase ${i + 1}`,
                max_lots: (i + 1) * 2, // Grows by 2 lots
                max_daily_loss: (i + 1) * 500,
                progression_rules: [
                    { condition: "days_positive" as const, value: 3 },
                ],
                regression_rules: [
                    { condition: "max_daily_loss_streak" as const, value: 1 },
                ],
            })),
        },
        aggressive: {
            name: $t("settings.risk.form.presets.aggressive"),
            description: $t("settings.risk.growthPlan.enableDesc"),
            phases: Array.from({ length: 10 }, (_, i) => ({
                id: crypto.randomUUID(),
                name: `${$t("general.items")} ${i + 1}`,
                description: `Fase ${i + 1}`,
                max_lots: (i + 1) * 5, // Grows fast
                max_daily_loss: (i + 1) * 1000,
                progression_rules: [
                    { condition: "days_positive" as const, value: 2 },
                ],
                regression_rules: [
                    {
                        condition: "drawdown_limit" as const,
                        value: (i + 1) * 800,
                    },
                ],
            })),
        },
    });

    let selectedGrowthPreset = $state<string>("custom");

    function applyGrowthPreset(key: string) {
        selectedGrowthPreset = key;
        if (key === "custom") return;

        const p = growthPresets[key as keyof typeof growthPresets];
        // Keep ID if exists, or generate new ones? Prefer new to avoid conflicts if saving multiple profiles
        // Actually, we replace the whole array
        formData.growth_phases = p.phases.map((phase) => ({
            ...phase,
            id: crypto.randomUUID(),
        }));
    }

    function addPhase() {
        formData.growth_phases = [
            ...formData.growth_phases,
            {
                id: crypto.randomUUID(),
                name: `${$t("settings.risk.growthPlan.phases")} ${formData.growth_phases.length}`,
                description: "",
                max_lots: 1,
                max_daily_loss: formData.max_daily_loss,
                progression_rules: [],
                regression_rules: [],
            },
        ];
        selectedGrowthPreset = "custom";
    }

    function removePhase(index: number) {
        formData.growth_phases = formData.growth_phases.filter(
            (_, i) => i !== index,
        );
        selectedGrowthPreset = "custom";
    }

    function save() {
        onSave(formData);
    }
</script>

<div class="space-y-4 py-4">
    <!-- Presets Selection -->
    <div class="space-y-2">
        <Label class="text-xs text-muted-foreground uppercase font-bold"
            >{$t("settings.risk.form.presets.title")}</Label
        >
        <div class="flex flex-wrap gap-2">
            {#each Object.entries(presets) as [key, p]}
                <button
                    class="px-3 py-1.5 rounded-full text-xs font-medium border transition-all {selectedPreset ===
                    key
                        ? 'bg-primary text-primary-foreground border-primary'
                        : 'bg-muted/40 text-muted-foreground hover:bg-muted border-transparent'}"
                    onclick={() => applyPreset(key)}
                >
                    {p.name}
                </button>
            {/each}
            <button
                class="px-3 py-1.5 rounded-full text-xs font-medium border transition-all {selectedPreset ===
                'custom'
                    ? 'bg-primary text-primary-foreground border-primary'
                    : 'bg-muted/40 text-muted-foreground hover:bg-muted border-transparent'}"
                onclick={() => (selectedPreset = "custom")}
            >
                {$t("general.custom") || "Personalizado"}
            </button>
        </div>
    </div>

    <div class="space-y-2">
        <Label>{$t("settings.risk.name")}</Label>
        <Input
            bind:value={formData.name}
            placeholder={$t("settings.risk.namePlaceholder")}
            oninput={() => (selectedPreset = "custom")}
        />
    </div>

    <Tabs.Root value="general" class="w-full">
        <Tabs.List class="grid w-full grid-cols-3">
            <Tabs.Trigger value="general"
                >{$t("settings.risk.form.tabs.general")}</Tabs.Trigger
            >
            <Tabs.Trigger value="risk-engine"
                >{$t("settings.risk.form.tabs.engine")}</Tabs.Trigger
            >
            <Tabs.Trigger value="growth"
                >{$t("settings.risk.form.tabs.growth")}</Tabs.Trigger
            >
        </Tabs.List>

        <Tabs.Content value="general" class="space-y-4 pt-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <!-- Downside Protection -->
                <div
                    class="space-y-4 p-4 rounded-lg border border-red-500/20 bg-red-500/5"
                >
                    <h3 class="flex items-center gap-2 font-bold text-red-500">
                        <Shield class="w-4 h-4" />
                        {$t("settings.risk.downside")}
                    </h3>
                    <div class="space-y-2">
                        <Label>{$t("settings.risk.dailyLossLimit")}</Label>
                        <Input
                            type="number"
                            bind:value={formData.max_daily_loss}
                        />
                    </div>
                    <div class="space-y-2">
                        <Label>{$t("settings.risk.maxRiskPerTrade")}</Label>
                        <Input
                            type="number"
                            step="0.1"
                            bind:value={formData.max_risk_per_trade_percent}
                        />
                    </div>
                </div>

                <!-- Upside Targets -->
                <div
                    class="space-y-4 p-4 rounded-lg border border-green-500/20 bg-green-500/5"
                >
                    <h3
                        class="flex items-center gap-2 font-bold text-green-500"
                    >
                        <Target class="w-4 h-4" />
                        {$t("settings.risk.upside")}
                    </h3>
                    <div class="space-y-2">
                        <Label>{$t("settings.risk.dailyGoal")}</Label>
                        <Input
                            type="number"
                            bind:value={formData.daily_target}
                        />
                    </div>
                    <div class="space-y-2">
                        <Label>{$t("settings.risk.minRiskReward")}</Label>
                        <Input
                            type="number"
                            step="0.1"
                            bind:value={formData.min_risk_reward}
                        />
                    </div>
                </div>
            </div>

            <!-- Discipline -->
            <div
                class="space-y-4 p-4 rounded-lg border border-muted bg-muted/20"
            >
                <h3
                    class="flex items-center gap-2 font-bold text-muted-foreground"
                >
                    <Lock class="w-4 h-4" />
                    {$t("settings.risk.discipline")}
                </h3>
                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-2">
                        <Label>{$t("settings.risk.maxTradesDay")}</Label>
                        <Input
                            type="number"
                            bind:value={formData.max_trades_per_day}
                        />
                    </div>
                    <div class="space-y-2">
                        <Label>{$t("settings.risk.applicability")}</Label>
                        <Select.Root
                            type="single"
                            bind:value={formData.account_type_applicability}
                        >
                            <Select.Trigger>
                                {accountTypes.find(
                                    (t) =>
                                        t.value ===
                                        formData.account_type_applicability,
                                )?.label ?? formData.account_type_applicability}
                            </Select.Trigger>
                            <Select.Content>
                                {#each accountTypes as type}
                                    <Select.Item value={type.value}
                                        >{type.label}</Select.Item
                                    >
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>
                </div>

                {#if formData.account_type_applicability === "Specific"}
                    <div
                        class="space-y-2 pt-2 border-t animate-in fade-in slide-in-from-top-1"
                    >
                        <Label class="text-xs font-semibold"
                            >{$t("settings.risk.form.accounts.title")}</Label
                        >
                        <div class="grid grid-cols-2 gap-2">
                            {#each settingsStore.accounts as account}
                                <div
                                    class="flex items-center space-x-2 p-2 rounded border bg-background/50"
                                >
                                    <Switch
                                        id="acc-{account.id}"
                                        checked={formData.account_ids.includes(
                                            account.id,
                                        )}
                                        onCheckedChange={(checked) => {
                                            if (checked) {
                                                formData.account_ids = [
                                                    ...formData.account_ids,
                                                    account.id,
                                                ];
                                            } else {
                                                formData.account_ids =
                                                    formData.account_ids.filter(
                                                        (id) =>
                                                            id !== account.id,
                                                    );
                                            }
                                        }}
                                    />
                                    <Label
                                        for="acc-{account.id}"
                                        class="text-xs cursor-pointer truncate"
                                    >
                                        {account.nickname}
                                    </Label>
                                </div>
                            {/each}
                        </div>
                        {#if formData.account_ids.length === 0}
                            <p class="text-[10px] text-amber-500 italic">
                                {$t(
                                    "settings.risk.form.accounts.noneSelected",
                                ) ||
                                    "Nenhuma conta selecionada. O perfil não será aplicado a nenhuma conta."}
                            </p>
                        {/if}
                    </div>
                {/if}

                <div class="flex items-center space-x-4 pt-2">
                    <div class="flex items-center space-x-2">
                        <Switch
                            id="lock-mode"
                            bind:checked={formData.lock_on_loss}
                        />
                        <Label for="lock-mode"
                            >{$t("settings.risk.lockOnLoss")}</Label
                        >
                    </div>
                </div>
                {#if formData.lock_on_loss}
                    <p class="text-xs text-red-400 flex items-center gap-1">
                        <AlertTriangle class="w-3 h-3" />
                        {$t("settings.risk.lockWarning")}
                    </p>
                {/if}
            </div>
        </Tabs.Content>

        <Tabs.Content value="risk-engine" class="space-y-4 pt-4">
            <!-- Psychological Coupling -->
            <div
                class="space-y-4 p-4 rounded-lg border border-indigo-500/20 bg-indigo-500/5"
            >
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <Brain class="w-5 h-5 text-indigo-400" />
                        <div class="space-y-0.5">
                            <h4 class="font-bold text-indigo-400">
                                {$t("settings.risk.engine.psychological.title")}
                            </h4>
                            <p
                                class="text-[10px] text-muted-foreground uppercase tracking-tight"
                            >
                                {$t(
                                    "settings.risk.engine.psychological.description",
                                    {
                                        values: {
                                            count: formData.psychological_lookback_count,
                                        },
                                    },
                                )}
                            </p>
                        </div>
                    </div>
                    <Switch
                        bind:checked={formData.psychological_coupling_enabled}
                    />
                </div>
                {#if formData.psychological_coupling_enabled}
                    <div class="grid grid-cols-2 gap-4 pt-2">
                        <div class="space-y-2">
                            <Label class="text-xs"
                                >{$t(
                                    "settings.risk.engine.psychological.strategy",
                                )}</Label
                            >
                            <Select.Root
                                type="single"
                                bind:value={
                                    formData.psychological_search_strategy
                                }
                            >
                                <Select.Trigger class="h-8 text-xs">
                                    {formData.psychological_search_strategy ===
                                    "Strict"
                                        ? $t(
                                              "settings.risk.engine.psychological.strategyStrict",
                                          )
                                        : $t(
                                              "settings.risk.engine.psychological.strategySequence",
                                          )}
                                </Select.Trigger>
                                <Select.Content>
                                    <Select.Item value="Strict" class="text-xs"
                                        >{$t(
                                            "settings.risk.engine.psychological.strategyStrict",
                                        )}</Select.Item
                                    >
                                    <Select.Item
                                        value="Sequence"
                                        class="text-xs"
                                        >{$t(
                                            "settings.risk.engine.psychological.strategySequence",
                                        )}</Select.Item
                                    >
                                </Select.Content>
                            </Select.Root>
                        </div>
                        <div class="space-y-2">
                            <Label class="text-xs"
                                >{$t(
                                    "settings.risk.engine.psychological.lookback",
                                )}</Label
                            >
                            <Input
                                type="number"
                                bind:value={
                                    formData.psychological_lookback_count
                                }
                                class="h-8 text-xs"
                            />
                        </div>
                        <div class="space-y-2">
                            <Label class="text-xs"
                                >{$t(
                                    "settings.risk.engine.psychological.threshold",
                                )}</Label
                            >
                            <Input
                                type="number"
                                bind:value={formData.psychological_threshold}
                                class="h-8 text-xs"
                            />
                        </div>
                        <div class="space-y-2">
                            <Label class="text-xs"
                                >{$t(
                                    "settings.risk.engine.psychological.multiplier",
                                )}</Label
                            >
                            <Input
                                type="number"
                                step="0.1"
                                bind:value={formData.lot_reduction_multiplier}
                                class="h-8 text-xs"
                            />
                        </div>
                    </div>
                {/if}
            </div>

            <!-- Outlier Regression -->
            <div
                class="space-y-4 p-4 rounded-lg border border-amber-500/20 bg-amber-500/5"
            >
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <AlertTriangle class="w-5 h-5 text-amber-500" />
                        <div class="space-y-0.5">
                            <h4 class="font-bold text-amber-500">
                                {$t("settings.risk.engine.outlier.title")}
                            </h4>
                            <p
                                class="text-[10px] text-muted-foreground uppercase tracking-tight"
                            >
                                {$t("settings.risk.engine.outlier.description")}
                            </p>
                        </div>
                    </div>
                    <Switch
                        bind:checked={formData.outlier_regression_enabled}
                    />
                </div>
                {#if formData.outlier_regression_enabled}
                    <div class="pt-2">
                        <Label class="text-xs"
                            >{$t(
                                "settings.risk.engine.outlier.lookback",
                            )}</Label
                        >
                        <Input
                            type="number"
                            bind:value={formData.outlier_lookback_count}
                            class="h-8 text-xs w-24"
                        />
                    </div>
                {/if}
            </div>

            <!-- Sniper Mode -->
            <div
                class="space-y-4 p-4 rounded-lg border border-blue-500/20 bg-blue-500/5"
            >
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <Zap class="w-5 h-5 text-blue-400" />
                        <div class="space-y-0.5">
                            <h4 class="font-bold text-blue-400">
                                {$t("settings.risk.engine.sniper.title")}
                            </h4>
                            <p
                                class="text-[10px] text-muted-foreground uppercase tracking-tight"
                            >
                                {$t("settings.risk.engine.sniper.description")}
                            </p>
                        </div>
                    </div>
                    <Switch bind:checked={formData.sniper_mode_enabled} />
                </div>
                {#if formData.sniper_mode_enabled}
                    <div class="space-y-2 pt-2 animate-in fade-in duration-300">
                        <Label class="text-xs"
                            >{$t(
                                "settings.risk.engine.sniper.selectivity",
                            )}</Label
                        >
                        <div class="flex items-center gap-4">
                            <Input
                                type="number"
                                bind:value={formData.sniper_mode_selectivity}
                                class="w-24 h-8 text-sm"
                            />
                            <p class="text-[10px] text-muted-foreground italic">
                                {$t(
                                    "settings.risk.engine.sniper.selectivityTip",
                                )}
                            </p>
                        </div>
                    </div>
                {/if}
            </div>
        </Tabs.Content>

        <Tabs.Content value="growth" class="space-y-4 pt-4">
            <div
                class="flex items-center justify-between p-4 rounded-lg border bg-card"
            >
                <div class="flex items-center gap-2">
                    <TrendingUp class="w-5 h-5 text-primary" />
                    <div class="space-y-0.5">
                        <h4 class="font-medium text-sm">
                            {$t("settings.risk.growthPlan.enable")}
                        </h4>
                        <p class="text-xs text-muted-foreground">
                            {$t("settings.risk.growthPlan.enableDesc")}
                        </p>
                    </div>
                </div>
                <Switch bind:checked={formData.growth_plan_enabled} />
            </div>

            {#if formData.growth_plan_enabled}
                <div class="space-y-4">
                    <!-- Growth Presets Selection -->
                    <div
                        class="space-y-2 p-3 bg-muted/20 rounded-lg border border-dashed"
                    >
                        <Label
                            class="text-xs text-muted-foreground uppercase font-bold"
                            >{$t("settings.risk.form.presets.title")}</Label
                        >
                        <div class="flex flex-wrap gap-2">
                            {#each Object.entries(growthPresets) as [key, p]}
                                <button
                                    class="px-3 py-1.5 rounded-full text-xs font-medium border transition-all {selectedGrowthPreset ===
                                    key
                                        ? 'bg-primary text-primary-foreground border-primary'
                                        : 'bg-background hover:bg-muted border-input'}"
                                    onclick={() => applyGrowthPreset(key)}
                                    title={p.description}
                                >
                                    {p.name}
                                </button>
                            {/each}
                            <button
                                class="px-3 py-1.5 rounded-full text-xs font-medium border transition-all {selectedGrowthPreset ===
                                'custom'
                                    ? 'bg-primary text-primary-foreground border-primary'
                                    : 'bg-background hover:bg-muted border-input'}"
                                onclick={() =>
                                    (selectedGrowthPreset = "custom")}
                            >
                                {$t("general.custom") || "Personalizado"}
                            </button>
                        </div>
                        {#if selectedGrowthPreset !== "custom"}
                            <p class="text-xs text-muted-foreground italic">
                                {growthPresets[
                                    selectedGrowthPreset as keyof typeof growthPresets
                                ].description}
                            </p>
                        {/if}
                    </div>

                    <div class="flex items-center justify-between">
                        <h4 class="text-sm font-medium">
                            {$t("settings.risk.growthPlan.phases")} ({formData
                                .growth_phases.length})
                        </h4>
                        <Button size="sm" variant="outline" onclick={addPhase}>
                            <Plus class="w-3 h-3 mr-1" />
                            {$t("settings.risk.growthPlan.addPhase")}
                        </Button>
                    </div>

                    <div class="grid gap-3 max-h-[400px] overflow-y-auto pr-2">
                        {#each formData.growth_phases as phase, i}
                            <Card.Root class="relative">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="absolute top-2 right-2 h-6 w-6 text-muted-foreground hover:text-destructive"
                                    onclick={() => removePhase(i)}
                                >
                                    <Trash2 class="w-3 h-3" />
                                </Button>
                                <Card.Header class="p-3 pb-0">
                                    <div class="flex items-center gap-2">
                                        <div
                                            class="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center text-xs font-bold text-primary"
                                        >
                                            {i}
                                        </div>
                                        <Input
                                            class="h-7 text-sm font-medium border-0 px-0 focus-visible:ring-0"
                                            bind:value={phase.name}
                                        />
                                    </div>
                                </Card.Header>
                                <Card.Content class="p-3 pt-2 space-y-3">
                                    <div class="grid grid-cols-2 gap-3">
                                        <div class="space-y-1">
                                            <Label class="text-[10px]"
                                                >{$t(
                                                    "settings.risk.growthPlan.maxLots",
                                                )}</Label
                                            >
                                            <Input
                                                type="number"
                                                class="h-7 text-xs"
                                                bind:value={phase.max_lots}
                                            />
                                        </div>
                                        <div class="space-y-1">
                                            <Label class="text-[10px]"
                                                >{$t(
                                                    "settings.risk.growthPlan.dailyLoss",
                                                )}</Label
                                            >
                                            <Input
                                                type="number"
                                                class="h-7 text-xs"
                                                bind:value={
                                                    phase.max_daily_loss
                                                }
                                            />
                                        </div>
                                    </div>

                                    <!-- Rules Summary (Simplified for now) -->
                                    <div
                                        class="grid grid-cols-2 gap-3 pt-2 border-t text-xs text-muted-foreground"
                                    >
                                        <div>
                                            <span
                                                class="font-semibold block text-green-500"
                                                >{$t(
                                                    "settings.risk.growthPlan.progression",
                                                )}</span
                                            >
                                            <div
                                                class="flex items-center gap-1 mt-1"
                                            >
                                                <span
                                                    >{$t(
                                                        "settings.risk.growthPlan.profitTarget",
                                                    )}</span
                                                >
                                                <Input
                                                    type="number"
                                                    class="h-6 w-16 px-1 text-xs"
                                                    value={phase.progression_rules.find(
                                                        (r) =>
                                                            r.condition ===
                                                            "profit_target",
                                                    )?.value ?? 0}
                                                    oninput={(e) => {
                                                        const val = Number(
                                                            e.currentTarget
                                                                .value,
                                                        );
                                                        const idx =
                                                            phase.progression_rules.findIndex(
                                                                (r) =>
                                                                    r.condition ===
                                                                    "profit_target",
                                                            );
                                                        if (val > 0) {
                                                            if (idx >= 0)
                                                                phase.progression_rules[
                                                                    idx
                                                                ].value = val;
                                                            else
                                                                phase.progression_rules.push(
                                                                    {
                                                                        condition:
                                                                            "profit_target",
                                                                        value: val,
                                                                    },
                                                                );
                                                        } else if (idx >= 0) {
                                                            phase.progression_rules.splice(
                                                                idx,
                                                                1,
                                                            );
                                                        }
                                                    }}
                                                />
                                            </div>
                                            <div
                                                class="flex items-center gap-1 mt-1"
                                            >
                                                <span
                                                    >{$t(
                                                        "settings.risk.growthPlan.consistency",
                                                    )}</span
                                                >
                                                <Input
                                                    type="number"
                                                    class="h-6 w-12 px-1 text-xs"
                                                    value={phase.progression_rules.find(
                                                        (r) =>
                                                            r.condition ===
                                                            "consistency_days",
                                                    )?.value ?? 0}
                                                    oninput={(e) => {
                                                        const val = Number(
                                                            e.currentTarget
                                                                .value,
                                                        );
                                                        const idx =
                                                            phase.progression_rules.findIndex(
                                                                (r) =>
                                                                    r.condition ===
                                                                    "consistency_days",
                                                            );
                                                        if (val > 0) {
                                                            if (idx >= 0)
                                                                phase.progression_rules[
                                                                    idx
                                                                ].value = val;
                                                            else
                                                                phase.progression_rules.push(
                                                                    {
                                                                        condition:
                                                                            "consistency_days",
                                                                        value: val,
                                                                    },
                                                                );
                                                        } else if (idx >= 0) {
                                                            phase.progression_rules.splice(
                                                                idx,
                                                                1,
                                                            );
                                                        }
                                                    }}
                                                />
                                                <span class="text-[10px]"
                                                    >{$t(
                                                        "settings.risk.growthPlan.tradingSessions",
                                                    )}</span
                                                >
                                            </div>
                                        </div>
                                        <div>
                                            <span
                                                class="font-semibold block text-red-500"
                                                >{$t(
                                                    "settings.risk.growthPlan.regression",
                                                )}</span
                                            >
                                            <div
                                                class="flex items-center gap-1 mt-1"
                                            >
                                                <span
                                                    >{$t(
                                                        "settings.risk.growthPlan.drawdown",
                                                    )}</span
                                                >
                                                <Input
                                                    type="number"
                                                    class="h-6 w-16 px-1 text-xs"
                                                    value={phase.regression_rules.find(
                                                        (r) =>
                                                            r.condition ===
                                                            "drawdown_limit",
                                                    )?.value ?? 0}
                                                    oninput={(e) => {
                                                        const val = Number(
                                                            e.currentTarget
                                                                .value,
                                                        );
                                                        const idx =
                                                            phase.regression_rules.findIndex(
                                                                (r) =>
                                                                    r.condition ===
                                                                    "drawdown_limit",
                                                            );
                                                        if (val > 0) {
                                                            if (idx >= 0)
                                                                phase.regression_rules[
                                                                    idx
                                                                ].value = val;
                                                            else
                                                                phase.regression_rules.push(
                                                                    {
                                                                        condition:
                                                                            "drawdown_limit",
                                                                        value: val,
                                                                    },
                                                                );
                                                        } else if (idx >= 0) {
                                                            phase.regression_rules.splice(
                                                                idx,
                                                                1,
                                                            );
                                                        }
                                                    }}
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </Card.Content>
                            </Card.Root>
                        {/each}
                        {#if formData.growth_phases.length === 0}
                            <div
                                class="text-center py-8 text-muted-foreground text-sm border border-dashed rounded-lg"
                            >
                                {$t("settings.risk.growthPlan.noPhases")}
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}
        </Tabs.Content>
    </Tabs.Root>

    <div class="flex justify-end gap-2 pt-4 border-t">
        <Button variant="outline" onclick={onCancel}
            >{$t("general.cancel")}</Button
        >
        <Button onclick={save}>{$t("general.save")}</Button>
    </div>
</div>
