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
  } from "lucide-svelte";
  import { isSameDay, startOfMonth, format } from "date-fns";
  import PerformanceCalendar from "$lib/components/dashboard/PerformanceCalendar.svelte";
  import { cn } from "$lib/utils";
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

  // --- Active Components (Elder / Velez) ---
  const activeProfile = $derived.by(() => {
    if (selectedAccountId === "all") return settingsStore.riskProfiles[0];
    const account = selectedAccount;
    return (
      settingsStore.riskProfiles.find(
        (p) =>
          p.account_type_applicability === account?.account_type ||
          p.account_type_applicability === "All",
      ) || settingsStore.riskProfiles[0]
    );
  });

  const activePhase = $derived(
    activeProfile?.growth_phases?.[activeProfile.current_phase_index],
  );

  // --- Mastery Stats Engine ---
  const stats = $derived.by(() => {
    const trades = filteredTrades;
    if (trades.length === 0)
      return {
        net: 0,
        winRate: 0,
        profitFactor: 0,
        payoff: 0,
        equity: [],
        monthResult: 0,
        dayResult: 0,
        discipline: 100,
      };

    const sorted = [...trades].sort(
      (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime(),
    );
    let current = 0,
      totalW = 0,
      totalL = 0,
      wins = 0,
      dayRes = 0,
      monthRes = 0,
      discSum = 0;
    const today = new Date();
    const thisMonth = startOfMonth(today);

    const equity = sorted.map((t) => {
      current += t.result || 0;
      if (t.result > 0) {
        wins++;
        totalW += t.result;
      } else {
        totalL += Math.abs(t.result);
      }
      if (isSameDay(new Date(t.date), today)) dayRes += t.result;
      if (new Date(t.date) >= thisMonth) monthRes += t.result;
      discSum += t.followed_plan ? 100 : 0;
      return [new Date(t.date).getTime(), current];
    });

    return {
      net: current,
      winRate: (wins / trades.length) * 100,
      profitFactor: totalL === 0 ? 10 : totalW / totalL,
      payoff:
        wins > 0 && trades.length - wins > 0
          ? totalW / wins / (totalL / (trades.length - wins))
          : 0,
      equity,
      monthResult: monthRes,
      dayResult: dayRes,
      discipline: discSum / trades.length,
    };
  });

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
      axisLine: { lineStyle: { color: "var(--border)" } },
      axisLabel: { color: "var(--muted-foreground)", fontSize: 10 },
    },
    yAxis: {
      type: "value",
      splitLine: { lineStyle: { color: "var(--border)", type: "dashed" } },
      axisLabel: { color: "var(--muted-foreground)", fontSize: 10 },
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

<div class="flex-1 space-y-8 p-8 min-h-screen">
  <Dialog.Root bind:open={isNewTradeOpen}>
    <Dialog.Content
      class="max-w-3xl max-h-[85vh] p-0 border-0 bg-transparent shadow-none"
    >
      <NewTradeWizard close={() => (isNewTradeOpen = false)} />
    </Dialog.Content>
  </Dialog.Root>

  <!-- TOP Navigation & Survival (Elder Focus) -->
  <div
    class="flex flex-col lg:flex-row justify-between items-start lg:items-center gap-6"
  >
    <div class="flex flex-col md:flex-row md:items-center gap-6">
      <div>
        <h2
          class="text-4xl font-extrabold tracking-tight text-foreground font-outfit"
        >
          Trading <span class="text-primary">Mastery</span>
        </h2>
        <p class="text-slate-500 font-medium text-sm">
          Controle de Operações e Mindset
        </p>
      </div>

      <div class="flex items-center gap-3">
        <Select type="single" bind:value={selectedAccountId}>
          <SelectTrigger class="w-[220px]">
            <div class="flex items-center gap-2">
              <Wallet class="w-4 h-4 text-muted-foreground" />
              <SelectValue placeholder="Selecione a Conta" />
            </div>
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">Todas as Contas</SelectItem>
            {#each settingsStore.accounts as acc}
              <SelectItem value={acc.id}>{acc.nickname}</SelectItem>
            {/each}
          </SelectContent>
        </Select>

        <Button
          onclick={() => (isNewTradeOpen = true)}
          class="bg-emerald-600 hover:bg-emerald-700 shadow-sm gap-2"
        >
          <Plus class="w-4 h-4" /> Novo Trade
        </Button>
      </div>
    </div>

    <!-- Active Risk Monitors (Elder 2% / 6%) -->
    <div class="flex flex-wrap gap-4">
      <Card
        class="border-border shadow-sm px-6 py-3 flex items-center gap-4 min-w-[200px]"
      >
        <div class="p-2 bg-emerald-500/10 rounded-lg">
          <ShieldCheck class="w-5 h-5 text-emerald-500" />
        </div>
        <div>
          <p
            class="text-[10px] uppercase font-bold text-muted-foreground tracking-wider"
          >
            Risco por Trade (2%)
          </p>
          <p class="text-lg font-black text-foreground">OK</p>
        </div>
      </Card>

      <Card
        class="border-border shadow-sm px-6 py-3 flex items-center gap-4 min-w-[200px]"
      >
        <div class="p-2 bg-rose-500/10 rounded-lg">
          <TrendingDown class="w-5 h-5 text-rose-500" />
        </div>
        <div>
          <p
            class="text-[10px] uppercase font-bold text-muted-foreground tracking-wider"
          >
            Risco Mensal (6%)
          </p>
          <div class="flex items-center gap-2">
            {#if stats}
              {@const monthlyExposure =
                (Math.abs(stats.monthResult) /
                  (selectedAccount?.balance || 10000)) *
                100}
              <span class="text-lg font-black text-foreground"
                >{monthlyExposure.toFixed(1)}%</span
              >
              <div class="w-16 h-1.5 bg-muted rounded-full overflow-hidden">
                <div
                  class={cn(
                    "h-full",
                    monthlyExposure > 4 ? "bg-rose-500" : "bg-emerald-500",
                  )}
                  style="width: {Math.min((monthlyExposure / 6) * 100, 100)}%"
                ></div>
              </div>
            {/if}
          </div>
        </div>
      </Card>
    </div>
  </div>

  <Separator class="bg-slate-200 opacity-50" />

  <!-- Main Grid: Growth & Performance -->
  <div class="grid gap-8 lg:grid-cols-7">
    <!-- Left Wing: Analytical Hub (5 Cols) -->
    <div class="col-span-full xl:col-span-5 space-y-8">
      <!-- Primary Metrics Bento -->
      <div class="grid gap-6 sm:grid-cols-2 xl:grid-cols-4">
        {#each [{ label: "Resultado Total", val: formatCurrency(stats.net), icon: TrendingUp, color: "text-emerald-600", bg: "bg-emerald-50" }, { label: "Taxa de Acerto", val: `${stats.winRate.toFixed(1)}%`, icon: Trophy, color: "text-blue-600", bg: "bg-blue-50" }, { label: "Fator de Lucro", val: stats.profitFactor.toFixed(2), icon: Activity, color: "text-amber-600", bg: "bg-amber-50" }, { label: "Disciplina (Douglas)", val: `${stats.discipline.toFixed(0)}%`, icon: Zap, color: "text-purple-600", bg: "bg-purple-50" }] as kpi}
          <Card class="hover:shadow-md transition-shadow group">
            <CardContent class="p-6">
              <div class="flex items-center justify-between mb-4">
                <div
                  class={cn(
                    "p-2 rounded-xl transition-transform group-hover:scale-110",
                    kpi.bg,
                  )}
                >
                  <kpi.icon class={cn("w-5 h-5", kpi.color)} />
                </div>
                <Badge
                  variant="secondary"
                  class="bg-muted text-muted-foreground text-[9px] border-none font-bold uppercase tracking-widest"
                  >Mastery</Badge
                >
              </div>
              <h3
                class="text-2xl font-black text-foreground tracking-tight font-outfit"
              >
                {kpi.val}
              </h3>
              <p
                class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest mt-1"
              >
                {kpi.label}
              </p>
            </CardContent>
          </Card>
        {/each}
      </div>

      <!-- Performance Timeline Central (Calendar request) -->
      <Card class="overflow-hidden">
        <CardHeader class="border-b border-border/50 px-8 py-6 bg-muted/20">
          <div class="flex items-center justify-between">
            <div>
              <CardTitle
                class="text-xl font-black text-foreground uppercase tracking-tighter"
                >Calendário de Performance</CardTitle
              >
              <CardDescription
                class="text-xs font-bold text-muted-foreground uppercase tracking-widest mt-1"
                >Clique para detalhar o histórico</CardDescription
              >
            </div>
            <Button variant="outline" size="sm" class="gap-2"
              ><Calendar class="w-4 h-4" /> Detalhes Mensais</Button
            >
          </div>
        </CardHeader>
        <CardContent class="p-8">
          <PerformanceCalendar trades={filteredTrades} />
        </CardContent>
      </Card>

      <!-- Equity Curve -->
      <Card class="p-8">
        <div class="flex justify-between items-center mb-10">
          <div>
            <h3
              class="text-xl font-black text-foreground uppercase tracking-tighter"
            >
              Curva de Patrimônio
            </h3>
            <p
              class="text-xs font-bold text-muted-foreground uppercase tracking-widest"
            >
              Evolução Líquida Acumulada
            </p>
          </div>
          <div class="flex gap-2">
            <Badge
              class="bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20 border-emerald-500/20 font-bold px-3 py-1"
              >CONSISTÊNCIA ALTA</Badge
            >
          </div>
        </div>
        <div class="h-[350px]">
          <EChart options={equityOptions} />
        </div>
      </Card>
    </div>

    <!-- Right Wing: Strategy & Growth (2 Cols) -->
    <div class="col-span-full xl:col-span-2 space-y-8">
      <!-- Growth Plan Widget (Velez Focus) -->
      <Card class="overflow-hidden border-t-4 border-t-emerald-500">
        <CardContent class="p-8 space-y-8">
          <div class="flex items-start justify-between">
            <div class="space-y-1">
              <h3
                class="font-black text-foreground uppercase tracking-tighter text-2xl"
              >
                Plano Evolutivo
              </h3>
              <p
                class="text-[10px] font-bold text-primary uppercase tracking-widest border border-primary/20 bg-primary/5 px-2 py-0.5 rounded-full inline-block"
              >
                {activeProfile?.name || "Nenhum Perfil Ativo"}
              </p>
            </div>
            <div class="p-3 bg-emerald-500/10 rounded-2xl">
              <ArrowUpRight class="w-6 h-6 text-emerald-500" />
            </div>
          </div>

          {#if activeProfile?.growth_plan_enabled}
            <div class="space-y-8">
              <div class="space-y-3">
                <div
                  class="flex justify-between text-[11px] font-black text-muted-foreground uppercase tracking-widest"
                >
                  <span>Fase Atual: {activePhase?.name}</span>
                  <span>45%</span>
                </div>
                <div class="h-2.5 bg-muted rounded-full overflow-hidden">
                  <div
                    class="h-full bg-emerald-500 transition-all duration-1000 shadow-[0_0_10px_rgba(16,185,129,0.3)]"
                    style="width: 45%"
                  ></div>
                </div>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <div class="p-5 bg-muted/40 rounded-2xl border border-border">
                  <p
                    class="text-[9px] font-black text-muted-foreground uppercase tracking-widest mb-1"
                  >
                    Lote Máximo
                  </p>
                  <p class="text-xl font-black text-foreground font-outfit">
                    {activePhase?.max_lots || 1}
                  </p>
                </div>
                <div class="p-5 bg-muted/40 rounded-2xl border border-border">
                  <p
                    class="text-[9px] font-black text-muted-foreground uppercase tracking-widest mb-1"
                  >
                    Loss Máx/Dia
                  </p>
                  <p class="text-xl font-black text-rose-500 font-outfit">
                    {formatCurrency(activePhase?.max_daily_loss || 500)}
                  </p>
                </div>
              </div>

              <div
                class="p-6 bg-emerald-500/5 rounded-2xl border border-emerald-500/10"
              >
                <p
                  class="text-[10px] font-bold text-emerald-500 uppercase tracking-widest mb-3 flex items-center gap-2"
                >
                  <Target class="w-3 h-3" /> Metas Próxima Fase
                </p>
                <div class="space-y-2">
                  {#each activePhase?.progression_rules || [] as rule}
                    <div
                      class="flex items-center justify-between text-xs font-bold"
                    >
                      <div
                        class="flex items-center gap-2 text-muted-foreground"
                      >
                        <div
                          class="w-1.5 h-1.5 rounded-full bg-emerald-500"
                        ></div>
                        <span
                          >{rule.condition === "profit_target"
                            ? "Alvo Financeiro:"
                            : rule.condition === "consistency_days"
                              ? "Consistência:"
                              : "Dias Positivos:"}
                        </span>
                      </div>
                      <span class="text-foreground">
                        {rule.condition === "profit_target"
                          ? formatCurrency(rule.value)
                          : `${rule.value} pregões`}
                      </span>
                    </div>
                  {/each}
                </div>
              </div>
            </div>
          {:else}
            <div class="py-12 text-center space-y-6">
              <div
                class="w-16 h-16 bg-muted rounded-full flex items-center justify-center mx-auto"
              >
                <Target class="w-8 h-8 text-muted-foreground/20" />
              </div>
              <p
                class="text-sm text-muted-foreground font-bold uppercase tracking-widest"
              >
                Nenhum plano ativado
              </p>
              <Button variant="outline" size="sm" class="border-border"
                >Configurar Agora</Button
              >
            </div>
          {/if}
        </CardContent>
      </Card>

      <!-- Retired Mindset section removed -->

      <!-- Distribution Mini (Clean) -->
      <Card class="p-8">
        <h3
          class="text-xs font-black text-muted-foreground mb-8 uppercase tracking-widest border-b border-border pb-4"
        >
          Eficiência por Horário
        </h3>
        <div class="space-y-6">
          {#each [{ label: "Manhã (09h-12h)", val: 68, color: "bg-emerald-500", text: "text-emerald-500" }, { label: "Tarde (12h-18h)", val: 32, color: "bg-muted", text: "text-muted-foreground" }] as item}
            <div class="space-y-2">
              <div
                class="flex justify-between text-[10px] font-black uppercase tracking-widest"
              >
                <span class="text-muted-foreground">{item.label}</span>
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
            Baseado nos últimos 20 trades
          </p>
        </div>
      </Card>
    </div>
  </div>
</div>

<style>
  :global(.font-outfit) {
    font-family: "Outfit", sans-serif;
  }
  :global(.font-inter) {
    font-family: "Inter", sans-serif;
  }
</style>
