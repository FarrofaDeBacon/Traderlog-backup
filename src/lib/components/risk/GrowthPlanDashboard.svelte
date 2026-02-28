<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
    import * as Card from "$lib/components/ui/card";
    import { Progress } from "$lib/components/ui/progress";
    import { Button } from "$lib/components/ui/button";
    import { TrendingUp, TrendingDown, Lock, CheckCircle } from "lucide-svelte";
    import { evaluateGrowthPhase } from "$lib/utils/riskLogic";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";

    // Mock das estatísticas (Em produção viria do Store de Trades)
    let currentStats = {
        totalProfit: 450,
        daysPositive: 2,
        currentDrawdown: 150,
        lossStreak: 1,
    };

    let activeProfile = $derived(settingsStore.activeProfile);
    let currentPhase = $derived(
        activeProfile?.growth_phases?.[activeProfile.current_phase_index],
    );

    let evaluation = $derived(
        activeProfile && currentPhase
            ? evaluateGrowthPhase(activeProfile, currentStats)
            : null,
    );

    function applyProgression() {
        if (evaluation?.action === "promote" && activeProfile) {
            settingsStore.updateRiskProfilePhase(
                activeProfile.id,
                evaluation.newPhaseIndex,
            );
            toast.success(
                $t("settings.risk.growthPlan.toasts.promote", {
                    values: {
                        name: activeProfile.growth_phases[
                            evaluation.newPhaseIndex
                        ].name,
                    },
                }),
            );
        }
    }

    function applyRegression() {
        if (evaluation?.action === "demote" && activeProfile) {
            settingsStore.updateRiskProfilePhase(
                activeProfile.id,
                evaluation.newPhaseIndex,
            );
            toast.warning(
                $t("settings.risk.growthPlan.toasts.demote", {
                    values: {
                        name: activeProfile.growth_phases[
                            evaluation.newPhaseIndex
                        ].name,
                    },
                }),
            );
        }
    }
</script>

{#if activeProfile && activeProfile.growth_plan_enabled && currentPhase}
    <Card.Root class="border-2 border-primary/20 bg-primary/5">
        <Card.Header class="pb-2">
            <div class="flex justify-between items-center">
                <div class="space-y-1">
                    <Card.Title class="flex items-center gap-2">
                        <TrendingUp class="w-5 h-5 text-primary" />
                        {$t("settings.risk.growthPlan.title")}: {currentPhase.name}
                    </Card.Title>
                    <Card.Description
                        >{$t("settings.risk.growthPlan.maxLotsLabel")}
                        <strong>{currentPhase.max_lots}</strong>
                        |
                        {$t("settings.risk.growthPlan.dailyLossLabel")}
                        <strong>R$ {currentPhase.max_daily_loss}</strong
                        ></Card.Description
                    >
                </div>
                <div class="text-right">
                    <span
                        class="text-xs text-muted-foreground uppercase font-bold"
                        >{$t("settings.risk.growthPlan.status.title")}</span
                    >
                    <div class="font-mono text-lg">
                        {evaluation?.action === "promote"
                            ? $t("settings.risk.growthPlan.status.approved")
                            : evaluation?.action === "demote"
                              ? $t("settings.risk.growthPlan.status.regression")
                              : $t(
                                    "settings.risk.growthPlan.status.inProgress",
                                )}
                    </div>
                </div>
            </div>
        </Card.Header>
        <Card.Content>
            <div class="grid grid-cols-2 gap-4">
                <!-- Progression Status -->
                <div class="space-y-2">
                    <span class="text-xs font-semibold text-green-600"
                        >{$t(
                            "settings.risk.growthPlan.requirements.promote",
                        )}</span
                    >
                    {#each currentPhase?.progression_rules || [] as rule}
                        <div class="flex justify-between text-sm">
                            <span
                                >{rule.condition === "profit_target"
                                    ? $t(
                                          "settings.risk.growthPlan.requirements.profit",
                                      )
                                    : $t(
                                          "settings.risk.growthPlan.requirements.days",
                                      )}</span
                            >
                            <span>
                                {rule.condition === "profit_target"
                                    ? `R$ ${currentStats.totalProfit} / ${rule.value}`
                                    : `${currentStats.daysPositive} / ${rule.value}`}
                            </span>
                        </div>
                        <Progress
                            value={(rule.condition === "profit_target"
                                ? currentStats.totalProfit / rule.value
                                : currentStats.daysPositive / rule.value) * 100}
                            class="h-2"
                        />
                    {/each}
                </div>

                <!-- Regression Status -->
                <div class="space-y-2">
                    <span class="text-xs font-semibold text-red-600"
                        >{$t(
                            "settings.risk.growthPlan.requirements.demote",
                        )}</span
                    >
                    {#each currentPhase?.regression_rules || [] as rule}
                        <div class="flex justify-between text-sm">
                            <span
                                >{rule.condition === "drawdown_limit"
                                    ? $t(
                                          "settings.risk.growthPlan.requirements.drawdown",
                                      )
                                    : $t(
                                          "settings.risk.growthPlan.requirements.lossStreak",
                                      )}</span
                            >
                            <span
                                class={currentStats.currentDrawdown > rule.value
                                    ? "text-red-500 font-bold"
                                    : ""}
                            >
                                {rule.condition === "drawdown_limit"
                                    ? `R$ ${currentStats.currentDrawdown} / ${rule.value}`
                                    : `${currentStats.lossStreak} / ${rule.value}`}
                            </span>
                        </div>
                        <Progress
                            value={(rule.condition === "drawdown_limit"
                                ? currentStats.currentDrawdown / rule.value
                                : currentStats.lossStreak / rule.value) * 100}
                            class="h-2 bg-red-100"
                        />
                    {/each}
                </div>
            </div>

            <!-- Action Buttons -->
            <div class="pt-4 flex justify-end gap-2">
                {#if evaluation?.action === "promote"}
                    <Button
                        onclick={applyProgression}
                        class="bg-green-600 hover:bg-green-700 text-white"
                    >
                        <CheckCircle class="w-4 h-4 mr-2" />
                        {$t("settings.risk.growthPlan.actions.promote")}
                    </Button>
                {:else if evaluation?.action === "demote"}
                    <Button onclick={applyRegression} variant="destructive">
                        <Lock class="w-4 h-4 mr-2" />
                        {$t("settings.risk.growthPlan.actions.demote")}
                    </Button>
                {/if}
            </div>
        </Card.Content>
    </Card.Root>
{/if}
