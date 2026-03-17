<script lang="ts">
    import { t } from "svelte-i18n";
    import { Activity, Shield, Target, TrendingUp, TrendingDown, CheckCircle2, XCircle, ArrowRight, Gauge, Layers, AlertTriangle, ChevronRight, ChevronLeft } from "lucide-svelte";
    import * as Card from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import EChart from "$lib/components/ui/echart.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { formatCurrency } from "$lib/utils";
    import { evaluateCondition, evaluateGrowthPhase, computeRiskStats } from "$lib/utils/riskLogic";
    import { toast } from "svelte-sonner";

    let activeProfiles = $derived(
        settingsStore.riskProfiles.filter(p => p.active)
    );

    let selectedProfileId = $state(activeProfiles[0]?.id || null);

    // Derived the single selected profile
    let currentProfile = $derived(
        activeProfiles.find(p => p.id === selectedProfileId) || activeProfiles[0]
    );

    /** Promote the growth plan phase by 1 */
    async function promotePhase() {
        if (!currentProfile) return;
        const next = Math.min(currentProfile.current_phase_index + 1, currentProfile.growth_phases.length - 1);
        await settingsStore.updateRiskProfile(currentProfile.id, { ...currentProfile, current_phase_index: next });
        toast.success(`Promovido para a Fase ${next + 1}: ${currentProfile.growth_phases[next]?.name}`);
    }

    /** Demote the growth plan phase by 1 */
    async function demotePhase() {
        if (!currentProfile) return;
        const prev = Math.max(currentProfile.current_phase_index - 1, 0);
        await settingsStore.updateRiskProfile(currentProfile.id, { ...currentProfile, current_phase_index: prev });
        toast.warning(`Regredido para a Fase ${prev + 1}: ${currentProfile.growth_phases[prev]?.name}`);
    }

    // Compute cockpit data using REAL trades
    let cockpitData = $derived(() => {
        if (!currentProfile) return null;

        const stats = computeRiskStats(tradesStore.trades, currentProfile);
        const evaluation = evaluateGrowthPhase(currentProfile, stats);

        const curve = stats.dailyEquityCurve;
        const lastValue = curve[curve.length - 1]?.value ?? 0;
        const isPositive = lastValue >= 0;

        const chartOptions = {
            animation: true,
            backgroundColor: "transparent",
            tooltip: { 
                trigger: 'axis', 
                textStyle: { fontFamily: "Inter", fontSize: 12 },
                formatter: (params: any[]) => {
                    const p = params[0];
                    return `${p.name}<br/>Resultado Acumulado: <b>${Number(p.value).toFixed(2)}</b>`;
                }
            },
            grid: { top: 10, right: 10, bottom: 30, left: 55 },
            xAxis: { 
                type: 'category', 
                boundaryGap: false, 
                data: curve.map(c => c.day),
                axisLine: { lineStyle: { color: "rgba(255,255,255,0.15)" } },
                axisLabel: { color: "#71717a", fontSize: 10 }
            },
            yAxis: { 
                type: 'value', 
                splitLine: {
                    lineStyle: { color: "rgba(255,255,255,0.1)", type: "dashed" }
                },
                axisLabel: { color: "#71717a", fontSize: 10 }
            },
            series: [{
                data: curve.map(c => c.value),
                type: 'line',
                smooth: true,
                symbol: 'none',
                lineStyle: { width: 3, color: isPositive ? "#10b981" : "#f43f5e" },
                areaStyle: {
                    color: {
                        type: 'linear',
                        x: 0, y: 0, x2: 0, y2: 1,
                        colorStops: [
                            { offset: 0, color: isPositive ? 'rgba(16, 185, 129, 0.15)' : 'rgba(244, 63, 94, 0.15)' },
                            { offset: 1, color: "transparent" }
                        ]
                    }
                }
            }]
        };

        return {
            profile: currentProfile,
            stats,
            evaluation,
            chartOptions
        };
    });

    function getProfileCurrencyCode(profile: any): string {
        if (profile.capital_source === 'LinkedAccount' && profile.linked_account_id) {
            const acc = settingsStore.accounts.find(a => a.id === profile.linked_account_id);
            if (acc && acc.currency) return acc.currency;
        }
        return settingsStore.userProfile.main_currency || 'USD';
    }

    $effect(() => {
        // Automatically select the first profile if id is null
        if (!selectedProfileId && activeProfiles.length > 0) {
            selectedProfileId = activeProfiles[0].id;
        }
    });
</script>

<svelte:head>
    <title>{$t("nav.risk", { default: "Controle de Risco" })} - TraderLog Pro</title>
</svelte:head>

<div class="flex-1 flex flex-col space-y-6 p-4 md:p-8 min-h-screen w-full">
    
    <!-- Top Bar Navigation & Cockpit Selector -->
    <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 border-b border-border/10 pb-6 mb-2">
        <div>
            <h2 class="text-3xl font-bold tracking-tight flex items-center gap-2">
                <Activity class="w-8 h-8 text-primary" />
                Cockpit de Risco
            </h2>
            <p class="text-muted-foreground">
                Terminal analítico de risco, capital e evolução em tempo real.
            </p>
        </div>

        {#if activeProfiles.length > 0}
            <div class="flex items-center gap-3">
                <Shield class="w-5 h-5 text-muted-foreground hidden md:block" />
                <!-- Profile Base Selector (Badges) -->
                <div class="flex flex-wrap items-center gap-2">
                    {#each activeProfiles as profile}
                        <button 
                            class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider transition-all border {selectedProfileId === profile.id ? 'bg-primary/20 border-primary text-primary shadow-[0_0_10px_rgba(255,255,255,0.1)]' : 'bg-black/20 border-border/20 text-muted-foreground hover:bg-black/40 hover:text-foreground'}"
                            onclick={() => selectedProfileId = profile.id}
                        >
                            {profile.name}
                        </button>
                    {/each}
                </div>
                <div class="px-2 py-1.5 rounded bg-emerald-500/10 border border-emerald-500/20 text-emerald-500 text-xs font-bold uppercase tracking-wider hidden sm:block">
                    Monitoramento Online
                </div>
            </div>
        {/if}
    </div>

    <!-- Empty State -->
    {#if activeProfiles.length === 0}
        <Card.Root class="border-dashed bg-muted/10">
            <Card.Content class="pt-6 flex flex-col items-center justify-center min-h-[400px] text-center space-y-4">
                <div class="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center text-primary">
                    <Shield class="w-8 h-8" />
                </div>
                <div class="space-y-1">
                    <h3 class="text-xl font-bold">Nenhum Plano Ativo</h3>
                    <p class="text-muted-foreground max-w-sm mx-auto">
                        Você não possui nenhum plano de risco ou conta vinculada no momento. Ajuste as configurações para começar.
                    </p>
                </div>
                <a href="/settings/risk" class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2">
                    Configurar Perfil de Risco
                </a>
            </Card.Content>
        </Card.Root>
    {:else}
        {@const data = cockpitData()}
        {#if data}
            <!-- Cockpit Dashboard Grid -->
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">

                <!-- Widget 1: Thermometer & Capital (4 cols) -->
                <Card.Root class="lg:col-span-4 border-border/10 shadow-lg card-glass flex flex-col justify-between overflow-hidden relative group">
                    <div class="absolute inset-0 bg-gradient-to-br from-primary/5 to-transparent opacity-50 pointer-events-none"></div>
                    
                    <div class="p-5 flex-1 relative z-10 space-y-5">
                        <div class="flex items-center gap-2 mb-2">
                           <Gauge class="w-5 h-5 text-primary" />
                           <h3 class="text-sm font-bold uppercase tracking-widest text-muted-foreground/80">Capital & Limites</h3>
                        </div>

                        <!-- Capital base -->
                        <div class="space-y-1">
                            <p class="text-[10px] text-muted-foreground/60 uppercase tracking-widest font-bold">Base de Operação</p>
                            <h2 class="text-3xl font-black font-mono tracking-tight glow-text flex items-end gap-2">
                                {#if data.profile.capital_source === "LinkedAccount" && data.profile.linked_account_id}
                                    <!-- Use dynamic balance logic here in production -->
                                    <span class="text-foreground/80">{getProfileCurrencyCode(data.profile)}</span>
                                    {formatCurrency(settingsStore.accounts.find(a => a.id === data.profile.linked_account_id)?.balance || 0, getProfileCurrencyCode(data.profile)).replace(getProfileCurrencyCode(data.profile), '')}
                                {:else}
                                    <span class="text-foreground/80">{getProfileCurrencyCode(data.profile)}</span>
                                    {formatCurrency(data.profile.fixed_capital, getProfileCurrencyCode(data.profile)).replace(getProfileCurrencyCode(data.profile), '')}
                                {/if}
                            </h2>
                        </div>

                        <!-- Drawdown Gauge -->
                        <div class="p-4 rounded-xl border border-border/10 bg-black/20 shadow-inner space-y-3">
                            <div class="flex justify-between items-center text-sm">
                                <span class="text-muted-foreground font-medium uppercase tracking-widest text-[10px]">Tolerância Diária Utilizada</span>
                                <span class="font-bold font-mono tracking-tight text-rose-500">
                                    {data.profile.target_type === 'Financial' ? formatCurrency(data.stats.currentDrawdown, getProfileCurrencyCode(data.profile)) : `${data.stats.currentDrawdown} pts`} 
                                    <span class="text-muted-foreground/40 font-normal">
                                        / {data.profile.target_type === 'Financial' ? formatCurrency(data.profile.max_daily_loss, getProfileCurrencyCode(data.profile)) : `${data.profile.max_daily_loss} pts`}
                                    </span>
                                </span>
                            </div>
                            <!-- Styled Progress Bar -->
                            <div class="h-3 bg-rose-500/10 rounded-full overflow-hidden shadow-inner relative">
                                <div class="absolute top-0 bottom-0 left-0 bg-rose-500 transition-all duration-700 rounded-full" style="width: {(data.stats.currentDrawdown / data.profile.max_daily_loss) * 100}%">
                                    <div class="absolute inset-0 bg-white/20 blur-[2px]"></div>
                                </div>
                            </div>
                            <p class="text-[10px] text-rose-500/80 uppercase tracking-wider text-right font-bold w-full">
                                {((data.stats.currentDrawdown / data.profile.max_daily_loss) * 100).toFixed(1)}% do Limite Consumido
                            </p>
                        </div>
                    </div>

                    <!-- Bottom Quick Stats -->
                    <div class="grid grid-cols-2 divide-x divide-border/10 border-t border-border/10 bg-black/20 p-2 z-10 relative">
                        <div class="p-2 text-center flex flex-col items-center">
                            <span class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/60 mb-1">Win Rate Geral</span>
                            <span class="font-mono text-lg font-black tabular-nums {data.stats.winRate >= 50 ? 'text-emerald-500' : 'text-rose-500'}">{data.stats.winRate}%</span>
                        </div>
                        <div class="p-2 text-center flex flex-col items-center">
                            <span class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/60 mb-1">Resultado (Base)</span>
                            <span class="font-mono text-lg font-black tabular-nums {data.stats.totalProfit >= 0 ? 'text-emerald-500' : 'text-rose-500'}">
                                {data.profile.target_type === 'Financial' ? formatCurrency(data.stats.totalProfit, getProfileCurrencyCode(data.profile)) : `${data.stats.totalProfit}pts`}
                            </span>
                        </div>
                    </div>
                </Card.Root>

                <!-- Widget 2: Equity Curve (8 cols) -->
                <Card.Root class="lg:col-span-8 border-border/10 shadow-lg card-glass p-5 flex flex-col min-h-[300px]">
                    <div class="flex items-center justify-between mb-4">
                        <div class="flex items-center gap-2">
                           <TrendingUp class="w-5 h-5 text-emerald-500" />
                           <h3 class="text-sm font-bold uppercase tracking-widest text-muted-foreground/80">Curva de Capital Recente</h3>
                        </div>
                        <span class="px-2 py-0.5 rounded-full bg-muted/20 text-[10px] uppercase tracking-widest font-bold text-muted-foreground/60 border border-border/10">14 Dias</span>
                    </div>
                    <div class="flex-1 w-full relative">
                        <!-- Chart component fills the space -->
                        <EChart options={data.chartOptions} />
                    </div>
                </Card.Root>

                <!-- Widget 3: Discipline Checklist (Full Width List) -->
                <Card.Root class="lg:col-span-12 border-border/10 shadow-lg card-glass p-5">
                    <div class="flex items-center gap-2 mb-6 border-b border-border/10 pb-4">
                        <Shield class="w-5 h-5 text-primary" />
                        <h3 class="text-sm font-bold uppercase tracking-widest text-foreground">Checklist de Disciplina do Perfil</h3>
                    </div>

                    <!-- Checklist Rules Table -->
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <!-- Stat 1 -->
                        <div class="flex items-center p-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 gap-4 shadow-sm group hover:bg-emerald-500/10 transition-colors">
                            <div class="w-10 h-10 rounded-full bg-emerald-500/20 flex items-center justify-center">
                                <CheckCircle2 class="w-6 h-6 text-emerald-500 filter drop-shadow-[0_0_2px_rgba(16,185,129,0.8)]" />
                            </div>
                            <div class="flex flex-col">
                                <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground/60">Dias de Lucro</span>
                                <span class="font-mono text-xl font-black text-foreground">{data.stats.daysPositive} <span class="text-xs font-medium text-muted-foreground lowercase tracking-normal">dias positivos</span></span>
                            </div>
                        </div>

                         <!-- Stat 2 -->
                        <div class="flex items-center p-4 rounded-xl border border-border/10 bg-black/20 gap-4 shadow-sm group hover:bg-black/30 transition-colors">
                            <div class="w-10 h-10 rounded-full bg-muted/10 flex items-center justify-center">
                                <Activity class="w-6 h-6 text-muted-foreground" />
                            </div>
                            <div class="flex flex-col">
                                <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground/60">Trades Máximo/Dia</span>
                                <span class="font-mono text-xl font-black text-foreground">{data.profile.max_trades_per_day} <span class="text-xs font-medium text-muted-foreground lowercase tracking-normal">limite</span></span>
                            </div>
                        </div>

                         <!-- Stat 3 -->
                        <div class="flex items-center p-4 rounded-xl border {data.stats.currentDrawdown >= data.profile.max_daily_loss ? 'border-rose-500/30 bg-rose-500/10' : 'border-emerald-500/20 bg-emerald-500/5'} gap-4 shadow-sm">
                            <div class="w-10 h-10 rounded-full {data.stats.currentDrawdown >= data.profile.max_daily_loss ? 'bg-rose-500/20' : 'bg-emerald-500/20'} flex items-center justify-center">
                                {#if data.stats.currentDrawdown >= data.profile.max_daily_loss}
                                    <XCircle class="w-6 h-6 text-rose-500 filter drop-shadow-[0_0_2px_rgba(244,63,94,0.8)]" />
                                {:else}
                                    <CheckCircle2 class="w-6 h-6 text-emerald-500 filter drop-shadow-[0_0_2px_rgba(16,185,129,0.8)]" />
                                {/if}
                            </div>
                            <div class="flex flex-col">
                                <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground/60">Sobrevivência Diária</span>
                                <span class="font-mono text-xl font-black {data.stats.currentDrawdown >= data.profile.max_daily_loss ? 'text-rose-500' : 'text-emerald-500'}">{data.stats.currentDrawdown >= data.profile.max_daily_loss ? 'STOPPADO' : 'OPERACIONAL'}</span>
                            </div>
                        </div>
                    </div>
                </Card.Root>

                <!-- Widget 4: Growth Phase (Dependent on whether it's enabled) -->
                {#if data.profile.growth_plan_enabled && data.profile.growth_phases.length > 0}
                    {@const currentPhase = data.profile.growth_phases[data.profile.current_phase_index] || data.profile.growth_phases[0]}
                    <Card.Root class="lg:col-span-12 border-primary/20 shadow-lg bg-black/10 overflow-hidden relative">
                        <div class="absolute top-0 left-0 w-1 h-full bg-primary"></div>
                        <div class="p-6">
                            <div class="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-4">
                                
                                <div class="flex items-center gap-3">
                                   <div class="p-2.5 bg-primary/10 rounded-lg">
                                       <Layers class="w-6 h-6 text-primary" />
                                   </div>
                                   <div>
                                       <span class="text-[10px] font-black uppercase tracking-widest text-primary mb-0.5 block">Plano de Crescimento Ativo</span>
                                       <h2 class="text-2xl font-black text-foreground tracking-tight">
                                           {currentPhase.name || `Fase ${currentPhase.level}`}
                                       </h2>
                                   </div>
                                </div>

                                <div class="flex gap-3 items-center">
                                     <div class="flex flex-col p-3 rounded-lg border border-border/10 bg-black/20 min-w[100px] items-center text-center">
                                         <span class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/50 mb-1">Mão Máxima (Contratos)</span>
                                         <span class="text-xl font-bold font-mono tracking-tight text-emerald-500">{currentPhase.lot_size}x</span>
                                     </div>
                                     <div class="flex flex-col p-3 rounded-lg border border-border/10 bg-black/20 items-center text-center">
                                         <span class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/50 mb-1">Fase</span>
                                         <span class="text-xl font-bold font-mono tracking-tight text-primary">{data.profile.current_phase_index + 1}<span class="text-xs text-muted-foreground font-normal">/{data.profile.growth_phases.length}</span></span>
                                     </div>
                                     <div class="flex flex-col gap-2">
                                         <Button size="sm" variant="outline"
                                             class="h-8 text-xs border-emerald-500/40 text-emerald-500 hover:bg-emerald-500/10 hover:border-emerald-500 disabled:opacity-30"
                                             disabled={data.profile.current_phase_index >= data.profile.growth_phases.length - 1}
                                             onclick={promotePhase}>
                                             <ChevronRight class="w-4 h-4 mr-1" /> Promover
                                         </Button>
                                         <Button size="sm" variant="outline"
                                             class="h-8 text-xs border-rose-500/40 text-rose-500 hover:bg-rose-500/10 hover:border-rose-500 disabled:opacity-30"
                                             disabled={data.profile.current_phase_index <= 0}
                                             onclick={demotePhase}>
                                             <ChevronLeft class="w-4 h-4 mr-1" /> Regredir
                                         </Button>
                                     </div>
                                </div>

                            </div>

                            <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                                <!-- Rules to advance -->
                                <div>
                                    <h4 class="text-[11px] font-bold uppercase tracking-widest text-muted-foreground mb-3 flex items-center gap-1.5"><ArrowRight class="w-3 h-3 text-emerald-500"/> Requisitos para Avançar (Promote)</h4>
                                    <div class="space-y-2">
                                        {#if !currentPhase.conditions_to_advance || currentPhase.conditions_to_advance.length === 0}
                                            <div class="p-4 rounded-lg bg-muted/5 border border-dashed border-border/20 text-center text-sm font-medium text-muted-foreground/60">
                                                Avanço Manual (Sem regras automáticas criadas).
                                            </div>
                                        {:else}
                                            {#each currentPhase.conditions_to_advance as rule}
                                                {@const evalResult = evaluateCondition(data.stats, rule)}
                                                <div class="flex items-center justify-between p-3 rounded-lg border border-border/10 bg-black/20 group hover:border-emerald-500/30 transition-colors">
                                                    <div class="flex items-center gap-3">
                                                       {#if evalResult.passed}
                                                           <CheckCircle2 class="w-4 h-4 text-emerald-500 drop-shadow-[0_0_2px_rgba(16,185,129,0.5)]" />
                                                       {:else}
                                                           <div class="w-4 h-4 rounded-full border-2 border-muted-foreground/30"></div>
                                                       {/if}
                                                       <span class="text-xs font-bold uppercase tracking-widest text-foreground/80 group-hover:text-foreground transition-colors">{rule.metric.replace('_', ' ')} <span class="text-muted-foreground lowercase mx-1">{rule.operator}</span> {rule.value}</span>
                                                    </div>
                                                    <span class="font-mono text-sm font-bold {evalResult.passed ? 'text-emerald-500' : 'text-muted-foreground'}">
                                                        {Number.isInteger(evalResult.actual) ? evalResult.actual : evalResult.actual.toFixed(2)}
                                                    </span>
                                                </div>
                                            {/each}
                                        {/if}
                                    </div>
                                </div>

                                <!-- Rules to demote -->
                                <div>
                                    <h4 class="text-[11px] font-bold uppercase tracking-widest text-muted-foreground mb-3 flex items-center gap-1.5"><TrendingDown class="w-3 h-3 text-rose-500"/> Alertas de Regressão (Demote)</h4>
                                    <div class="space-y-2">
                                        {#if !currentPhase.conditions_to_demote || currentPhase.conditions_to_demote.length === 0}
                                            <div class="p-4 rounded-lg bg-muted/5 border border-dashed border-border/20 text-center text-sm font-medium text-muted-foreground/60">
                                                Nenhuma regra de regressão criada para esta fase.
                                            </div>
                                        {:else}
                                            {#each currentPhase.conditions_to_demote as rule}
                                                {@const evalResult = evaluateCondition(data.stats, rule)}
                                                <!-- For Demote, passing means BAD (condition met means demote triggers) -->
                                                <div class="flex items-center justify-between p-3 rounded-lg border border-border/10 bg-black/20 group hover:border-rose-500/30 transition-colors">
                                                    <div class="flex items-center gap-3">
                                                       {#if evalResult.passed}
                                                           <AlertTriangle class="w-4 h-4 text-rose-500 drop-shadow-[0_0_2px_rgba(244,63,94,0.5)]" />
                                                       {:else}
                                                           <div class="w-4 h-4 rounded-full bg-emerald-500/20 flex items-center justify-center"><CheckCircle2 class="w-3 h-3 text-emerald-500" /></div>
                                                       {/if}
                                                       <span class="text-xs font-bold uppercase tracking-widest text-foreground/80 group-hover:text-foreground transition-colors">{rule.metric.replace('_', ' ')} <span class="text-muted-foreground lowercase mx-1">{rule.operator}</span> {rule.value}</span>
                                                    </div>
                                                    <span class="font-mono text-sm font-bold {evalResult.passed ? 'text-rose-500' : 'text-muted-foreground'}">
                                                        {Number.isInteger(evalResult.actual) ? evalResult.actual : evalResult.actual.toFixed(2)}
                                                    </span>
                                                </div>
                                            {/each}
                                        {/if}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Card.Root>
                {/if}
            </div>
        {/if}
    {/if}
</div>
