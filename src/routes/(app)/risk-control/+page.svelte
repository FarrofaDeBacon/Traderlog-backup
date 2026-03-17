```
<script lang="ts">
    import { t } from "svelte-i18n";
    import { 
        Activity, Shield, Target, TrendingUp, TrendingDown, 
        CheckCircle2, XCircle, ArrowRight, Gauge, Layers, 
        AlertTriangle, ChevronRight, ChevronLeft, Calendar, 
        History, Lock, HelpCircle, Box, Info 
    } from 'lucide-svelte';
    import * as Card from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import EChart from "$lib/components/ui/echart.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { riskStore } from "$lib/stores/riskStore.svelte";
    import { formatCurrency } from "$lib/utils";
    import { evaluateCondition, computeRiskStats } from "$lib/utils/riskLogic";
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

    // TODO: migrar este trecho para riskStore/riskCockpitState
    let stats = $derived(currentProfile ? computeRiskStats(tradesStore.trades, currentProfile) : null);
    
    let riskData = $derived(riskStore.riskCockpitState);

    let chartOptions = $derived(() => {
        if (!stats) return undefined;

        const curve = stats.dailyEquityCurve;
        const lastValue = curve[curve.length - 1]?.value ?? 0;
        const isPositive = lastValue >= 0;

        return {
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
                <!-- Sizing Asset Selector (Temporary) -->
                <div class="hidden sm:flex items-center gap-2">
                    <span class="text-xs text-muted-foreground uppercase font-bold tracking-wider">Ativo:</span>
                    <select 
                        class="bg-black/20 border border-border/20 text-foreground text-xs rounded px-2 py-1 outline-none min-w-[80px]"
                        bind:value={riskStore.activeAssetId}
                    >
                        <option value={null}>Selecione...</option>
                        {#each settingsStore.assets as asset}
                            <option value={asset.id}>{asset.symbol}</option>
                        {/each}
                    </select>
                </div>
                
                <!-- Profile Base Selector (Badges) -->
                <div class="flex flex-wrap items-center gap-2 border-l border-border/10 pl-3">
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

        {#if currentProfile && stats && riskData && chartOptions()}
            <!-- Cockpit Dashboard Grid -->
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">

                <!-- Widget 1: Thermometer & Capital (4 cols) -->
                <Card.Root class="lg:col-span-4 border-border/10 shadow-lg card-glass flex flex-col justify-between overflow-hidden relative group">
                    <div class="absolute inset-0 bg-gradient-to-br from-primary/5 to-transparent opacity-50 pointer-events-none"></div>
                    
                    <div class="p-5 flex-1 relative z-10 space-y-5">
                        <div class="flex items-center justify-between mb-2">
                            <div class="flex items-center gap-2">
                               <Gauge class="w-5 h-5 text-primary" />
                               <h3 class="text-sm font-bold uppercase tracking-widest text-muted-foreground/80">Capital & Limites</h3>
                            </div>
                            <div class="px-2 py-0.5 rounded text-[10px] font-black uppercase tracking-widest {riskData?.dailyRiskStatus.isLocked ? 'bg-rose-500/10 text-rose-500 border border-rose-500/20' : 'bg-primary/10 text-primary border border-primary/20'}">
                                {riskData?.dailyRiskStatus.statusLabel.replace(/_/g, ' ') || 'RUNNING'}
                            </div>
                        </div>

                        <!-- Capital base -->
                        <div class="space-y-1">
                            <p class="text-[10px] text-muted-foreground/60 uppercase tracking-widest font-bold">Base de Operação</p>
                            <h2 class="text-3xl font-black font-mono tracking-tight glow-text flex items-end gap-2">
                                {#if currentProfile.capital_source === "LinkedAccount" && currentProfile.linked_account_id}
                                    <!-- Use dynamic balance logic here in production -->
                                    <span class="text-foreground/80">{getProfileCurrencyCode(currentProfile)}</span>
                                    {formatCurrency(settingsStore.accounts.find(a => a.id === currentProfile.linked_account_id)?.balance || 0, getProfileCurrencyCode(currentProfile)).replace(getProfileCurrencyCode(currentProfile), '')}
                                {:else}
                                    <span class="text-foreground/80">{getProfileCurrencyCode(currentProfile)}</span>
                                    {formatCurrency(currentProfile.fixed_capital, getProfileCurrencyCode(currentProfile)).replace(getProfileCurrencyCode(currentProfile), '')}
                                {/if}
                            </h2>
                        </div>

                        <!-- Drawdown Gauge -->
                        <div class="p-4 rounded-xl border border-border/10 bg-black/20 shadow-inner space-y-3">
                            <div class="flex justify-between items-center text-sm">
                                <span class="text-muted-foreground font-medium uppercase tracking-widest text-[10px]">Tolerância Diária Utilizada</span>
                                <!-- TODO: migrar este trecho para riskStore/riskCockpitState -->
                                <span class="font-bold font-mono tracking-tight text-rose-500">
                                    {currentProfile.target_type === 'Financial' ? formatCurrency(stats.currentDrawdown, getProfileCurrencyCode(currentProfile)) : `${stats.currentDrawdown} pts`} 
                                    <span class="text-muted-foreground/40 font-normal">
                                        / {currentProfile.target_type === 'Financial' ? formatCurrency(currentProfile.max_daily_loss, getProfileCurrencyCode(currentProfile)) : `${currentProfile.max_daily_loss} pts`}
                                    </span>
                                </span>
                            </div>
                            <!-- Styled Progress Bar -->
                            <div class="h-3 bg-rose-500/10 rounded-full overflow-hidden shadow-inner relative">
                                <div class="absolute top-0 bottom-0 left-0 bg-rose-500 transition-all duration-700 rounded-full" style="width: {(stats.currentDrawdown / currentProfile.max_daily_loss) * 100}%">
                                    <div class="absolute inset-0 bg-white/20 blur-[2px]"></div>
                                </div>
                            </div>
                            <p class="text-[10px] text-rose-500/80 uppercase tracking-wider text-right font-bold w-full">
                                {((stats.currentDrawdown / currentProfile.max_daily_loss) * 100).toFixed(1)}% do Limite Consumido
                            </p>
                        </div>
                    </div>

                    <!-- Bottom Quick Stats -->
                    <div class="grid grid-cols-2 divide-x divide-border/10 border-t border-border/10 bg-black/20 p-2 z-10 relative">
                        <div class="p-2 text-center flex flex-col items-center">
                            <span class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/60 mb-1">Win Rate Geral</span>
                            <!-- TODO: migrar este trecho para riskStore/riskCockpitState -->
                            <span class="font-mono text-lg font-black tabular-nums {stats.winRate >= 50 ? 'text-emerald-500' : 'text-rose-500'}">{stats.winRate}%</span>
                        </div>
                        <div class="p-2 text-center flex flex-col items-center group relative">
                            <span class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/60 mb-1 flex items-center gap-1">
                                Resultado Diário
                                {#if riskData.dailyRiskStatus.dailyTargetHit}
                                    <Target class="w-3 h-3 text-emerald-500" />
                                {/if}
                            </span>
                            <span class="font-mono text-lg font-black tabular-nums {riskData.dailyRiskStatus.dailyTargetHit ? 'text-emerald-500 drop-shadow-[0_0_5px_rgba(16,185,129,0.5)]' : ((riskData.dailyRiskStatus.dailyPnL ?? 0) >= 0 ? 'text-emerald-500' : 'text-rose-500')}">
                                {currentProfile.target_type === 'Financial' ? formatCurrency(riskData.dailyRiskStatus.dailyPnL || 0, getProfileCurrencyCode(currentProfile)) : `${riskData.dailyRiskStatus.dailyPnLPoints || 0}pts`}
                            </span>
                        </div>
                    </div>
                </Card.Root>

                <!-- Chart Column (8 cols) -->
                <Card.Root class="lg:col-span-8 border-border/10 shadow-lg card-glass flex flex-col h-[400px]">
                    <Card.Header class="pb-2 flex-none">
                        <Card.Title class="text-lg flex items-center gap-2">
                            <TrendingUp class="w-5 h-5 text-emerald-400" />
                            Curva de Capital (Drawdown de Topo)
                        </Card.Title>
                        <Card.Description>
                            Evolução financeira baseada apenas nas operações deste Perfil
                        </Card.Description>
                    </Card.Header>
                    <Card.Content class="flex-1 w-full relative -ml-4">
                        <EChart options={chartOptions()} />
                    </Card.Content>
                </Card.Root>

                <!-- New Card: Position Sizing Engine Result -->
                <Card.Root class="lg:col-span-4 border-border/10 shadow-lg card-glass">
                    <Card.Header class="pb-2 border-b border-border/10">
                        <Card.Title class="text-base flex items-center gap-2">
                            <Box class="w-5 h-5 text-blue-400" />
                            Position Sizing
                        </Card.Title>
                        <Card.Description>Contratos Permitidos Hoje</Card.Description>
                    </Card.Header>
                    <Card.Content class="pt-4 space-y-4">
                        {#if !riskStore.activeAssetId}
                            <div class="text-sm text-muted-foreground flex items-center gap-2 p-3 bg-black/20 rounded-md border border-border/10">
                                <Info class="w-4 h-4 text-blue-400 shrink-0" />
                                Selecione um Ativo no topo da tela para calcular o tamanho de posição.
                            </div>
                        {:else if riskStore.positionSizingResult}
                            {@const b = riskStore.positionSizingResult}
                            
                            <div class="flex items-center justify-between pb-3 border-b border-border/10">
                                <span class="text-sm text-muted-foreground">Lotes Permitidos</span>
                                {#if b.isValid}
                                    <span class="text-3xl font-bold text-emerald-400 tracking-tighter">{b.allowedContracts}</span>
                                {:else}
                                    <span class="text-3xl font-bold text-rose-500 tracking-tighter">0</span>
                                {/if}
                            </div>
                            
                            <div class="space-y-2">
                                <div class="flex justify-between text-xs">
                                    <span class="text-muted-foreground">Risco Máx (Hedge)</span>
                                    <span class="font-medium">R$ {b.allowedRisk.toFixed(2)}</span>
                                </div>
                                <div class="flex justify-between text-xs">
                                    <span class="text-muted-foreground">Custo por Contrato</span>
                                    <span class="font-medium">R$ {b.riskPerContract.toFixed(2)}</span>
                                </div>
                            </div>

                            {#if !b.isValid && b.reasons.length > 0}
                                <div class="mt-4 p-3 bg-rose-500/10 border border-rose-500/20 rounded-md space-y-1">
                                    <div class="flex items-center gap-2 text-rose-400 font-bold text-xs uppercase tracking-wider mb-2">
                                        <AlertTriangle class="w-3 h-3" />
                                        Operação Bloqueada
                                    </div>
                                    <ul class="text-xs text-rose-300/80 list-disc pl-4 space-y-1">
                                        {#each b.reasons as reason}
                                            <li>{reason}</li>
                                        {/each}
                                    </ul>
                                </div>
                            {:else if b.reasons.length > 0}
                                <div class="mt-4 p-3 bg-amber-500/10 border border-amber-500/20 rounded-md space-y-1">
                                    <div class="flex items-center gap-2 text-amber-400 font-bold text-xs uppercase tracking-wider mb-2">
                                        <AlertTriangle class="w-3 h-3" />
                                        Limites de Fase Aplicados
                                    </div>
                                    <ul class="text-xs text-amber-300/80 list-disc pl-4 space-y-1">
                                        {#each b.reasons as reason}
                                            <li>{reason}</li>
                                        {/each}
                                    </ul>
                                </div>
                            {/if}
                        {:else}
                             <div class="text-sm text-muted-foreground">
                                Dados insuficientes para calcular dimensionamento (sem parâmetros no ativo).
                             </div>
                        {/if}
                    </Card.Content>
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
                                <!-- TODO: migrar este trecho para riskStore/riskCockpitState -->
                                <span class="font-mono text-xl font-black text-foreground">{stats.daysPositive} <span class="text-xs font-medium text-muted-foreground lowercase tracking-normal">dias positivos</span></span>
                            </div>
                        </div>

                         <!-- Stat 2: Consuming DOMAIN disciplineEvaluation -->
                        <div class="flex items-center p-4 rounded-xl border {riskData.disciplineEvaluation?.overtradingDetected ? 'border-rose-500/30 bg-rose-500/10' : 'border-border/10 bg-black/20'} gap-4 shadow-sm group hover:border-border/30 transition-colors">
                            <div class="w-10 h-10 rounded-full {riskData.disciplineEvaluation?.overtradingDetected ? 'bg-rose-500/20' : 'bg-muted/10'} flex items-center justify-center">
                                {#if riskData.disciplineEvaluation?.overtradingDetected}
                                    <AlertTriangle class="w-6 h-6 text-rose-500 filter drop-shadow-[0_0_2px_rgba(244,63,94,0.8)]" />
                                {:else}
                                    <Activity class="w-6 h-6 text-muted-foreground" />
                                {/if}
                            </div>
                            <div class="flex flex-col">
                                <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground/60 flex gap-1 items-center">
                                    Trades Máximo/Dia
                                </span>
                                <!-- TODO: migrar este trecho para riskStore/riskCockpitState -->
                                <span class="font-mono text-xl font-black {riskData.disciplineEvaluation?.overtradingDetected ? 'text-rose-500' : 'text-foreground'}">
                                    {currentProfile.max_trades_per_day} <span class="text-xs font-medium text-muted-foreground lowercase tracking-normal">limite</span>
                                </span>
                            </div>
                        </div>

                         <!-- Stat 3: Consuming DOMAIN riskData -->
                        <div class="flex items-center p-4 rounded-xl border {riskData.dailyRiskStatus.dailyLossHit ? 'border-rose-500/30 bg-rose-500/10' : (riskData.dailyRiskStatus.isLocked ? 'border-amber-500/30 bg-amber-500/10' : 'border-emerald-500/20 bg-emerald-500/5')} gap-4 shadow-sm">
                            <div class="w-10 h-10 rounded-full {riskData.dailyRiskStatus.dailyLossHit ? 'bg-rose-500/20' : (riskData.dailyRiskStatus.isLocked ? 'bg-amber-500/20' : 'bg-emerald-500/20')} flex items-center justify-center">
                                {#if riskData.dailyRiskStatus.dailyLossHit}
                                    <XCircle class="w-6 h-6 text-rose-500 filter drop-shadow-[0_0_2px_rgba(244,63,94,0.8)]" />
                                {:else if riskData.dailyRiskStatus.isLocked}
                                    <Shield class="w-6 h-6 text-amber-500 filter drop-shadow-[0_0_2px_rgba(245,158,11,0.8)]" />
                                {:else}
                                    <CheckCircle2 class="w-6 h-6 text-emerald-500 filter drop-shadow-[0_0_2px_rgba(16,185,129,0.8)]" />
                                {/if}
                            </div>
                            <div class="flex flex-col">
                                <span class="text-[10px] uppercase tracking-widest font-bold text-muted-foreground/60">Sobrevivência Diária</span>
                                <span class="font-mono text-xl font-black {riskData.dailyRiskStatus.dailyLossHit ? 'text-rose-500' : (riskData.dailyRiskStatus.isLocked ? 'text-amber-500' : 'text-emerald-500')}">{riskData.dailyRiskStatus.dailyLossHit ? 'STOPPADO' : (riskData.dailyRiskStatus.isLocked ? 'BLOQUEADO' : 'OPERACIONAL')}</span>
                            </div>
                        </div>
                    </div>
                </Card.Root>

                <!-- Widget 4: Growth Phase (Dependent on whether it's enabled) -->
                {#if currentProfile.growth_plan_enabled && currentProfile.growth_phases.length > 0}
                    {@const currentPhase = currentProfile.growth_phases[currentProfile.current_phase_index] || currentProfile.growth_phases[0]}
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
                                         <span class="text-xl font-bold font-mono tracking-tight text-primary">{currentProfile.current_phase_index + 1}<span class="text-xs text-muted-foreground font-normal">/{currentProfile.growth_phases.length}</span></span>
                                     </div>
                                     <div class="flex flex-col gap-2">
                                         <!-- Consuming DOMAIN growthEvaluation -->
                                         <Button size="sm" variant="outline"
                                             class="h-8 text-xs border-emerald-500/40 text-emerald-500 hover:bg-emerald-500/10 hover:border-emerald-500 disabled:opacity-30 relative"
                                             disabled={currentProfile.current_phase_index >= currentProfile.growth_phases.length - 1 || riskData.growthEvaluation?.canPromote === false}
                                             onclick={promotePhase}>
                                             <ChevronRight class="w-4 h-4 mr-1" /> Promover
                                             {#if riskData.growthEvaluation?.canPromote}
                                                <div class="absolute -top-1 -right-1 w-2.5 h-2.5 bg-emerald-500 rounded-full animate-pulse blur-[1px]"></div>
                                             {/if}
                                         </Button>
                                         <Button size="sm" variant="outline"
                                             class="h-8 text-xs border-rose-500/40 text-rose-500 hover:bg-rose-500/10 hover:border-rose-500 disabled:opacity-30 relative"
                                             disabled={currentProfile.current_phase_index <= 0 || riskData.growthEvaluation?.shouldRegress === false}
                                             onclick={demotePhase}>
                                             <ChevronLeft class="w-4 h-4 mr-1" /> Regredir
                                             {#if riskData.growthEvaluation?.shouldRegress}
                                                <div class="absolute -top-1 -right-1 w-2.5 h-2.5 bg-rose-500 rounded-full animate-pulse blur-[1px]"></div>
                                             {/if}
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
                                            <!-- TODO: migrar este trecho para riskStore/riskCockpitState -->
                                            {#each currentPhase.conditions_to_advance as rule}
                                                {@const evalResult = evaluateCondition(stats, rule)}
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
                                    <h4 class="text-[11px] font-bold uppercase tracking-widest text-muted-foreground mb-3 flex items-center gap-1.5">
                                        <TrendingDown class="w-3 h-3 text-rose-500"/> Alertas de Regressão (Demote)
                                        <!-- TODO: migrar este trecho para riskStore/riskCockpitState -->
                                    </h4>
                                    <div class="space-y-2">
                                        {#if !currentPhase.conditions_to_demote || currentPhase.conditions_to_demote.length === 0}
                                            <div class="p-4 rounded-lg bg-muted/5 border border-dashed border-border/20 text-center text-sm font-medium text-muted-foreground/60">
                                                Nenhuma regra de regressão criada para esta fase.
                                            </div>
                                        {:else}
                                            <!-- TODO: migrar este trecho para riskStore/riskCockpitState -->
                                            {#each currentPhase.conditions_to_demote as rule}
                                                {@const evalResult = evaluateCondition(stats, rule)}
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
