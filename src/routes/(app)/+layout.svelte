<script lang="ts">
    import { onMount } from "svelte";
    import AppSidebar from "$lib/components/app-sidebar/AppSidebar.svelte";
    import { sidebarState } from "$lib/stores/sidebar.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import OnboardingWizard from "$lib/components/setup/OnboardingWizard.svelte";
    import LicenseBanner from "$lib/components/layout/LicenseBanner.svelte";
    import AutoTradeDetectionDialog from "$lib/components/trades/AutoTradeDetectionDialog.svelte";
    import { invoke } from "@tauri-apps/api/core";

    let { children } = $props();
    let dataLoaded = $state(false);

    onMount(async () => {
        console.log("[Layout] Initializing stores and dumping diagnostics...");
        
        // --- DIAGNOSTICS ---
        invoke("diagnostic_dump_trades").catch(e => console.error(e));
        invoke("diagnostic_dump_users").catch(e => console.error(e));
        // -------------------

        await Promise.all([settingsStore.loadData(), tradesStore.loadTrades()]);
        dataLoaded = true;
        console.log(
            "[Layout] Settings store loaded. Onboarding completed:",
            settingsStore.userProfile.onboarding_completed,
        );
    });

    $inspect(settingsStore.userProfile.onboarding_completed).with(
        (type, val) => {
            console.log(`[Layout] Onboarding record state (${type}):`, val);
        },
    );

    async function handleOnboardingComplete() {
        await Promise.all([settingsStore.loadData(), tradesStore.loadTrades()]);
    }
</script>

{#if dataLoaded && !settingsStore.userProfile.onboarding_completed}
    <OnboardingWizard onComplete={handleOnboardingComplete} />
{/if}

<div
    class="grid min-h-screen w-full transition-all duration-300 bg-muted/40 md:grid-cols-[var(--sidebar-width)_1fr] {dataLoaded &&
    !settingsStore.userProfile.onboarding_completed
        ? ''
        : ''}"
    style="--sidebar-width: {sidebarState.isCollapsed ? '70px' : '280px'};"
>
    <AppSidebar />
    <div class="flex flex-col min-w-0">
        <LicenseBanner />
        <AutoTradeDetectionDialog />
        <main class="flex flex-1 flex-col gap-4 p-4 lg:gap-6 lg:p-6 min-w-0">
            {@render children()}
        </main>
    </div>
</div>
