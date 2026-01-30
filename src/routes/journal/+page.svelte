<script lang="ts">
    import { Button } from "$lib/components/ui/button";
</script>

<div class="flex items-center justify-between">
    <h1 class="text-lg font-semibold md:text-2xl">Diário de Trading</h1>
    <Button>Nova Nota</Button>
</div>

<div
    class="mt-4 flex flex-1 items-center justify-center rounded-lg border border-dashed shadow-sm h-[400px]"
>
    <div class="flex flex-col items-center gap-1 text-center">
        <h3 class="text-2xl font-bold tracking-tight">Sem notas ainda</h3>
        <p class="text-sm text-muted-foreground">
            Comece a registrar seu mindset hoje.
        </p>
    </div>
</div>
