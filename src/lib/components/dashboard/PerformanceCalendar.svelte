<script lang="ts">
    import {
        format,
        startOfMonth,
        endOfMonth,
        startOfWeek,
        endOfWeek,
        eachDayOfInterval,
        isSameMonth,
        isSameDay,
        addMonths,
        subMonths,
        addWeeks,
        subWeeks,
    } from "date-fns";
    import { ptBR, enUS, es, fr } from "date-fns/locale";
    import {
        ChevronLeft,
        ChevronRight,
        Calendar as CalendarIcon,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { cn } from "$lib/utils";
    import { goto } from "$app/navigation";
    import { t, locale } from "svelte-i18n";

    let {
        trades = [],
        onFilterChange,
        onDateClick,
        compact = false,
    } = $props<{
        trades: any[];
        onFilterChange?: (start: Date, end: Date) => void;
        onDateClick?: (day: Date) => void;
        compact?: boolean;
    }>();

    let viewMode = $state<"month" | "week">("month");
    let currentDate = $state(new Date());

    const days = $derived.by(() => {
        const start =
            viewMode === "month"
                ? startOfWeek(startOfMonth(currentDate))
                : startOfWeek(currentDate);
        const end =
            viewMode === "month"
                ? endOfWeek(endOfMonth(currentDate))
                : endOfWeek(currentDate);
        return eachDayOfInterval({ start, end });
    });

    const pnlByDay = $derived.by(() => {
        const map = new Map<string, number>();
        trades.forEach((t: import("$lib/stores/settings.svelte").Trade) => {
            const day = format(new Date(t.date), "yyyy-MM-dd");
            map.set(day, (map.get(day) || 0) + (t.result || 0));
        });
        return map;
    });

    function next() {
        currentDate =
            viewMode === "month"
                ? addMonths(currentDate, 1)
                : addWeeks(currentDate, 1);
    }

    function prev() {
        currentDate =
            viewMode === "month"
                ? subMonths(currentDate, 1)
                : subWeeks(currentDate, 1);
    }

    function handleDateClick(day: Date) {
        if (onDateClick) {
            onDateClick(day);
        } else {
            const dateStr = format(day, "yyyy-MM-dd");
            goto(`/journal?date=${dateStr}`);
        }
    }

    $effect(() => {
        if (onFilterChange) {
            const start =
                viewMode === "month"
                    ? startOfMonth(currentDate)
                    : startOfWeek(currentDate);
            const end =
                viewMode === "month"
                    ? endOfMonth(currentDate)
                    : endOfWeek(currentDate);
            onFilterChange(start, end);
        }
    });

    const weekDayLabels = $derived([
        $t("calendar.days.sun"),
        $t("calendar.days.mon"),
        $t("calendar.days.tue"),
        $t("calendar.days.wed"),
        $t("calendar.days.thu"),
        $t("calendar.days.fri"),
        $t("calendar.days.sat"),
    ]);
    const dateLocale = $derived.by(() => {
        const currentLocale = $locale;
        if (currentLocale?.startsWith("en")) return enUS;
        if (currentLocale?.startsWith("es")) return es;
        if (currentLocale?.startsWith("fr")) return fr;
        return ptBR;
    });
</script>

<div class={cn("flex flex-col", compact ? "gap-2" : "gap-4")}>
    <div
        class={cn(
            "flex items-center justify-between",
            compact ? "mb-1" : "mb-4",
        )}
    >
        <div class="flex items-center gap-2">
            <span
                class={cn(
                    "font-bold text-foreground uppercase tracking-widest",
                    compact ? "text-[10px]" : "text-sm",
                )}
            >
                {viewMode === "month"
                    ? format(currentDate, "MMMM yyyy", { locale: dateLocale })
                    : `${$t("calendar.weekOf")} ${format(currentDate, "dd", { locale: dateLocale })} ${$t("calendar.of")} ${format(currentDate, "MMMM", { locale: dateLocale })}`}
            </span>
        </div>
        <div
            class="flex items-center gap-1 bg-muted/50 p-1 rounded-lg border border-border"
        >
            <Button
                variant={viewMode === "month" ? "secondary" : "ghost"}
                size="sm"
                class={cn(
                    "font-bold px-3 transition-all",
                    compact ? "h-6 text-[8px]" : "h-7 text-[10px]",
                )}
                onclick={() => (viewMode = "month")}
            >
                {compact ? $t("calendar.monthlyShort") : $t("calendar.monthly")}
            </Button>
            <Button
                variant={viewMode === "week" ? "secondary" : "ghost"}
                size="sm"
                class={cn(
                    "font-bold px-3 transition-all",
                    compact ? "h-6 text-[8px]" : "h-7 text-[10px]",
                )}
                onclick={() => (viewMode = "week")}
            >
                {compact ? $t("calendar.weeklyShort") : $t("calendar.weekly")}
            </Button>
            <div class="w-px h-4 bg-border mx-1"></div>
            <Button
                variant="ghost"
                size="icon"
                class={cn(
                    "text-muted-foreground hover:text-foreground",
                    compact ? "h-6 w-6" : "h-7 w-7",
                )}
                onclick={prev}
            >
                <ChevronLeft class={compact ? "w-3 h-3" : "w-4 h-4"} />
            </Button>
            <Button
                variant="ghost"
                size="icon"
                class={cn(
                    "text-muted-foreground hover:text-foreground",
                    compact ? "h-6 w-6" : "h-7 w-7",
                )}
                onclick={next}
            >
                <ChevronRight class={compact ? "w-3 h-3" : "w-4 h-4"} />
            </Button>
        </div>
    </div>

    <div
        class="rounded-xl border border-border overflow-hidden bg-card shadow-sm"
    >
        <div class="grid grid-cols-7 bg-muted/20 border-b border-border">
            {#each weekDayLabels as label}
                <div
                    class="py-3 text-center text-[9px] font-black uppercase text-muted-foreground tracking-widest"
                >
                    {label}
                </div>
            {/each}
        </div>
        <div class="grid grid-cols-7 auto-rows-fr">
            {#each days as day, i}
                {@const dateStr = format(day, "yyyy-MM-dd")}
                {@const pnl = pnlByDay.get(dateStr) || 0}
                {@const isCurrentMonth = isSameMonth(day, currentDate)}
                {@const isToday = isSameDay(day, new Date())}

                <button
                    class={cn(
                        compact ? "min-h-[55px] p-1.5" : "min-h-[85px] p-3",
                        "border-r border-b border-border group transition-all relative flex flex-col text-left",
                        !isCurrentMonth &&
                            viewMode === "month" &&
                            "bg-muted/10 opacity-30",
                        i % 7 === 6 && "border-r-0",
                        "hover:bg-muted/40 cursor-pointer active:scale-[0.98]",
                    )}
                    onclick={() => handleDateClick(day)}
                >
                    <span
                        class={cn(
                            "text-[10px] font-black transition-all",
                            isToday
                                ? "bg-emerald-600 text-white px-1.5 py-0.5 rounded-full"
                                : "text-muted-foreground group-hover:text-foreground",
                        )}
                    >
                        {format(day, "dd")}
                    </span>

                    {#if pnl !== 0}
                        <div class="mt-auto pt-1 flex flex-col">
                            <span
                                class={cn(
                                    compact ? "text-[9px]" : "text-[11px]",
                                    "font-black tracking-tight",
                                    pnl > 0
                                        ? "text-emerald-600"
                                        : "text-rose-600",
                                )}
                            >
                                {pnl > 0 ? "+" : ""}{pnl.toLocaleString(
                                    "pt-BR",
                                    { style: "currency", currency: "BRL" },
                                )}
                            </span>
                            {#if !compact}
                                <div
                                    class={cn(
                                        "w-full h-1 rounded-full mt-1.5 opacity-20",
                                        pnl > 0
                                            ? "bg-emerald-500"
                                            : "bg-rose-500",
                                    )}
                                    style="width: {Math.min(
                                        Math.abs(pnl) / 10,
                                        100,
                                    )}%"
                                ></div>
                            {/if}
                        </div>
                    {/if}
                </button>
            {/each}
        </div>
    </div>
</div>
