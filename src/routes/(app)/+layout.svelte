<script lang="ts">
    import { onMount } from "svelte";
    import AppSidebar from "$lib/components/app-sidebar/AppSidebar.svelte";
    import { sidebarState } from "$lib/stores/sidebar.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import OnboardingWizard from "$lib/components/setup/OnboardingWizard.svelte";
    import LicenseBanner from "$lib/components/layout/LicenseBanner.svelte";
    import AutoTradeDetectionDialog from "$lib/components/trades/AutoTradeDetectionDialog.svelte";
    import { ScrollArea } from "$lib/components/ui/scroll-area";
    import { invoke } from "@tauri-apps/api/core";

    let { children } = $props();
    let dataLoaded = $state(false);

    onMount(async () => {
        console.log("[Layout] App layout mounted, data handled by root.");

        // --- DIAGNOSTICS ---
        invoke("diagnostic_dump_trades").catch((e) => console.error(e));
        invoke("diagnostic_dump_users").catch((e) => console.error(e));
        // -------------------

        dataLoaded = true;
    });

    $inspect(settingsStore.userProfile.onboarding_completed).with(
        (type, val) => {
            console.log(`[Layout] Onboarding record state (${type}):`, val);
        },
    );

    async function handleOnboardingComplete() {
        console.log("[Layout] Onboarding complete, refreshing data...");
        await Promise.all([settingsStore.loadData(), tradesStore.loadTrades()]);
    }
</script>

{#if dataLoaded}
    {#if !settingsStore.userProfile.onboarding_completed}
        <OnboardingWizard onComplete={handleOnboardingComplete} />
    {:else}
        <div
            class="grid min-h-screen w-full transition-all duration-300 bg-background md:grid-cols-[var(--sidebar-width)_1fr]"
            style="--sidebar-width: {sidebarState.isCollapsed
                ? '70px'
                : '280px'};"
        >
            <AppSidebar />
            <div class="flex flex-col min-w-0">
                <LicenseBanner />
                <AutoTradeDetectionDialog />
                <main
                    class="flex flex-1 flex-col gap-4 p-4 lg:gap-6 lg:p-6 min-w-0"
                >
                    {@render children()}
                </main>
            </div>
        </div>
    {/if}
{/if}
