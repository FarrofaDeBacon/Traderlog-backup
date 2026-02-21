<script lang="ts">
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";

    onMount(() => {
        goto("/settings/fiscal/rules");
    });
</script>

<div class="flex items-center justify-center h-[50vh]">
    <span class="loading loading-spinner loading-lg"></span>
</div>
