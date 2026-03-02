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
        ChevronDown,
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
        const diff = d.getDate() - day + (day === 0 ? -6 : 1); // adjust when day is sunday
        const monday = new Date(d.setDate(diff));
        const sunday = new Date(new Date(monday).setDate(monday.getDate() + 6));
        return { start: monday, end: sunday };
    }

    let pageSize = $state(25);

    let expandedMonths = $state<Set<string>>(new Set());

    let groupedTransactions = $derived.by(() => {
        const dayGroups: Record<string, any[]> = {};

        // 1. Filter and group transactions by day
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

        // 1.5 Sort transactions within each day by date descending (strict chronological order)
        // Tie-breaker: id descending
        Object.values(dayGroups || {}).forEach((group) => {
            group.sort((a, b) => {
                const dateSort = b.date.localeCompare(a.date);
                if (dateSort !== 0) return dateSort;
                return b.id.localeCompare(a.id);
            });
        });

        // 2. Aggregate into Months
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
                const monthKey = date.substring(0, 7); // YYYY-MM
                if (!months[monthKey]) {
                    months[monthKey] = {
                        monthKey,
                        days: [],
                        monthlyPnl: {},
                    };
                }

                // Calculate P&L for this day
                const pnl: Record<string, number> = {};
                dayGroups[date].forEach((tx) => {
                    const acc = settingsStore.accounts.find(
                        (a) => String(a.id) === String(tx.account_id),
                    );
                    const curr = acc?.currency || "BRL";
                    pnl[curr] = (pnl[curr] || 0) + tx.amount;

                    // Add to monthly P&L
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

    // Expansion state
    let expandedDays = $state<Set<string>>(new Set());

    // Auto-expand the first month and day
    $effect(() => {
        if (groupedTransactions.length > 0) {
            untrack(() => {
                if (expandedMonths.size === 0) {
                    expandedMonths = new Set([groupedTransactions[0].monthKey]);
                }
                if (
                    expandedDays.size === 0 &&
                    groupedTransactions[0].days.length > 0
                ) {
                    expandedDays = new Set([
                        groupedTransactions[0].days[0].date,
                    ]);
                }
            });
        }
    });

    function toggleMonth(month: string) {
        const newSet = new Set(expandedMonths);
        if (newSet.has(month)) newSet.delete(month);
        else newSet.add(month);
        expandedMonths = newSet;
    }

    function toggleDay(date: string) {
        const newSet = new Set(expandedDays);
        if (newSet.has(date)) {
            newSet.delete(date);
        } else {
            newSet.add(date);
        }
        expandedDays = newSet;
    }

    function formatCurrencyValue(val: number, currency: string) {
        try {
            return new Intl.NumberFormat($locale || "pt-BR", {
                style: "currency",
                currency: currency,
            }).format(val);
        } catch (e) {
            // Fallback for non-standard currencies like USDT
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
        console.log(
            "[StatementTable] opening details for tx:",
            tx.id,
            "trades:",
            tx.trade_ids,
        );
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
        const normalizedId = id.split(":").pop() || id;
        return tradesStore.trades.find((t) => {
            const tid = t.id.split(":").pop() || t.id;
            return tid === normalizedId;
        });
    }
</script>

<div class="space-y-4">
    <!-- Filters Header -->
    <div
        class="flex flex-col lg:flex-row gap-4 justify-between items-start lg:items-center bg-muted/30 p-4 rounded-xl border border-border/50"
    >
        <div class="flex flex-wrap gap-3 items-center w-full lg:w-auto">
            <div class="relative w-full md:w-64">
                <Search
                    class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground"
                />
                <Input
                    placeholder={$t("finance.statement.searchPlaceholder")}
                    class="pl-10 bg-background/50 border-border"
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
                    class="w-[180px] bg-background/50 border-border"
                >
                    <div class="flex items-center gap-2">
                        <Wallet class="w-3.5 h-3.5 text-muted-foreground" />
                        <span class="truncate">
                            {accountOptions.find(
                                (o) => o.value === accountFilter,
                            )?.label ?? $t("finance.statement.account")}
                        </span>
                    </div>
                </Select.Trigger>
                <Select.Content>
                    {#each accountOptions as opt}
                        <Select.Item value={opt.value}>{opt.label}</Select.Item>
                    {/each}
                </Select.Content>
            </Select.Root>

            <Select.Root type="single" bind:value={currencyFilter}>
                <Select.Trigger
                    class="w-[180px] bg-background/50 border-border"
                >
                    <div class="flex items-center gap-2">
                        <Coins class="w-3.5 h-3.5 text-muted-foreground" />
                        <span class="truncate">
                            {currencyOptions.find(
                                (o) => o.value === currencyFilter,
                            )?.label ?? $t("general.currency")}
                        </span>
                    </div>
                </Select.Trigger>
                <Select.Content>
                    {#each currencyOptions as opt}
                        <Select.Item value={opt.value}>{opt.label}</Select.Item>
                    {/each}
                </Select.Content>
            </Select.Root>
        </div>

        <div class="flex items-center gap-3 w-full lg:w-auto mt-4 lg:mt-0">
            <span
                class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest whitespace-nowrap"
                >{$t("general.show")}</span
            >
            <Select.Root
                type="single"
                value={pageSize.toString()}
                onValueChange={(v) => (pageSize = parseInt(v))}
            >
                <Select.Trigger
                    class="w-[100px] bg-background/50 border-border"
                >
                    <span class="truncate"
                        >{pageSize} {$t("general.time.months")}</span
                    >
                </Select.Trigger>
                <Select.Content>
                    <Select.Item value="1"
                        >1 {$t("general.time.month")}</Select.Item
                    >
                    <Select.Item value="3"
                        >3 {$t("general.time.months")}</Select.Item
                    >
                    <Select.Item value="6"
                        >6 {$t("general.time.months")}</Select.Item
                    >
                    <Select.Item value="12"
                        >1 {$t("general.time.year")}</Select.Item
                    >
                    <Select.Item value="24"
                        >2 {$t("general.time.years")}</Select.Item
                    >
                    <Select.Item value="100">{$t("general.all")}</Select.Item>
                </Select.Content>
            </Select.Root>
        </div>
    </div>

    <!-- Nested Monthly Table -->
    <div class="space-y-6">
        {#each groupedTransactions as { monthKey, days, monthlyPnl }}
            {@const isMonthExpanded = expandedMonths.has(monthKey)}
            <div class="space-y-3">
                <!-- Month Header -->
                <button
                    class="w-full flex items-center justify-between p-3 rounded-xl bg-primary/10 border border-primary/20 hover:bg-primary/15 transition-colors sticky top-0 z-10 backdrop-blur-md"
                    onclick={() => toggleMonth(monthKey)}
                >
                    <div class="flex items-center gap-3">
                        <div class="p-2 rounded-lg bg-primary/20">
                            <Calendar class="w-4 h-4 text-primary" />
                        </div>
                        <h4
                            class="text-sm font-black text-foreground uppercase tracking-tight"
                        >
                            {new Date(
                                monthKey + "-02T12:00:00",
                            ).toLocaleDateString($locale || "pt-BR", {
                                month: "long",
                                year: "numeric",
                            })}
                        </h4>
                    </div>

                    <div class="flex items-center gap-6">
                        <div class="flex gap-4">
                            {#each Object.entries(monthlyPnl) as [curr, total]}
                                <div class="flex flex-col items-end">
                                    <span
                                        class="text-[9px] font-bold text-muted-foreground uppercase tracking-widest"
                                        >{$t("general.balance")} {curr}</span
                                    >
                                    <span
                                        class="text-xs font-mono font-bold {total >=
                                        0
                                            ? 'text-emerald-500'
                                            : 'text-rose-500'}"
                                    >
                                        {formatCurrencyValue(total, curr)}
                                    </span>
                                </div>
                            {/each}
                        </div>
                        <div
                            class="w-6 h-6 rounded-full bg-primary/20 flex items-center justify-center transition-transform {isMonthExpanded
                                ? 'rotate-180'
                                : ''}"
                        >
                            <ChevronDown class="w-3 h-3 text-primary" />
                        </div>
                    </div>
                </button>

                {#if isMonthExpanded}
                    <div
                        class="pl-4 space-y-3 border-l-2 border-border/30 ml-6 animate-in slide-in-from-top-2 duration-300"
                    >
                        {#each days as { date, transactions, pnl }}
                            {@const isExpanded = expandedDays.has(date)}
                            <div
                                class="rounded-xl border border-border/50 bg-muted/20 overflow-hidden"
                            >
                                <button
                                    class="w-full flex items-center justify-between p-2 hover:bg-muted/40 transition-colors"
                                    onclick={() => toggleDay(date)}
                                >
                                    <div class="flex items-center gap-4">
                                        <div
                                            class="flex flex-col items-center justify-center bg-muted/80 rounded-lg h-9 w-9 border border-border/50"
                                        >
                                            <span
                                                class="text-[10px] font-black leading-none"
                                                >{new Date(date + "T12:00:00")
                                                    .toLocaleString(
                                                        $locale || "pt-BR",
                                                        {
                                                            weekday: "short",
                                                        },
                                                    )
                                                    .toUpperCase()}</span
                                            >
                                            <span
                                                class="text-sm font-black leading-none mt-0.5"
                                                >{new Date(
                                                    date + "T12:00:00",
                                                ).getDate()}</span
                                            >
                                        </div>
                                        <div
                                            class="flex flex-col items-start px-1"
                                        >
                                            <span
                                                class="text-xs font-bold text-foreground"
                                                >{$t(
                                                    "finance.dailyClosure",
                                                )}</span
                                            >
                                            <span
                                                class="text-[9px] font-medium text-muted-foreground uppercase tracking-widest"
                                                >{transactions.length}
                                                {$t(
                                                    "finance.statement.entries",
                                                )}</span
                                            >
                                        </div>
                                    </div>

                                    <div class="flex items-center gap-6">
                                        <div class="flex gap-4">
                                            {#each Object.entries(pnl) as [curr, total]}
                                                <div
                                                    class="flex items-center gap-2"
                                                >
                                                    <span
                                                        class="text-[10px] font-mono font-bold {(total as number) >=
                                                        0
                                                            ? 'text-emerald-500'
                                                            : 'text-rose-500'}"
                                                    >
                                                        {formatCurrencyValue(
                                                            total as number,
                                                            curr,
                                                        )}
                                                    </span>
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        class="h-5 w-5 hover:bg-accent/10"
                                                        onclick={(e) => {
                                                            e.stopPropagation();
                                                            openDayDetails(
                                                                date,
                                                                curr,
                                                            );
                                                        }}
                                                    >
                                                        <Eye
                                                            class="w-2.5 h-2.5 text-muted-foreground hover:text-foreground"
                                                        />
                                                    </Button>
                                                </div>
                                            {/each}
                                        </div>
                                        <div
                                            class="w-6 h-6 rounded-full bg-muted flex items-center justify-center transition-transform {isExpanded
                                                ? 'rotate-180'
                                                : ''}"
                                        >
                                            <ChevronDown
                                                class="w-3 h-3 text-muted-foreground"
                                            />
                                        </div>
                                    </div>
                                </button>

                                {#if isExpanded}
                                    <div class="border-t border-border/50">
                                        <Table.Root>
                                            <Table.Header class="bg-muted/20">
                                                <Table.Row
                                                    class="hover:bg-transparent border-border"
                                                >
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
                                                    <Table.Head class="w-[80px]"
                                                    ></Table.Head>
                                                </Table.Row>
                                            </Table.Header>
                                            <Table.Body>
                                                {#each transactions as tx}
                                                    {@const acc = getAccount(
                                                        tx.account_id,
                                                    )}
                                                    <Table.Row
                                                        class="border-border/50 hover:bg-muted/20"
                                                    >
                                                        <Table.Cell>
                                                            {#if tx.id && tx.id.includes("daily_closure_")}
                                                                <div
                                                                    class="flex items-center text-blue-400 text-[10px] font-bold uppercase tracking-tighter bg-blue-500/10 px-2 py-0.5 rounded border border-blue-500/20 w-fit"
                                                                >
                                                                    <CalendarCheck
                                                                        class="w-3 h-3 mr-1"
                                                                    />
                                                                    {$t(
                                                                        "finance.dailyClosure",
                                                                    )}
                                                                </div>
                                                            {:else if tx.type === "Deposit"}
                                                                <div
                                                                    class="flex items-center text-emerald-500 text-[10px] font-bold uppercase tracking-tighter bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20 w-fit"
                                                                >
                                                                    <ArrowDownLeft
                                                                        class="w-3 h-3 mr-1"
                                                                    />
                                                                    {$t(
                                                                        "finance.statement.types.deposit",
                                                                    )}
                                                                </div>
                                                            {:else if tx.type === "Withdraw"}
                                                                <div
                                                                    class="flex items-center text-red-500 text-[10px] font-bold uppercase tracking-tighter bg-red-500/10 px-2 py-0.5 rounded border border-red-500/20 w-fit"
                                                                >
                                                                    <ArrowUpRight
                                                                        class="w-3 h-3 mr-1"
                                                                    />
                                                                    {$t(
                                                                        "finance.statement.types.withdraw",
                                                                    )}
                                                                </div>
                                                            {:else if tx.type === "DailyResult"}
                                                                <div
                                                                    class="flex items-center text-muted-foreground text-[10px] font-mono font-bold uppercase tracking-tighter bg-muted/30 px-2 py-0.5 rounded border border-border/50 w-fit"
                                                                >
                                                                    <CalendarCheck
                                                                        class="w-3 h-3 mr-1"
                                                                    />
                                                                    {$t(
                                                                        "finance.statement.types.result",
                                                                    )}
                                                                </div>
                                                            {:else}
                                                                <div
                                                                    class="flex items-center text-blue-500 text-[10px] font-mono font-bold uppercase tracking-tighter bg-blue-500/10 px-2 py-0.5 rounded border border-blue-500/20 w-fit"
                                                                >
                                                                    <RefreshCw
                                                                        class="w-3 h-3 mr-1"
                                                                    />
                                                                    {$t(
                                                                        "finance.statement.types.result",
                                                                    )}
                                                                </div>
                                                            {/if}
                                                        </Table.Cell>
                                                        <Table.Cell
                                                            class="font-medium text-foreground"
                                                        >
                                                            {#if tx.id && tx.id.includes("daily_closure_")}
                                                                {$t(
                                                                    "finance.dailyClosure",
                                                                )} ({tx.trade_ids
                                                                    ? tx
                                                                          .trade_ids
                                                                          .length
                                                                    : 0}
                                                                {$t(
                                                                    "sidebar.trades",
                                                                ).toLowerCase()})
                                                            {:else}
                                                                {tx.description}
                                                            {/if}
                                                        </Table.Cell>
                                                        <Table.Cell
                                                            class="text-muted-foreground text-xs"
                                                        >
                                                            <div
                                                                class="flex items-center gap-1.5"
                                                            >
                                                                <span
                                                                    class="font-bold text-foreground/80"
                                                                    >{acc?.nickname ??
                                                                        "---"}</span
                                                                >
                                                                <span
                                                                    class="text-[10px] bg-muted px-1 rounded text-muted-foreground uppercase"
                                                                    >{acc?.currency ??
                                                                        ""}</span
                                                                >
                                                            </div>
                                                        </Table.Cell>
                                                        <Table.Cell
                                                            class="text-right font-mono font-bold {tx.amount >
                                                            0
                                                                ? 'text-emerald-500'
                                                                : 'text-rose-500'}"
                                                        >
                                                            {formatCurrencyValue(
                                                                tx.amount,
                                                                acc?.currency ||
                                                                    "BRL",
                                                            )}
                                                        </Table.Cell>
                                                        <Table.Cell>
                                                            <div
                                                                class="flex items-center justify-end gap-1"
                                                            >
                                                                {#if tx.trade_ids && tx.trade_ids.length > 0}
                                                                    <Button
                                                                        variant="ghost"
                                                                        size="icon"
                                                                        class="h-7 w-7 text-muted-foreground hover:text-foreground hover:bg-muted"
                                                                        onclick={() =>
                                                                            openDetails(
                                                                                tx,
                                                                            )}
                                                                    >
                                                                        <Eye
                                                                            class="h-3.5 w-3.5"
                                                                        />
                                                                    </Button>
                                                                {/if}

                                                                {#if tx.system_linked && (!tx.id || !tx.id.includes("daily_closure_"))}
                                                                    <Tooltip.Root
                                                                    >
                                                                        <Tooltip.Trigger
                                                                        >
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
                                                                        class="h-7 w-7 text-red-500/50 hover:text-red-500 hover:bg-red-500/10"
                                                                        onclick={() =>
                                                                            requestDelete(
                                                                                tx.id,
                                                                            )}
                                                                    >
                                                                        <Trash2
                                                                            class="h-3.5 w-3.5"
                                                                        />
                                                                    </Button>
                                                                {/if}
                                                            </div>
                                                        </Table.Cell>
                                                    </Table.Row>
                                                {/each}
                                            </Table.Body>
                                        </Table.Root>
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {:else}
            <div
                class="rounded-xl border border-border/50 bg-muted/20 p-12 text-center text-muted-foreground font-medium"
            >
                <div class="flex flex-col items-center gap-2">
                    <Search class="w-8 h-8 opacity-20" />
                    {$t("finance.statement.noTransactions")}
                </div>
            </div>
        {/each}
    </div>
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

<Dialog.Root bind:open={showDetailsDialog}>
    <Dialog.Content class="sm:max-w-[700px] glass border-border">
        <Dialog.Header>
            <Dialog.Title class="text-foreground"
                >{$t("finance.details.title")}</Dialog.Title
            >
        </Dialog.Header>
        {#if selectedTransaction}
            {@const acc = getAccount(selectedTransaction.account_id)}
            <div class="space-y-6 py-4">
                <div class="grid grid-cols-3 gap-6 text-sm">
                    <div class="space-y-1">
                        <span
                            class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest"
                            >{$t("finance.details.date")}</span
                        >
                        <div class="text-foreground/80 font-medium font-mono">
                            {new Date(
                                selectedTransaction.date.includes("T")
                                    ? selectedTransaction.date
                                    : selectedTransaction.date + "T12:00:00",
                            ).toLocaleDateString()}
                        </div>
                    </div>
                    <div class="space-y-1">
                        <span
                            class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest"
                            >{$t("finance.details.account")}</span
                        >
                        <div class="text-foreground/80 font-medium">
                            {acc?.nickname}
                        </div>
                    </div>
                    <div class="space-y-1">
                        <span
                            class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest"
                            >{$t("finance.transactionDialog.amount")} ({acc?.currency})</span
                        >
                        <div class="text-xl font-black text-foreground">
                            {formatCurrencyValue(
                                selectedTransaction.amount,
                                acc?.currency || "BRL",
                            )}
                        </div>
                    </div>
                </div>

                <div
                    class="border border-border rounded-xl overflow-hidden bg-muted/40"
                >
                    <Table.Root>
                        <Table.Header class="bg-muted/50">
                            <Table.Row
                                class="hover:bg-transparent border-border"
                            >
                                <Table.Head
                                    class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest"
                                    >{$t(
                                        "finance.details.columns.asset",
                                    )}</Table.Head
                                >
                                <Table.Head
                                    class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest"
                                    >{$t(
                                        "finance.details.columns.side",
                                    )}</Table.Head
                                >
                                <Table.Head
                                    class="text-right text-[10px] font-bold text-muted-foreground uppercase tracking-widest"
                                    >{$t(
                                        "finance.details.columns.result",
                                    )}</Table.Head
                                >
                            </Table.Row>
                        </Table.Header>
                        <Table.Body>
                            {#if selectedTransaction.trade_ids}
                                {#each selectedTransaction.trade_ids as tradeId}
                                    {@const trade = findTradeById(tradeId)}
                                    {#if trade}
                                        <Table.Row
                                            class="border-border/50 hover:bg-muted/20"
                                        >
                                            <Table.Cell
                                                class="font-bold text-foreground/80"
                                                >{trade.asset_symbol}</Table.Cell
                                            >
                                            <Table.Cell>
                                                <Badge
                                                    variant="outline"
                                                    class={cn(
                                                        "text-[10px] font-bold",
                                                        trade.direction ===
                                                            "Buy"
                                                            ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20"
                                                            : "bg-red-500/10 text-red-500 border-red-500/20",
                                                    )}
                                                >
                                                    {trade.direction.toUpperCase()}
                                                </Badge>
                                            </Table.Cell>
                                            <Table.Cell
                                                class="text-right font-mono font-bold {trade.result >=
                                                0
                                                    ? 'text-emerald-500'
                                                    : 'text-rose-500'}"
                                            >
                                                {formatCurrencyValue(
                                                    trade.result,
                                                    acc?.currency || "BRL",
                                                )}
                                            </Table.Cell>
                                        </Table.Row>
                                    {/if}
                                {/each}
                            {:else}
                                <Table.Row>
                                    <Table.Cell
                                        colspan={3}
                                        class="text-center h-12 text-muted-foreground"
                                    >
                                        {$t("finance.details.noTrades")}
                                    </Table.Cell>
                                </Table.Row>
                            {/if}
                        </Table.Body>
                    </Table.Root>
                </div>
            </div>
        {/if}
    </Dialog.Content>
</Dialog.Root>

<DarfDetailsDialog
    bind:open={isDarfDetailOpen}
    transactionId={selectedDarfTxId}
/>
