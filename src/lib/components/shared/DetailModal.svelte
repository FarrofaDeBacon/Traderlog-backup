<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { cn } from "$lib/utils";
    import { t } from "svelte-i18n";

    let {
        open = $bindable(false),
        title = "",
        description = "",
        icon: Icon = null,
        loading = false,
        size = "default", // default, lg, xl
        children,
        footer,
    } = $props();

    const sizeClasses = {
        default: "sm:max-w-[500px]",
        lg: "sm:max-w-[700px]",
        xl: "sm:max-w-[900px]",
    };
</script>

<Dialog.Root bind:open>
    <Dialog.Content
        class={cn(
            sizeClasses[size] || sizeClasses.default,
            "bg-popover/80 backdrop-blur-md border-border text-foreground overflow-hidden flex flex-col max-h-[90vh]"
        )}
    >
        <Dialog.Header class="px-6 pt-6 pb-2">
            <Dialog.Title class="flex items-center gap-2 text-xl font-black uppercase tracking-tight">
                {#if Icon}
                    <Icon class="w-5 h-5 text-amber-500" />
                {/if}
                {title}
            </Dialog.Title>
            {#if description}
                <Dialog.Description class="text-xs text-muted-foreground font-bold uppercase tracking-widest opacity-60">
                    {description}
                </Dialog.Description>
            {/if}
        </Dialog.Header>

        <div class="flex-1 overflow-y-auto px-6 py-4 custom-scrollbar">
            {#if loading}
                <div class="py-12 flex flex-col items-center justify-center gap-4">
                    <div
                        class="w-8 h-8 border-4 border-amber-500/20 border-t-amber-500 rounded-full animate-spin"
                    ></div>
                    <p class="text-[10px] font-black uppercase tracking-widest text-muted-foreground animate-pulse">
                        {loading === true ? $t("general.loading") : loading}
                    </p>
                </div>
            {:else}
                {@render children?.()}
            {/if}
        </div>

        <Dialog.Footer
            class="px-6 py-4 bg-muted/20 border-t border-border/30 flex flex-col sm:flex-row gap-2"
        >
            {#if footer}
                {@render footer()}
            {:else}
                <Button
                    variant="secondary"
                    onclick={() => (open = false)}
                    class="bg-muted hover:bg-accent text-foreground border-border font-bold uppercase text-[10px] px-6"
                >
                    {$t("general.close")}
                </Button>
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: hsl(var(--border) / 0.5);
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: hsl(var(--border));
    }
</style>
