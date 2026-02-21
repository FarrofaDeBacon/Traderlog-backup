<script lang="ts">
    import { cn } from "$lib/utils";

    let {
        value = $bindable(0),
        max = 100,
        min = 0,
        step = 1,
        class: className = undefined,
    } = $props();
</script>

<input
    type="range"
    bind:value
    {min}
    {max}
    {step}
    class={cn(
        "w-full h-2 bg-secondary rounded-lg appearance-none cursor-pointer accent-primary focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        className,
    )}
/>
