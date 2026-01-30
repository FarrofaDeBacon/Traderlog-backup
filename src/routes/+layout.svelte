<script lang="ts">
  import "../app.css";
  import AppSidebar from "$lib/components/app-sidebar/AppSidebar.svelte";
  import AppHeader from "$lib/components/app-header/AppHeader.svelte";
  import { ModeWatcher } from "mode-watcher";

  let { children } = $props();
</script>

<ModeWatcher />

<div
  class="grid min-h-screen w-full md:grid-cols-[220px_1fr] lg:grid-cols-[280px_1fr]"
>
  <AppSidebar />
  <div class="flex flex-col">
    <AppHeader />
    <main class="flex flex-1 flex-col gap-4 p-4 lg:gap-6 lg:p-6">
      {@render children()}
    </main>
  </div>
</div>
