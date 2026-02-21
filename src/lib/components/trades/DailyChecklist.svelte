<script lang="ts">
    import { onMount } from "svelte";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import { Label } from "$lib/components/ui/label";
    import { Brain, CheckCircle2 } from "lucide-svelte";

    interface ChecklistItem {
        id: string;
        label: string;
        checked: boolean;
    }

    let items = $state<ChecklistItem[]>([
        { id: "sleep", label: "Dormi pelo menos 7h?", checked: false },
        { id: "calm", label: "Estado emocional equilibrado?", checked: false },
        { id: "plan", label: "Cenários de trade definidos?", checked: false },
        {
            id: "distraction",
            label: "Foco total / Sem distrações?",
            checked: false,
        },
        { id: "risk", label: "Aceito o stop do dia?", checked: false },
    ]);

    const STORAGE_KEY = "traderlog_daily_checklist_v1";

    onMount(() => {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved) {
            try {
                const parsed = JSON.parse(saved);
                if (Array.isArray(parsed)) items = parsed;
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
                    Condição Ideal
                </p>
                <p class="text-xs font-bold text-foreground italic">
                    PRONTO PARA O MERCADO!
                </p>
            </div>
        </div>
    {/if}
</div>
