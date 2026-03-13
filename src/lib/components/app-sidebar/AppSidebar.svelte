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
        Target,
        Wallet,
        Brain,
        Activity,
        PanelLeftClose,
        PanelLeftOpen,
        FileText,
        DollarSign,
    } from "lucide-svelte";
    import * as Tooltip from "$lib/components/ui/tooltip";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { sidebarState } from "$lib/stores/sidebar.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";
    import { page } from "$app/stores";
    import ModeToggle from "$lib/components/mode-toggle/ModeToggle.svelte";

    // Workspace Items (Principal)
    let workspaceItems = $derived([
        { label: $t("sidebar.trades"), href: "/trades", icon: TrendingUp },
        { label: $t("sidebar.strategies"), href: "/strategies", icon: Target },
        { label: $t("sidebar.finance"), href: "/finance", icon: Wallet },
    ]);

    // Análise Items
    let analysisItems = $derived([
        { label: $t("sidebar.dashboard"), href: "/", icon: LayoutDashboard },
        { label: $t("sidebar.psychology"), href: "/psychology", icon: Brain },
    ]);

    // Fiscal Items
    let fiscalItems = $derived([
        {
            label: $t("sidebar.irpf"),
            href: "/fiscal/irpf",
            icon: FileText,
        },
    ]);

    // System Items
    let systemItems = $derived([
        {
            label: $t("sidebar.settings"),
            href: "/settings",
            icon: Settings,
        },
    ]);

    let isOpen = $state(false);

    // Helper to check active route
    function isActive(href: string) {
        if (href === "/" && $page.url.pathname !== "/") return false;
        return (
            $page.url.pathname.startsWith(href) &&
            (href !== "/" || $page.url.pathname === "/")
        );
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
                    <img src="/branding/navbar-logo.png" alt="Logo" class="h-6 w-6 object-contain" />
                    <span
                        >TraderLog <span
                            class="text-[9px] font-black uppercase text-primary px-1 rounded border border-primary/20 bg-primary/5 align-top"
                            >Beta</span
                        ></span
                    >
                </h2>
            </div>
            <Separator />
            <ScrollArea class="flex-1 py-4">
                <nav class="grid gap-1 px-2">
                    <div
                        class="px-3 py-2 text-[10px] font-bold uppercase text-muted-foreground tracking-widest"
                    >
                        {$t("sidebar.categories.workspace")}
                    </div>
                    {#each workspaceItems as item}
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

                    <Separator class="my-2" />
                    <div
                        class="px-3 py-2 text-[10px] font-bold uppercase text-muted-foreground tracking-widest"
                    >
                        {$t("sidebar.categories.analysis")}
                    </div>
                    {#each analysisItems as item}
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

                    <Separator class="my-2" />
                    <div
                        class="px-3 py-2 text-[10px] font-bold uppercase text-muted-foreground tracking-widest"
                    >
                        {$t("sidebar.categories.fiscal")}
                    </div>
                    {#each fiscalItems as item}
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
<!-- Desktop Sidebar -->
<!-- Desktop Sidebar -->
<aside
    class="hidden border-r bg-sidebar/40 backdrop-blur-xl md:flex md:flex-col min-h-screen transition-all duration-300 relative group"
>
    <!-- Floating Toggle Button -->
    <Button
        variant="outline"
        size="icon"
        class="absolute -right-3 top-20 z-50 h-6 w-6 rounded-full border shadow-md p-0 flex items-center justify-center bg-background"
        onclick={() => sidebarState.toggle()}
    >
        {#if sidebarState.isCollapsed}
            <PanelLeftOpen class="h-4 w-4" />
        {:else}
            <PanelLeftClose class="h-4 w-4" />
        {/if}
    </Button>

    <div
        class="flex h-14 items-center border-b px-4 lg:h-[60px] lg:px-6 justify-between"
    >
        <a
            href="/"
            class="flex items-center gap-2 font-semibold overflow-hidden"
        >
            <img src="/branding/navbar-logo.png" alt="Logo" class="h-6 w-6 shrink-0 object-contain" />
            {#if !sidebarState.isCollapsed}
                <div class="flex items-center gap-2 overflow-hidden">
                    <span class="truncate">TraderLog Pro</span>
                    <span
                        class="text-[9px] font-black uppercase text-primary px-1 rounded border border-primary/20 bg-primary/5"
                        >Beta</span
                    >
                </div>
            {/if}
        </a>
        <div class="flex items-center gap-2">
            <ModeToggle />
        </div>
    </div>
    <div class="flex-1 overflow-x-hidden">
        <nav class="grid gap-1 px-2 pt-4">
            {#if !sidebarState.isCollapsed}
                <div
                    class="px-3 py-2 text-[10px] font-bold uppercase text-muted-foreground tracking-widest"
                >
                    {$t("sidebar.categories.workspace")}
                </div>
            {/if}
            {#each workspaceItems as item}
                {#if sidebarState.isCollapsed}
                    <Tooltip.Root>
                        <Tooltip.Trigger>
                            {#snippet child({
                                props,
                            }: {
                                props: Record<string, any>;
                            })}
                                <a
                                    href={item.href}
                                    class="flex items-center justify-center rounded-lg px-2 py-2 transition-all hover:text-primary {isActive(
                                        item.href,
                                    )
                                        ? 'bg-muted text-primary'
                                        : 'text-muted-foreground'}"
                                    {...props}
                                >
                                    <item.icon class="h-5 w-5 shrink-0" />
                                </a>
                            {/snippet}
                        </Tooltip.Trigger>
                        <Tooltip.Content side="right"
                            >{item.label}</Tooltip.Content
                        >
                    </Tooltip.Root>
                {:else}
                    <a
                        href={item.href}
                        class="flex items-center gap-3 rounded-lg px-3 py-2 transition-all hover:text-primary {isActive(
                            item.href,
                        )
                            ? 'bg-muted text-primary'
                            : 'text-muted-foreground'}"
                    >
                        <item.icon class="h-4 w-4 shrink-0" />
                        {item.label}
                    </a>
                {/if}
            {/each}

            <Separator class="my-2" />
            {#if !sidebarState.isCollapsed}
                <div
                    class="px-3 py-2 text-[10px] font-bold uppercase text-muted-foreground tracking-widest"
                >
                    {$t("sidebar.categories.analysis")}
                </div>
            {/if}
            {#each analysisItems as item}
                {#if sidebarState.isCollapsed}
                    <Tooltip.Root>
                        <Tooltip.Trigger>
                            {#snippet child({
                                props,
                            }: {
                                props: Record<string, any>;
                            })}
                                <a
                                    href={item.href}
                                    class="flex items-center justify-center rounded-lg px-2 py-2 transition-all hover:text-primary {isActive(
                                        item.href,
                                    )
                                        ? 'bg-muted text-primary'
                                        : 'text-muted-foreground'}"
                                    {...props}
                                >
                                    <item.icon class="h-5 w-5 shrink-0" />
                                </a>
                            {/snippet}
                        </Tooltip.Trigger>
                        <Tooltip.Content side="right"
                            >{item.label}</Tooltip.Content
                        >
                    </Tooltip.Root>
                {:else}
                    <a
                        href={item.href}
                        class="flex items-center gap-3 rounded-lg px-3 py-2 transition-all hover:text-primary {isActive(
                            item.href,
                        )
                            ? 'bg-muted text-primary'
                            : 'text-muted-foreground'}"
                    >
                        <item.icon class="h-4 w-4 shrink-0" />
                        {item.label}
                    </a>
                {/if}
            {/each}

            <Separator class="my-2" />
            {#if !sidebarState.isCollapsed}
                <div
                    class="px-3 py-2 text-[10px] font-bold uppercase text-muted-foreground tracking-widest"
                >
                    {$t("sidebar.categories.fiscal")}
                </div>
            {/if}
            {#each fiscalItems as item}
                {#if sidebarState.isCollapsed}
                    <Tooltip.Root>
                        <Tooltip.Trigger>
                            {#snippet child({
                                props,
                            }: {
                                props: Record<string, any>;
                            })}
                                <a
                                    href={item.href}
                                    class="flex items-center justify-center rounded-lg px-2 py-2 transition-all hover:text-primary {isActive(
                                        item.href,
                                    )
                                        ? 'bg-muted text-primary'
                                        : 'text-muted-foreground'}"
                                    {...props}
                                >
                                    <item.icon class="h-5 w-5 shrink-0" />
                                </a>
                            {/snippet}
                        </Tooltip.Trigger>
                        <Tooltip.Content side="right"
                            >{item.label}</Tooltip.Content
                        >
                    </Tooltip.Root>
                {:else}
                    <a
                        href={item.href}
                        class="flex items-center gap-3 rounded-lg px-3 py-2 transition-all hover:text-primary {isActive(
                            item.href,
                        )
                            ? 'bg-muted text-primary'
                            : 'text-muted-foreground'}"
                    >
                        <item.icon class="h-4 w-4 shrink-0" />
                        {item.label}
                    </a>
                {/if}
            {/each}
        </nav>
    </div>

    <!-- Footer: Settings & Logout -->
    <div class="mt-auto p-4 border-t flex flex-col gap-2">
        {#each systemItems as item}
            {#if sidebarState.isCollapsed}
                <Tooltip.Root>
                    <Tooltip.Trigger>
                        {#snippet child({
                            props,
                        }: {
                            props: Record<string, any>;
                        })}
                            <a
                                href={item.href}
                                class="flex items-center justify-center rounded-lg px-2 py-2 transition-all hover:text-primary {isActive(
                                    item.href,
                                )
                                    ? 'bg-muted text-primary'
                                    : 'text-muted-foreground'}"
                                {...props}
                            >
                                <item.icon class="h-5 w-5 shrink-0" />
                            </a>
                        {/snippet}
                    </Tooltip.Trigger>
                    <Tooltip.Content side="right">{item.label}</Tooltip.Content>
                </Tooltip.Root>
            {:else}
                <a
                    href={item.href}
                    class="flex items-center gap-3 rounded-lg px-3 py-2 transition-all hover:text-primary {isActive(
                        item.href,
                    )
                        ? 'bg-muted text-primary'
                        : 'text-muted-foreground'}"
                >
                    <item.icon class="h-4 w-4 shrink-0" />
                    {item.label}
                </a>
            {/if}
        {/each}
        <!-- Logout Button -->
        <AlertDialog.Root>
            <AlertDialog.Trigger>
                {#snippet child({ props }: { props: Record<string, any> })}
                    <Button
                        variant="ghost"
                        size="sm"
                        class="w-full justify-start text-destructive hover:text-destructive hover:bg-destructive/10 {sidebarState.isCollapsed
                            ? 'px-2 justify-center'
                            : ''}"
                        {...props}
                    >
                        <LogOut
                            class="h-4 w-4 {sidebarState.isCollapsed
                                ? ''
                                : 'mr-2'}"
                        />
                        {#if !sidebarState.isCollapsed}
                            <span>{$t("settings.profile.security.logout")}</span
                            >
                        {/if}
                    </Button>
                {/snippet}
            </AlertDialog.Trigger>
            <AlertDialog.Content>
                <AlertDialog.Header>
                    <AlertDialog.Title
                        >{$t(
                            "settings.profile.security.logout",
                        )}</AlertDialog.Title
                    >
                    <AlertDialog.Description>
                        {$t("settings.profile.security.confirmLogout")}
                    </AlertDialog.Description>
                </AlertDialog.Header>
                <AlertDialog.Footer>
                    <AlertDialog.Cancel
                        >{$t("general.cancel")}</AlertDialog.Cancel
                    >
                    <AlertDialog.Action
                        onclick={() => {
                            toast.success(
                                $t("settings.profile.security.loggingOut"),
                            );
                            settingsStore.logout();
                            setTimeout(() => goto("/login"), 1000);
                        }}
                        class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                    >
                        {$t("settings.profile.security.logout")}
                    </AlertDialog.Action>
                </AlertDialog.Footer>
            </AlertDialog.Content>
        </AlertDialog.Root>
    </div>
</aside>
