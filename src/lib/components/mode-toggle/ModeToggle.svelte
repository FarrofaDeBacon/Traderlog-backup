<script lang="ts">
    import { Sun, Moon } from "lucide-svelte";
    import { toggleMode } from "mode-watcher";
    import { Button } from "$lib/components/ui/button";
    import { settingsStore } from "$lib/stores/settings.svelte";

    function handleToggle() {
        toggleMode();
        // Sync with store
        const newTheme =
            settingsStore.userProfile.theme === "dark" ? "light" : "dark";
        settingsStore.userProfile.theme = newTheme;
    }
</script>

<Button onclick={handleToggle} variant="outline" size="icon" class="h-8 w-8">
    <Sun
        class="h-[1rem] w-[1rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
    />
    <Moon
        class="absolute h-[1rem] w-[1rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
    />
    <span class="sr-only">Toggle theme</span>
</Button>
