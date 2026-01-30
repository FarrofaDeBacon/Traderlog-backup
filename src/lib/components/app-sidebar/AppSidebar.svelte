<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { ScrollArea } from "$lib/components/ui/scroll-area";
    import { Separator } from "$lib/components/ui/separator";
    import * as Sheet from "$lib/components/ui/sheet";
    import {
        LayoutDashboard,
        BookOpen,
        TrendingUp,
        Settings,
        LogOut,
        Menu,
    } from "lucide-svelte";
    import { page } from "$app/stores";

    // Navigation Items
    const navItems = [
        { label: "Dashboard", href: "/", icon: LayoutDashboard },
        { label: "Diário", href: "/journal", icon: BookOpen },
        { label: "Trades", href: "/trades", icon: TrendingUp },
        { label: "Configurações", href: "/settings", icon: Settings },
    ];

    let isOpen = $state(false);

    // Helper to check active route
    function isActive(href: string) {
        return $page.url.pathname === href;
    }
</script>

<!-- Mobile Trigger -->
<Sheet.Root bind:open={isOpen}>
    <Sheet.Trigger class="md:hidden">
        {#snippet child({ props }: { props: Record<string, any> })}
            <Button variant="ghost" size="icon" {...props}>
                <Menu class="h-5 w-5" />
            </Button>
        {/snippet}
    </Sheet.Trigger>
    <Sheet.Content side="left" class="w-[240px] p-0">
        <div class="flex h-full flex-col">
            <div class="p-6">
                <h2
                    class="text-lg font-bold tracking-tight text-primary flex items-center gap-2"
                >
                    <TrendingUp class="h-6 w-6" />
                    TraderLog
                </h2>
            </div>
            <Separator />
            <ScrollArea class="flex-1 py-4">
                <nav class="grid gap-1 px-2">
                    {#each navItems as item}
                        <a
                            href={item.href}
                            class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground {isActive(
                                item.href,
                            )
                                ? 'bg-accent text-accent-foreground'
                                : 'text-muted-foreground'}"
                            onclick={() => (isOpen = false)}
                        >
                            <item.icon class="h-4 w-4" />
                            {item.label}
                        </a>
                    {/each}
                </nav>
            </ScrollArea>
        </div>
    </Sheet.Content>
</Sheet.Root>

<!-- Desktop Sidebar -->
<aside
    class="hidden border-r bg-muted/40 md:block md:w-[220px] lg:w-[280px] h-screen sticky top-0"
>
    <div class="flex h-full flex-col gap-2">
        <div class="flex h-14 items-center border-b px-4 lg:h-[60px] lg:px-6">
            <a href="/" class="flex items-center gap-2 font-semibold">
                <TrendingUp class="h-6 w-6" />
                <span class="">TraderLog Pro</span>
            </a>
        </div>
        <div class="flex-1">
            <nav class="grid items-start px-2 text-sm font-medium lg:px-4 pt-4">
                {#each navItems as item}
                    <a
                        href={item.href}
                        class="flex items-center gap-3 rounded-lg px-3 py-2 transition-all hover:text-primary {isActive(
                            item.href,
                        )
                            ? 'bg-muted text-primary'
                            : 'text-muted-foreground'}"
                    >
                        <item.icon class="h-4 w-4" />
                        {item.label}
                    </a>
                {/each}
            </nav>
        </div>
        <div class="mt-auto p-4">
            <!-- Future User Profile or Credits area -->
        </div>
    </div>
</aside>
