<script lang="ts">
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  import { TooltipProvider } from "$lib/components/ui/tooltip";
  import { setupI18n } from "$lib/i18n";
  import { isLoading } from "svelte-i18n";
  import { Toaster } from "$lib/components/ui/sonner";

  setupI18n();

  let { children } = $props();
</script>

<ModeWatcher />

{#if $isLoading}
  <div class="flex items-center justify-center h-screen w-full bg-zinc-950">
    <div class="flex flex-col items-center gap-4">
      <div
        class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin"
      ></div>
      <p class="text-zinc-500 font-medium">Carregando PsicoTrade Pro...</p>
    </div>
  </div>
{:else}
  <TooltipProvider>
    {@render children()}
    <Toaster />
  </TooltipProvider>
{/if}
