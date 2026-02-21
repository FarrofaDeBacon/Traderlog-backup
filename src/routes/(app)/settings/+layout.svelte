<script lang="ts">
    import { Separator } from "$lib/components/ui/separator";
    import SettingsSidebar from "./SettingsSidebar.svelte";

    let { children } = $props();
</script>

<div class="hidden space-y-6 p-10 pb-16 md:block">
    <div class="space-y-0.5">
        <h2 class="text-2xl font-bold tracking-tight">Configurações</h2>
        <p class="text-muted-foreground">
            Gerencie suas preferências de conta e do aplicativo.
        </p>
    </div>
    <Separator class="my-6" />
    <div class="flex flex-col space-y-8 lg:flex-row lg:space-x-12 lg:space-y-0">
        <aside class="lg:w-64 overflow-visible shrink-0">
            <SettingsSidebar />
        </aside>
        <div class="flex-1 lg:max-w-4xl">
            {@render children()}
        </div>
    </div>
</div>
