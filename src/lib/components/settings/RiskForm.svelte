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
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import type { RiskProfile, GrowthPhase } from "$lib/stores/settings.svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Card from "$lib/components/ui/card";

    import { Badge } from "$lib/components/ui/badge";

    let { initialData, onSave, onCancel } = $props<{
        initialData?: RiskProfile;
        onSave: (data: Omit<RiskProfile, "id">) => void;
        onCancel: () => void;
    }>();

    // Presets Configuration
    const presets = {
        conservative: {
            name: "Conservador",
            max_daily_loss: 500,
            daily_target: 300,
            max_risk_per_trade_percent: 1.0,
            max_trades_per_day: 3,
            min_risk_reward: 2.0,
            lock_on_loss: true,
        },
        moderate: {
            name: "Moderado",
            max_daily_loss: 1000,
            daily_target: 800,
            max_risk_per_trade_percent: 2.0,
            max_trades_per_day: 5,
            min_risk_reward: 1.5,
            lock_on_loss: true,
        },
        aggressive: {
            name: "Agressivo",
            max_daily_loss: 2000,
            daily_target: 2000,
            max_risk_per_trade_percent: 5.0,
            max_trades_per_day: 10,
            min_risk_reward: 1.0,
            lock_on_loss: false,
        },
    };

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
            formData = {
                name: initialData.name,
                max_daily_loss: initialData.max_daily_loss,
                daily_target: initialData.daily_target,
                max_risk_per_trade_percent:
                    initialData.max_risk_per_trade_percent,
                max_trades_per_day: initialData.max_trades_per_day,
                min_risk_reward: initialData.min_risk_reward,
                lock_on_loss: initialData.lock_on_loss,
                account_type_applicability:
                    initialData.account_type_applicability,
                growth_plan_enabled: initialData.growth_plan_enabled ?? false,
                current_phase_index: initialData.current_phase_index ?? 0,
                growth_phases: initialData.growth_phases
                    ? [...initialData.growth_phases]
                    : [],
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
    ]);

    // Growth Plan Presets
    const growthPresets = {
        conservative: {
            name: "Lento e Consistente",
            description:
                "Aumenta a mão a cada 5 dias de meta batida. Regride ao atingir 50% do drawdown.",
            phases: Array.from({ length: 5 }, (_, i) => ({
                id: crypto.randomUUID(),
                name: `Nível ${i + 1}`,
                description: `Fase ${i + 1} do plano conservador`,
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
            name: "Crescimento Equilibrado",
            description:
                "Aumenta a mão a cada 3 dias de meta. Regride com 1 dia de loss total.",
            phases: Array.from({ length: 8 }, (_, i) => ({
                id: crypto.randomUUID(),
                name: `Nível ${i + 1}`,
                description: `Fase ${i + 1} do plano moderado`,
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
            name: "Alavancagem Rápida",
            description:
                "Aumenta lote a cada 2 dias de meta. Suporta drawdowns maiores.",
            phases: Array.from({ length: 10 }, (_, i) => ({
                id: crypto.randomUUID(),
                name: `Nível ${i + 1}`,
                description: `Fase ${i + 1} agressiva`,
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
    };

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
                name: `Fase ${formData.growth_phases.length}`,
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
            >Modelos (Presets)</Label
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
                Personalizado
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
        <Tabs.List class="grid w-full grid-cols-2">
            <Tabs.Trigger value="general">Geral</Tabs.Trigger>
            <Tabs.Trigger value="growth">Plano de Crescimento</Tabs.Trigger>
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

        <Tabs.Content value="growth" class="space-y-4 pt-4">
            <div
                class="flex items-center justify-between p-4 rounded-lg border bg-card"
            >
                <div class="flex items-center gap-2">
                    <TrendingUp class="w-5 h-5 text-primary" />
                    <div class="space-y-0.5">
                        <h4 class="font-medium text-sm">
                            Habilitar Plano de Crescimento
                        </h4>
                        <p class="text-xs text-muted-foreground">
                            Regras automáticas para aumentar ou diminuir a mão.
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
                            >Modelos (Presets)</Label
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
                                Personalizado
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
                            Fases ({formData.growth_phases.length})
                        </h4>
                        <Button size="sm" variant="outline" onclick={addPhase}>
                            <Plus class="w-3 h-3 mr-1" />
                            Adicionar Fase
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
                                                >Lote Máx.</Label
                                            >
                                            <Input
                                                type="number"
                                                class="h-7 text-xs"
                                                bind:value={phase.max_lots}
                                            />
                                        </div>
                                        <div class="space-y-1">
                                            <Label class="text-[10px]"
                                                >Loss Diário (R$)</Label
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
                                                >Progressão:</span
                                            >
                                            <div
                                                class="flex items-center gap-1 mt-1"
                                            >
                                                <span>Meta de Lucro: R$</span>
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
                                                <span>Consistência: </span>
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
                                                    >pregões</span
                                                >
                                            </div>
                                        </div>
                                        <div>
                                            <span
                                                class="font-semibold block text-red-500"
                                                >Regressão:</span
                                            >
                                            <div
                                                class="flex items-center gap-1 mt-1"
                                            >
                                                <span>Drawdown: R$</span>
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
                                Nenhuma fase configurada. Adicione a "Fase 0"
                                para começar.
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
