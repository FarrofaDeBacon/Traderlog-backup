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

    const data = $state.snapshot(initialData);
    let formData = $state<Omit<RiskProfile, "id">>({
        name: data?.name ?? "",
        max_daily_loss: data?.max_daily_loss ?? 0,
        daily_target: data?.daily_target ?? 0,
        max_risk_per_trade_percent: data?.max_risk_per_trade_percent ?? 1.0,
        max_trades_per_day: data?.max_trades_per_day ?? 5,
        min_risk_reward: data?.min_risk_reward ?? 1.5,
        lock_on_loss: data?.lock_on_loss ?? false,
        account_type_applicability: data?.account_type_applicability ?? "All",
        target_type: data?.target_type ?? "Financial",
        capital_source: data?.capital_source ?? "Fixed",
        fixed_capital: data?.fixed_capital ?? 0,
        linked_account_id: data?.linked_account_id ?? null,
        growth_plan_enabled: data?.growth_plan_enabled ?? false,
        current_phase_index: data?.current_phase_index ?? 0,
        growth_phases: data?.growth_phases ?? [],
        psychological_coupling_enabled: data?.psychological_coupling_enabled ?? false,
        outlier_regression_enabled: data?.outlier_regression_enabled ?? false,
        sniper_mode_enabled: data?.sniper_mode_enabled ?? false,
        sniper_mode_selectivity: data?.sniper_mode_selectivity ?? 3,
        psychological_lookback_count: data?.psychological_lookback_count ?? 10,
        outlier_lookback_count: data?.outlier_lookback_count ?? 20,
        psychological_threshold: data?.psychological_threshold ?? -2,
        lot_reduction_multiplier: data?.lot_reduction_multiplier ?? 0.5,
        psychological_search_strategy: data?.psychological_search_strategy ?? "Strict",
        account_ids: data?.account_ids ?? [],
        active: data?.active ?? false,
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
                name: fd.name,
                max_daily_loss: fd.max_daily_loss,
                daily_target: fd.daily_target,
                max_risk_per_trade_percent: fd.max_risk_per_trade_percent,
                max_trades_per_day: fd.max_trades_per_day,
                min_risk_reward: fd.min_risk_reward,
                lock_on_loss: fd.lock_on_loss,
                account_type_applicability: fd.account_type_applicability,
                target_type: fd.target_type ?? "Financial",
                capital_source: fd.capital_source ?? "Fixed",
                fixed_capital: fd.fixed_capital ?? 0,
                linked_account_id: fd.linked_account_id ?? null,
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
                level: i + 1,
                name: `${$t("general.items")} ${i + 1}`,
                lot_size: i + 1,
                conditions_to_advance: [
                    { metric: "days_positive", operator: ">=", value: 5 },
                ],
                conditions_to_demote: [
                    { metric: "drawdown_limit", operator: ">=", value: (i + 1) * 100 },
                ],
            })),
        },
        moderate: {
            name: $t("settings.risk.form.presets.moderate"),
            description: $t("settings.risk.growthPlan.enableDesc"),
            phases: Array.from({ length: 8 }, (_, i) => ({
                level: i + 1,
                name: `${$t("general.items")} ${i + 1}`,
                lot_size: (i + 1) * 2,
                conditions_to_advance: [
                    { metric: "days_positive", operator: ">=", value: 3 },
                ],
                conditions_to_demote: [
                    { metric: "max_daily_loss_streak", operator: ">=", value: 1 },
                ],
            })),
        },
        aggressive: {
            name: $t("settings.risk.form.presets.aggressive"),
            description: $t("settings.risk.growthPlan.enableDesc"),
            phases: Array.from({ length: 10 }, (_, i) => ({
                level: i + 1,
                name: `${$t("general.items")} ${i + 1}`,
                lot_size: (i + 1) * 5,
                conditions_to_advance: [
                    { metric: "days_positive", operator: ">=", value: 2 },
                ],
                conditions_to_demote: [
                    { metric: "drawdown_limit", operator: ">=", value: (i + 1) * 800 },
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
                level: formData.growth_phases.length + 1,
                name: `${$t("settings.risk.growthPlan.phases")} ${formData.growth_phases.length}`,
                lot_size: 1,
                conditions_to_advance: [],
                conditions_to_demote: [],
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

        <Tabs.Content value="general" class="space-y-6 pt-4">
            
            <!-- Target Type & Capital Source -->
            <div class="space-y-5 p-5 rounded-xl border border-border/10 bg-black/5 shadow-sm">
                <h3 class="flex items-center gap-2 font-bold text-primary">
                    <Target class="w-4 h-4" />
                    Configuração de Capital e Alvo
                </h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                    <div class="space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Tipo de Alvo</Label>
                        <Select.Root
                            type="single"
                            bind:value={formData.target_type}
                        >
                            <Select.Trigger class="w-full">
                                {formData.target_type === "Financial" ? "Financeiro ($)" : "Pontos (pts)"}
                            </Select.Trigger>
                            <Select.Content>
                                <Select.Item value="Financial">Financeiro ($)</Select.Item>
                                <Select.Item value="Points">Pontos (pts)</Select.Item>
                            </Select.Content>
                        </Select.Root>
                    </div>

                    <div class="space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Origem do Capital</Label>
                        <Select.Root
                            type="single"
                            bind:value={formData.capital_source}
                        >
                            <Select.Trigger class="w-full">
                                {formData.capital_source === "Fixed" ? "Valor Fixo" : "Vincular a Conta"}
                            </Select.Trigger>
                            <Select.Content>
                                <Select.Item value="Fixed">Valor Fixo</Select.Item>
                                <Select.Item value="LinkedAccount">Vincular a Conta</Select.Item>
                            </Select.Content>
                        </Select.Root>
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-5 pt-2">
                    {#if formData.capital_source === "Fixed"}
                        <div class="space-y-2.5 animate-in fade-in slide-in-from-top-1">
                            <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Capital Base (Valor Fixo)</Label>
                            <Input
                                type="number"
                                step="0.01"
                                bind:value={formData.fixed_capital}
                                placeholder="ex: 1000.00"
                            />
                        </div>
                    {:else}
                         <div class="space-y-2.5 animate-in fade-in slide-in-from-top-1">
                            <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Selecionar Conta Vinculada</Label>
                            <Select.Root
                                type="single"
                                bind:value={formData.linked_account_id}
                            >
                                <Select.Trigger class="w-full">
                                    {settingsStore.accounts.find(a => a.id === formData.linked_account_id)?.nickname ?? "Selecione uma Conta"}
                                </Select.Trigger>
                                <Select.Content>
                                    {#each settingsStore.accounts as account}
                                        <Select.Item value={account.id}>{account.nickname}</Select.Item>
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        </div>
                    {/if}
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                <!-- Downside Protection -->
                <div class="space-y-5 p-5 rounded-xl border border-border/10 bg-black/5 shadow-sm">
                    <h3 class="flex items-center gap-2 font-bold text-rose-500">
                        <Shield class="w-4 h-4" />
                        {$t("settings.risk.downside")}
                    </h3>
                    <div class="space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">{$t("settings.risk.dailyLossLimit")}</Label>
                        <Input
                            type="number"
                            bind:value={formData.max_daily_loss}
                        />
                    </div>
                    <div class="space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">{$t("settings.risk.maxRiskPerTrade")}</Label>
                        <Input
                            type="number"
                            step="0.1"
                            bind:value={formData.max_risk_per_trade_percent}
                        />
                    </div>
                </div>

                <!-- Upside Targets -->
                <div class="space-y-5 p-5 rounded-xl border border-border/10 bg-black/5 shadow-sm">
                    <h3 class="flex items-center gap-2 font-bold text-emerald-500">
                        <Target class="w-4 h-4" />
                        {$t("settings.risk.upside")}
                    </h3>
                    <div class="space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">{$t("settings.risk.dailyGoal")}</Label>
                        <Input
                            type="number"
                            bind:value={formData.daily_target}
                        />
                    </div>
                    <div class="space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">{$t("settings.risk.minRiskReward")}</Label>
                        <Input
                            type="number"
                            step="0.1"
                            bind:value={formData.min_risk_reward}
                        />
                    </div>
                </div>
            </div>

            <!-- Discipline -->
            <div class="space-y-5 p-5 rounded-xl border border-border/10 bg-black/5 shadow-sm">
                <h3 class="flex items-center gap-2 font-bold text-muted-foreground">
                    <Lock class="w-4 h-4" />
                    {$t("settings.risk.discipline")}
                </h3>
                <div class="grid grid-cols-2 gap-5">
                    <div class="space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">{$t("settings.risk.maxTradesDay")}</Label>
                        <Input
                            type="number"
                            bind:value={formData.max_trades_per_day}
                        />
                    </div>
                    <div class="space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">{$t("settings.risk.applicability")}</Label>
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
                                        onCheckedChange={(checked: boolean) => {
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

        <Tabs.Content value="risk-engine" class="space-y-6 pt-4">
            <!-- Psychological Coupling -->
            <div class="space-y-5 p-5 rounded-xl border border-indigo-500/20 bg-indigo-500/5 shadow-sm">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <div class="p-2 rounded-lg bg-indigo-500/10"><Brain class="w-5 h-5 text-indigo-400" /></div>
                        <div class="space-y-1">
                            <h4 class="font-bold text-indigo-400 text-sm">
                                {$t("settings.risk.engine.psychological.title")}
                            </h4>
                            <p class="text-[10px] text-muted-foreground/80 uppercase tracking-widest font-semibold">
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
                    <div class="grid grid-cols-2 gap-5 pt-3 border-t border-indigo-500/10">
                        <div class="space-y-2.5">
                            <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                                {$t("settings.risk.engine.psychological.strategy")}
                            </Label>
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
                        <div class="space-y-2.5">
                            <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                                {$t("settings.risk.engine.psychological.lookback")}
                            </Label>
                            <Input
                                type="number"
                                bind:value={
                                    formData.psychological_lookback_count
                                }
                                class="h-8 text-xs"
                            />
                        </div>
                        <div class="space-y-2.5">
                            <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                                {$t("settings.risk.engine.psychological.threshold")}
                            </Label>
                            <Input
                                type="number"
                                bind:value={formData.psychological_threshold}
                                class="h-8 text-xs"
                            />
                        </div>
                        <div class="space-y-2.5">
                            <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                                {$t("settings.risk.engine.psychological.multiplier")}
                            </Label>
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
            <div class="space-y-5 p-5 rounded-xl border border-amber-500/20 bg-amber-500/5 shadow-sm">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <div class="p-2 rounded-lg bg-amber-500/10"><AlertTriangle class="w-5 h-5 text-amber-500" /></div>
                        <div class="space-y-1">
                            <h4 class="font-bold text-amber-500 text-sm">
                                {$t("settings.risk.engine.outlier.title")}
                            </h4>
                            <p class="text-[10px] text-muted-foreground/80 uppercase tracking-widest font-semibold">
                                {$t("settings.risk.engine.outlier.description")}
                            </p>
                        </div>
                    </div>
                    <Switch
                        bind:checked={formData.outlier_regression_enabled}
                    />
                </div>
                {#if formData.outlier_regression_enabled}
                    <div class="pt-3 border-t border-amber-500/10 space-y-2.5">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                            {$t("settings.risk.engine.outlier.lookback")}
                        </Label>
                        <Input
                            type="number"
                            bind:value={formData.outlier_lookback_count}
                            class="h-8 text-xs w-24"
                        />
                    </div>
                {/if}
            </div>

            <!-- Sniper Mode -->
            <div class="space-y-5 p-5 rounded-xl border border-blue-500/20 bg-blue-500/5 shadow-sm">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <div class="p-2 rounded-lg bg-blue-500/10"><Zap class="w-5 h-5 text-blue-400" /></div>
                        <div class="space-y-1">
                            <h4 class="font-bold text-blue-400 text-sm">
                                {$t("settings.risk.engine.sniper.title")}
                            </h4>
                            <p class="text-[10px] text-muted-foreground/80 uppercase tracking-widest font-semibold">
                                {$t("settings.risk.engine.sniper.description")}
                            </p>
                        </div>
                    </div>
                    <Switch bind:checked={formData.sniper_mode_enabled} />
                </div>
                {#if formData.sniper_mode_enabled}
                    <div class="space-y-2.5 pt-3 border-t border-blue-500/10 animate-in fade-in duration-300">
                        <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                            {$t("settings.risk.engine.sniper.selectivity")}
                        </Label>
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

        <Tabs.Content value="growth" class="space-y-6 pt-4">
            <div class="flex items-center justify-between p-5 rounded-xl border border-border/10 bg-black/5 shadow-sm cursor-pointer" onclick={() => formData.growth_plan_enabled = !formData.growth_plan_enabled}>
                <div class="flex items-center gap-3">
                    <div class="p-2 rounded-lg bg-primary/10"><TrendingUp class="w-5 h-5 text-primary" /></div>
                    <div class="space-y-1">
                        <h4 class="font-bold text-sm text-foreground">
                            {$t("settings.risk.growthPlan.enable")}
                        </h4>
                        <p class="text-[10px] text-muted-foreground/80 uppercase tracking-widest font-semibold">
                            {$t("settings.risk.growthPlan.enableDesc")}
                        </p>
                    </div>
                </div>
                <Switch bind:checked={formData.growth_plan_enabled} />
            </div>

            {#if formData.growth_plan_enabled}
                <div class="space-y-6">
                    <!-- Growth Presets Selection -->
                    <div class="space-y-3 p-4 bg-black/5 rounded-xl border border-dashed border-border/20">
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
                            <Card.Root class="relative border-border/10 bg-black/5 shadow-sm rounded-xl overflow-hidden mb-4">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="absolute top-3 right-3 h-8 w-8 text-muted-foreground/50 hover:text-destructive transition-colors"
                                    onclick={() => removePhase(i)}
                                >
                                    <Trash2 class="w-4 h-4" />
                                </Button>
                                <Card.Header class="p-5 pb-2">
                                    <div class="flex items-center gap-3">
                                        <div
                                            class="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center text-sm font-bold text-primary shadow-inner"
                                        >
                                            {i}
                                        </div>
                                        <Input
                                            class="h-9 text-base font-bold border-0 px-0 focus-visible:ring-0 bg-transparent"
                                            bind:value={phase.name}
                                        />
                                    </div>
                                </Card.Header>
                                <Card.Content class="p-5 pt-3 space-y-5">
                                    <div class="grid grid-cols-2 gap-5">
                                        <div class="space-y-2.5">
                                            <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground"
                                                >{$t(
                                                    "settings.risk.growthPlan.maxLots",
                                                )}</Label
                                            >
                                            <Input
                                                type="number"
                                                class="h-9 text-sm"
                                                bind:value={phase.max_lots}
                                            />
                                        </div>
                                        <div class="space-y-2.5">
                                            <Label class="text-xs font-semibold uppercase tracking-wider text-muted-foreground"
                                                >{$t(
                                                    "settings.risk.growthPlan.dailyLoss",
                                                )}</Label
                                            >
                                            <Input
                                                type="number"
                                                class="h-9 text-sm"
                                                bind:value={
                                                    phase.max_daily_loss
                                                }
                                            />
                                        </div>
                                    </div>

                                    <!-- Dynamic Rule Builder -->
                                    <div class="pt-4 border-t border-border/10 space-y-6">
                                        <!-- Advance Rules -->
                                        <div class="space-y-3">
                                            <div class="flex items-center justify-between">
                                                <Label class="text-[11px] font-bold text-emerald-500 uppercase flex items-center gap-1.5 tracking-wider">
                                                    <TrendingUp class="w-3.5 h-3.5" />
                                                    {$t("settings.risk.growthPlan.progression") || "Avançar de Fase se:"}
                                                </Label>
                                                <Button variant="ghost" size="sm" class="h-6 px-2 text-[10px] font-bold text-emerald-500 hover:text-emerald-400 hover:bg-emerald-500/10" onclick={() => phase.conditions_to_advance.push({metric: 'profit_target', operator: '>=', value: 0})}>
                                                    <Plus class="w-3 h-3 mr-1" /> {$t("general.add")}
                                                </Button>
                                            </div>
                                            {#each phase.conditions_to_advance as rule, ri}
                                                <div class="flex items-center gap-2 animate-in fade-in slide-in-from-top-1 bg-black/5 p-1 rounded-md border border-border/5">
                                                    <Select.Root type="single" bind:value={rule.metric}>
                                                        <Select.Trigger class="h-8 text-xs flex-1 border-0 bg-transparent shadow-none"><span class="truncate font-medium">{rule.metric === 'profit_target' ? 'Lucro' : rule.metric === 'days_positive' ? 'Dias O.K' : rule.metric === 'win_rate' ? 'Win Rate %' : rule.metric === 'consistency_days' ? 'Consistência' : rule.metric}</span></Select.Trigger>
                                                        <Select.Content>
                                                            <Select.Item value="profit_target" class="text-xs">Lucro ($/pts)</Select.Item>
                                                            <Select.Item value="days_positive" class="text-xs">Dias O.K</Select.Item>
                                                            <Select.Item value="win_rate" class="text-xs">Win Rate %</Select.Item>
                                                            <Select.Item value="consistency_days" class="text-xs">Consistência</Select.Item>
                                                        </Select.Content>
                                                    </Select.Root>
                                                    <Select.Root type="single" bind:value={rule.operator}>
                                                        <Select.Trigger class="h-8 text-xs w-[60px] px-2 font-mono font-bold border-0 bg-transparent shadow-none">{rule.operator}</Select.Trigger>
                                                        <Select.Content>
                                                            <Select.Item value=">=" class="text-xs font-mono">&gt;=</Select.Item>
                                                            <Select.Item value=">" class="text-xs font-mono">&gt;</Select.Item>
                                                            <Select.Item value="<=" class="text-xs font-mono">&lt;=</Select.Item>
                                                            <Select.Item value="<" class="text-xs font-mono">&lt;</Select.Item>
                                                        </Select.Content>
                                                    </Select.Root>
                                                    <Input type="number" step="0.1" class="h-8 text-xs w-[75px] font-mono font-bold" bind:value={rule.value} />
                                                    <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground/50 hover:text-destructive shrink-0" onclick={() => phase.conditions_to_advance.splice(ri, 1)}>
                                                        <Trash2 class="w-3.5 h-3.5" />
                                                    </Button>
                                                </div>
                                            {/each}
                                            {#if phase.conditions_to_advance.length === 0}
                                                <div class="text-[11px] font-semibold text-muted-foreground/60 p-3 border border-dashed border-border/10 rounded-lg bg-black/5 text-center">Nenhuma regra para avançar. O avanço será manual.</div>
                                            {/if}
                                        </div>

                                        <!-- Demote Rules -->
                                        <div class="space-y-3">
                                            <div class="flex items-center justify-between">
                                                <Label class="text-[11px] font-bold text-rose-500 uppercase flex items-center gap-1.5 tracking-wider">
                                                    <TrendingUp class="w-3.5 h-3.5 rotate-180" />
                                                    {$t("settings.risk.growthPlan.regression") || "Rebaixar de Fase se:"}
                                                </Label>
                                                <Button variant="ghost" size="sm" class="h-6 px-2 text-[10px] font-bold text-rose-500 hover:text-rose-400 hover:bg-rose-500/10" onclick={() => phase.conditions_to_demote.push({metric: 'drawdown_limit', operator: '<=', value: 0})}>
                                                    <Plus class="w-3 h-3 mr-1" /> {$t("general.add")}
                                                </Button>
                                            </div>
                                            {#each phase.conditions_to_demote as rule, ri}
                                                <div class="flex items-center gap-2 animate-in fade-in slide-in-from-top-1 bg-black/5 p-1 rounded-md border border-border/5">
                                                    <Select.Root type="single" bind:value={rule.metric}>
                                                        <Select.Trigger class="h-8 text-xs flex-1 border-0 bg-transparent shadow-none"><span class="truncate font-medium">{rule.metric === 'drawdown_limit' ? 'Drawdown' : rule.metric === 'daily_loss_limit' ? 'Perda Diária' : rule.metric === 'max_daily_loss_streak' ? 'Dias de Loss' : rule.metric}</span></Select.Trigger>
                                                        <Select.Content>
                                                            <Select.Item value="drawdown_limit" class="text-xs">Drawdown ($/pts)</Select.Item>
                                                            <Select.Item value="daily_loss_limit" class="text-xs">Perda Diária ($/pts)</Select.Item>
                                                            <Select.Item value="max_daily_loss_streak" class="text-xs">Dias de Loss Seguidos</Select.Item>
                                                        </Select.Content>
                                                    </Select.Root>
                                                    <Select.Root type="single" bind:value={rule.operator}>
                                                        <Select.Trigger class="h-8 text-xs w-[60px] px-2 font-mono font-bold border-0 bg-transparent shadow-none">{rule.operator}</Select.Trigger>
                                                        <Select.Content>
                                                            <Select.Item value=">=" class="text-xs font-mono">&gt;=</Select.Item>
                                                            <Select.Item value=">" class="text-xs font-mono">&gt;</Select.Item>
                                                            <Select.Item value="<=" class="text-xs font-mono">&lt;=</Select.Item>
                                                            <Select.Item value="<" class="text-xs font-mono">&lt;</Select.Item>
                                                        </Select.Content>
                                                    </Select.Root>
                                                    <Input type="number" step="0.1" class="h-8 text-xs w-[75px] font-mono font-bold" bind:value={rule.value} />
                                                    <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground/50 hover:text-destructive shrink-0" onclick={() => phase.conditions_to_demote.splice(ri, 1)}>
                                                        <Trash2 class="w-3.5 h-3.5" />
                                                    </Button>
                                                </div>
                                            {/each}
                                            {#if phase.conditions_to_demote.length === 0}
                                                <div class="text-[11px] font-semibold text-muted-foreground/60 p-3 border border-dashed border-border/10 rounded-lg bg-black/5 text-center">Nenhuma regra armada para rebaixamento de fase.</div>
                                            {/if}
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
