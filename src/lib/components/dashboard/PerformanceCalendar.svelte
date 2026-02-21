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
    import { ptBR } from "date-fns/locale";
    import {
        ChevronLeft,
        ChevronRight,
        Calendar as CalendarIcon,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { cn } from "$lib/utils";
    import { goto } from "$app/navigation";

    let { trades = [], onFilterChange } = $props<{
        trades: any[];
        onFilterChange?: (start: Date, end: Date) => void;
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
        const dateStr = format(day, "yyyy-MM-dd");
        goto(`/journal?date=${dateStr}`);
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

    const weekDayLabels = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"];
</script>

<div class="space-y-4">
    <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-2">
            <span
                class="text-sm font-bold text-foreground uppercase tracking-tighter"
            >
                {format(
                    currentDate,
                    viewMode === "month"
                        ? "MMMM yyyy"
                        : "'Semana de' dd 'de' MMMM",
                    { locale: ptBR },
                )}
            </span>
        </div>
        <div
            class="flex items-center gap-1 bg-muted/50 p-1 rounded-lg border border-border"
        >
            <Button
                variant={viewMode === "month" ? "secondary" : "ghost"}
                size="sm"
                class="h-7 text-[10px] uppercase font-bold px-3 transition-all"
                onclick={() => (viewMode = "month")}
            >
                Mensal
            </Button>
            <Button
                variant={viewMode === "week" ? "secondary" : "ghost"}
                size="sm"
                class="h-7 text-[10px] uppercase font-bold px-3 transition-all"
                onclick={() => (viewMode = "week")}
            >
                Semanal
            </Button>
            <div class="w-px h-4 bg-border mx-1"></div>
            <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 text-muted-foreground hover:text-foreground"
                onclick={prev}
            >
                <ChevronLeft class="w-4 h-4" />
            </Button>
            <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 text-muted-foreground hover:text-foreground"
                onclick={next}
            >
                <ChevronRight class="w-4 h-4" />
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
                        "min-h-[85px] p-3 border-r border-b border-border group transition-all relative flex flex-col text-left",
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
                                ? "bg-emerald-600 text-white px-2 py-0.5 rounded-full"
                                : "text-muted-foreground group-hover:text-foreground",
                        )}
                    >
                        {format(day, "dd")}
                    </span>

                    {#if pnl !== 0}
                        <div class="mt-auto pt-2 flex flex-col">
                            <span
                                class={cn(
                                    "text-[11px] font-black tracking-tight",
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
                            <div
                                class={cn(
                                    "w-full h-1 rounded-full mt-1.5 opacity-20",
                                    pnl > 0 ? "bg-emerald-500" : "bg-rose-500",
                                )}
                                style="width: {Math.min(
                                    Math.abs(pnl) / 10,
                                    100,
                                )}%"
                            ></div>
                        </div>
                    {/if}
                </button>
            {/each}
        </div>
    </div>
</div>
