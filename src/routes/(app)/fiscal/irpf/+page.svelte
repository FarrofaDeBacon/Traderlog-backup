<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { toast } from "svelte-sonner";
    import { _ as t } from "svelte-i18n";
    import * as Card from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Dialog from "$lib/components/ui/dialog";
    import {
        Calendar,
        DollarSign,
        FileText,
        TrendingDown,
        TrendingUp,
        Eye,
        Trash2,
        AlertTriangle,
        RefreshCw,
        ArrowRight,
        ChevronLeft,
        ChevronRight,
        CheckCircle2,
        AlertCircle,
        Search,
        Filter,
    } from "lucide-svelte";
    import { irpfStore } from "$lib/stores/irpfStore.svelte";
    import TaxEvolutionChart from "$lib/components/dashboard/TaxEvolutionChart.svelte";
    import * as Select from "$lib/components/ui/select";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Tooltip from "$lib/components/ui/tooltip";
    import DarfDetailsDialog from "$lib/components/finance/DarfDetailsDialog.svelte";
    import HierarchicalList from "$lib/components/shared/HierarchicalList.svelte";
    import { Badge } from "$lib/components/ui/badge";

    // View Modal State
    let isViewModalOpen = $state(false);
    let selectedAppraisal = $state<any>(null);

    // Delete Modal State
    let isDeleteModalOpen = $state(false);
    let appraisalToDelete = $state<any>(null);

    let isAppraisalModalOpen = $state(false);
    let appraisalMonth = $state(String(new Date().getMonth() + 1));
    let appraisalYear = $state(irpfStore.selectedYear);
    let appraisalResults = $state<any[]>([]);

    let currentMonth = new Date().getMonth() + 1;
    let selectedMonth = $state<number | null>(null); // null = "Todos"
    let statusFilter = $state<"all" | "pending">("all");
    let searchText = $state("");

    // Define months *before* using in derived stores
    const months = [
        { val: 1, key: "months.january" },
        { val: 2, key: "months.february" },
        { val: 3, key: "months.march" },
        { val: 4, key: "months.april" },
        { val: 5, key: "months.may" },
        { val: 6, key: "months.june" },
        { val: 7, key: "months.july" },
        { val: 8, key: "months.august" },
        { val: 9, key: "months.september" },
        { val: 10, key: "months.october" },
        { val: 11, key: "months.november" },
        { val: 12, key: "months.december" },
    ];

    let taxEvolutionData = $derived.by(() => {
        const result = [];
        const currentYear = Number(irpfStore.selectedYear);
        const apps = irpfStore.appraisals || [];
        const darfs = irpfStore.darfs || [];

        for (let i = 1; i <= 12; i++) {
            // Find appraisals for this month and year
            const monthApps = apps.filter(
                (a) =>
                    Number(a.period_month) === i &&
                    Number(a.period_year) === currentYear,
            );

            // Find paid darfs for this month and year
            const monthPaid = darfs.filter((d) => {
                const parts = (d.period || "").split("/");
                if (parts.length < 2) return false;
                const m = Number(parts[0]);
                const y = Number(parts[1]);
                return m === i && y === currentYear && d.status === "Paid";
            });

            const taxDueValue = monthApps.reduce((acc, a) => {
                const val = Number(a.total_payable || 0);
                return acc + (isNaN(val) ? 0 : val);
            }, 0);

            const taxPaidValue = monthPaid.reduce((acc, d) => {
                const val = Number(d.total_value || 0);
                return acc + (isNaN(val) ? 0 : val);
            }, 0);

            const activeMonth = months.find((m) => m.val === i);
            result.push({
                month: activeMonth ? $t(activeMonth.key) : "Mês Inválido",
                taxDue: taxDueValue,
                taxPaid: taxPaidValue,
            });
        }
        return result;
    });

    // Filter appraisals by selected month and hide paid ones
    let filteredAppraisals = $derived.by(() => {
        let list = irpfStore.appraisals.filter(
            (a) => Number(a.period_year) === Number(irpfStore.selectedYear),
        );

        // Apply Month Filter + Carry-over Pending
        if (selectedMonth !== null) {
            list = list.filter(a => 
                Number(a.period_month) === selectedMonth || 
                (a.status === 'Pending' && a.total_payable > 0)
            );
        }

        // Apply Status Filter
        if (statusFilter === "pending") {
            list = list.filter(a => a.status === 'Pending');
        }

        // Apply Search
        if (searchText.trim()) {
            const q = searchText.toLowerCase();
            list = list.filter(a => 
                (a.trade_type || "").toLowerCase().includes(q) ||
                (a.revenue_code || "").toLowerCase().includes(q)
            );
        }

        return list;
    });

    let hierarchicalAppraisals = $derived.by(() => {
        const dataByMonth: Record<number, any> = {};

        filteredAppraisals.forEach((item) => {
            if (!dataByMonth[item.period_month]) {
                const monthName = $t(
                    months.find((m) => m.val === Number(item.period_month))
                        ?.key || "general.error",
                );
                dataByMonth[item.period_month] = {
                    key: `month-${item.period_month}-${item.period_year}`,
                    label: `${monthName} / ${item.period_year}`,
                    days: [], // flatMode reads 'days' array directly
                    originalItems: [],
                };
            }

            const existingDarf = irpfStore.darfs.find(
                (d) =>
                    irpfStore.getId(d.appraisal_id) ===
                    irpfStore.getId(item.id),
            );
            dataByMonth[item.period_month].originalItems.push(item);
            dataByMonth[item.period_month].days.push({
                key: `appraisal-${item.id}`,
                date: "",
                label:
                    item.trade_type === "DayTrade"
                        ? "Day Trade"
                        : "Swing Trade",
                originalItem: item,
                existingDarf: existingDarf, // Otimizado: lookup feito apenas uma vez na derivação
            });
        });

        return Object.values(dataByMonth).sort((a: any, b: any) => {
            const m1 = parseInt(a.key.split("-")[1]);
            const m2 = parseInt(b.key.split("-")[1]);
            return m2 - m1;
        });
    });

    onMount(() => {
        irpfStore.loadAllData();
    });

    function deleteAppraisal(item: any) {
        appraisalToDelete = item;
        isDeleteModalOpen = true;
    }

    async function confirmDelete() {
        if (!appraisalToDelete) return;
        const idStr = irpfStore.getId(appraisalToDelete.id);
        await irpfStore.deleteAppraisal(idStr);
        isDeleteModalOpen = false;
    }

    async function generateDarf(item: any) {
        let idStr = irpfStore.getId(item.id);

        // Block if not saved
        if (!idStr) {
            toast.error($t("fiscal.irpf.saveBeforeDarf"));
            return;
        }

        try {
            await irpfStore.generateDarf(idStr);
            item.darfGenerated = true;
        } catch (error) {
            // Error handled in store
        }
    }

    function openViewModal(item: any) {
        // If there's an existing DARF, we show the DARF details
        // Otherwise we show the appraisal details (standardizing on DarfDetailsDialog)
        selectedAppraisal = item;
        isViewModalOpen = true;
    }

    function formatCurrency(val: number) {
        return new Intl.NumberFormat("pt-BR", {
            style: "currency",
            currency: "BRL",
        }).format(val || 0);
    }

    async function calculateAppraisal() {
        try {
            appraisalResults = await irpfStore.calculateMonthlyTax(
                Number(appraisalMonth),
                Number(appraisalYear),
            );
        } catch (error) {
            // Handled in store
        }
    }

    async function saveAppraisalResult(item: any) {
        const saved = await irpfStore.saveAppraisal(item);
        // Update item.id so "Gerar DARF" button appears immediately
        if (saved?.id) {
            item.id = saved.id;
            // Removed: isAppraisalModalOpen = false;
            irpfStore.loadAllData(); // Refresh history
        }
    }

    function loadData() {
        irpfStore.loadAllData();
    }

    function getTotalLoss(type: string) {
        return irpfStore.losses
            .filter((l) => l.trade_type === type)
            .reduce((acc, curr) => acc + curr.balance, 0);
    }

    async function deleteLoss(id: any) {
        const idStr = irpfStore.getId(id);
        await irpfStore.deleteLoss(idStr);
    }

    let expandedMonths = $state(new Set<string>());

    $effect(() => {
        if (hierarchicalAppraisals.length > 0) {
            untrack(() => {
                const today = new Date();
                const currentMonth = today.getMonth() + 1;
                const currentYear = today.getFullYear();
                const currentMonthKey = `month-${currentMonth}-${currentYear}`;

                if (expandedMonths.size === 0) {
                    if (hierarchicalAppraisals.some(m => m.key === currentMonthKey)) {
                        expandedMonths.add(currentMonthKey);
                    } else {
                        // Fallback to latest month
                        expandedMonths.add(hierarchicalAppraisals[0].key);
                    }
                    expandedMonths = new Set(expandedMonths);
                }
            });
        }
    });
</script>

<div
    class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20"
>
    <!-- Header & Actions -->
    <div
        class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 border-b border-border/30 pb-6"
    >
        <div>
            <h2
                class="text-3xl font-bold text-foreground tracking-tight flex items-center gap-3"
            >
                <FileText class="w-8 h-8 text-primary" />
                {$t("fiscal.irpf.title")}
            </h2>
            <p class="text-muted-foreground mt-1">
                {$t("fiscal.irpf.description")}
            </p>
        </div>
        <div class="flex gap-2">
            <Button
                variant="outline"
                href="/fiscal/irpf/darf"
                class="border-border"
            >
                <FileText class="w-4 h-4 mr-2" />
                {$t("fiscal.irpf.manageDarfs")}
            </Button>

            <Button
                onclick={() => (isAppraisalModalOpen = true)}
                class="neon-glow bg-primary text-primary-foreground font-bold"
            >
                <Calendar class="w-4 h-4 mr-2" />
                {$t("fiscal.irpf.newAppraisal")}
            </Button>
        </div>
    </div>

    <!-- KPI Cards (Standardized - Single Container for Perfect Height) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- Total Devido -->
        <div class="card-glass border-l-4 border-l-rose-500 overflow-hidden">
            <div class="flex items-start justify-between py-2 px-4">
                <span
                    class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground"
                >
                    {$t("fiscal.irpf.kpis.totalDue")} ({irpfStore.selectedYear})
                </span>
                <DollarSign class="h-3.5 w-3.5 text-rose-500" />
            </div>
            <div class="py-2 px-4">
                <div
                    class="text-base font-mono font-bold text-foreground uppercase tracking-tighter"
                >
                    {formatCurrency(irpfStore.totalDue)}
                </div>
                <p class="text-[10px] text-muted-foreground mt-0.5">
                    {$t("fiscal.irpf.kpis.dueYearHint")}
                </p>
            </div>
        </div>

        <!-- Total Pago -->
        <div class="card-glass border-l-4 border-l-emerald-500 overflow-hidden">
            <div class="flex items-start justify-between py-2 px-4">
                <span
                    class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground"
                >
                    {$t("fiscal.irpf.kpis.totalPaid")} ({irpfStore.selectedYear})
                </span>
                <CheckCircle2 class="h-3.5 w-3.5 text-emerald-500" />
            </div>
            <div class="py-2 px-4">
                <div
                    class="text-base font-mono font-bold text-emerald-500 uppercase tracking-tighter"
                >
                    {formatCurrency(irpfStore.totalPaid)}
                </div>
                <p class="text-[10px] text-muted-foreground mt-0.5">
                    {$t("fiscal.irpf.kpis.paidYearHint")}
                </p>
            </div>
        </div>

        <!-- Card 3: Pendente Atual -->
        <div class="card-glass border-l-4 border-l-amber-500 overflow-hidden">
            <div class="flex items-start justify-between py-2 px-4">
                <span
                    class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground"
                >
                    {$t("fiscal.irpf.kpis.pending")}
                </span>
                <AlertTriangle class="h-3.5 w-3.5 text-amber-500" />
            </div>
            <div class="py-2 px-4">
                <div
                    class="text-base font-mono font-bold text-amber-500 uppercase tracking-tighter"
                >
                    {formatCurrency(irpfStore.pendingAmount)}
                </div>
                <div class="flex justify-between items-center mt-0.5">
                    <p class="text-[10px] text-muted-foreground">
                        {$t("fiscal.irpf.kpis.pendingHint", {
                            values: { count: irpfStore.pendingGuiasCount },
                        })}
                    </p>
                    <Button
                        variant="link"
                        href="/fiscal/irpf/darf"
                        class="h-auto p-0 text-[10px] text-amber-500/80 hover:text-amber-500 hover:no-underline font-bold uppercase"
                    >
                        Verificar &rarr;
                    </Button>
                </div>
            </div>
        </div>

        <!-- Card 4: Prejuízos Acumulados -->
        <div class="card-glass border-l-4 border-l-blue-500 overflow-hidden">
            <div class="flex items-start justify-between py-2 px-4">
                <span
                    class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground"
                >
                    {$t("fiscal.irpf.kpis.losses")}
                </span>
                <TrendingDown class="h-3.5 w-3.5 text-blue-500" />
            </div>
            <div class="py-2 px-4 space-y-1">
                <div class="flex justify-between items-center">
                    <span
                        class="text-[10px] font-bold text-muted-foreground uppercase opacity-60"
                        >Day Trade</span
                    >
                    <span
                        class="text-sm font-mono font-bold text-rose-500 tabular-nums"
                    >
                        {formatCurrency(getTotalLoss("DayTrade"))}
                    </span>
                </div>
                <div class="flex justify-between items-center">
                    <span
                        class="text-[10px] font-bold text-muted-foreground uppercase opacity-60"
                        >Swing Trade</span
                    >
                    <span
                        class="text-sm font-mono font-bold text-rose-500 tabular-nums"
                    >
                        {formatCurrency(getTotalLoss("SwingTrade"))}
                    </span>
                </div>
            </div>
        </div>
    </div>

    <!-- Charts Section -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <Card.Root class="lg:col-span-2 card-glass">
            <Card.Header>
                <Card.Title class="text-lg font-medium"
                    >{$t("fiscal.irpf.evolution")} ({irpfStore.selectedYear})</Card.Title
                >
            </Card.Header>
            <Card.Content class="h-[300px]">
                {#key irpfStore.selectedYear}
                    <TaxEvolutionChart data={taxEvolutionData} />
                {/key}
                <div
                    class="mt-2 text-[10px] text-muted-foreground flex justify-between px-2 italic"
                >
                    <span>
                        {$t("fiscal.irpf.history")}: {irpfStore.appraisals.filter(
                            (a) =>
                                Number(a.period_year) == irpfStore.selectedYear,
                        ).length} / {irpfStore.darfs.filter((d) =>
                            d.period.includes(
                                irpfStore.selectedYear.toString(),
                            ),
                        ).length} DARFs
                    </span>
                    <span
                        >{formatCurrency(
                            Number(
                                taxEvolutionData.reduce(
                                    (acc, curr) => acc + curr.taxDue,
                                    0,
                                ),
                            ),
                        )}</span
                    >
                </div>
            </Card.Content>
        </Card.Root>

        <!-- Year Selector & Quick Actions (Side) -->
        <div class="space-y-6">
            <Card.Root class="card-glass">
                <Card.Header>
                    <Card.Title class="text-sm font-medium"
                        >{$t("fiscal.irpf.periodFilter")}</Card.Title
                    >
                </Card.Header>
                <Card.Content>
                    <div class="flex items-center gap-2">
                        <Select.Root
                            type="single"
                            value={irpfStore.selectedYear.toString()}
                            onValueChange={(v) => {
                                if (v) {
                                    irpfStore.loadAllData(parseInt(v));
                                }
                            }}
                        >
                            <Select.Trigger
                                class="w-full bg-muted/20 border-border/30 text-foreground"
                            >
                                {irpfStore.selectedYear}
                            </Select.Trigger>
                            <Select.Content>
                                {#each Array.from({ length: 5 }, (_, i) => new Date().getFullYear() - i) as y}
                                    <Select.Item
                                        value={y.toString()}
                                        label={y.toString()}>{y}</Select.Item
                                    >
                                {/each}
                            </Select.Content>
                        </Select.Root>
                        <Button
                            variant="outline"
                            size="icon"
                            onclick={loadData}
                        >
                            <RefreshCw class="w-4 h-4" />
                        </Button>
                    </div>
                </Card.Content>
            </Card.Root>

            <Card.Root
                class="card-glass bg-amber-500/5 hover:bg-amber-500/10 transition-all"
            >
                <Card.Content class="p-4 flex flex-col gap-3">
                    <div class="flex items-start gap-3">
                        <div class="p-2 bg-amber-500/20 rounded-full">
                            <FileText class="w-5 h-5 text-amber-500" />
                        </div>
                        <div>
                            <h4 class="font-bold text-amber-500">
                                {$t("fiscal.darf.title")}
                            </h4>
                            <p
                                class="text-xs text-amber-900/60 dark:text-amber-200/70 mt-1"
                            >
                                {$t("fiscal.darf.description")}
                            </p>
                        </div>
                    </div>
                    <Button
                        variant="secondary"
                        class="w-full bg-amber-500/20 hover:bg-amber-500/30 text-amber-900 dark:text-amber-100 border-amber-500/20 font-bold"
                        href="/fiscal/irpf/darf"
                    >
                        {$t("fiscal.irpf.manageDarfs")}
                    </Button>
                </Card.Content>
            </Card.Root>
        </div>
    </div>

    <!-- Appraisals List -->
    <Card.Root class="card-glass">
        <Card.Header>
            <Card.Title>{$t("fiscal.irpf.history")}</Card.Title>
        </Card.Header>
        <Card.Content>
            <!-- Month Filter Tabs -->
            <Tabs.Root
                value={selectedMonth === null
                    ? "all"
                    : selectedMonth.toString()}
                onValueChange={(v) => {
                    selectedMonth = v === "all" ? null : parseInt(v);
                }}
                class="mb-6"
            >
                <Tabs.List
                    class="grid grid-cols-7 lg:grid-cols-13 gap-1 bg-muted/20 p-1"
                >
                    <Tabs.Trigger value="all" class="text-xs"
                        >{$t("general.all")}</Tabs.Trigger
                    >
                    {#each months as m}
                        <Tabs.Trigger value={m.val.toString()} class="text-xs">
                            {$t(m.key).substring(0, 3)}
                        </Tabs.Trigger>
                    {/each}
                </Tabs.List>
            </Tabs.Root>

            <!-- Search and Status Filters -->
            <div class="flex flex-col md:flex-row gap-4 mb-6 items-center">
                <div class="relative w-full md:w-96 group">
                    <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                    <Input 
                        placeholder="Buscar por tipo ou código..."
                        bind:value={searchText}
                        class="pl-10 bg-muted/20 border-border/30 focus-visible:ring-primary/20"
                    />
                </div>

                <div class="flex items-center gap-2 ml-auto">
                    <span class="text-[10px] font-black uppercase text-muted-foreground tracking-widest hidden sm:block">Filtro:</span>
                    <Select.Root type="single" bind:value={statusFilter}>
                        <Select.Trigger class="w-40 bg-muted/20 border-border/30 h-10">
                            <span class="flex items-center gap-2">
                                <Filter class="w-3.5 h-3.5 text-muted-foreground" />
                                {statusFilter === 'all' ? 'Todas' : 'Somente Pendentes'}
                            </span>
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="all">Todas</Select.Item>
                            <Select.Item value="pending">Pendentes</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>
            {#if irpfStore.loading}
                <div class="p-8 text-center text-muted-foreground">
                    {$t("general.loading")}
                </div>
            {:else if filteredAppraisals.length === 0}
                <div class="p-12 text-center flex flex-col items-center gap-3">
                    <div
                        class="w-12 h-12 rounded-full bg-muted/30 flex items-center justify-center"
                    >
                        <FileText class="w-6 h-6 text-muted-foreground" />
                    </div>
                    <p class="text-muted-foreground">
                        {selectedMonth === null
                            ? $t("fiscal.darf.messages.emptyHistory", {
                                  values: { year: irpfStore.selectedYear },
                              }).replace("DARF", "")
                            : `${$t("general.empty")} ${$t(months.find((m) => m.val === selectedMonth)?.key || "")}/${irpfStore.selectedYear}.`}
                    </p>
                    <Button
                        variant="outline"
                        onclick={() => (isAppraisalModalOpen = true)}
                        class="mt-2"
                    >
                        {$t("fiscal.irpf.newAppraisal")}
                    </Button>
                </div>
            {:else}
                <HierarchicalList
                    data={hierarchicalAppraisals}
                    flatMode={true}
                    omitDays={true}
                    bind:expandedMonths
                    mutualExclusion={true}
                >
                    {#snippet monthBadges(month: any)}
                        {@const totalPayable = month.originalItems.reduce(
                            (acc: number, curr: any) =>
                                acc + curr.total_payable,
                            0,
                        )}
                        {@const totalProfit = month.originalItems.reduce(
                            (acc: number, curr: any) => acc + curr.net_profit,
                            0,
                        )}

                        <Badge
                            variant="outline"
                            class="text-[9px] px-1.5 h-4 bg-muted border-border font-bold uppercase"
                        >
                            {month.days.length}
                            {$t("general.records", { default: "REGISTROS" })}
                        </Badge>

                        <div class="flex gap-3 ml-2">
                            <div class="flex flex-col items-end">
                                <span
                                    class="text-[9px] font-bold text-muted-foreground uppercase opacity-70"
                                >
                                    {$t("fiscal.irpf.table.netProfit")}
                                </span>
                                <span
                                    class="text-[10px] font-mono font-bold {totalProfit >=
                                    0
                                        ? 'text-emerald-500'
                                        : 'text-rose-500'}"
                                >
                                    {formatCurrency(totalProfit)}
                                </span>
                            </div>

                            <div class="flex flex-col items-end hidden sm:flex">
                                <span
                                    class="text-[9px] font-bold text-muted-foreground uppercase opacity-70"
                                >
                                    {$t("fiscal.irpf.kpis.totalDue")}
                                </span>
                                <span
                                    class="text-[10px] font-mono font-bold text-foreground"
                                >
                                    {formatCurrency(totalPayable)}
                                </span>
                            </div>
                        </div>
                    {/snippet}

                    {#snippet monthContent(month: any)}
                        <div class="space-y-2">
                            <!-- Column Headers -->
                            <div class="hidden md:flex items-center justify-between p-3 border-b border-border/10 bg-muted/5 rounded-t-lg">
                                <div class="flex items-center gap-3 w-32">
                                    <span class="text-[9px] font-black uppercase text-muted-foreground tracking-widest">Tipo de Operação</span>
                                </div>
                                <div class="flex items-center gap-8 pr-4">
                                    <span class="text-[9px] font-black uppercase text-muted-foreground tracking-widest w-24 text-right">Lucro Líquido</span>
                                    <span class="text-[9px] font-black uppercase text-muted-foreground tracking-widest w-24 text-right">Total a Pagar</span>
                                    <span class="text-[9px] font-black uppercase text-muted-foreground tracking-widest w-24 text-right">Compensado</span>
                                    <span class="text-[9px] font-black uppercase text-muted-foreground tracking-widest w-24 text-center">Status</span>
                                    <span class="text-[9px] font-black uppercase text-muted-foreground tracking-widest w-24 text-right">Ações</span>
                                </div>
                            </div>
                            {#each month.originalItems as item}
                                {@const revenueCode =
                                    item.trade_type === "DayTrade"
                                        ? "6015"
                                        : "6015"}
                                {@const period = `${String(item.period_month).padStart(2, "0")}/${item.period_year}`}
                                {@const existingDarf = irpfStore.darfs.find(
                                    (d) =>
                                        irpfStore.getId(d.appraisal_id) ===
                                        irpfStore.getId(item.id),
                                )}

                                <div
                                    class="flex flex-col md:flex-row items-start md:items-center justify-between p-3 gap-4 w-full group hover:bg-muted/10 transition-colors"
                                >
                                    <div class="flex items-center gap-3">
                                        <span
                                            class="px-2 py-1 rounded text-xs font-bold {item.trade_type ===
                                            'DayTrade'
                                                ? 'bg-blue-500/10 text-blue-500'
                                                : 'bg-indigo-500/10 text-indigo-500'}"
                                        >
                                            {item.trade_type === "DayTrade"
                                                ? "Day Trade"
                                                : "Swing Trade"}
                                        </span>
                                        {#if item.is_complementary}
                                            <span
                                                class="text-[9px] font-bold text-amber-500 uppercase tracking-tighter"
                                            >
                                                {$t(
                                                    "fiscal.irpf.complementary",
                                                )}
                                            </span>
                                        {/if}
                                    </div>

                                    <div
                                        class="flex items-center gap-8 pr-4"
                                    >
                                        <div class="flex flex-col items-end w-24">
                                            <span
                                                class="text-[9px] text-muted-foreground uppercase font-bold"
                                                >Líquido</span
                                            >
                                            <span
                                                class="font-mono text-sm leading-none font-bold {item.net_profit >=
                                                0
                                                    ? 'text-green-400'
                                                    : 'text-red-400'}"
                                            >
                                                {formatCurrency(
                                                    item.net_profit,
                                                )}
                                            </span>
                                        </div>
                                        <div class="flex flex-col items-end w-24">
                                            <span
                                                class="text-[9px] text-muted-foreground uppercase font-bold"
                                                title={item.tax_accumulated > 0
                                                    ? `Corrente: ${formatCurrency(item.tax_payable)} | Acum.: ${formatCurrency(item.tax_accumulated)}`
                                                    : ""}
                                            >
                                                {$t("fiscal.irpf.table.toPay")}
                                            </span>
                                            <span
                                                class="font-mono text-sm leading-none font-bold text-foreground"
                                            >
                                                {formatCurrency(
                                                    item.total_payable,
                                                )}
                                            </span>
                                        </div>
                                        <div
                                            class="flex flex-col items-end hidden md:flex w-24"
                                        >
                                            <span
                                                class="text-[9px] text-muted-foreground uppercase font-bold"
                                                >{$t(
                                                    "fiscal.irpf.table.compensated",
                                                )}</span
                                            >
                                            <span
                                                class="font-mono text-sm leading-none font-bold text-yellow-500/80"
                                            >
                                                {item.compensated_loss > 0
                                                    ? `-${formatCurrency(item.compensated_loss)}`
                                                    : "-"}
                                            </span>
                                        </div>

                                        <div class="w-24 text-center">
                                                {#if item.status === "Paid"}
                                                    <span
                                                        class="px-2 py-1 rounded text-[10px] font-bold bg-green-500/10 text-green-500 border border-green-500/20 uppercase"
                                                    >
                                                        {$t(
                                                            "fiscal.irpf.table.paid",
                                                        )}
                                                    </span>
                                                {:else if item.status === "Pending"}
                                                    <span
                                                        class="px-2 py-1 rounded text-[10px] font-bold bg-yellow-500/10 text-yellow-500 border border-yellow-500/20 uppercase"
                                                    >
                                                        {$t(
                                                            "fiscal.irpf.table.pending",
                                                        )}
                                                    </span>
                                                {:else}
                                                    <span
                                                        class="px-2 py-1 rounded text-[10px] font-bold bg-green-500/10 text-green-500 border border-green-500/20 uppercase"
                                                    >
                                                        {$t(
                                                            "fiscal.irpf.table.ok",
                                                        )}
                                                    </span>
                                                {/if}
                                        </div>
                                        <div
                                            class="w-24 flex items-center justify-end gap-1"
                                        >
                                                {#if item.status !== "Paid" && item.total_payable > 0}
                                                    {#if existingDarf}
                                                        <Button
                                                            variant="ghost"
                                                            size="icon"
                                                            class="h-8 w-8 text-green-500 hover:text-green-400"
                                                            title={$t(
                                                                "fiscal.irpf.table.alreadyGenerated",
                                                            )}
                                                            href="/fiscal/irpf/darf"
                                                        >
                                                            <FileText
                                                                class="w-4 h-4"
                                                            />
                                                        </Button>
                                                    {:else if item.total_payable >= 10}
                                                        <Button
                                                            variant="ghost"
                                                            size="icon"
                                                            class="h-8 w-8 text-amber-500 hover:text-amber-400"
                                                            title={$t(
                                                                "fiscal.irpf.table.generateDarf",
                                                            )}
                                                            onclick={() =>
                                                                generateDarf(
                                                                    item,
                                                                )}
                                                        >
                                                            <FileText
                                                                class="w-4 h-4"
                                                            />
                                                        </Button>
                                                    {/if}
                                                {/if}

                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    class="h-8 w-8 text-blue-500 hover:text-blue-400"
                                                    onclick={() =>
                                                        openViewModal(item)}
                                                >
                                                    <Eye class="w-4 h-4" />
                                                </Button>

                                                {#if item.status === "Paid" || item.status === "Ok"}
                                                    <Tooltip.Root>
                                                        <Tooltip.Trigger>
                                                            <Button
                                                                variant="ghost"
                                                                size="icon"
                                                                class="h-8 w-8 opacity-50 cursor-not-allowed"
                                                                disabled
                                                            >
                                                                <Trash2
                                                                    class="w-4 h-4 text-muted-foreground/40"
                                                                />
                                                            </Button>
                                                        </Tooltip.Trigger>
                                                        <Tooltip.Content
                                                            class="bg-popover border-border text-foreground text-xs"
                                                        >
                                                            {$t(
                                                                "fiscal.irpf.table.cannotDelete",
                                                            )}
                                                        </Tooltip.Content>
                                                    </Tooltip.Root>
                                                {:else}
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-8 w-8 text-rose-500 hover:bg-rose-500/10"
                                                        onclick={() =>
                                                            deleteAppraisal(
                                                                item,
                                                            )}
                                                    >
                                                        <Trash2
                                                            class="w-4 h-4 text-rose-500"
                                                        />
                                                    </Button>
                                                {/if}
                                        </div>
                                    </div>
                                </div>
                            {/each}
                        </div>
                    {/snippet}
                </HierarchicalList>
            {/if}
        </Card.Content>
    </Card.Root>

    <!-- Appraisal Modal -->
    <Dialog.Root bind:open={isAppraisalModalOpen}>
        <Dialog.Content
            class="sm:max-w-[800px] max-h-[90vh] overflow-y-auto bg-popover/80 border-border/50 backdrop-blur-xl"
        >
            <Dialog.Header>
                <Dialog.Title class="text-foreground"
                    >{$t("fiscal.irpf.modal.title")}</Dialog.Title
                >
                <Dialog.Description class="text-muted-foreground"
                    >{$t("fiscal.irpf.modal.description")}</Dialog.Description
                >
            </Dialog.Header>

            <div class="space-y-6 py-4">
                <div
                    class="flex flex-col md:flex-row gap-4 items-end bg-muted/20 p-4 rounded-lg border border-border/20"
                >
                    <div class="space-y-2 w-full md:w-40">
                        <Label class="text-foreground"
                            >{$t("fiscal.irpf.modal.month")}</Label
                        >
                        <Select.Root type="single" bind:value={appraisalMonth}>
                            <Select.Trigger
                                class="h-10 w-full bg-muted/20 border-border/30 text-foreground"
                            >
                                {$t(
                                    months.find(
                                        (m) => String(m.val) === appraisalMonth,
                                    )?.key || "fiscal.irpf.modal.month",
                                )}
                            </Select.Trigger>
                            <Select.Content>
                                {#each months as m}
                                    <Select.Item
                                        value={String(m.val)}
                                        label={$t(m.key)}
                                        >{$t(m.key)}</Select.Item
                                    >
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>
                    <div class="space-y-2 w-full md:w-32">
                        <Label class="text-foreground"
                            >{$t("fiscal.irpf.modal.year")}</Label
                        >
                        <div class="relative w-full">
                            <Input
                                type="number"
                                bind:value={appraisalYear}
                                class="h-10 bg-muted/20 border-border/30 text-foreground text-left pl-3 pr-8 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                            />
                            <div
                                class="absolute inset-y-0 right-0 flex flex-col border-l border-border/10 w-8"
                            >
                                <button
                                    type="button"
                                    class="flex-1 flex items-center justify-center hover:bg-accent/10 transition-colors"
                                    onclick={() => appraisalYear++}
                                >
                                    <ChevronLeft
                                        class="w-3 h-3 text-muted-foreground/70 rotate-90"
                                    />
                                </button>
                                <div class="h-px w-full bg-border/20"></div>
                                <button
                                    type="button"
                                    class="flex-1 flex items-center justify-center hover:bg-accent/10 transition-colors"
                                    onclick={() => appraisalYear--}
                                >
                                    <ChevronRight
                                        class="w-3 h-3 text-white/70 rotate-90"
                                    />
                                </button>
                            </div>
                        </div>
                    </div>
                    <Button
                        class="neon-glow bg-primary text-primary-foreground font-bold w-full md:w-auto"
                        onclick={calculateAppraisal}
                        disabled={irpfStore.loading}
                    >
                        {#if irpfStore.loading}
                            {$t("fiscal.irpf.calculating")}
                        {:else}
                            <FileText class="w-4 h-4 mr-2" />
                            {$t("fiscal.irpf.calculate")}
                        {/if}
                    </Button>
                </div>

                {#if appraisalResults.length > 0}
                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        {#each appraisalResults as item}
                            <Card.Root
                                class="bg-card/30 border-border/40 glass overflow-hidden relative group"
                            >
                                <div
                                    class="absolute top-0 left-0 w-1 h-full {item.trade_type ===
                                    'DayTrade'
                                        ? 'bg-blue-500'
                                        : 'bg-indigo-500'}"
                                ></div>
                                <Card.Header>
                                    <Card.Title
                                        class="flex justify-between items-center"
                                    >
                                        <span
                                            class="text-lg {item.trade_type ===
                                            'DayTrade'
                                                ? 'text-blue-400'
                                                : 'text-indigo-400'}"
                                        >
                                            {item.trade_type === "DayTrade"
                                                ? "Day Trade"
                                                : "Swing Trade"}
                                        </span>
                                        {#if item.is_exempt}
                                            <span
                                                class="px-2 py-1 rounded text-xs bg-green-500/10 text-green-500 border border-green-500/20"
                                                >Isento</span
                                            >
                                        {/if}
                                    </Card.Title>
                                </Card.Header>
                                <Card.Content class="space-y-4">
                                    <div class="grid grid-cols-2 gap-4 text-sm">
                                        <div class="space-y-1">
                                            <span class="text-muted-foreground"
                                                >Lucro Bruto</span
                                            >
                                            <div
                                                class="text-green-400 font-mono font-bold tabular-nums leading-none"
                                            >
                                                {formatCurrency(
                                                    item.gross_profit,
                                                )}
                                            </div>
                                        </div>
                                        <div class="space-y-1">
                                            <span class="text-muted-foreground"
                                                >Prejuízo</span
                                            >
                                            <div
                                                class="text-red-400 font-mono font-bold tabular-nums leading-none"
                                            >
                                                {formatCurrency(item.loss)}
                                            </div>
                                        </div>
                                        <div
                                            class="space-y-1 pt-2 border-t border-border/10 col-span-2"
                                        >
                                            <span
                                                class="text-muted-foreground font-bold"
                                                >Lucro Líquido</span
                                            >
                                            <div
                                                class="text-xl font-mono font-bold tabular-nums {item.net_profit >=
                                                0
                                                    ? 'text-foreground'
                                                    : 'text-red-400'}"
                                            >
                                                {formatCurrency(
                                                    item.net_profit,
                                                )}
                                            </div>
                                        </div>

                                        {#if item.net_profit > 0}
                                            <div
                                                class="space-y-1 col-span-2 bg-accent/5 p-3 rounded border border-border/10"
                                            >
                                                <div
                                                    class="flex justify-between"
                                                >
                                                    <span
                                                        class="text-muted-foreground"
                                                        >Base de Cálculo:</span
                                                    >
                                                    <span
                                                        class="font-mono font-bold tabular-nums leading-none"
                                                        >{formatCurrency(
                                                            item.calculation_basis,
                                                        )}</span
                                                    >
                                                </div>
                                                <div
                                                    class="flex justify-between"
                                                >
                                                    <span
                                                        class="text-muted-foreground"
                                                        >Alíquota:</span
                                                    >
                                                    <span
                                                        class="font-mono font-bold tabular-nums leading-none"
                                                        >{item.tax_rate}%</span
                                                    >
                                                </div>
                                                <div
                                                    class="flex justify-between text-red-600 dark:text-red-400"
                                                >
                                                    <span
                                                        class="text-muted-foreground"
                                                        >Imposto Devido:</span
                                                    >
                                                    <span
                                                        class="font-mono font-bold tabular-nums leading-none"
                                                        >{formatCurrency(
                                                            item.tax_due,
                                                        )}</span
                                                    >
                                                </div>
                                                {#if item.withheld_tax > 0}
                                                    <div
                                                        class="flex justify-between text-emerald-600 dark:text-emerald-400"
                                                    >
                                                        <span
                                                            class="text-muted-foreground"
                                                            >IRRF (Dedo-Duro):</span
                                                        >
                                                        <span
                                                            class="font-mono font-bold tabular-nums leading-none"
                                                            >-{formatCurrency(
                                                                item.withheld_tax,
                                                            )}</span
                                                        >
                                                    </div>
                                                {/if}
                                                {#if item.withholding_credit_used > 0}
                                                    <div
                                                        class="flex justify-between text-blue-600 dark:text-blue-400"
                                                    >
                                                        <span
                                                            class="text-muted-foreground"
                                                            >Crédito IRRF
                                                            Aplicado:</span
                                                        >
                                                        <span
                                                            class="font-mono font-bold tabular-nums leading-none"
                                                            >-{formatCurrency(
                                                                item.withholding_credit_used,
                                                            )}</span
                                                        >
                                                    </div>
                                                {/if}
                                                <div
                                                    class="flex justify-between pt-2 border-t border-border/10 mt-2"
                                                >
                                                    <span
                                                        class="text-muted-foreground"
                                                        >Imposto do Mês:</span
                                                    >
                                                    <span
                                                        class="font-mono font-bold tabular-nums leading-none text-foreground"
                                                        >{formatCurrency(
                                                            item.tax_payable,
                                                        )}</span
                                                    >
                                                </div>
                                                {#if item.withholding_credit_remaining > 0}
                                                    <div
                                                        class="flex justify-between text-cyan-600 dark:text-cyan-400"
                                                    >
                                                        <span
                                                            class="text-muted-foreground"
                                                            >Saldo IRRF p/
                                                            Futuro:</span
                                                        >
                                                        <span
                                                            class="font-mono font-bold tabular-nums leading-none"
                                                            >{formatCurrency(
                                                                item.withholding_credit_remaining,
                                                            )}</span
                                                        >
                                                    </div>
                                                {/if}
                                                {#if item.tax_accumulated > 0}
                                                    <div
                                                        class="flex justify-between"
                                                    >
                                                        <span
                                                            class="text-muted-foreground"
                                                            >Acumulado Anterior:</span
                                                        >
                                                        <span
                                                            class="font-black tabular-nums leading-none text-yellow-400"
                                                            >+{formatCurrency(
                                                                item.tax_accumulated,
                                                            )}</span
                                                        >
                                                    </div>
                                                {/if}
                                                <div
                                                    class="flex justify-between pt-2 border-t border-border/10 mt-1"
                                                >
                                                    <span
                                                        class="font-bold text-foreground"
                                                        >Total a Pagar:</span
                                                    >
                                                    <span
                                                        class="font-bold text-xl text-primary font-mono"
                                                        >{formatCurrency(
                                                            item.total_payable,
                                                        )}</span
                                                    >
                                                </div>
                                            </div>
                                        {:else}
                                            <div
                                                class="col-span-2 p-3 rounded bg-yellow-500/10 border border-yellow-500/20 flex items-center gap-3"
                                            >
                                                <AlertTriangle
                                                    class="w-5 h-5 text-amber-500"
                                                />
                                                <div class="flex flex-col">
                                                    <span
                                                        class="text-xs text-amber-600 dark:text-amber-400 font-medium"
                                                        >Prejuízo a compensar: {formatCurrency(
                                                            item.net_profit,
                                                        )}</span
                                                    >
                                                    {#if item.withholding_credit_remaining > 0}
                                                        <span
                                                            class="text-xs text-cyan-600 dark:text-cyan-400 font-bold"
                                                        >
                                                            Crédito IRRF
                                                            (Dedo-Duro): {formatCurrency(
                                                                item.withholding_credit_remaining,
                                                            )}
                                                        </span>
                                                    {/if}
                                                </div>
                                            </div>
                                        {/if}
                                    </div>
                                </Card.Content>
                                <Card.Footer
                                    class="bg-muted/10 border-t border-border/10 p-4 flex justify-end gap-2"
                                >
                                    {#if item.id}
                                        {#if item.total_payable >= 10}
                                            <Button
                                                variant="secondary"
                                                size="sm"
                                                onclick={() =>
                                                    generateDarf(item)}
                                            >
                                                <FileText
                                                    class="w-4 h-4 mr-2"
                                                /> Gerar DARF
                                            </Button>
                                        {:else if item.total_payable > 0}
                                            <div
                                                class="flex items-center px-3 py-1 bg-yellow-500/10 border border-yellow-500/20 rounded text-yellow-500 text-xs mr-auto"
                                            >
                                                <AlertCircle
                                                    class="w-3 h-3 mr-1"
                                                />
                                                Valor menor que R$ 10,00 (Transportado)
                                            </div>
                                        {/if}
                                    {/if}
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        class="border-border hover:bg-accent/10"
                                        onclick={() =>
                                            saveAppraisalResult(item)}
                                    >
                                        <FileText class="w-4 h-4 mr-2" /> Salvar
                                    </Button>
                                </Card.Footer>
                            </Card.Root>
                        {/each}
                    </div>
                {/if}
            </div>
        </Dialog.Content>
    </Dialog.Root>

    <!-- Delete Modal -->
    <Dialog.Root bind:open={isDeleteModalOpen}>
        <Dialog.Content class="sm:max-w-[400px]">
            <Dialog.Header>
                <Dialog.Title>Confirmar Exclusão</Dialog.Title>
                <Dialog.Description
                    >Tem certeza que deseja excluir esta apuração? Isso pode
                    afetar o saldo de prejuízos.</Dialog.Description
                >
            </Dialog.Header>
            <Dialog.Footer>
                <Button
                    variant="outline"
                    onclick={() => (isDeleteModalOpen = false)}>Cancelar</Button
                >
                <Button variant="destructive" onclick={confirmDelete}
                    >Excluir</Button
                >
            </Dialog.Footer>
        </Dialog.Content>
    </Dialog.Root>

    <!-- Standardized DARF/Appraisal Details Dialog -->
    <DarfDetailsDialog
        darfId={selectedAppraisal
            ? irpfStore.darfs.find(
                  (d) =>
                      irpfStore.getId(d.appraisal_id) ===
                      irpfStore.getId(selectedAppraisal.id),
              )?.id || ""
            : ""}
        appraisalId={selectedAppraisal
            ? irpfStore.getId(selectedAppraisal.id)
            : ""}
        bind:open={isViewModalOpen}
    />
</div>
