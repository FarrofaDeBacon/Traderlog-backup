<script lang="ts">
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
        CardDescription,
    } from "$lib/components/ui/card";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import { Label } from "$lib/components/ui/label";
    import { Progress } from "$lib/components/ui/progress";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import {
        ClipboardCheck,
        TrendingUp,
        AlertTriangle,
        PlayCircle,
        CalendarDays,
        RefreshCw,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import { onMount } from "svelte";
    import { marketMock } from "$lib/services/marketMock";
    import type { EconomicEvent } from "$lib/services/llmService";
    import { diaryService } from "$lib/services/diaryService";

    // Item definition
    interface ChecklistItem {
        id: string;
        label: string;
        checked: boolean;
        type?: "boolean" | "input" | "info";
        inputValue?: string; // For input types
        conditional?: boolean; // If true, only shows under specific conditions
        meta?: any; // To store extra data like news events
    }

    let itemsData: ChecklistItem[] = $state([
        {
            id: "news",
            label: "",
            checked: false,
            conditional: true,
            type: "info",
        },
        { id: "trend_20ma", label: "trend_20ma", checked: false },
        { id: "trend_200ma", label: "trend_200ma", checked: false },
        { id: "gap_analysis", label: "gap_analysis", checked: false },
        { id: "prior_levels", label: "prior_levels", checked: false },
        {
            id: "risk_limit",
            label: "risk_limit",
            checked: false,
            type: "input",
            inputValue: "",
        },
        { id: "mindset", label: "mindset", checked: false },
    ]);

    let loading = $state(true);
    let newsLoading = $state(false);
    let showWidget = $state(false); // Controls iframe visibility separately from data

    async function fetchNews() {
        newsLoading = true;
        try {
            const news = await marketMock.checkEconomicCalendar();
            itemsData = itemsData.map((item) => {
                if (item.id === "news") {
                    if (news.length > 0) {
                        return {
                            ...item,
                            conditional: false,
                            meta: news, // Stores the ACTUAL news array
                        };
                    } else {
                        // If no news, hide the item but keep it checked as "done"
                        return {
                            ...item,
                            conditional: true,
                            checked: true,
                            meta: [],
                        };
                    }
                }
                return item;
            });
            toast.success("Notícias atualizadas via IA!");
        } catch (e) {
            toast.error("Erro ao buscar notícias: " + (e as Error).message);
        } finally {
            newsLoading = false;
        }
    }

    // Load/Save Logic
    const TARGET_DATE = new Date().toISOString().split("T")[0]; // YYYY-MM-DD
    // We still use a temp key for instant-save while editing, but we ALSO commit to the diaryService
    const TEMP_STORAGE_KEY = `traderlog:checklist_temp:${TARGET_DATE}`;

    onMount(async () => {
        // 1. Try to load from "Diary Entry" (Official Record) first
        const officialEntry = diaryService.getEntry(TARGET_DATE);

        let savedState: any[] = [];

        if (officialEntry) {
            savedState = officialEntry.items;
        } else {
            // Fallback to temp storage if no official entry yet
            const temp = localStorage.getItem(TEMP_STORAGE_KEY);
            if (temp)
                try {
                    savedState = JSON.parse(temp);
                } catch {}
        }

        if (savedState.length > 0) {
            itemsData = itemsData.map((item) => {
                const found = savedState.find((p: any) => p.id === item.id);
                if (found) {
                    return {
                        ...item,
                        checked: found.checked,
                        inputValue: found.inputValue ?? "",
                        // Preserve meta if loading from storage, but ensure we don't lose news structure
                        meta:
                            item.id === "news" && !found.meta
                                ? item.meta
                                : found.meta,
                    };
                }
                return item;
            });
        }

        // 2. Initial fetch
        await fetchNews();

        loading = false;
    });

    // Auto-save to Service (which handles the "Index" for future filtering)
    $effect(() => {
        if (loading) return;

        // Save to temp (fast access)
        localStorage.setItem(TEMP_STORAGE_KEY, JSON.stringify(itemsData));

        // Save to Persistent Diary Service
        diaryService.saveEntry(TARGET_DATE, {
            items: itemsData,
            news: itemsData.find((i) => i.id === "news")?.meta || [],
            riskLimit:
                itemsData.find((i) => i.id === "risk_limit")?.inputValue || "",
        });
    });

    // Derived
    let visibleItems = $derived(itemsData.filter((i) => !i.conditional));
    let checkedCount = $derived(visibleItems.filter((i) => i.checked).length);
    let totalCount = $derived(visibleItems.length);
    let progress = $derived(
        totalCount === 0 ? 100 : (checkedCount / totalCount) * 100,
    );
    let readyToTrade = $derived(progress === 100);

    function resetChecklist() {
        itemsData = itemsData.map((i) => {
            if (i.id === "news" && i.conditional) return i;
            return { ...i, checked: false, inputValue: "" };
        });
        showWidget = false;
        toast.info("Checklist resetado para hoje.");
    }

    function startTrading() {
        if (!readyToTrade) return;
        toast.success($t("strategy.dashboard.checklist.success"));
    }
</script>

<Card class="w-full">
    <CardHeader class="py-1 px-3 border-b border-border/50">
        <div class="flex items-center justify-between">
            <div class="space-y-1">
                <CardTitle
                    class="text-[11px] font-black uppercase tracking-wider"
                >
                    <ClipboardCheck class="w-4 h-4 text-primary" />
                    {$t("strategy.dashboard.checklist.title")}
                </CardTitle>
                <CardDescription
                    class="text-[8px] mt-0 opacity-60 leading-none"
                >
                    {$t("strategy.dashboard.checklist.subtitle")}
                </CardDescription>
            </div>
            <div class="flex items-center gap-3">
                <span
                    class="text-xs text-muted-foreground bg-muted px-2 py-1 rounded"
                >
                    {TARGET_DATE}
                </span>
                {#if readyToTrade}
                    <div
                        class="px-3 py-1 bg-green-100 text-green-700 rounded-full text-xs font-bold animate-pulse dark:bg-green-900/30 dark:text-green-400"
                    >
                        {$t("strategy.dashboard.checklist.ready")}
                    </div>
                {/if}
            </div>
        </div>
    </CardHeader>
    <CardContent class="p-2 space-y-2">
        <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
            <!-- Coluna Esquerda: Checklist (Ocupa 3 colunas se Widget ativo, ou toda a largura se inativo) -->
            <div
                class="{showWidget
                    ? 'lg:col-span-3'
                    : 'lg:col-span-4'} space-y-2 transition-all duration-300"
            >
                <!-- Progress Bar -->
                <div class="space-y-2">
                    <div class="flex justify-between text-xs">
                        <span class="text-muted-foreground">
                            {checkedCount}
                            {$t("general.of")}
                            {totalCount}
                            {$t("strategy.dashboard.checklist.verified")}
                        </span>
                        <span
                            class="font-bold {progress === 100
                                ? 'text-green-600'
                                : 'text-blue-600'}">{progress.toFixed(0)}%</span
                        >
                    </div>
                    <Progress value={progress} class="h-2" />
                </div>

                <!-- Checklist Items -->
                <div
                    class="grid gap-2 {showWidget
                        ? 'md:grid-cols-1'
                        : 'md:grid-cols-2'}"
                >
                    {#each visibleItems as item}
                        <div
                            class="flex flex-col py-0.5 px-2 rounded-lg border hover:bg-muted/50 transition-colors {item.checked
                                ? 'bg-muted/20'
                                : ''}"
                        >
                            <div class="flex items-start space-x-3 w-full">
                                <Checkbox
                                    id={item.id}
                                    bind:checked={item.checked}
                                    class="mt-1"
                                />
                                <div class="flex-1 space-y-2">
                                    <Label
                                        for={item.id}
                                        class="text-sm font-medium leading-none cursor-pointer w-full {item.checked
                                            ? 'text-muted-foreground line-through'
                                            : ''}"
                                    >
                                        {#if item.id === "news"}
                                            <div
                                                class="flex flex-col gap-2 w-full"
                                            >
                                                <div
                                                    class="flex items-center justify-between w-full"
                                                >
                                                    <span>
                                                        {$t(
                                                            "strategy.dashboard.checklist.items.news",
                                                        )}
                                                    </span>
                                                    <!-- Botão de Refresh -->
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        class="h-7 text-xs gap-2 ml-2 hover:bg-primary/10 hover:text-primary"
                                                        onclick={(e) => {
                                                            e.stopPropagation();
                                                            fetchNews();
                                                        }}
                                                        disabled={newsLoading}
                                                    >
                                                        {#if newsLoading}
                                                            <RefreshCw
                                                                class="w-3 h-3 animate-spin"
                                                            />
                                                        {:else}
                                                            <RefreshCw
                                                                class="w-3 h-3"
                                                            />
                                                        {/if}
                                                        {$t(
                                                            "strategy.dashboard.checklist.newsUpdate",
                                                        )}
                                                    </Button>
                                                </div>
                                            </div>
                                        {:else}
                                            {$t(
                                                `strategy.dashboard.checklist.items.${item.label}`,
                                            )}
                                        {/if}
                                    </Label>

                                    <!-- ALERTA DA IA: Se houver notícias de alto impacto no meta -->
                                    {#if item.id === "news" && item.meta && Array.isArray(item.meta) && item.meta.length > 0}
                                        <div
                                            class="mt-2 text-xs bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-900 rounded p-2 animate-in slide-in-from-top-1"
                                        >
                                            <div
                                                class="font-bold text-amber-700 dark:text-amber-400 flex items-center gap-1 mb-1"
                                            >
                                                <AlertTriangle
                                                    class="w-3 h-3"
                                                />
                                                {$t(
                                                    "strategy.dashboard.checklist.volatilityAlert",
                                                )}
                                                <span
                                                    class="text-[10px] font-normal text-muted-foreground ml-auto"
                                                    >{$t(
                                                        "strategy.dashboard.checklist.scrollMore",
                                                    )}</span
                                                >
                                            </div>
                                            <ul
                                                class="divide-y divide-amber-200/50 dark:divide-amber-800/50 max-h-[160px] overflow-y-auto pr-1"
                                            >
                                                {#each item.meta as event}
                                                    <li
                                                        class="flex items-center gap-2 py-1 hover:bg-amber-100/50 dark:hover:bg-amber-900/30 transition-colors px-1 rounded-sm"
                                                    >
                                                        <span
                                                            class="font-mono font-bold text-amber-600 whitespace-nowrap text-[10px]"
                                                            >{event.time}</span
                                                        >

                                                        {#if event.importance === "high" || event.importance === "3"}
                                                            <span
                                                                class="text-[9px] bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-100 px-1 rounded border border-red-200 dark:border-red-800 whitespace-nowrap"
                                                                >HIGH</span
                                                            >
                                                        {/if}

                                                        <span
                                                            class="font-bold whitespace-nowrap text-[10px]"
                                                            >{event.currency}</span
                                                        >
                                                        <span
                                                            class="truncate flex-1 min-w-0"
                                                            title={event.event}
                                                            >{event.event}</span
                                                        >
                                                    </li>
                                                {/each}
                                            </ul>
                                        </div>
                                    {/if}

                                    <!-- Special Render: Risk Input -->
                                    {#if item.id === "risk_limit"}
                                        <div
                                            class="flex items-center gap-2 mt-1"
                                        >
                                            <span
                                                class="text-xs text-muted-foreground"
                                                >{$t(
                                                    "strategy.dashboard.checklist.items.riskInput",
                                                )}</span
                                            >
                                            <Input
                                                type="number"
                                                placeholder="Ex: 500"
                                                class="h-7 w-24 text-xs"
                                                bind:value={item.inputValue}
                                                disabled={item.checked}
                                            />
                                        </div>
                                    {/if}
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>

            <!-- Coluna Direita: Calendário (Apenas visível se ativado) -->
            {#if showWidget}
                <div
                    class="lg:col-span-1 h-full min-h-[500px] border-l pl-0 lg:pl-6 animate-in slide-in-from-right-10 fade-in duration-500"
                >
                    <div
                        class="flex items-center gap-2 mb-3 text-sm font-semibold text-muted-foreground"
                    >
                        <CalendarDays class="w-4 h-4" />
                        {$t("strategy.dashboard.checklist.economicCalendar")}
                    </div>

                    <div
                        class="w-full h-full rounded-md overflow-hidden bg-white dark:bg-black border shadow-inner min-h-[500px]"
                    >
                        <iframe
                            src="https://sslecal2.forexprostools.com/?columns=exc_flags,exc_currency,exc_importance,exc_actual,exc_forecast,exc_previous&features=datepicker,timezone&countries=110,17,37,12,25,32,6,36,5,72&calType=day&timeZone=12&lang=12"
                            width="100%"
                            height="100%"
                            title={$t(
                                "strategy.dashboard.checklist.calendarTitle",
                            )}
                            frameborder="0"
                            allowtransparency={null}
                            marginwidth="0"
                            marginheight="0"
                            class="dark:invert dark:hue-rotate-180 dark:brightness-90 w-full h-full min-h-[500px]"
                        >
                        </iframe>
                    </div>
                </div>
            {/if}
        </div>

        <!-- Action -->
        {#if readyToTrade}
            <div class="pt-4 animate-in slide-in-from-bottom-2">
                <Button
                    class="w-full bg-green-600 hover:bg-green-700 text-white"
                    onclick={startTrading}
                >
                    <PlayCircle class="w-4 h-4 mr-2" />
                    {$t("strategy.dashboard.checklist.start")}
                </Button>
            </div>
        {:else}
            <div class="pt-4 flex justify-end">
                <Button variant="ghost" size="sm" onclick={resetChecklist}>
                    {$t("strategy.dashboard.checklist.reset")}
                </Button>
            </div>
        {/if}
    </CardContent>
</Card>
