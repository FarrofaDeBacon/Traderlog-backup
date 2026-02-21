<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { CalendarIcon } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { cn } from "$lib/utils";

    let {
        value = $bindable("today"),
        startDate = $bindable(""),
        endDate = $bindable(""),
    } = $props();

    const presets = [
        { value: "today", label: "filters.date.today" },
        { value: "yesterday", label: "filters.date.yesterday" },
        { value: "this_week", label: "filters.date.this_week" },
        { value: "last_week", label: "filters.date.last_week" },
        { value: "this_month", label: "filters.date.this_month" },
        { value: "last_month", label: "filters.date.last_month" },
        { value: "this_year", label: "filters.date.this_year" },
        { value: "last_year", label: "filters.date.last_year" },
        { value: "all", label: "filters.date.all" },
        { value: "custom", label: "filters.date.custom" },
    ];

    const triggerContent = $derived(
        presets.find((p) => p.value === value)?.label ?? "filters.date.select",
    );
</script>

<div class="flex items-center gap-2">
    <Select.Root type="single" bind:value>
        <Select.Trigger class="w-[180px]">
            <CalendarIcon class="mr-2 h-4 w-4" />
            {$t(triggerContent)}
        </Select.Trigger>
        <Select.Content>
            {#each presets as preset}
                <Select.Item value={preset.value}
                    >{$t(preset.label)}</Select.Item
                >
            {/each}
        </Select.Content>
    </Select.Root>

    {#if value === "custom"}
        <div
            class="flex items-center gap-2 animate-in fade-in slide-in-from-left-5"
        >
            <Input type="date" bind:value={startDate} class="w-[140px]" />
            <span class="text-muted-foreground">-</span>
            <Input type="date" bind:value={endDate} class="w-[140px]" />
        </div>
    {/if}
</div>
