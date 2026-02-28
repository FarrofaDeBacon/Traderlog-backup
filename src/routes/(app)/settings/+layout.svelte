<script lang="ts">
    import { t } from "svelte-i18n";
    import { page } from "$app/stores";
    import { Separator } from "$lib/components/ui/separator";
    import SettingsSidebar from "./SettingsSidebar.svelte";

    let { children } = $props();

    let section = $derived(
        $page.url.pathname.includes("/fiscal/rules")
            ? "fiscal.rules"
            : $page.url.pathname.includes("/fiscal/profiles")
              ? "fiscal.profiles"
              : $page.url.pathname.includes("/fiscal/assignments")
                ? "fiscal.assignments"
                : $page.url.pathname.split("/").pop() || "general",
    );

    let navKey = $derived(
        section === "settings"
            ? "general"
            : section === "asset-types"
              ? "assetTypes"
              : section === "emotional-states"
                ? "emotionalStates"
                : section === "chart-types"
                  ? "chartTypes"
                  : section === "api-integrations"
                    ? "integrations"
                    : section,
    );
</script>

<div class="hidden space-y-6 p-10 pb-16 md:block">
    <div class="space-y-0.5">
        <h2 class="text-2xl font-bold tracking-tight capitalize">
            {$t(`settings.nav.${navKey}`)}
        </h2>
        <p class="text-muted-foreground">
            {$t(`settings.nav.descriptions.${navKey}`)}
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
