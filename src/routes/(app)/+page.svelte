<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { t } from "svelte-i18n";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { tradesStore } from "$lib/stores/trades.svelte";
  import EChart from "$lib/components/ui/echart.svelte";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
    CardDescription,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Separator } from "$lib/components/ui/separator";
  import * as Dialog from "$lib/components/ui/dialog";
  import NewTradeWizard from "$lib/components/trades/NewTradeWizard.svelte";
  import CurrencyTicker from "$lib/components/finance/CurrencyTicker.svelte";
  import {
    TrendingUp,
    TrendingDown,
    Activity,
    Calendar,
    BarChart3,
    ShieldCheck,
    Zap,
    Trophy,
    ArrowUpRight,
    Target,
    BookOpen,
    Wallet,
    Plus,
    Timer,
  } from "lucide-svelte";
  import { isSameDay } from "date-fns/isSameDay";
  import { startOfMonth } from "date-fns/startOfMonth";
  import { format } from "date-fns/format";
  import { parseISO } from "date-fns/parseISO";
  import PerformanceCalendar from "$lib/components/dashboard/PerformanceCalendar.svelte";
  import { ptBR } from "date-fns/locale/pt-BR";
  import { cn, parseSafeDate } from "$lib/utils";
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from "$lib/components/ui/select";


  // --- State ---
  let selectedAccountId = $state<string>("all");
  let isNewTradeOpen = $state(false);
  let isDailyDetailOpen = $state(false);
  let selectedDateForDetail = $state<Date | null>(null);

  const filteredTrades = $derived.by(() => {
    let trades = tradesStore.trades || [];
    if (selectedAccountId !== "all") {
      trades = trades.filter((t: any) => t.account_id === selectedAccountId);
    }
    return trades;
  });

  const selectedAccount = $derived(
    settingsStore.accounts.find((a) => a.id === selectedAccountId),
  );

  const activeProfile = $derived(
    settingsStore.activeProfile || settingsStore.riskProfiles[0],
  );

  const lastPrice = $derived.by(() => {
    if (!filteredTrades || filteredTrades.length === 0) return 0;
    const sorted = [...filteredTrades].sort(
      (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime(),
    );
    return Number(sorted[0].entry_price) || 0;
  });

  const activePhase = $derived.by(() => {
    const profile = activeProfile;
    if (
      !profile ||
      !profile.growth_phases ||
      profile.current_phase_index === undefined
    )
      return null;
    return profile.growth_phases[profile.current_phase_index] || null;
  });

  // --- Mastery Stats Engine ---
  const totalBalanceBRL = $derived(
    tradesStore.getTotalBalanceBRL(
      settingsStore.accounts,
      settingsStore.currencies,
    ),
  );

  const stats = $derived.by(() => {
    try {
      const trades = filteredTrades;
      const today = new Date();
      const thisMonth = startOfMonth(today);
      const yearMonthStr = format(today, "yyyy-MM");

      if (!trades || trades.length === 0)
        return {
          net: 0,
          winRate: 0,
          profitFactor: 0,
          payoff: 0,
          equity: [],
          monthResult: 0,
          dayResult: 0,
          discipline: 100,
          drawdown: 0,
          tradesToday: 0,
          avgRFactor: 0,
        };

      const sorted = [...trades].sort((a, b) => {
        const da = parseSafeDate(a.date).getTime();
        const db = parseSafeDate(b.date).getTime();
        return (isNaN(da) ? 0 : da) - (isNaN(db) ? 0 : db);
      });

      let current = 0,
        totalW = 0,
        totalL = 0,
        wins = 0,
        dayRes = 0,
        monthRes = 0,
        discSum = 0,
        peak = 0,
        maxDD = 0,
        tradesToday = 0,
        rFactorSum = 0,
        rFactorCount = 0;

      const equity = sorted.map((t) => {
        const res = Number(t.result) || 0;
        current += res;

        // Drawdown Logic
        if (current > peak) peak = current;
        const dd = peak === 0 ? 0 : ((peak - current) / peak) * 100;
        if (dd > maxDD) maxDD = dd;

        if (res > 0) {
          wins++;
          totalW += res;
        } else if (res < 0) {
          totalL += Math.abs(res);
        }

        try {
          const tDate = parseISO(t.date);
          const tExitDate = t.exit_date ? parseISO(t.exit_date) : tDate;

          if (isSameDay(tExitDate, today)) {
            dayRes += res;
            tradesToday++;
          }

          // Use consistent year-month detection for parity with Finance
          const tMonthStr = format(tExitDate, "yyyy-MM");
          if (tMonthStr === yearMonthStr) {
            monthRes += res;
          }
        } catch (e) {
          // ignore single date error
        }

        discSum += t.followed_plan ? 100 : 0;

        // R-Factor calculation (Reward/Risk)
        if ((t as any).risk_amount && (t as any).risk_amount > 0) {
          rFactorSum += t.result / (t as any).risk_amount;
          rFactorCount++;
        }

        return [parseSafeDate(t.date).getTime(), current];
      });

      return {
        net: current,
        winRate: (wins / (trades.length || 1)) * 100,
        profitFactor: totalL === 0 ? (totalW > 0 ? 99 : 0) : totalW / totalL,
        payoff:
          wins > 0 && trades.length - wins > 0
            ? totalW / wins / (totalL / (trades.length - wins) || 1)
            : 0,
        equity,
        monthResult: monthRes,
        dayResult: dayRes,
        discipline: discSum / (trades.length || 1),
        drawdown: maxDD,
        tradesToday,
        avgRFactor: rFactorCount > 0 ? rFactorSum / rFactorCount : 0,
      };
    } catch (err) {
      console.error("[Dashboard] Error in stats derived:", err);
      return {
        net: 0,
        winRate: 0,
        profitFactor: 0,
        payoff: 0,
        equity: [],
        monthResult: 0,
        dayResult: 0,
        discipline: 0,
        drawdown: 0,
        tradesToday: 0,
        avgRFactor: 0,
      };
    }
  });

  const growthProgress = $derived.by(() => {
    if (!activePhase) return 0;
    const rule = activePhase.progression_rules?.find(
      (r: any) => r.condition === "profit_target",
    );
    if (!rule) return 0;
    const target = rule.value;
    return Math.min(Math.max((stats.net / (target || 1)) * 100, 0), 100);
  });

  function handleDayClick(day: Date) {
    selectedDateForDetail = day;
    isDailyDetailOpen = true;
  }

  function formatCurrency(val: number) {
    return new Intl.NumberFormat("pt-BR", {
      style: "currency",
      currency: "BRL",
    }).format(val);
  }

  // --- ECharts Clean Options ---
  const equityOptions = $derived({
    animation: true,
    backgroundColor: "transparent",
    tooltip: {
      trigger: "axis",
      textStyle: { fontFamily: "Inter", fontSize: 12 },
    },
    grid: { top: 10, right: 10, bottom: 30, left: 50 },
    xAxis: {
      type: "time",
      axisLine: { lineStyle: { color: "rgba(255,255,255,0.15)" } },
      axisLabel: { color: "#71717a", fontSize: 10 },
    },
    yAxis: {
      type: "value",
      splitLine: {
        lineStyle: { color: "rgba(255,255,255,0.1)", type: "dashed" },
      },
      axisLabel: { color: "#71717a", fontSize: 10 },
    },
    series: [
      {
        data: stats.equity,
        type: "line",
        smooth: true,
        symbol: "none",
        lineStyle: { width: 3, color: "#10b981" }, // Keeping primary green for trend
        areaStyle: {
          color: {
            type: "linear",
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              { offset: 0, color: "rgba(16, 185, 129, 0.15)" },
              { offset: 1, color: "transparent" },
            ],
          },
        },
      },
    ],
  });
</script>

{#if settingsStore.isLoadingData}
  <div class="flex items-center justify-center p-20 min-h-[60vh] w-full">
    <div class="flex flex-col items-center gap-4 text-center">
      <div
        class="w-10 h-10 border-4 border-primary border-t-transparent rounded-full animate-spin"
      ></div>
      <div class="space-y-1">
        <p
          class="text-sm font-bold text-foreground uppercase tracking-widest animate-pulse"
        >
          {$t("dashboard.syncingTerminal") || "Sincronizando Terminal"}
        </p>
        <p
          class="text-xs text-muted-foreground uppercase font-medium tracking-tighter opacity-70"
        >
          {$t("dashboard.waitingSurvivorData") ||
            "Aguardando dados do Survivor Hub..."}
        </p>
      </div>
    </div>
  </div>
{:else}
  <div class="space-y-6">
    <div class="px-4 pt-4 -mb-8">
      <CurrencyTicker />
    </div>

    <div class="flex-1 flex flex-col space-y-8 p-4 md:p-8 min-h-screen">
      <Dialog.Root bind:open={isNewTradeOpen}>
        <Dialog.Content
          class="max-w-3xl max-h-[85vh] p-0 border-0 bg-transparent shadow-none"
        >
          <NewTradeWizard close={() => (isNewTradeOpen = false)} />
        </Dialog.Content>
      </Dialog.Root>

      <!-- TOP Navigation & Survivor Hub (Compact Header) -->
      <div
        class="flex flex-col md:flex-row justify-between items-start md:items-center gap-6"
      >
        <div class="space-y-0.5">
          <h1 class="text-3xl font-bold tracking-tight text-foreground">
            {$t("dashboard.title")}
          </h1>
          <div class="flex items-center gap-3">
            <p class="text-sm text-muted-foreground">
              {$t("dashboard.subtitle")}
            </p>
            <Separator orientation="vertical" class="h-4" />
            <div class="flex items-center gap-2">
              <p class="text-xs font-medium text-muted-foreground/80">
                {$t("dashboard.terminalHub")}
              </p>
              <Select type="single" bind:value={selectedAccountId}>
                <SelectTrigger class="w-[180px] h-8 text-xs font-medium">
                  <div class="flex items-center gap-2">
                    <Wallet class="w-3 h-3 text-muted-foreground shrink-0" />
                    <span class="truncate">
                      {selectedAccountId === "all"
                        ? $t("general.all")
                        : settingsStore.accounts.find(
                            (a) => a.id === selectedAccountId,
                          )?.nickname || $t("trades.filters.account")}
                    </span>
                  </div>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{$t("general.all")}</SelectItem>
                  {#each settingsStore.accounts as acc}
                    <SelectItem value={acc.id}>{acc.nickname}</SelectItem>
                  {/each}
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <Button
            onclick={() => (isNewTradeOpen = true)}
            class="bg-emerald-600 hover:bg-emerald-700 shadow-md gap-2 font-semibold px-6"
          >
            <Plus class="w-4 h-4" />
            {$t("dashboard.newTrade")}
          </Button>
        </div>
      </div>

      <!-- ELITE KPI LINE: 7 Professional Horizontal Cards -->
      <div class="grid grid-cols-2 md:grid-cols-4 xl:grid-cols-7 gap-4">
        {#each [{ label: $t("general.balance") + " (Est.)", val: formatCurrency(totalBalanceBRL), icon: Wallet, color: "text-emerald-500", borderColor: "border-l-emerald-500" }, { label: $t("dashboard.kpis.netResult"), val: formatCurrency(stats.net), icon: TrendingUp, color: stats.net >= 0 ? "text-emerald-500" : "text-rose-500", borderColor: stats.net >= 0 ? "border-l-emerald-500" : "border-l-rose-500" }, { label: $t("dashboard.kpis.winRate"), val: `${stats.winRate.toFixed(1)}%`, icon: Trophy, color: "text-blue-500", borderColor: "border-l-blue-500" }, { label: $t("dashboard.kpis.profitFactor"), val: stats.profitFactor.toFixed(2), icon: Activity, color: "text-amber-500", borderColor: "border-l-amber-500" }, { label: $t("dashboard.kpis.discipline"), val: `${stats.discipline.toFixed(0)}%`, icon: Zap, color: "text-indigo-500", borderColor: "border-l-indigo-500" }, { label: $t("dashboard.kpis.payoff"), val: stats.payoff.toFixed(2), icon: ArrowUpRight, color: "text-cyan-400", borderColor: "border-l-cyan-400" }, { label: $t("dashboard.kpis.maxDrawdown"), val: `${(stats.drawdown || 0).toFixed(1)}%`, icon: Activity, color: "text-rose-500", borderColor: "border-l-rose-500" }] as kpi}
          {@const Icon = kpi.icon}
          <Card class="card-glass border-l-2 {kpi.borderColor}">
            <CardContent class="py-0.5 px-2.5">
              <div class="flex items-center justify-between space-y-0">
                <span
                  class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60 leading-none"
                  >{kpi.label}</span
                >
                <Icon class={cn("w-3 h-3", kpi.color)} />
              </div>
              <div class="mt-0">
                <h3
                  class="text-base font-mono font-bold text-foreground tabular-nums tracking-tight leading-none"
                >
                  {kpi.val}
                </h3>
              </div>
            </CardContent>
          </Card>
        {/each}
      </div>

      <Separator />

      <div class="grid gap-4 lg:grid-cols-12 mt-2">
        <!-- Left Analytical Hub (8 Cols) -->
        <div class="col-span-full lg:col-span-8 space-y-2">
          <!-- Unified Survivor Control Center -->
          {#if activeProfile?.growth_plan_enabled}
            <div class="space-y-1.5">
              <div class="flex items-center justify-between">
                <h4
                  class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest pl-1 flex items-center gap-2"
                >
                  <Zap class="w-3 h-3 text-emerald-500 fill-emerald-500/20" />
                  {$t("dashboard.survivor.title")}
                </h4>

                <div class="flex items-center gap-2.5">
                  <div class="flex items-center gap-2">
                    <p class="text-xs font-medium text-muted-foreground/80">
                      {$t("dashboard.survivor.activePlan")}
                    </p>
                    <Select
                      type="single"
                      value={activeProfile?.id}
                      onValueChange={(val: string) =>
                        settingsStore.setActiveRiskProfile(val)}
                    >
                      <SelectTrigger
                        class="w-[150px] h-8 text-xs font-medium bg-emerald-500/5 border-emerald-500/20 text-emerald-500"
                      >
                        <div class="flex items-center gap-2">
                          <ShieldCheck class="w-3 h-3 shrink-0" />
                          <span class="truncate"
                            >{activeProfile?.name ||
                              $t("dashboard.survivor.select")}</span
                          >
                        </div>
                      </SelectTrigger>
                      <SelectContent>
                        {#each settingsStore.riskProfiles as profile}
                          <SelectItem value={profile.id}
                            >{profile.name}</SelectItem
                          >
                        {/each}
                      </SelectContent>
                    </Select>
                  </div>
                  <Separator orientation="vertical" class="h-4" />

                  <div class="flex items-center gap-1.5">
                    <span
                      class="text-[9px] font-bold text-muted-foreground/60 uppercase"
                      >{$t("dashboard.survivor.operationalStatus")}</span
                    >
                    {#if stats.dayResult <= -(activePhase?.max_daily_loss || 0) && (activePhase?.max_daily_loss ?? 0) > 0}
                      <Badge
                        variant="outline"
                        class="bg-rose-500/10 text-rose-500 border-rose-500/20 text-[9px] font-black uppercase tracking-tighter"
                        >{$t("dashboard.survivor.statusBlocked")}</Badge
                      >
                    {:else if stats.dayResult <= -(activePhase?.max_daily_loss || 0) * 0.7 && (activePhase?.max_daily_loss ?? 0) > 0}
                      <Badge
                        variant="outline"
                        class="bg-orange-500/10 text-orange-500 border-orange-500/20 text-[9px] font-black uppercase tracking-tighter"
                        >{$t("dashboard.survivor.statusRisk")}</Badge
                      >
                    {:else}
                      <Badge
                        variant="outline"
                        class="bg-emerald-500/10 text-emerald-500 border-emerald-500/20 text-[9px] font-black uppercase tracking-tighter"
                        >{$t("dashboard.survivor.statusAlpha")}</Badge
                      >
                    {/if}
                  </div>

                  <Separator orientation="vertical" class="h-3" />

                  <!-- Phase Roadmap -->
                  <div class="flex items-center gap-1.5">
                    {#each activeProfile?.growth_phases || [] as phase, i}
                      <div class="flex items-center">
                        <div
                          class={cn(
                            "w-1.5 h-1.5 rounded-full transition-all duration-300",
                            i === activeProfile?.current_phase_index
                              ? "bg-emerald-500 scale-125 shadow-[0_0_8px_rgba(16,185,129,0.5)]"
                              : i < (activeProfile?.current_phase_index || 0)
                                ? "bg-emerald-500/40"
                                : "bg-zinc-800",
                          )}
                        ></div>
                        {#if i < (activeProfile?.growth_phases?.length || 0) - 1}
                          <div
                            class={cn(
                              "w-3 h-[1px]",
                              i < (activeProfile?.current_phase_index || 0)
                                ? "bg-emerald-500/40"
                                : "bg-zinc-800",
                            )}
                          ></div>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
              </div>

              <div class="grid grid-cols-1 xl:grid-cols-5 gap-2.5">
                <!-- Card 1: Growth Engine (Span 3) -->
                <Card class="xl:col-span-4 card-glass overflow-hidden">
                  <CardContent class="p-0">
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-0">
                      <!-- SubCol 1: Current Phase -->
                      <div
                        class="py-0.5 px-2.5 border-r border-border/10 space-y-1"
                      >
                        <div class="flex items-center justify-between">
                          <span
                            class="text-[9px] font-black uppercase tracking-widest text-muted-foreground/50"
                            >{$t("dashboard.survivor.currentPhase")}</span
                          >
                          <span
                            class="text-[9px] font-mono font-bold text-emerald-400"
                            >{growthProgress.toFixed(0)}%</span
                          >
                        </div>
                        <h3
                          class="font-black text-foreground uppercase tracking-tight text-xs truncate"
                        >
                          {activePhase?.name}
                        </h3>
                        <div
                          class="h-1 bg-zinc-800 rounded-full overflow-hidden"
                        >
                          <div
                            class="h-full bg-emerald-500 transition-all duration-700"
                            style="width: {growthProgress}%"
                          ></div>
                        </div>
                      </div>

                      <!-- SubCol 2: Day Progress -->
                      <div
                        class="py-0.5 px-2.5 border-r border-border/10 space-y-1"
                      >
                        <div class="flex items-center justify-between">
                          <span
                            class="text-[9px] font-black uppercase tracking-widest text-muted-foreground/50"
                            >{$t("dashboard.survivor.dailyEvolution")}</span
                          >
                          <span
                            class="text-[9px] font-bold {stats.dayResult >= 0
                              ? 'text-emerald-500'
                              : 'text-rose-500'}"
                          >
                            {activeProfile?.daily_target > 0
                              ? (
                                  (stats.dayResult /
                                    activeProfile.daily_target) *
                                  100
                                ).toFixed(0)
                              : 0}%
                          </span>
                        </div>
                        <div class="flex items-center justify-between h-4">
                          <span
                            class="text-[11px] font-mono font-bold tabular-nums {stats.dayResult >=
                            0
                              ? 'text-emerald-400'
                              : 'text-rose-400'}"
                          >
                            {formatCurrency(stats.dayResult)}
                          </span>
                          <span
                            class="text-[9px] text-muted-foreground/50 font-medium"
                            >{$t("dashboard.survivor.target")}{formatCurrency(
                              activeProfile?.daily_target || 0,
                            )}</span
                          >
                        </div>
                        <div
                          class="h-1 bg-zinc-800 rounded-full overflow-hidden"
                        >
                          <div
                            class={cn(
                              "h-full transition-all duration-700",
                              stats.dayResult >= 0
                                ? "bg-emerald-500"
                                : "bg-rose-500",
                            )}
                            style="width: {Math.min(
                              activeProfile?.daily_target > 0
                                ? (Math.abs(stats.dayResult) /
                                    activeProfile.daily_target) *
                                    100
                                : 0,
                              100,
                            )}%"
                          ></div>
                        </div>
                      </div>

                      <!-- SubCol 3: Usage Counters -->
                      <div
                        class="py-0.5 px-2.5 border-r border-border/10 space-y-1"
                      >
                        <div class="flex justify-between items-center">
                          <span
                            class="text-[9px] font-black uppercase tracking-widest text-muted-foreground/50"
                            >{$t("dashboard.survivor.tradesUsage")}</span
                          >
                          <span
                            class="text-[9px] font-mono font-bold {stats.tradesToday >=
                            (activeProfile?.max_trades_per_day || 0)
                              ? 'text-amber-500'
                              : 'text-emerald-500'}"
                          >
                            {stats.tradesToday}/{activeProfile?.max_trades_per_day ||
                              0}
                          </span>
                        </div>
                        <div class="flex items-center gap-1">
                          {#each Array(activeProfile?.max_trades_per_day || 3) as _, i}
                            <div
                              class={cn(
                                "h-1 flex-1 rounded-full",
                                i < stats.tradesToday
                                  ? stats.tradesToday >
                                    (activeProfile?.max_trades_per_day || 0)
                                    ? "bg-rose-500"
                                    : "bg-emerald-600"
                                  : "bg-zinc-800/80",
                              )}
                            ></div>
                          {/each}
                        </div>
                        <div class="flex justify-between items-center">
                          <span
                            class="text-[9px] font-bold uppercase text-muted-foreground/40"
                            >{$t("dashboard.survivor.avgRFactor")}</span
                          >
                          <span
                            class="text-[10px] font-mono font-bold tabular-nums {stats.avgRFactor >=
                            (activeProfile?.min_risk_reward || 1)
                              ? 'text-emerald-500'
                              : 'text-amber-500'}"
                          >
                            {stats.avgRFactor.toFixed(2)}R
                          </span>
                        </div>
                      </div>

                      <!-- SubCol 4: Next Level Roadmap -->
                      <div class="py-0.5 px-2.5 space-y-1">
                        <p
                          class="text-[9px] font-black uppercase tracking-widest text-muted-foreground/50 flex items-center gap-1.5"
                        >
                          <Trophy class="w-2.5 h-2.5 text-amber-500" />
                          {$t("dashboard.survivor.roadmapProgress")}
                        </p>
                        <div class="flex flex-col gap-1">
                          {#each activePhase?.progression_rules || [] as rule}
                            <div class="flex items-center justify-between">
                              <span
                                class="text-[9px] font-medium text-muted-foreground/60"
                              >
                                {rule.condition === "profit_target"
                                  ? $t("dashboard.survivor.pnl")
                                  : $t("dashboard.survivor.consistency")}:
                              </span>
                              <span
                                class="text-[9px] font-black text-foreground tabular-nums"
                              >
                                {rule.condition === "profit_target"
                                  ? formatCurrency(rule.value)
                                  : `${rule.value}d`}
                              </span>
                            </div>
                          {/each}
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>

                <!-- Card 2: Risk Monitor (Survivor Focus) -->
                <Card class="card-glass overflow-hidden">
                  <CardContent class="p-0 flex flex-col h-full">
                    <div
                      class="py-0.5 px-2.5 flex-1 border-b border-border/10 space-y-0.5"
                    >
                      <div class="flex items-center justify-between">
                        <span
                          class="text-[9px] font-black uppercase tracking-widest text-muted-foreground/50 flex items-center gap-1.5"
                        >
                          <ShieldCheck class="w-2.5 h-2.5 text-rose-500" />
                          {$t("dashboard.survivor.riskMonth")}
                        </span>
                        <span class="text-[9px] font-bold text-rose-500"
                          >6% {$t("dashboard.survivor.limit")}</span
                        >
                      </div>
                      <div class="flex items-center justify-between">
                        <span
                          class="text-[11px] font-mono font-bold tabular-nums"
                        >
                          {(
                            (Math.abs(stats.monthResult) /
                              (selectedAccount?.balance || 10000)) *
                            100
                          ).toFixed(1)}%
                        </span>
                        <div
                          class="flex-1 max-w-[40px] h-1 bg-zinc-800 rounded-full ml-2 overflow-hidden"
                        >
                          <div
                            class="h-full bg-rose-500"
                            style="width: {Math.min(
                              (((Math.abs(stats.monthResult) /
                                (selectedAccount?.balance || 10000)) *
                                100) /
                                6) *
                                100,
                              100,
                            )}%"
                          ></div>
                        </div>
                      </div>
                    </div>
                    <div class="py-0.5 px-2.5 flex-1 bg-emerald-500/5">
                      <div class="flex items-center justify-between mb-1">
                        <span
                          class="text-[9px] font-black uppercase tracking-widest text-emerald-500/70 shrink-0"
                          >{$t("dashboard.survivor.eligibility")}</span
                        >
                        <Badge
                          variant="outline"
                          class="h-4 px-1 text-[8px] bg-emerald-500/10 border-emerald-500/20 text-emerald-500"
                          >{$t("dashboard.survivor.greenZone")}</Badge
                        >
                      </div>
                      <p
                        class="text-[10px] font-black text-emerald-400 uppercase tracking-tighter"
                      >
                        {$t("dashboard.survivor.operateTrade")}
                      </p>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </div>
          {/if}

          <!-- Equity Curve -->

          <Card class="p-4 card-glass mb-4">
            <div class="flex justify-between items-start mb-2">
              <div class="flex items-center gap-3">
                <div class="p-1 bg-emerald-500/10 rounded-lg">
                  <TrendingUp class="w-3.5 h-3.5 text-emerald-500" />
                </div>
                <div>
                  <h3
                    class="text-xs font-bold text-foreground tracking-tight leading-none"
                  >
                    {$t("dashboard.charts.equity")}
                  </h3>
                  <p class="text-[11px] text-muted-foreground">
                    {$t("dashboard.charts.equitySubtitle")}
                  </p>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <Badge
                  variant="outline"
                  class="bg-emerald-500/10 text-emerald-500 border-none font-bold px-2 py-0.5 text-[9px] uppercase"
                >
                  {filteredTrades.filter((t) => t.result > 0).length}
                  {$t("dashboard.charts.wins")}
                </Badge>
                <Badge
                  variant="outline"
                  class="bg-rose-500/10 text-rose-500 border-none font-bold px-2 py-0.5 text-[9px] uppercase"
                >
                  {filteredTrades.filter((t) => (t.result || 0) < 0).length}
                  {$t("dashboard.charts.losses")}
                </Badge>
                <Badge
                  variant="outline"
                  class="bg-blue-500/10 text-blue-500 border-none font-bold px-2 py-0.5 text-[9px] uppercase"
                >
                  {$t("dashboard.charts.consistency")}
                </Badge>
              </div>
            </div>
            <div class="h-[280px] w-full">
              <EChart options={equityOptions} />
            </div>
          </Card>
        </div>

        <div class="col-span-full lg:col-span-4 space-y-2">
          <!-- Compact Performance Calendar (Restored) -->
          <Card class="card-glass overflow-hidden p-2">
            <CardContent class="p-0">
              <PerformanceCalendar
                trades={filteredTrades}
                compact={true}
                onDateClick={handleDayClick}
              />
            </CardContent>
          </Card>



          <!-- Daily Detail Modal -->
          <Dialog.Root bind:open={isDailyDetailOpen}>
            <Dialog.Content
              class="max-w-4xl max-h-[90vh] overflow-y-auto text-left"
            >
              <Dialog.Header>
                <Dialog.Title
                  class="text-xl font-bold uppercase tracking-tight"
                >
                  {$t("dashboard.dailyDetail.title")}
                </Dialog.Title>
                <Dialog.Description
                  class="text-xs font-semibold text-muted-foreground uppercase tracking-widest"
                >
                  {$t("dashboard.dailyDetail.subtitle")}
                </Dialog.Description>
              </Dialog.Header>

              <div class="space-y-6 pt-4">
                <div class="flex items-center justify-between">
                  <h4 class="text-lg font-bold tracking-tight">
                    {#if selectedDateForDetail}
                      {format(selectedDateForDetail, "dd 'de' MMMM, yyyy", {
                        locale: ptBR,
                      })}
                    {/if}
                  </h4>
                </div>
                {#if selectedDateForDetail}
                  {@const dayTrades = filteredTrades.filter((t) =>
                    isSameDay(parseSafeDate(t.date), selectedDateForDetail as Date),
                  )}
                  {@const dayRes = dayTrades.reduce(
                    (acc, t) => acc + (t.result || 0),
                    0,
                  )}

                  <div class="grid grid-cols-3 gap-4">
                    <div class="p-4 card-glass border-border/10 shadow-none">
                      <p
                        class="text-[9px] font-bold text-muted-foreground uppercase tracking-widest mb-1"
                      >
                        {$t("dashboard.dailyDetail.summary")}
                      </p>
                      <p
                        class={cn(
                          "text-base font-mono font-bold tabular-nums",
                          dayRes >= 0 ? "text-emerald-500" : "text-rose-500",
                        )}
                      >
                        {formatCurrency(dayRes)}
                      </p>
                    </div>
                    <div class="p-4 card-glass border-border/10 shadow-none">
                      <p
                        class="text-[9px] font-bold text-muted-foreground uppercase tracking-widest mb-1"
                      >
                        {$t("dashboard.dailyDetail.tradeQty")}
                      </p>
                      <p
                        class="text-base font-black text-foreground tabular-nums"
                      >
                        {dayTrades.length}
                      </p>
                    </div>
                    <div class="p-4 card-glass border-border/10 shadow-none">
                      <p
                        class="text-[9px] font-bold text-muted-foreground uppercase tracking-widest mb-1"
                      >
                        {$t("dashboard.dailyDetail.efficiency")}
                      </p>
                      <p
                        class="text-base font-black text-blue-500 tabular-nums"
                      >
                        {dayTrades.length > 0
                          ? (
                              (dayTrades.filter((t) => t.result > 0).length /
                                dayTrades.length) *
                              100
                            ).toFixed(0)
                          : 0}%
                      </p>
                    </div>
                  </div>

                  <Separator />

                  <div class="space-y-4">
                    <h4
                      class="text-[10px] font-bold uppercase tracking-widest text-foreground"
                    >
                      {$t("dashboard.dailyDetail.operationsList")}
                    </h4>
                    <div
                      class="rounded-xl border border-border overflow-hidden"
                    >
                      <table class="w-full text-left text-xs">
                        <thead
                          class="bg-muted/50 font-bold uppercase tracking-tight text-[10px]"
                        >
                          <tr>
                            <th class="px-4 py-3"
                              >{$t("dashboard.dailyDetail.asset")}</th
                            >
                            <th class="px-4 py-3 text-center"
                              >{$t("dashboard.dailyDetail.direction")}</th
                            >
                            <th class="px-4 py-3 text-right"
                              >{$t("dashboard.dailyDetail.result")}</th
                            >
                            <th class="px-4 py-3 text-right"
                              >{$t("general.quantity")}</th
                            >
                          </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                          {#each dayTrades as trade}
                            <tr class="hover:bg-muted/30 transition-colors">
                              <td class="px-4 py-3 font-medium"
                                >{trade.asset_symbol}</td
                              >
                              <td class="px-4 py-3 text-center">
                                <Badge
                                  variant="outline"
                                  class={cn(
                                    "text-[10px] font-semibold uppercase tracking-tight py-0.5 h-5 border-none",
                                    trade.direction === "Buy"
                                      ? "bg-emerald-500/10 text-emerald-500"
                                      : "bg-rose-500/10 text-rose-500",
                                  )}
                                >
                                  {trade.direction === "Buy"
                                    ? $t("dashboard.dailyDetail.buy")
                                    : $t("dashboard.dailyDetail.sell")}
                                </Badge>
                              </td>
                              <td
                                class={cn(
                                  "px-4 py-3 text-right font-medium font-mono",
                                  trade.result >= 0
                                    ? "text-emerald-500"
                                    : "text-rose-500",
                                )}
                              >
                                {formatCurrency(trade.result)}
                              </td>
                              <td
                                class="px-4 py-3 text-right text-muted-foreground font-mono"
                              >
                                {trade.quantity}
                              </td>
                            </tr>
                          {:else}
                            <tr>
                              <td
                                colspan="4"
                                class="px-4 py-12 text-center text-muted-foreground font-medium uppercase text-[10px] tracking-widest"
                              >
                                {$t("dashboard.dailyDetail.noTrades")}
                              </td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </div>
                  </div>
                {/if}
              </div>
            </Dialog.Content>
          </Dialog.Root>

          <!-- Distribution Mini (Clean) -->
          <Card class="p-4 card-glass">
            <h3
              class="text-xs font-bold text-muted-foreground mb-2 uppercase tracking-wider border-b border-border/50 pb-1"
            >
              {$t("dashboard.schedule.title")}
            </h3>
            <div class="space-y-2">
              {#each [{ label: "Manhã (09h-12h)", val: 68, color: "bg-emerald-500", text: "text-emerald-500" }, { label: "Tarde (12h-18h)", val: 32, color: "bg-muted", text: "text-muted-foreground" }] as item}
                <div class="space-y-2">
                  <div
                    class="flex justify-between text-[8px] font-black uppercase tracking-wider"
                  >
                    <span class="text-muted-foreground">
                      {item.label === "Manhã (09h-12h)"
                        ? $t("dashboard.schedule.morning")
                        : $t("dashboard.schedule.afternoon")} ({item.label.split(
                        "(",
                      )[1]}
                    </span>
                    <span class={item.text}>{item.val}%</span>
                  </div>
                  <div class="h-2 w-full bg-muted rounded-full overflow-hidden">
                    <div
                      class={cn("h-full", item.color)}
                      style="width: {item.val}%"
                    ></div>
                  </div>
                </div>
              {/each}
            </div>
            <div class="mt-8 pt-6 border-t border-border">
              <p
                class="text-[9px] font-bold text-muted-foreground/40 uppercase tracking-widest text-center"
              >
                {$t("dashboard.schedule.basedOn", { values: { count: 20 } })}
              </p>
            </div>
          </Card>
        </div>
      </div>
    </div>
  </div>
{/if}
