<script lang="ts">
    import { page } from "$app/stores";
    import { buttonVariants } from "$lib/components/ui/button";
    import { cn } from "$lib/utils";
    import {
        Settings,
        User,
        Wallet,
        Banknote,
        Receipt,
        Globe,
        Layers,
        Bitcoin,
        ShieldAlert,
        Target,
        Workflow,
        HeartPulse,
        Tag,
        Activity,
        Clock,
        BarChart3,
        Plug,
        Database,
        Crown,
        Scale,
    } from "lucide-svelte";

    import { t } from "svelte-i18n";

    let sidebarNavItems = $derived([
        {
            title: $t("settings.nav.sections.general"),
            href: "sep",
        },
        {
            href: "/settings/profile",
            title: $t("settings.nav.profile"),
            icon: User,
        },
        {
            href: "/settings/license",
            title: $t("settings.nav.license"),
            icon: Crown,
        },
        {
            title: $t("settings.nav.sections.registration"),
            href: "sep",
        },
        {
            href: "/settings/accounts",
            title: $t("settings.nav.accounts"),
            icon: Wallet,
        },
        {
            href: "/settings/currencies",
            title: $t("settings.nav.currencies"),
            icon: Banknote,
        },
        {
            href: "/settings/fees",
            title: $t("settings.nav.fees"),
            icon: Receipt,
        },
        {
            href: "/settings/markets",
            title: $t("settings.nav.markets"),
            icon: Globe,
        },
        {
            href: "/settings/asset-types",
            title: $t("settings.nav.assetTypes"),
            icon: Layers,
        },
        {
            href: "/settings/assets",
            title: $t("settings.nav.assets"),
            icon: Bitcoin,
        },

        {
            title: $t("settings.nav.sections.fiscal"),
            href: "sep",
        },
        {
            href: "/settings/fiscal/rules",
            title: $t("settings.nav.fiscal.rules"),
            icon: Scale,
        },
        {
            href: "/settings/fiscal/profiles",
            title: $t("settings.nav.fiscal.profiles"),
            icon: Layers, // Using Layers for profiles/grouping
        },
        {
            href: "/settings/fiscal/assignments",
            title: $t("settings.nav.fiscal.assignments"),
            icon: Tag, // Using Tag for assignments
        },

        {
            title: $t("settings.nav.sections.operational"),
            href: "sep",
        },
        {
            href: "/settings/risk",
            title: $t("settings.nav.risk"),
            icon: ShieldAlert,
        },
        {
            href: "/settings/strategies",
            title: $t("settings.nav.strategies"),
            icon: Target,
        },
        {
            href: "/settings/modalities",
            title: $t("settings.nav.modalities"),
            icon: Workflow,
        },
        {
            title: $t("settings.nav.sections.analysis"),
            href: "sep",
        },
        {
            href: "/settings/emotional-states",
            title: $t("settings.nav.emotionalStates"),
            icon: HeartPulse,
        },
        { href: "/settings/tags", title: $t("settings.nav.tags"), icon: Tag },
        {
            href: "/settings/indicators",
            title: $t("settings.nav.indicators"),
            icon: Activity,
        },
        {
            href: "/settings/timeframes",
            title: $t("settings.nav.timeframes"),
            icon: Clock,
        },
        {
            href: "/settings/chart-types",
            title: $t("settings.nav.chartTypes"),
            icon: BarChart3,
        },
        {
            title: $t("settings.nav.sections.system"),
            href: "sep",
        },
        {
            href: "/settings/api-integrations",
            title: $t("settings.nav.integrations"),
            icon: Plug,
        },
        {
            href: "/settings/database",
            title: $t("settings.nav.database"),
            icon: Database,
        },
    ]);
</script>

<nav
    class={cn(
        "flex flex-nowrap space-x-2 lg:flex-col lg:space-x-0 lg:space-y-1 overflow-x-auto lg:overflow-y-auto max-h-[calc(100vh-10rem)] custom-scrollbar pr-2 pb-2 lg:pb-10",
        "",
    )}
>
    {#each sidebarNavItems as item}
        {#if item.href === "sep"}
            <div
                class="mt-6 mb-2 px-4 text-xs font-bold text-muted-foreground/60 uppercase tracking-widest"
            >
                {item.title}
            </div>
        {:else}
            {@const Icon = item.icon}
            <a
                href={item.href}
                class={cn(
                    "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-all",
                    $page.url.pathname === item.href
                        ? "bg-secondary text-secondary-foreground shadow-sm"
                        : "text-muted-foreground hover:bg-muted hover:text-foreground",
                    "justify-start w-full",
                )}
            >
                <Icon class="w-4 h-4 shrink-0" />
                <span class="truncate">{item.title}</span>
            </a>
        {/if}
    {/each}
</nav>
