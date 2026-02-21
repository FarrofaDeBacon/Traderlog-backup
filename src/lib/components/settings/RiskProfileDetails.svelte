<script lang="ts">
    import {
        Shield,
        Target,
        Lock,
        AlertTriangle,
        TrendingUp,
        Clock,
        Ban,
        CheckCircle2,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import type { RiskProfile } from "$lib/stores/settings.svelte";
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import * as Card from "$lib/components/ui/card";

    let { profile } = $props<{ profile: RiskProfile }>();
</script>

<div class="space-y-6 py-4">
    <!-- Header Summary -->
    <div class="flex items-center justify-between">
        <div class="space-y-1">
            <h2 class="text-2xl font-bold flex items-center gap-2">
                <Shield class="w-6 h-6 text-primary" />
                {profile.name}
            </h2>
            <div class="flex items-center gap-2 text-sm text-muted-foreground">
                <Badge variant="outline">
                    {$t(
                        `settings.risk.accountTypes.${profile.account_type_applicability}`,
                    ) || profile.account_type_applicability}
                </Badge>
                {#if profile.growth_plan_enabled}
                    <Badge
                        variant="default"
                        class="bg-green-500/10 text-green-600 hover:bg-green-500/20 border-green-500/20"
                    >
                        <TrendingUp class="w-3 h-3 mr-1" />
                        Plano de Crescimento Ativo
                    </Badge>
                {/if}
            </div>
        </div>
    </div>

    <Separator />

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Downside Protection -->
        <Card.Root class="border-red-500/20 bg-red-500/5">
            <Card.Header class="pb-2">
                <Card.Title
                    class="text-base flex items-center gap-2 text-red-600 dark:text-red-400"
                >
                    <Shield class="w-4 h-4" />
                    {$t("settings.risk.downside")}
                </Card.Title>
            </Card.Header>
            <Card.Content class="space-y-2">
                <div class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground"
                        >{$t("settings.risk.dailyLossLimit")}</span
                    >
                    <span
                        class="font-bold text-red-600 dark:text-red-400 text-lg"
                    >
                        R$ {profile.max_daily_loss.toFixed(2)}
                    </span>
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground"
                        >{$t("settings.risk.maxRiskPerTrade")}</span
                    >
                    <span class="font-mono font-medium"
                        >{profile.max_risk_per_trade_percent}%</span
                    >
                </div>
            </Card.Content>
        </Card.Root>

        <!-- Upside Targets -->
        <Card.Root class="border-green-500/20 bg-green-500/5">
            <Card.Header class="pb-2">
                <Card.Title
                    class="text-base flex items-center gap-2 text-green-600 dark:text-green-400"
                >
                    <Target class="w-4 h-4" />
                    {$t("settings.risk.upside")}
                </Card.Title>
            </Card.Header>
            <Card.Content class="space-y-2">
                <div class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground"
                        >{$t("settings.risk.dailyGoal")}</span
                    >
                    <span
                        class="font-bold text-green-600 dark:text-green-400 text-lg"
                    >
                        R$ {profile.daily_target.toFixed(2)}
                    </span>
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-sm text-muted-foreground"
                        >{$t("settings.risk.minRiskReward")}</span
                    >
                    <span class="font-mono font-medium"
                        >1:{profile.min_risk_reward}</span
                    >
                </div>
            </Card.Content>
        </Card.Root>
    </div>

    <!-- Discipline & Rules -->
    <Card.Root>
        <Card.Header class="pb-2">
            <Card.Title class="text-base flex items-center gap-2">
                <Lock class="w-4 h-4" />
                {$t("settings.risk.discipline")}
            </Card.Title>
        </Card.Header>
        <Card.Content class="grid grid-cols-2 gap-4">
            <div class="space-y-1">
                <span
                    class="text-xs text-muted-foreground block uppercase font-bold"
                    >Max Trades / Dia</span
                >
                <div class="flex items-center gap-2">
                    <Clock class="w-4 h-4 text-muted-foreground" />
                    <span class="text-lg font-medium"
                        >{profile.max_trades_per_day}</span
                    >
                </div>
            </div>

            <div class="space-y-1">
                <span
                    class="text-xs text-muted-foreground block uppercase font-bold"
                    >Bloqueio de Plataforma</span
                >
                <div class="flex items-center gap-2">
                    {#if profile.lock_on_loss}
                        <Ban class="w-4 h-4 text-red-500" />
                        <span class="font-medium text-red-500"
                            >{$t("general.yes") || "Sim"}</span
                        >
                    {:else}
                        <CheckCircle2 class="w-4 h-4 text-muted-foreground" />
                        <span class="font-medium text-muted-foreground"
                            >{$t("general.no") || "Não"}</span
                        >
                    {/if}
                </div>
            </div>

            {#if profile.lock_on_loss}
                <div
                    class="col-span-2 mt-2 p-2 bg-red-100 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-xs rounded flex items-start gap-2"
                >
                    <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
                    <span>{$t("settings.risk.lockWarning")}</span>
                </div>
            {/if}
        </Card.Content>
    </Card.Root>

    <!-- Growth Plan Summary -->
    {#if profile.growth_plan_enabled && profile.growth_phases && profile.growth_phases.length > 0}
        <div class="space-y-2">
            <h3
                class="text-sm font-medium flex items-center gap-2 text-muted-foreground"
            >
                <TrendingUp class="w-4 h-4" />
                Fases do Plano de Crescimento
            </h3>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
                {#each profile.growth_phases as phase, i}
                    <div
                        class="p-2 rounded border bg-muted/20 text-xs relative {i ===
                        profile.current_phase_index
                            ? 'ring-2 ring-primary border-primary bg-primary/5'
                            : ''}"
                    >
                        {#if i === profile.current_phase_index}
                            <div
                                class="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-primary animate-pulse"
                            ></div>
                        {/if}
                        <div class="font-bold mb-1 truncate" title={phase.name}>
                            {phase.name}
                        </div>
                        <div class="text-muted-foreground">
                            Max Lote: <span class="font-mono text-foreground"
                                >{phase.max_lots}</span
                            >
                        </div>
                        <div class="text-muted-foreground">
                            Loss: <span class="font-mono text-foreground"
                                >R$ {phase.max_daily_loss}</span
                            >
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    {/if}
</div>
