<script lang="ts">
    import * as Table from "$lib/components/ui/table";
    import * as Select from "$lib/components/ui/select";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import DateFilter from "$lib/components/filters/DateFilter.svelte";
    import {
        Search,
        Wallet,
        ArrowDownLeft,
        ArrowUpRight,
        RefreshCw,
        Eye,
        Trash2,
        Coins,
        Calendar,
        CalendarCheck,
    } from "lucide-svelte";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { Badge } from "$lib/components/ui/badge";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Tooltip from "$lib/components/ui/tooltip";
    import { t, locale } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import { cn, getLocalDatePart } from "$lib/utils";
    import { untrack } from "svelte";
    import DarfDetailsDialog from "./DarfDetailsDialog.svelte";
    import HierarchicalList from "$lib/components/shared/HierarchicalList.svelte";
    import DetailModal from "$lib/components/shared/DetailModal.svelte";
    import TradeDetailView from "./TradeDetailView.svelte";

    let selectedTransaction = $state<any>(null);
    let showDetailsDialog = $state(false);

    let isDarfDetailOpen = $state(false);
    let selectedDarfTxId = $state("");

    function openDarfDetails(txId: string) {
        selectedDarfTxId = txId;
        isDarfDetailOpen = true;
    }

    let selectedDay = $state<string | null>(null);
    let selectedDayCurrency = $state<string | null>(null);
    let showDayDetailsDialog = $state(false);

    function getWeekRange(date: Date) {
        const d = new Date(date);
        const day = d.getDay();
        const diff = d.getDate() - day + (day === 0 ? -6 : 1);
        const monday = new Date(d.setDate(diff));
        const sunday = new Date(new Date(monday).setDate(monday.getDate() + 6));
        return { start: monday, end: sunday };
    }

    let pageSize = $state(25);

    let groupedTransactions = $derived.by(() => {
        const dayGroups: Record<string, any[]> = {};

        settingsStore.cashTransactions
            .filter((tx) => {
                const txDateStr = tx.date.substring(0, 10);
                const txDate = new Date(txDateStr + "T00:00:00");
                const today = new Date();
                today.setHours(0, 0, 0, 0);
                const todayStr = today.toISOString().split("T")[0];

                if (dateFilter === "today") return txDateStr === todayStr;
                if (dateFilter === "yesterday") {
                    const yesterday = new Date(today);
                    yesterday.setDate(yesterday.getDate() - 1);
                    return txDateStr === yesterday.toISOString().split("T")[0];
                }
                if (dateFilter === "this_week") {
                    const { start, end } = getWeekRange(today);
                    return txDate >= start && txDate <= end;
                }
                if (dateFilter === "last_week") {
                    const lastWeek = new Date(today);
                    lastWeek.setDate(lastWeek.getDate() - 7);
                    const { start, end } = getWeekRange(lastWeek);
                    return txDate >= start && txDate <= end;
                }
                if (dateFilter === "this_month")
                    return (
                        txDateStr.substring(0, 7) === todayStr.substring(0, 7)
                    );
                if (dateFilter === "last_month") {
                    const lastMonth = new Date(today);
                    lastMonth.setMonth(lastMonth.getMonth() - 1);
                    const lastMonthStr = lastMonth
                        .toISOString()
                        .split("T")[0]
                        .substring(0, 7);
                    return txDateStr.substring(0, 7) === lastMonthStr;
                }
                if (dateFilter === "this_year")
                    return (
                        txDateStr.substring(0, 4) === todayStr.substring(0, 4)
                    );
                if (dateFilter === "last_year") {
                    const lastYear = new Date(today);
                    lastYear.setFullYear(lastYear.getFullYear() - 1);
                    const lastYearStr = lastYear
                        .toISOString()
                        .split("T")[0]
                        .substring(0, 4);
                    return txDateStr.substring(0, 4) === lastYearStr;
                }
                if (dateFilter === "custom") {
                    if (!customStartDate && !customEndDate) return true;
                    let match = true;
                    if (customStartDate)
                        match = match && txDateStr >= customStartDate;
                    if (customEndDate)
                        match = match && txDateStr <= customEndDate;
                    return match;
                }
                return true;
            })
            .filter(
                (tx) =>
                    accountFilter === "all" ||
                    String(tx.account_id) === String(accountFilter),
            )
            .filter((tx) => {
                if (currencyFilter === "all") return true;
                const acc = settingsStore.accounts.find(
                    (a) => String(a.id) === String(tx.account_id),
                );
                return acc?.currency === currencyFilter;
            })
            .filter(
                (tx) =>
                    tx.description
                        .toLowerCase()
                        .includes(searchTerm.toLowerCase()) ||
                    tx.amount.toString().includes(searchTerm),
            )
            .forEach((tx) => {
                const dateKey = getLocalDatePart(tx.date);
                if (!dayGroups[dateKey]) dayGroups[dateKey] = [];
                dayGroups[dateKey].push(tx);
            });

        // Sort transactions within each day
        Object.values(dayGroups || {}).forEach((group) => {
            group.sort((a, b) => {
                const dateSort = b.date.localeCompare(a.date);
                if (dateSort !== 0) return dateSort;
                return b.id.localeCompare(a.id);
            });
        });

        // Aggregate into Months
        const months: Record<
            string,
            {
                monthKey: string;
                days: any[];
                monthlyPnl: Record<string, number>;
            }
        > = {};

        Object.keys(dayGroups)
            .sort((a, b) => b.localeCompare(a))
            .forEach((date) => {
                const monthKey = date.substring(0, 7);
                if (!months[monthKey]) {
                    months[monthKey] = { monthKey, days: [], monthlyPnl: {} };
                }

                const pnl: Record<string, number> = {};
                dayGroups[date].forEach((tx) => {
                    const acc = settingsStore.accounts.find(
                        (a) => String(a.id) === String(tx.account_id),
                    );
                    const curr = acc?.currency || "BRL";
                    pnl[curr] = (pnl[curr] || 0) + tx.amount;
                    months[monthKey].monthlyPnl[curr] =
                        (months[monthKey].monthlyPnl[curr] || 0) + tx.amount;
                });

                months[monthKey].days.push({
                    date,
                    transactions: dayGroups[date],
                    pnl,
                });
            });

        return Object.values(months || {})
            .sort((a, b) => b.monthKey.localeCompare(a.monthKey))
            .slice(0, pageSize);
    });

    import { endOfWeek } from "date-fns/endOfWeek";
    import { format } from "date-fns/format";
    import { isSameMonth } from "date-fns/isSameMonth";
    import { startOfWeek } from "date-fns/startOfWeek";

    // Map groupedTransactions → HierarchicalList format (Month → Week → Day)
    const hierarchicalData = $derived.by(() => {
        return groupedTransactions.map(({ monthKey, days, monthlyPnl }) => {
            // 1. Group days into weeks for this month
            const weekGroups: Record<string, any> = {};

            days.forEach(({ date, transactions, pnl }) => {
                const dayDate = new Date(date + "T12:00:00");
                const weekStart = startOfWeek(dayDate, { weekStartsOn: 1 }); // Monday start

                // Ensure the week label belongs to the current month to avoid "Week of undefined" overlaps
                const weekLabelDate = isSameMonth(weekStart, dayDate)
                    ? weekStart
                    : dayDate;
                const weekKey = format(weekStart, "yyyy-MM-dd");

                if (!weekGroups[weekKey]) {
                    weekGroups[weekKey] = {
                        key: weekKey,
                        label: `${new Date(weekLabelDate).toLocaleDateString($locale || "pt-BR", { day: "numeric" })} - ${new Date(endOfWeek(weekStart, { weekStartsOn: 1 })).toLocaleDateString($locale || "pt-BR", { day: "numeric", month: "short" })}`,
                        days: [],
                        pnlEntries: {},
                    };
                }

                weekGroups[weekKey].days.push({
                    key: date,
                    date,
                    label: new Date(dayDate).toLocaleDateString(
                        $locale || "pt-BR",
                        {
                            weekday: "long",
                            day: "numeric",
                            month: "short",
                        },
                    ),
                    trades: transactions,
                    pnlEntries: Object.entries(pnl).map(([curr, val]) => ({
                        curr,
                        val: val as number,
                    })),
                    meta: { pnl },
                });

                // Accumulate weekly PnL
                Object.entries(pnl).forEach(([curr, val]) => {
                    weekGroups[weekKey].pnlEntries[curr] =
                        (weekGroups[weekKey].pnlEntries[curr] || 0) +
                        (val as number);
                });
            });

            // 2. Format weeks and overall month structure
            return {
                key: monthKey,
                label: new Date(monthKey + "-02T12:00:00").toLocaleDateString(
                    $locale || "pt-BR",
                    { month: "long", year: "numeric" },
                ),
                trades: [],
                weeks: Object.values(weekGroups)
                    .sort((a, b) => b.key.localeCompare(a.key))
                    .map((w) => ({
                        ...w,
                        pnlEntries: Object.entries(w.pnlEntries).map(
                            ([curr, val]) => ({ curr, val: val as number }),
                        ),
                    })),
                days: [], // Not used when we have weeks
                pnlEntries: Object.entries(monthlyPnl).map(([curr, val]) => ({
                    curr,
                    val,
                })),
            };
        });
    });

    function formatCurrencyValue(val: number, currency: string) {
        try {
            return new Intl.NumberFormat($locale || "pt-BR", {
                style: "currency",
                currency: currency,
            }).format(val);
        } catch (e) {
            return `${currency} ${val.toLocaleString($locale || "pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
        }
    }

    let accountOptions = $derived([
        { value: "all", label: $t("finance.statement.allAccounts") },
        ...settingsStore.accounts.map((a) => ({
            value: a.id,
            label: a.nickname,
        })),
    ]);

    let currencyOptions = $derived([
        { value: "all", label: $t("finance.statement.allCurrencies") },
        ...settingsStore.currencies.map((c) => ({
            value: c.code,
            label: `${c.name} (${c.code})`,
        })),
    ]);

    function openDayDetails(date: string, currency: string) {
        selectedDay = date;
        selectedDayCurrency = currency;
        showDayDetailsDialog = true;
    }

    function openDetails(tx: any) {
        selectedTransaction = tx;
        showDetailsDialog = true;
    }

    let dayDetailsStats = $derived.by(() => {
        if (!selectedDay) return null;

        let foundDay = null;
        for (const m of groupedTransactions) {
            foundDay = m.days.find((d) => d.date === selectedDay);
            if (foundDay) break;
        }
        if (!foundDay) return null;

        const txs = foundDay.transactions.filter((tx: any) => {
            const acc = getAccount(tx.account_id);
            return (
                !selectedDayCurrency || acc?.currency === selectedDayCurrency
            );
        });

        let deposits = 0;
        let withdrawals = 0;

        txs.forEach((tx: any) => {
            if (tx.amount > 0) deposits += tx.amount;
            else withdrawals += Math.abs(tx.amount);
        });

        return {
            date: selectedDay,
            currency: selectedDayCurrency,
            deposits,
            withdrawals,
            net: foundDay.pnl[selectedDayCurrency || "BRL"] || 0,
            count: txs.length,
            transactions: txs,
        };
    });

    let dateFilter = $state("all");
    let customStartDate = $state("");
    let customEndDate = $state("");
    let accountFilter = $state("all");
    let currencyFilter = $state("all");
    let searchTerm = $state("");
    let expandedMonths = $state(new Set<string>());
    let expandedWeeks = $state(new Set<string>());

    // Delete State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);
    let isDeleteWithJournalOpen = $state(false);
    let deleteJournalId = $state<string | null>(null);

    function requestDelete(id: string) {
        const tx = settingsStore.cashTransactions.find((t) => t.id === id);
        if (tx && tx.id.includes("daily_closure_")) {
            const txDate = getLocalDatePart(tx.date);
            const hasJournal = settingsStore.getJournalEntryByDate(txDate);
            if (hasJournal) {
                deleteId = id;
                deleteJournalId = hasJournal.id;
                isDeleteWithJournalOpen = true;
                return;
            }
        }
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDeleteWithJournal(deleteJournal: boolean) {
        if (deleteId) {
            const result = await settingsStore.removeCashTransaction(deleteId);
            if (result.success) {
                if (deleteJournal && deleteJournalId) {
                    const jRes =
                        await settingsStore.removeJournalEntry(deleteJournalId);
                    if (jRes.success === false) {
                        toast.error(
                            "Financeiro removido, mas falhou ao remover psicológico.",
                        );
                    } else {
                        toast.success(
                            $t("general.deleteSuccess") ||
                                "Removido com sucesso",
                        );
                    }
                } else {
                    toast.success(
                        $t("general.deleteSuccess") || "Removido com sucesso",
                    );
                }
            } else {
                toast.error(
                    result.error ||
                        $t("finance.statement.messages.deleteError"),
                );
            }
            deleteId = null;
            deleteJournalId = null;
            isDeleteWithJournalOpen = false;
        }
    }

    async function confirmDelete() {
        if (deleteId) {
            const result = await settingsStore.removeCashTransaction(deleteId);
            if (result.success) {
                toast.success($t("general.deleteSuccess"));
            } else {
                toast.error(
                    result.error ||
                        $t("finance.statement.messages.deleteError"),
                );
            }
            deleteId = null;
        }
    }

    import { onMount } from "svelte";

    onMount(() => {
        if (tradesStore.trades.length === 0) {
            tradesStore.loadTrades();
        }
    });

    function getAccount(id: string) {
        return settingsStore.accounts.find((a) => String(a.id) === String(id));
    }

    function findTradeById(id: string) {
        if (!id) return null;
        const normalizedId = tradesStore.normalizeId(id);
        return tradesStore.trades.find((t) => {
            const tid = tradesStore.normalizeId(t.id);
            return tid === normalizedId;
        });
    }

    $effect(() => {
        if (hierarchicalData.length > 0) {
            untrack(() => {
                const today = new Date();
                const currentMonthKey = today.toISOString().slice(0, 7);
                const currentWeekKey = format(startOfWeek(today, { weekStartsOn: 1 }), "yyyy-MM-dd");

                // Auto-expand current month and week if not already set
                if (expandedMonths.size === 0) {
                    // Try to find current month
                    if (hierarchicalData.some(m => m.key === currentMonthKey)) {
                        expandedMonths.add(currentMonthKey);
                    } else {
                        // Fallback to latest month
                        expandedMonths.add(hierarchicalData[0].key);
                    }
                    
                    // Try to find current week in the expanded month
                    const month = hierarchicalData.find(m => expandedMonths.has(m.key));
                    if (month && month.weeks.length > 0) {
                        const currentWeek = month.weeks.find((w: any) => w.key === currentWeekKey);
                        if (currentWeek) {
                            expandedWeeks.add(currentWeek.key);
                        } else {
                            expandedWeeks.add(month.weeks[0].key);
                        }
                    }
                    
                    expandedMonths = new Set(expandedMonths);
                    expandedWeeks = new Set(expandedWeeks);
                }
            });
        }
    });
</script>

<div class="space-y-4">
    <!-- Filters Header -->
    <div
        class="flex flex-col sm:flex-row gap-4 justify-between items-end card-glass p-4"
    >
        <div class="space-y-1">
            <h3
                class="text-sm font-black uppercase tracking-widest text-muted-foreground"
            >
                {$t("finance.statement.title")}
            </h3>
            <p
                class="text-[10px] text-muted-foreground/60 font-bold uppercase tracking-tighter"
            >
                {$t("finance.statement.hierarchy", { default: "HIERARQUIA" })}
            </p>
        </div>

        <div
            class="flex flex-wrap items-center justify-end gap-2 w-full sm:w-auto"
        >
            <div class="relative w-[140px] md:w-48">
                <Search
                    class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground"
                />
                <Input
                    placeholder={$t("finance.statement.searchPlaceholder")}
                    class="pl-10 h-9 bg-background/50 border-border"
                    bind:value={searchTerm}
                />
            </div>

            <DateFilter
                bind:value={dateFilter}
                bind:startDate={customStartDate}
                bind:endDate={customEndDate}
            />

            <Select.Root type="single" bind:value={accountFilter}>
                <Select.Trigger
                    class="w-[140px] h-9 bg-background/50 border-border"
                >
                    <div class="flex items-center gap-2 truncate">
                        <Wallet
                            class="w-3.5 h-3.5 text-muted-foreground shrink-0"
                        />
                        <span class="truncate text-[10px] font-bold uppercase">
                            {accountOptions.find(
                                (o) => o.value === accountFilter,
                            )?.label ?? $t("finance.statement.account")}
                        </span>
                    </div>
                </Select.Trigger>
                <Select.Content>
                    {#each accountOptions as opt}
                        <Select.Item
                            value={opt.value}
                            class="text-[10px] font-bold uppercase"
                            >{opt.label}</Select.Item
                        >
                    {/each}
                </Select.Content>
            </Select.Root>

            <Select.Root type="single" bind:value={currencyFilter}>
                <Select.Trigger
                    class="w-[140px] h-9 bg-background/50 border-border"
                >
                    <div class="flex items-center gap-2 truncate">
                        <Coins
                            class="w-3.5 h-3.5 text-muted-foreground shrink-0"
                        />
                        <span class="truncate text-[10px] font-bold uppercase">
                            {currencyOptions.find(
                                (o) => o.value === currencyFilter,
                            )?.label ?? $t("general.currency")}
                        </span>
                    </div>
                </Select.Trigger>
                <Select.Content>
                    {#each currencyOptions as opt}
                        <Select.Item
                            value={opt.value}
                            class="text-[10px] font-bold uppercase"
                            >{opt.label}</Select.Item
                        >
                    {/each}
                </Select.Content>
            </Select.Root>

            <div class="flex items-center gap-2 pl-2 border-l border-border/30">
                <span
                    class="text-[9px] font-bold text-muted-foreground uppercase tracking-widest hidden lg:inline"
                    >{$t("general.show")}</span
                >
                <Select.Root
                    type="single"
                    value={pageSize.toString()}
                    onValueChange={(v) => (pageSize = parseInt(v))}
                >
                    <Select.Trigger
                        class="w-[90px] h-9 bg-background/50 border-border text-[10px] font-bold uppercase"
                    >
                        <span class="truncate"
                            >{pageSize} {$t("general.time.months")}</span
                        >
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item
                            value="1"
                            class="text-[10px] font-bold uppercase"
                            >1 {$t("general.time.month")}</Select.Item
                        >
                        <Select.Item
                            value="3"
                            class="text-[10px] font-bold uppercase"
                            >3 {$t("general.time.months")}</Select.Item
                        >
                        <Select.Item
                            value="6"
                            class="text-[10px] font-bold uppercase"
                            >6 {$t("general.time.months")}</Select.Item
                        >
                        <Select.Item
                            value="12"
                            class="text-[10px] font-bold uppercase"
                            >1 {$t("general.time.year")}</Select.Item
                        >
                        <Select.Item
                            value="24"
                            class="text-[10px] font-bold uppercase"
                            >2 {$t("general.time.years")}</Select.Item
                        >
                        <Select.Item
                            value="100"
                            class="text-[10px] font-bold uppercase"
                            >{$t("general.all")}</Select.Item
                        >
                    </Select.Content>
                </Select.Root>
            </div>
        </div>
    </div>

    <!-- Hierarchical Statement List (Month → Day) -->
    {#if hierarchicalData.length === 0}
        <div
            class="rounded-xl border border-border/50 bg-muted/20 p-12 text-center text-muted-foreground font-medium"
        >
            <div class="flex flex-col items-center gap-2">
                <Search class="w-8 h-8 opacity-20" />
                {$t("finance.statement.noTransactions")}
            </div>
        </div>
    {:else}
        <HierarchicalList 
            data={hierarchicalData}
            autoExpandDefault={true}
            bind:expandedMonths
            bind:expandedWeeks
            mutualExclusion={true}
        >
            <!-- Month badges: monthly PnL per currency -->
            {#snippet monthBadges(month: any)}
                <Badge
                    variant="outline"
                    class="text-[9px] px-1.5 h-4 bg-muted border-border font-bold uppercase"
                >
                    {month.weeks.length}
                    {$t("general.weeks_upper", { default: "SEMANAS" })}
                </Badge>
                <div class="flex gap-1.5 opacity-60">
                    {#each month.pnlEntries ?? [] as entry}
                        <span
                            class="text-[9px] font-bold {entry.val >= 0
                                ? 'text-emerald-500'
                                : 'text-rose-500'}"
                        >
                            {formatCurrencyValue(entry.val, entry.curr)}
                        </span>
                    {/each}
                </div>
            {/snippet}

            <!-- Week badges: weekly PnL per currency -->
            {#snippet weekBadges(week: any)}
                <div class="flex gap-3">
                    {#each week.pnlEntries ?? [] as entry}
                        <div class="flex items-center gap-1.5">
                            <span
                                class="text-[8px] text-muted-foreground uppercase tracking-widest"
                                >{entry.curr}</span
                            >
                            <span
                                class="text-[10px] font-mono font-bold {entry.val >=
                                0
                                    ? 'text-emerald-500'
                                    : 'text-rose-500'}"
                            >
                                {entry.val.toFixed(2)}
                            </span>
                        </div>
                    {/each}
                </div>
            {/snippet}

            <!-- Week right content (optional, currently empty) -->
            {#snippet weekRight()}
                <div></div>
            {/snippet}

            <!-- Day right: PnL per currency + Eye button -->
            {#snippet dayRight(day: any)}
                <div class="flex items-center gap-4">
                    <div class="flex gap-4">
                        {#each day.pnlEntries ?? [] as entry}
                            <div class="flex items-center gap-2">
                                <span
                                    class="text-[10px] font-mono font-bold {entry.val >=
                                    0
                                        ? 'text-emerald-500'
                                        : 'text-rose-500'}"
                                >
                                    {formatCurrencyValue(entry.val, entry.curr)}
                                </span>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="h-5 w-5 hover:bg-accent/10"
                                    onclick={(e: MouseEvent) => {
                                        e.stopPropagation();
                                        openDayDetails(day.date, entry.curr);
                                    }}
                                >
                                    <Eye
                                        class="w-2.5 h-2.5 text-muted-foreground hover:text-foreground"
                                    />
                                </Button>
                            </div>
                        {/each}
                    </div>
                </div>
            {/snippet}

            <!-- Day content: transaction table -->
            {#snippet dayContent(day: any)}
                <Table.Root>
                    <Table.Header class="bg-muted/20">
                        <Table.Row class="hover:bg-transparent border-border">
                            <Table.Head
                                class="w-[120px] text-[10px] font-bold uppercase tracking-widest text-muted-foreground"
                                >{$t(
                                    "finance.statement.columns.type",
                                )}</Table.Head
                            >
                            <Table.Head
                                class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground"
                                >{$t(
                                    "finance.statement.columns.description",
                                )}</Table.Head
                            >
                            <Table.Head
                                class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground"
                                >{$t(
                                    "finance.statement.columns.account",
                                )}</Table.Head
                            >
                            <Table.Head
                                class="text-right text-[10px] font-bold uppercase tracking-widest text-muted-foreground"
                                >{$t(
                                    "finance.statement.columns.amount",
                                )}</Table.Head
                            >
                            <Table.Head class="w-[80px]"></Table.Head>
                        </Table.Row>
                    </Table.Header>
                    <Table.Body>
                        {#each day.trades as tx}
                            {@const acc = getAccount(tx.account_id)}
                            <Table.Row
                                class="border-border/50 hover:bg-muted/20"
                            >
                                <!-- Type badge -->
                                <Table.Cell>
                                    {#if tx.category === 'TaxPayment'}
                                        <div
                                            class="flex items-center text-indigo-400 text-[10px] font-black uppercase tracking-tighter bg-indigo-500/10 px-2 py-0.5 rounded border border-indigo-500/20 w-fit"
                                        >
                                            <Coins class="w-3 h-3 mr-1" />
                                            {$t("finance.statement.categories.tax_payment")}
                                        </div>
                                    {:else if tx.category === 'Trading' || (tx.id && tx.id.includes("daily_closure_"))}
                                        <div
                                            class="flex items-center text-blue-400 text-[10px] font-black uppercase tracking-tighter bg-blue-500/10 px-2 py-0.5 rounded border border-blue-500/20 w-fit"
                                        >
                                            <CalendarCheck class="w-3 h-3 mr-1" />
                                            {$t("finance.statement.categories.trading")}
                                        </div>
                                    {:else if tx.category === 'Transfer'}
                                        <div
                                            class="flex items-center text-orange-400 text-[10px] font-black uppercase tracking-tighter bg-orange-500/10 px-2 py-0.5 rounded border border-orange-500/20 w-fit"
                                        >
                                            <RefreshCw class="w-3 h-3 mr-1" />
                                            {$t("finance.statement.categories.transfer")}
                                        </div>
                                    {:else if tx.category === 'TaxRefund'}
                                        <div
                                            class="flex items-center text-cyan-400 text-[10px] font-black uppercase tracking-tighter bg-cyan-500/10 px-2 py-0.5 rounded border border-cyan-500/20 w-fit"
                                        >
                                            <ArrowDownLeft class="w-3 h-3 mr-1" />
                                            {$t("finance.statement.categories.tax_refund")}
                                        </div>
                                    {:else if tx.category === 'Adjustment' || tx.type === 'Adjustment'}
                                        <div
                                            class="flex items-center text-muted-foreground text-[10px] font-black uppercase tracking-tighter bg-muted/30 px-2 py-0.5 rounded border border-border/50 w-fit"
                                        >
                                            <RefreshCw class="w-3 h-3 mr-1" />
                                            {$t("finance.statement.categories.adjustment")}
                                        </div>
                                    {:else if tx.type === "Deposit"}
                                        <div
                                            class="flex items-center text-emerald-500 text-[10px] font-bold uppercase tracking-tighter bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20 w-fit"
                                        >
                                            <ArrowDownLeft class="w-3 h-3 mr-1" />
                                            {$t("finance.statement.types.deposit")}
                                        </div>
                                    {:else if tx.type === "Withdraw"}
                                        <div
                                            class="flex items-center text-rose-500 text-[10px] font-bold uppercase tracking-tighter bg-rose-500/10 px-2 py-0.5 rounded border border-rose-500/20 w-fit"
                                        >
                                            <ArrowUpRight class="w-3 h-3 mr-1" />
                                            {$t("finance.statement.types.withdraw")}
                                        </div>
                                    {:else}
                                        <div
                                            class="flex items-center text-blue-500 text-[10px] font-mono font-bold uppercase tracking-tighter bg-blue-500/10 px-2 py-0.5 rounded border border-blue-500/20 w-fit"
                                        >
                                            <RefreshCw class="w-3 h-3 mr-1" />
                                            {$t("finance.statement.types.result")}
                                        </div>
                                    {/if}
                                </Table.Cell>
                                <!-- Description -->
                                <Table.Cell class="font-medium text-foreground">
                                    {#if tx.id && tx.id.includes("daily_closure_")}
                                        {$t("finance.dailyClosure")} ({tx.trade_ids
                                            ? tx.trade_ids.length
                                            : 0}
                                        {$t("sidebar.trades").toLowerCase()})
                                    {:else}
                                        {tx.description}
                                    {/if}
                                </Table.Cell>
                                <!-- Account -->
                                <Table.Cell
                                    class="text-muted-foreground text-xs"
                                >
                                    <div class="flex items-center gap-1.5">
                                        <span
                                            class="font-bold text-foreground/80"
                                            >{acc?.nickname ?? "---"}</span
                                        >
                                        <span
                                            class="text-[10px] bg-muted px-1 rounded text-muted-foreground uppercase"
                                            >{acc?.currency ?? ""}</span
                                        >
                                    </div>
                                </Table.Cell>
                                <!-- Amount -->
                                <Table.Cell
                                    class="text-right font-mono font-bold {tx.amount >
                                    0
                                        ? 'text-emerald-500'
                                        : 'text-rose-500'}"
                                >
                                    {formatCurrencyValue(
                                        tx.amount,
                                        acc?.currency || "BRL",
                                    )}
                                </Table.Cell>
                                <!-- Actions -->
                                <Table.Cell>
                                    <div
                                        class="flex items-center justify-end gap-1"
                                    >
                                        {#if tx.category === 'TaxPayment' || tx.category === 'TaxRefund'}
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="h-7 w-7 text-amber-500 hover:text-amber-600 hover:bg-amber-500/10"
                                                onclick={() => openDarfDetails(tx.id)}
                                            >
                                                <Eye class="h-3.5 w-3.5" />
                                            </Button>
                                        {:else if tx.trade_ids && tx.trade_ids.length > 0}
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="h-7 w-7 text-muted-foreground hover:text-foreground hover:bg-muted"
                                                onclick={() => openDetails(tx)}
                                            >
                                                <Eye class="h-3.5 w-3.5" />
                                            </Button>
                                        {/if}

                                        {#if tx.system_linked && (!tx.id || !tx.id.includes("daily_closure_"))}
                                            <Tooltip.Root>
                                                <Tooltip.Trigger>
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-7 w-7 text-muted-foreground/30 cursor-not-allowed opacity-50"
                                                        disabled
                                                    >
                                                        <Trash2
                                                            class="h-3.5 w-3.5"
                                                        />
                                                    </Button>
                                                </Tooltip.Trigger>
                                                <Tooltip.Content
                                                    class="bg-popover border-border text-[10px] max-w-[200px]"
                                                >
                                                    {$t(
                                                        "finance.statement.messages.systemLinked",
                                                    )}
                                                </Tooltip.Content>
                                            </Tooltip.Root>
                                        {:else}
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="h-7 w-7 text-rose-500/50 hover:text-rose-500 hover:bg-rose-500/10"
                                                onclick={() =>
                                                    requestDelete(tx.id)}
                                            >
                                                <Trash2 class="h-3.5 w-3.5" />
                                            </Button>
                                        {/if}
                                    </div>
                                </Table.Cell>
                            </Table.Row>
                        {/each}
                    </Table.Body>
                </Table.Root>
            {/snippet}
        </HierarchicalList>
    {/if}
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDeleteWithJournalOpen}>
    <Dialog.Content class="max-w-[425px] w-full glass border-border">
        <Dialog.Header>
            <Dialog.Title class="text-foreground"
                >{$t("general.confirmDelete") ||
                    "Excluir Fechamento Diário"}</Dialog.Title
            >
            <Dialog.Description class="text-muted-foreground">
                Este fechamento possui um <strong
                    >Registro Diário Psicológico</strong
                > associado. Você deseja excluir apenas o registro financeiro ou
                ambos?
            </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer class="flex flex-col sm:flex-row gap-2 mt-4">
            <Button
                variant="outline"
                onclick={() => (isDeleteWithJournalOpen = false)}
                >{$t("general.cancel")}</Button
            >
            <Button
                variant="secondary"
                onclick={() => confirmDeleteWithJournal(false)}
            >
                Só Financeiro
            </Button>
            <Button
                variant="destructive"
                onclick={() => confirmDeleteWithJournal(true)}
            >
                Excluir Ambos
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={showDayDetailsDialog}>
    <Dialog.Content class="sm:max-w-[500px] glass border-border">
        <Dialog.Header>
            <Dialog.Title class="text-foreground flex items-center gap-2">
                <Calendar class="w-5 h-5 text-primary" />
                {$t("finance.statement.summary.title")}
            </Dialog.Title>
            <Dialog.Description
                class="text-muted-foreground font-mono text-[10px] uppercase tracking-widest"
            >
                {selectedDay
                    ? new Date(
                          selectedDay.includes("T")
                              ? selectedDay
                              : selectedDay + "T12:00:00",
                      ).toLocaleDateString($locale || "pt-BR", {
                          weekday: "long",
                          day: "numeric",
                          month: "long",
                      })
                    : ""}
            </Dialog.Description>
        </Dialog.Header>

        {#if dayDetailsStats}
            <div class="grid grid-cols-2 gap-4 py-4">
                <div class="bg-muted/50 p-4 rounded-xl border border-border/50">
                    <span
                        class="text-[10px] font-black text-muted-foreground uppercase tracking-widest block mb-1"
                        >{$t("finance.statement.summary.totalDeposits")}</span
                    >
                    <span class="text-lg font-mono font-bold text-emerald-500">
                        {formatCurrencyValue(
                            dayDetailsStats.deposits,
                            dayDetailsStats.currency || "BRL",
                        )}
                    </span>
                </div>
                <div class="bg-muted/50 p-4 rounded-xl border border-border/50">
                    <span
                        class="text-[10px] font-black text-muted-foreground uppercase tracking-widest block mb-1"
                        >{$t(
                            "finance.statement.summary.totalWithdrawals",
                        )}</span
                    >
                    <span class="text-lg font-mono font-bold text-rose-500">
                        {formatCurrencyValue(
                            dayDetailsStats.withdrawals,
                            dayDetailsStats.currency || "BRL",
                        )}
                    </span>
                </div>
                <div
                    class="col-span-2 bg-primary/5 p-4 rounded-xl border border-primary/20 flex justify-between items-center"
                >
                    <div>
                        <span
                            class="text-[10px] font-black text-primary uppercase tracking-widest block mb-1"
                            >{$t("finance.statement.summary.netResult")}</span
                        >
                        <span
                            class="text-2xl font-mono font-bold text-foreground leading-none"
                        >
                            {formatCurrencyValue(
                                dayDetailsStats.net,
                                dayDetailsStats.currency || "BRL",
                            )}
                        </span>
                    </div>
                    <div class="text-right">
                        <span
                            class="text-[10px] font-black text-muted-foreground uppercase tracking-widest block mb-1"
                            >{$t("finance.statement.entries")}</span
                        >
                        <span class="text-xl font-bold text-foreground/80"
                            >{dayDetailsStats.count}</span
                        >
                    </div>
                </div>
            </div>

            <div class="space-y-3 mt-2">
                <h4
                    class="text-[10px] font-black text-muted-foreground uppercase tracking-widest"
                >
                    {$t("finance.statement.summary.periodEntries")}
                </h4>
                <div class="max-h-[300px] overflow-y-auto space-y-2 pr-1">
                    {#each dayDetailsStats.transactions as tx}
                        <div
                            class="flex items-center justify-between p-3 rounded-lg bg-muted/30 border border-border/50"
                        >
                            <div class="flex flex-col">
                                <span class="text-xs font-bold text-foreground"
                                    >{tx.description}</span
                                >
                                <span class="text-[10px] text-muted-foreground"
                                    >{getAccount(tx.account_id)?.nickname}</span
                                >
                            </div>
                            <span
                                class="font-mono text-xs font-bold {tx.amount >
                                0
                                    ? 'text-emerald-500'
                                    : 'text-rose-500'}"
                            >
                                {formatCurrencyValue(
                                    tx.amount,
                                    getAccount(tx.account_id)?.currency ||
                                        "BRL",
                                )}
                            </span>
                        </div>
                    {/each}
                </div>
            </div>
        {/if}

        <Dialog.Footer>
            <Button
                variant="outline"
                class="w-full"
                onclick={() => (showDayDetailsDialog = false)}
            >
                {$t("general.close")}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

<DetailModal
    bind:open={showDetailsDialog}
    title={$t("finance.details.title")}
    icon={Eye}
    size="lg"
>
    {#if selectedTransaction}
        {@const acc = getAccount(selectedTransaction.account_id)}
        <div class="space-y-6">
            <div class="grid grid-cols-3 gap-6">
                <div class="space-y-1">
                    <span class="text-[10px] font-black text-muted-foreground uppercase tracking-widest opacity-60">
                        {$t("finance.details.date")}
                    </span>
                    <div class="text-foreground/80 font-bold font-mono text-sm">
                        {new Date(
                            selectedTransaction.date.includes("T")
                                ? selectedTransaction.date
                                : selectedTransaction.date + "T12:00:00",
                        ).toLocaleDateString()}
                    </div>
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] font-black text-muted-foreground uppercase tracking-widest opacity-60">
                        {$t("finance.details.account")}
                    </span>
                    <div class="text-foreground/80 font-bold text-sm">
                        {acc?.nickname}
                    </div>
                </div>
                <div class="space-y-1 text-right">
                    <span class="text-[10px] font-black text-muted-foreground uppercase tracking-widest opacity-60">
                        {$t("finance.transactionDialog.amount")}
                    </span>
                    <div class="text-xl font-black text-foreground font-mono tracking-tighter">
                        {formatCurrencyValue(
                            selectedTransaction.amount,
                            acc?.currency || "BRL",
                        )}
                    </div>
                </div>
            </div>

            <TradeDetailView 
                trades={selectedTransaction.trade_ids?.map(id => findTradeById(id)).filter(Boolean) || []} 
                currency={acc?.currency || "BRL"} 
            />
        </div>
    {/if}
</DetailModal>

<DarfDetailsDialog
    bind:open={isDarfDetailOpen}
    transactionId={selectedDarfTxId}
/>
