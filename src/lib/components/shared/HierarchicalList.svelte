<script lang="ts">
    import { locale } from "svelte-i18n";
    import { ChevronDown, TrendingUp, Calendar } from "lucide-svelte";
    import * as Separator from "$lib/components/ui/separator";
    import { untrack } from "svelte";

    // ---- Types -------------------------------------------------------
    type PnlEntry = { curr: string; val: number };

    interface HierarchicalDay {
        key: string;
        date: string;
        label: string;
        trades: any[];
        pnlEntries?: PnlEntry[];
        meta?: any;
        [key: string]: any;
    }

    interface HierarchicalWeek {
        key: string;
        label: string;
        days: HierarchicalDay[];
        pnlEntries?: PnlEntry[];
        meta?: any;
        [key: string]: any;
    }

    interface HierarchicalMonth {
        key: string;
        label: string;
        weeks: HierarchicalWeek[];
        trades: any[];
        pnlEntries?: PnlEntry[];
        meta?: any;
        [key: string]: any;
    }

    // ---- Props -------------------------------------------------------
    let {
        data = [],
        // Snippet for the right-hand side of the month header
        monthRight = undefined,
        // Snippet for the right-hand side of the week header
        weekRight = undefined,
        // Snippet for the right-hand side of the day card header
        dayRight = undefined,
        // Snippet for the expanded content inside the day card
        dayContent,
        // Snippet override for default PnL display in month (optional)
        monthBadges = undefined,
        // Snippet override for default PnL display in week (optional)
        weekBadges = undefined,
        // Snippet override for default PnL display in day (optional)
        dayBadges = undefined,
        // Callback fired when a month is toggled, useful for syncing external context
        onMonthToggle = (key: string, expanded: boolean) => {},
        onWeekToggle = (key: string, expanded: boolean) => {},
        onDayToggle = (key: string, expanded: boolean) => {},
        // When true, renders days directly under month (no week level) — for Month→Day hierarchies
        flatMode = false,
        // When true, skips rendering the individual days list explicitly, instead yielding a monthContent snippet directly inside the expanded month
        omitDays = false,
        // Content to render directly inside the month when omitDays is true
        monthContent = undefined,
    // When true, automatically expands the first available month (and week) on mount or data change if nothing is expanded yet
    autoExpandDefault = false,
        expandedMonths = $bindable(new Set<string>()),
        expandedWeeks = $bindable(new Set<string>()),
        expandedDays = $bindable(new Set<string>()),
        // Behavior
        mutualExclusion = true,
    }: {
        data: HierarchicalMonth[];
        monthRight?: import("svelte").Snippet<[HierarchicalMonth]>;
        weekRight?: import("svelte").Snippet<[HierarchicalWeek]>;
        dayRight?: import("svelte").Snippet<[HierarchicalDay]>;
        dayContent?: import("svelte").Snippet<[HierarchicalDay]>;
        monthContent?: import("svelte").Snippet<[HierarchicalMonth]>;
        monthBadges?: import("svelte").Snippet<[HierarchicalMonth]>;
        weekBadges?: import("svelte").Snippet<[HierarchicalWeek]>;
        dayBadges?: import("svelte").Snippet<[HierarchicalDay]>;
        onMonthToggle?: (key: string, expanded: boolean) => void;
        onWeekToggle?: (key: string, expanded: boolean) => void;
        onDayToggle?: (key: string, expanded: boolean) => void;
        flatMode?: boolean;
        omitDays?: boolean;
        autoExpandDefault?: boolean;
        expandedMonths?: Set<string>;
        expandedWeeks?: Set<string>;
        expandedDays?: Set<string>;
        mutualExclusion?: boolean;
    } = $props();

    // ---- Auto Expand Logic -------------------------------------------
    let hasAutoExpanded = $state(false);

    $effect(() => {
        if (autoExpandDefault && data.length > 0 && !hasAutoExpanded) {
            untrack(() => {
                if (expandedMonths.size === 0) {
                    const firstMonth = data[0];
                    if (firstMonth) {
                        expandedMonths = new Set([firstMonth.key]);
                        // If not omitting days, also expand the first week of that month
                        if (!omitDays && !flatMode && firstMonth.weeks && firstMonth.weeks.length > 0) {
                            expandedWeeks = new Set([firstMonth.weeks[0].key]);
                        }
                    }
                }
                hasAutoExpanded = true;
            });
        }
    });

    // Reset auto-expand when data changes completely
    $effect(() => {
        if (data && data.length > 0) {
           // We keep this empty just to trigger tracking on data, the logic is above
        }
    });

    // ---- Toggle helpers ----------------------------------------------
    function toggleMonth(key: string) {
        const next = new Set(expandedMonths);
        if (next.has(key)) {
            next.delete(key);
        } else {
            if (mutualExclusion) {
                next.clear();
                // When switching month in mutual exclusion, we must reassign to trigger reactivity in Svelte 5
                expandedWeeks = new Set();
                expandedDays = new Set();
            }
            next.add(key);
        }
        expandedMonths = next;
        if (onMonthToggle) onMonthToggle(key, next.has(key));
    }

    function toggleWeek(key: string) {
        const next = new Set(expandedWeeks);
        if (next.has(key)) {
            next.delete(key);
        } else {
            if (mutualExclusion) {
                next.clear();
                expandedDays = new Set();
            }
            next.add(key);
        }
        expandedWeeks = next;
        if (onWeekToggle) onWeekToggle(key, next.has(key));
    }

    function toggleDay(key: string) {
        const next = new Set(expandedDays);
        if (next.has(key)) {
            next.delete(key);
        } else {
            if (mutualExclusion) {
                next.clear();
            }
            next.add(key);
        }
        expandedDays = next;
        if (onDayToggle) onDayToggle(key, next.has(key));
    }

    // ---- Helpers -----------------------------------------------------
    function weekdayShort(dateStr: string) {
        return new Date(dateStr + "T12:00:00")
            .toLocaleString($locale || "pt-BR", { weekday: "short" })
            .toUpperCase();
    }

    function dayNumber(dateStr: string) {
        return new Date(dateStr + "T12:00:00").getDate();
    }
</script>

<div class="space-y-4">
    {#each data as month (month.key)}
        {@const isMonthExpanded = expandedMonths.has(month.key)}
        <div class="space-y-3">
            <!-- ═══ Month Header ════════════════════════════════════════ -->
            <button
                class="w-full flex items-center justify-between p-3 rounded-xl card-glass border-primary/20 hover:bg-primary/15 transition-all duration-300 sticky top-0 z-10 backdrop-blur-md"
                onclick={() => toggleMonth(month.key)}
            >
                <div
                    class="flex items-center gap-4 animate-in fade-in slide-in-from-bottom-2 duration-300"
                >
                    <div class="p-2 rounded-lg bg-primary/20">
                        <Calendar class="w-4 h-4 text-primary" />
                    </div>
                    <div class="text-left">
                        <h4
                            class="text-sm font-black text-foreground uppercase tracking-tight"
                        >
                            {month.label}
                        </h4>
                        <div class="flex items-center gap-2 mt-0.5">
                            <!-- Default PnL badges (overrideable) -->
                            {#if monthBadges}
                                {@render monthBadges(month)}
                            {:else if month.pnlEntries?.length}
                                <div class="flex gap-2">
                                    {#each month.pnlEntries as entry}
                                        <div
                                            class="flex items-baseline gap-0.5"
                                        >
                                            <span
                                                class="text-[8px] text-muted-foreground uppercase"
                                                >{entry.curr}</span
                                            >
                                            <span
                                                class="text-[9px] font-mono font-bold {entry.val >=
                                                0
                                                    ? 'text-emerald-500'
                                                    : 'text-rose-500'}"
                                            >
                                                {entry.val.toFixed(2)}
                                            </span>
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    </div>
                </div>

                <!-- RHS: custom content (score, eye, etc.) + chevron -->
                <div class="flex items-center gap-3">
                    {#if monthRight}
                        {@render monthRight(month)}
                    {/if}
                    <div
                        class="w-7 h-7 rounded-full bg-primary/10 flex items-center justify-center transition-transform duration-300 {isMonthExpanded
                            ? 'rotate-180'
                            : ''}"
                    >
                        <ChevronDown class="w-4 h-4 text-primary" />
                    </div>
                </div>
            </button>

            {#if isMonthExpanded}
                <div
                    class="pl-4 pb-4 space-y-3 border-l-2 border-border/30 ml-6 animate-in fade-in slide-in-from-top-2 duration-300"
                >
                    <Separator.Root class="bg-border/20 mb-3" />

                    {#if omitDays}
                        <div class="px-2 pb-2">
                            {#if monthContent}
                                {@render monthContent(month)}
                            {/if}
                        </div>
                    {:else if flatMode}
                        <!-- Flat mode: days directly under month -->
                        <div
                            class="pl-4 pb-2 space-y-3 border-l-2 border-border/30 ml-6"
                        >
                            {#each month.days ?? [] as day (day.key)}
                                {@const isDayExpanded = expandedDays.has(
                                    day.key,
                                )}
                                <div
                                    class="rounded-xl border border-border/50 card-glass overflow-hidden transition-all duration-300 hover:border-primary/30"
                                >
                                    <!-- Day Card Header -->
                                    <button
                                        class="w-full flex items-center justify-between p-2 hover:bg-primary/10 transition-all duration-300 border-none bg-transparent cursor-pointer"
                                        onclick={() => toggleDay(day.key)}
                                    >
                                        <div
                                            class="flex items-center gap-3 animate-in fade-in slide-in-from-bottom-2 duration-300"
                                        >
                                            <!-- Date badge -->
                                            <div
                                                class="flex flex-col items-center justify-center bg-muted/80 rounded-lg h-9 w-9 border border-border/50 shadow-inner"
                                            >
                                                <span
                                                    class="text-[10px] font-black leading-none text-muted-foreground/80"
                                                >
                                                    {weekdayShort(day.date)}
                                                </span>
                                                <span
                                                    class="text-sm font-black leading-none mt-0.5 text-foreground"
                                                >
                                                    {dayNumber(day.date)}
                                                </span>
                                            </div>

                                            <!-- Day info -->
                                            <div class="text-left">
                                                <span
                                                    class="text-[9px] font-bold text-muted-foreground uppercase tracking-tight"
                                                >
                                                    {day.label}
                                                </span>
                                                <div
                                                    class="flex items-center gap-2 mt-0.5"
                                                >
                                                    <span
                                                        class="text-[8px] text-muted-foreground/60 font-medium"
                                                    >
                                                        ({day.trades?.length ||
                                                            1} entradas)
                                                    </span>

                                                    <!-- Default day PnL (overrideable) -->
                                                    {#if dayBadges}
                                                        {@render dayBadges(day)}
                                                    {:else if day.pnlEntries?.length}
                                                        {#each day.pnlEntries as entry}
                                                            <div
                                                                class="flex items-baseline gap-0.5"
                                                            >
                                                                <span
                                                                    class="text-[7px] text-muted-foreground uppercase"
                                                                    >{entry.curr}</span
                                                                >
                                                                <span
                                                                    class="text-[8px] font-mono font-bold {entry.val >=
                                                                    0
                                                                        ? 'text-emerald-500'
                                                                        : 'text-rose-500'}"
                                                                >
                                                                    {entry.val.toFixed(
                                                                        2,
                                                                    )}
                                                                </span>
                                                            </div>
                                                        {/each}
                                                    {/if}
                                                </div>
                                            </div>
                                        </div>

                                        <!-- RHS content + chevron -->
                                        <div class="flex items-center gap-3">
                                            {#if dayRight}
                                                {@render dayRight(day)}
                                            {/if}
                                            <div
                                                class="w-6 h-6 rounded-full bg-muted flex items-center justify-center transition-transform duration-300 {isDayExpanded
                                                    ? 'rotate-180'
                                                    : ''}"
                                            >
                                                <ChevronDown
                                                    class="w-3 h-3 text-muted-foreground"
                                                />
                                            </div>
                                        </div>
                                    </button>

                                    <!-- Day Content (expanded) -->
                                    {#if isDayExpanded}
                                        <div
                                            class="border-t border-border/50 animate-in fade-in slide-in-from-top-2 duration-300"
                                        >
                                            {#if dayContent}
                                                {@render dayContent(day)}
                                            {/if}
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    {:else}
                        {#each month.weeks as week (week.key)}
                            {@const isWeekExpanded = expandedWeeks.has(
                                week.key,
                            )}
                            <div class="space-y-3 relative">
                                <!-- ═══ Week Divider (Clickable) ══════════════════════ -->
                                <button
                                    class="w-full flex items-center gap-2 mb-2 ml-4 p-1.5 rounded-lg hover:bg-muted/50 transition-colors animate-in fade-in slide-in-from-top-1 duration-300 cursor-pointer text-left"
                                    onclick={() => toggleWeek(week.key)}
                                >
                                    <div
                                        class="p-1 rounded bg-muted border border-border/40"
                                    >
                                        <TrendingUp
                                            class="w-3 h-3 text-primary"
                                        />
                                    </div>
                                    <span
                                        class="text-[10px] font-black uppercase tracking-widest text-muted-foreground whitespace-nowrap"
                                    >
                                        {week.label}
                                    </span>
                                    <div
                                        class="h-[1px] flex-1 bg-border/20 mx-2"
                                    ></div>

                                    <!-- Default week PnL (overrideable) -->
                                    <div class="flex items-center gap-2">
                                        {#if weekBadges}
                                            {@render weekBadges(week)}
                                        {:else if week.pnlEntries?.length}
                                            <div class="flex gap-2">
                                                {#each week.pnlEntries as entry}
                                                    <div
                                                        class="flex items-baseline gap-0.5"
                                                    >
                                                        <span
                                                            class="text-[7px] text-muted-foreground uppercase"
                                                            >{entry.curr}</span
                                                        >
                                                        <span
                                                            class="text-[9px] font-mono font-bold {entry.val >=
                                                            0
                                                                ? 'text-emerald-500'
                                                                : 'text-rose-500'}"
                                                        >
                                                            {entry.val.toFixed(
                                                                2,
                                                            )}
                                                        </span>
                                                    </div>
                                                {/each}
                                            </div>
                                        {/if}

                                        {#if weekRight}
                                            {@render weekRight(week)}
                                        {/if}

                                        <div
                                            class="w-5 h-5 rounded-full bg-muted flex items-center justify-center transition-transform duration-300 ml-1 {isWeekExpanded
                                                ? 'rotate-180'
                                                : ''}"
                                        >
                                            <ChevronDown
                                                class="w-3 h-3 text-muted-foreground"
                                            />
                                        </div>
                                    </div>
                                </button>

                                <!-- ═══ Days ════════════════════════════════ -->
                                {#if isWeekExpanded}
                                    <div
                                        class="pl-4 pb-2 space-y-3 border-l-2 border-border/30 ml-6 animate-in fade-in slide-in-from-top-2 duration-300"
                                    >
                                        {#each week.days as day (day.key)}
                                            {@const isDayExpanded =
                                                expandedDays.has(day.key)}
                                            <div
                                                class="rounded-xl border border-border/50 card-glass overflow-hidden transition-all duration-300 hover:border-primary/30"
                                            >
                                                <!-- Day Card Header -->
                                                <button
                                                    class="w-full flex items-center justify-between p-2 hover:bg-primary/10 transition-all duration-300 border-none bg-transparent cursor-pointer"
                                                    onclick={() =>
                                                        toggleDay(day.key)}
                                                >
                                                    <div
                                                        class="flex items-center gap-3 animate-in fade-in slide-in-from-bottom-2 duration-300"
                                                    >
                                                        <!-- Date badge -->
                                                        <div
                                                            class="flex flex-col items-center justify-center bg-muted/80 rounded-lg h-9 w-9 border border-border/50 shadow-inner"
                                                        >
                                                            <span
                                                                class="text-[10px] font-black leading-none text-muted-foreground/80"
                                                            >
                                                                {weekdayShort(
                                                                    day.date,
                                                                )}
                                                            </span>
                                                            <span
                                                                class="text-sm font-black leading-none mt-0.5 text-foreground"
                                                            >
                                                                {dayNumber(
                                                                    day.date,
                                                                )}
                                                            </span>
                                                        </div>

                                                        <!-- Day info -->
                                                        <div class="text-left">
                                                            <span
                                                                class="text-[9px] font-bold text-muted-foreground uppercase tracking-tight"
                                                            >
                                                                {day.label}
                                                            </span>
                                                            <div
                                                                class="flex items-center gap-2 mt-0.5"
                                                            >
                                                                <span
                                                                    class="text-[8px] text-muted-foreground/60 font-medium"
                                                                >
                                                                    ({day.trades
                                                                        .length}
                                                                    trades)
                                                                </span>

                                                                <!-- Default day PnL (overrideable) -->
                                                                {#if dayBadges}
                                                                    {@render dayBadges(
                                                                        day,
                                                                    )}
                                                                {:else if day.pnlEntries?.length}
                                                                    {#each day.pnlEntries as entry}
                                                                        <div
                                                                            class="flex items-baseline gap-0.5"
                                                                        >
                                                                            <span
                                                                                class="text-[7px] text-muted-foreground uppercase"
                                                                                >{entry.curr}</span
                                                                            >
                                                                            <span
                                                                                class="text-[8px] font-mono font-bold {entry.val >=
                                                                                0
                                                                                    ? 'text-emerald-500'
                                                                                    : 'text-rose-500'}"
                                                                            >
                                                                                {entry.val.toFixed(
                                                                                    2,
                                                                                )}
                                                                            </span>
                                                                        </div>
                                                                    {/each}
                                                                {/if}
                                                            </div>
                                                        </div>
                                                    </div>

                                                    <!-- RHS content + chevron -->
                                                    <div
                                                        class="flex items-center gap-3"
                                                    >
                                                        {#if dayRight}
                                                            {@render dayRight(
                                                                day,
                                                            )}
                                                        {/if}
                                                        <div
                                                            class="w-6 h-6 rounded-full bg-muted flex items-center justify-center transition-transform duration-300 {isDayExpanded
                                                                ? 'rotate-180'
                                                                : ''}"
                                                        >
                                                            <ChevronDown
                                                                class="w-3 h-3 text-muted-foreground"
                                                            />
                                                        </div>
                                                    </div>
                                                </button>

                                                <!-- Day Content (expanded) -->
                                                {#if isDayExpanded}
                                                    <div
                                                        class="border-t border-border/50 animate-in fade-in slide-in-from-top-2 duration-300"
                                                    >
                                                        {#if dayContent}
                                                            {@render dayContent(
                                                                day,
                                                            )}
                                                        {/if}
                                                    </div>
                                                {/if}
                                            </div>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    {/if}
                </div>
            {/if}
        </div>
    {/each}
</div>
