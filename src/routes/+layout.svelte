<script lang="ts">
  import "../app.css";
  import { ModeWatcher, setMode, mode } from "mode-watcher";
  import { TooltipProvider } from "$lib/components/ui/tooltip";
  import { setupI18n } from "$lib/i18n";
  import { isLoading } from "svelte-i18n";
  import { Toaster } from "$lib/components/ui/sonner";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { tradesStore } from "$lib/stores/trades.svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import StartupSplash from "$lib/components/ui/branding/StartupSplash.svelte";
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  let isI18nReady = $state(false);
  let isBypassLoading = $state(false);
  let isSplashFinished = $state(false);
  let isDetached = $state(false);

  // Check if window is detached (secondary) on mount
  onMount(() => {
    isDetached = window.location.pathname.includes("detached-trade");
    if (isDetached) {
      isSplashFinished = true; // Skip splash for secondary windows
    }
  });

  // Cross-window sync: reload trades when another window (detached) saves
  let _unlistenTradeSaved: (() => void) | null = null;
  // Ctrl+scroll zoom handler reference for cleanup
  let _handleWheel: ((e: WheelEvent) => void) | null = null;

  onMount(async () => {
    console.log("[Root Layout] Starting initial data load sequence...");
    try {
      console.log("[Root Layout] Step 1: Loading setupI18n...");
      await setupI18n();
      isI18nReady = true;
      console.log("[Root Layout] Step 1 complete: i18n ready.");

      console.log("[Root Layout] Step 2: Loading settingsStore data...");
      await settingsStore.loadData();
      console.log("[Root Layout] Step 2 complete: Settings loaded.");

      console.log("[Root Layout] Step 3: Loading tradesStore data...");
      await tradesStore.loadTrades();
      console.log("[Root Layout] Step 3 complete: Trades loaded.");

      console.log("[Root Layout] All initial data load complete.");
    } catch (e) {
      console.error("[Root Layout] FATAL Error during initial load:", e);
      // Fallback: force bypass loading to at least show something
      isBypassLoading = true;
      settingsStore.isLoadingData = false;
    }

    // Fire-and-forget: listen for trade-saved from detached windows
    listen("trade-saved", () => {
      tradesStore.loadTrades();
    }).then((fn) => {
      _unlistenTradeSaved = fn;
    });

    // Ctrl+scroll wheel zoom
    let currentZoom = 1.0;
    _handleWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      currentZoom += e.deltaY < 0 ? 0.05 : -0.05;
      currentZoom = Math.min(Math.max(currentZoom, 0.5), 2.0);
      (document.documentElement as HTMLElement).style.zoom =
        String(currentZoom);
    };
    window.addEventListener("wheel", _handleWheel, { passive: false });
  });

  onDestroy(() => {
    _unlistenTradeSaved?.();
    if (_handleWheel) window.removeEventListener("wheel", _handleWheel);
  });

  // Auth Guard: Force login if password is set and user is not authenticated
  $effect(() => {
    if (settingsStore.isLoadingData) return;

    const isAuthPage =
      $page.url.pathname === "/login" || $page.url.pathname === "/signup";
    const hasPassword = !!settingsStore.userProfile.password_hash;

    if (!settingsStore.isLoggedIn && hasPassword && !isAuthPage) {
      console.log("[Layout] Auth Guard: Redirecting to login");
      goto("/login");
    }
  });

  // Integrated Theme Sync (One source of truth: Store -> UI)
  $effect(() => {
    // 1. Check for theme override in URL (for detached windows)
    if (typeof window !== "undefined") {
      const urlParams = new URLSearchParams(window.location.search);
      const urlTheme = urlParams.get("theme");
      if (urlTheme && urlTheme !== mode.current) {
        console.log(`[Layout] Theme override from URL: ${urlTheme}`);
        setMode(urlTheme as any);
        return; // Prioritize URL param on initial load
      }
    }

    // 2. Otherwise sync from settingsStore
    // We only sync if theme is explicitly set (not empty string)
    let desiredTheme = settingsStore.userProfile.theme;

    if (desiredTheme && desiredTheme !== mode.current) {
      console.log(
        `[Layout] Syncing theme: ${desiredTheme} (current: ${mode.current})`,
      );
      setMode(desiredTheme as any);
    }
  });

  // Panic bypass for fixed loading screen (5s threshold)
  $effect(() => {
    const timer = setTimeout(() => {
      if ($isLoading || settingsStore.isLoadingData) {
        console.warn(
          "[Layout] Initial loading taking too long, forcing bypass.",
        );
        isBypassLoading = true;

        // Ensure store isn't stuck in loading state (prevent lockouts)
        if (settingsStore.isLoadingData) {
          settingsStore.isLoadingData = false;
        }
      }
    }, 5000);
    return () => clearTimeout(timer);
  });

  let { children } = $props();
</script>

<ModeWatcher />

{#if isI18nReady}
  {#if !isSplashFinished}
    <StartupSplash onFinish={() => (isSplashFinished = true)} />
  {:else if ($isLoading || settingsStore.isLoadingData) && !isBypassLoading}
    <div
      class="flex items-center justify-center h-screen w-full bg-background border-t-4 border-primary"
    >
      <div class="flex flex-col items-center gap-4">
        <div
          class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin"
        ></div>
        <p class="text-muted-foreground font-medium animate-pulse">
          Carregando PsicoTrade Pro...
        </p>
      </div>
    </div>
  {:else}
    <TooltipProvider>
      {@render children()}
      <Toaster />
    </TooltipProvider>
  {/if}
{/if}
