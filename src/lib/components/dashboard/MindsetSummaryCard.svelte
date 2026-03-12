<script lang="ts">
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import * as Card from "$lib/components/ui/card";
    import { Brain, Quote, Smile, Frown, Meh, Sparkles } from "lucide-svelte";
    import { cn } from "$lib/utils";
    import { t } from "svelte-i18n";

    const lastEntry = $derived(settingsStore.journalEntries[0]);
    const lastTrade = $derived(
        [...tradesStore.trades]
            .sort(
                (a, b) =>
                    new Date(b.date).getTime() - new Date(a.date).getTime(),
            )
            .find((t) => t.entry_emotional_state_id),
    );

    const currentState = $derived.by(() => {
        const stateId =
            lastEntry?.emotional_state_id ||
            lastTrade?.entry_emotional_state_id;
        if (!stateId) return null;
        return settingsStore.emotionalStates.find((s) => s.id === stateId);
    });

    const impactColors = {
        Positive: "text-emerald-500 bg-emerald-500/10",
        Negative: "text-rose-500 bg-rose-500/10",
        Neutral: "text-blue-500 bg-blue-500/10",
    };

    const impactIcons = {
        Positive: Smile,
        Negative: Frown,
        Neutral: Meh,
    };
</script>

<Card.Root class="border-l-2 border-l-indigo-500 shadow-sm bg-card h-full">
    <Card.Header
        class="flex flex-row items-center justify-between space-y-0 py-1.5 px-3"
    >
        <Card.Title
            class="text-[9px] font-black uppercase tracking-wider text-muted-foreground/60"
            >{$t("dashboard.mindset.title")}</Card.Title
        >
        <Brain class="h-3 w-3 text-indigo-500" />
    </Card.Header>
    <Card.Content class="py-1.5 px-3 space-y-2.5">
        {#if currentState}
            {@const ImpactIcon = impactIcons[currentState.impact]}
            <div class="flex items-center gap-3">
                <div
                    class={cn(
                        "p-2 rounded-lg",
                        impactColors[currentState.impact],
                    )}
                >
                    <ImpactIcon class="w-4 h-4" />
                </div>
                <div>
                    <h4
                        class="text-[11px] font-black text-foreground leading-none"
                    >
                        {currentState.name}
                    </h4>
                    <p
                        class="text-[10px] text-muted-foreground uppercase font-medium line-clamp-1"
                    >
                        {$t("dashboard.mindset.impact_detected", { values: { impact: currentState.impact } })}
                    </p>
                </div>
            </div>
        {:else}
            <div
                class="flex items-center gap-3 text-muted-foreground italic text-xs"
            >
                <Sparkles class="w-4 h-4 opacity-50" />
                {$t("dashboard.mindset.empty")}
            </div>
        {/if}

        <div
            class="bg-muted/30 py-0.5 px-2 rounded-lg border border-border/50 relative"
        >
            <Quote
                class="w-3 h-3 text-muted-foreground/30 absolute top-2 left-2"
            />
            <p
                class="text-xs text-muted-foreground leading-relaxed italic line-clamp-3 pl-4"
            >
                {lastEntry?.content || $t("dashboard.mindset.placeholder")}
            </p>
        </div>

        {#if lastEntry?.followed_plan !== undefined}
            <div class="flex gap-2">
                <div
                    class={cn(
                        "px-2 py-0.5 rounded text-[8px] font-bold uppercase tracking-widest",
                        lastEntry.followed_plan
                            ? "bg-emerald-500/10 text-emerald-500"
                            : "bg-rose-500/10 text-rose-500",
                    )}
                >
                    {lastEntry.followed_plan
                        ? $t("dashboard.mindset.followed_plan")
                        : $t("dashboard.mindset.deviated_plan")}
                </div>
            </div>
        {/if}
    </Card.Content>
</Card.Root>
