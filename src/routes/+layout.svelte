<script lang="ts">
  import "../app.css";
  import { ModeWatcher, setMode, mode } from "mode-watcher";
  import { TooltipProvider } from "$lib/components/ui/tooltip";
  import { setupI18n } from "$lib/i18n";
  import { isLoading } from "svelte-i18n";
  import { Toaster } from "$lib/components/ui/sonner";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import StartupSplash from "$lib/components/ui/branding/StartupSplash.svelte";

  let isI18nReady = $state(false);

  setupI18n().then(() => {
    isI18nReady = true;
  });

  let isBypassLoading = $state(false);
  let isSplashFinished = $state(false);

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
    if (settingsStore.isLoadingData) return;

    const desiredTheme = settingsStore.userProfile.theme;
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
      if ($isLoading) {
        console.warn("[Layout] i18n taking too long, forcing bypass.");
        isBypassLoading = true;
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
  {:else if $isLoading && !isBypassLoading}
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
