<script lang="ts">
    import { t } from "svelte-i18n";
    import { onMount } from "svelte";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import { Label } from "$lib/components/ui/label";
    import { CheckCircle2 } from "lucide-svelte";

    interface ChecklistItem {
        id: string;
        label: string;
        checked: boolean;
    }

    let items = $state<ChecklistItem[]>([
        {
            id: "sleep",
            label: $t("trades.wizard.checklist.sleep"),
            checked: false,
        },
        {
            id: "calm",
            label: $t("trades.wizard.checklist.calm"),
            checked: false,
        },
        {
            id: "plan",
            label: $t("trades.wizard.checklist.plan"),
            checked: false,
        },
        {
            id: "distraction",
            label: $t("trades.wizard.checklist.distraction"),
            checked: false,
        },
        {
            id: "risk",
            label: $t("trades.wizard.checklist.risk"),
            checked: false,
        },
    ]);

    const STORAGE_KEY = "traderlog_daily_checklist_v1";

    onMount(() => {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved) {
            try {
                const parsed = JSON.parse(saved);
                if (Array.isArray(parsed)) {
                    // Update labels from translations while keeping checked state
                    items = items.map((item) => {
                        const savedItem = parsed.find(
                            (p: any) => p.id === item.id,
                        );
                        return {
                            ...item,
                            checked: savedItem?.checked || false,
                        };
                    });
                }
            } catch (e) {
                console.error("Error loading checklist", e);
            }
        }
    });

    function handleCheck(id: string, checked: boolean) {
        items = items.map((i) => (i.id === id ? { ...i, checked } : i));
        localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
    }

    let allChecked = $derived(items.every((i) => i.checked));
</script>

<div class="space-y-3">
    <div class="grid gap-2">
        {#each items as item}
            <div
                class="flex items-center gap-3 p-3 rounded-lg border transition-all duration-300
                {item.checked
                    ? 'bg-primary/10 border-primary/20 opacity-60'
                    : 'bg-muted/30 border-muted hover:border-primary/20'}"
            >
                <Checkbox
                    id={item.id}
                    checked={item.checked}
                    onCheckedChange={(v) => handleCheck(item.id, v as boolean)}
                    class="data-[state=checked]:bg-primary data-[state=checked]:border-primary"
                />
                <Label
                    for={item.id}
                    class="text-xs font-bold uppercase tracking-wider cursor-pointer flex-1 {item.checked
                        ? 'text-primary/70 line-through'
                        : 'text-foreground'}"
                >
                    {item.label}
                </Label>
            </div>
        {/each}
    </div>

    {#if allChecked}
        <div
            class="mt-4 p-4 bg-primary/10 border border-primary/30 rounded-xl flex items-center gap-3 animate-in zoom-in-95 duration-500"
        >
            <div
                class="w-8 h-8 rounded-full bg-primary/20 flex items-center justify-center text-primary"
            >
                <CheckCircle2 class="w-5 h-5" />
            </div>
            <div>
                <p
                    class="text-[10px] font-bold uppercase tracking-tighter text-primary"
                >
                    {$t("trades.wizard.checklist.ideal_condition")}
                </p>
                <p class="text-xs font-bold text-foreground italic">
                    {$t("trades.wizard.checklist.ready")}
                </p>
            </div>
        </div>
    {/if}
</div>
