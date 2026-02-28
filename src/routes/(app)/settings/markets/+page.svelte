<script lang="ts">
    import {
        Plus,
        Pencil,
        Trash2,
        Globe,
        MapPin,
        Clock,
        X,
        Calendar,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Select from "$lib/components/ui/select";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Separator } from "$lib/components/ui/separator";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import {
        settingsStore,
        type Market,
        type TradingSession,
    } from "$lib/stores/settings.svelte";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";
    import { toast } from "svelte-sonner";
    import Skeleton from "$lib/components/ui/skeleton.svelte";

    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let formMarket = $state<Omit<Market, "id">>({
        code: "",
        name: "",
        timezone: "America/Sao_Paulo",
        trading_days: [1, 2, 3, 4, 5],
        trading_sessions: [{ start_time: "09:00", end_time: "18:00" }],
    });

    const weekdays = [
        { value: 0, label: "sun" },
        { value: 1, label: "mon" },
        { value: 2, label: "tue" },
        { value: 3, label: "wed" },
        { value: 4, label: "thu" },
        { value: 5, label: "fri" },
        { value: 6, label: "sat" },
    ];

    function toggleDay(day: number) {
        if (formMarket.trading_days.includes(day)) {
            formMarket.trading_days = formMarket.trading_days.filter(
                (d) => d !== day,
            );
        } else {
            formMarket.trading_days = [...formMarket.trading_days, day].sort();
        }
    }

    function addSession() {
        formMarket.trading_sessions = [
            ...formMarket.trading_sessions,
            { start_time: "09:00", end_time: "18:00" },
        ];
    }

    function removeSession(index: number) {
        formMarket.trading_sessions = formMarket.trading_sessions.filter(
            (_, i) => i !== index,
        );
    }

    const timezones = [
        { value: "America/Sao_Paulo", label: "Brasília (UTC-3)" },
        { value: "America/New_York", label: "Nova York (US Eastern)" },
        { value: "America/Chicago", label: "Chicago (US Central)" },
        { value: "UTC", label: "UTC (Universal)" },
        { value: "Europe/London", label: "Londres (GMT/BST)" },
        { value: "Asia/Tokyo", label: "Tóquio (JST)" },
        { value: "Asia/Hong_Kong", label: "Hong Kong (HKT)" },
        { value: "Australia/Sydney", label: "Sydney (AEST)" },
    ];

    // Filter and Sort
    let filteredMarkets = $derived(
        [...settingsStore.markets].sort((a, b) => a.code.localeCompare(b.code)),
    );

    // Group by Timezone
    let groupedMarkets = $derived.by(() => {
        const groups: Record<string, Market[]> = {};
        for (const item of filteredMarkets) {
            const tzLabel =
                timezones.find((t) => t.value === item.timezone)?.label ||
                item.timezone;
            if (!groups[tzLabel]) {
                groups[tzLabel] = [];
            }
            groups[tzLabel].push(item);
        }
        return groups;
    });

    function openNew() {
        editingId = null;
        formMarket = {
            code: "",
            name: "",
            timezone: "America/Sao_Paulo",
            trading_days: [1, 2, 3, 4, 5],
            trading_sessions: [{ start_time: "09:00", end_time: "18:00" }],
        };
        isDialogOpen = true;
    }

    function openEdit(item: Market) {
        editingId = item.id;
        formMarket = {
            code: item.code,
            name: item.name,
            timezone: item.timezone,
            trading_days: [...item.trading_days],
            trading_sessions: item.trading_sessions.map((s) => ({ ...s })),
        };
        isDialogOpen = true;
    }

    function save() {
        if (editingId) {
            settingsStore.updateMarket(editingId, formMarket);
        } else {
            settingsStore.addMarket(formMarket);
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    async function confirmDelete() {
        if (deleteId) {
            await settingsStore.deleteMarket(deleteId);
            toast.success($t("general.deleteSuccess"));
            deleteId = null;
        }
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">
                {$t("settings.markets.title")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.markets.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.markets.new")}
        </Button>
    </div>
    <Separator />

    <!-- Grouped Clickable List Cards -->
    <div class="space-y-6">
        {#if settingsStore.isLoadingData && Object.keys(groupedMarkets).length === 0}
            <div class="space-y-4">
                <Skeleton class="h-8 w-48" />
                <div class="space-y-3">
                    {#each Array(3) as _}
                        <Skeleton class="h-24 rounded-lg" />
                    {/each}
                </div>
            </div>
        {:else if Object.keys(groupedMarkets).length > 0}
            {#each Object.entries(groupedMarkets) as [timezone, items]}
                {@const isBrazil =
                    timezone.includes("Brasília") ||
                    timezone.includes("Sao_Paulo")}
                {@const isUS =
                    timezone.includes("York") || timezone.includes("Chicago")}
                {@const isAsia =
                    timezone.includes("Tokyo") ||
                    timezone.includes("Hong_Kong") ||
                    timezone.includes("Asia")}
                {@const isUTC = timezone.includes("UTC")}

                <div class="space-y-4">
                    <!-- Rich Header -->
                    <div class="flex items-center gap-2">
                        <div
                            class={`p-1.5 rounded ${
                                isBrazil
                                    ? "bg-green-500/10"
                                    : isUS
                                      ? "bg-blue-500/10"
                                      : isAsia
                                        ? "bg-red-500/10"
                                        : "bg-muted"
                            }`}
                        >
                            {#if isBrazil}
                                <MapPin class="w-4 h-4 text-green-500" />
                            {:else if isUS}
                                <MapPin class="w-4 h-4 text-blue-500" />
                            {:else if isAsia}
                                <MapPin class="w-4 h-4 text-red-500" />
                            {:else}
                                <Clock class="w-4 h-4 text-muted-foreground" />
                            {/if}
                        </div>
                        <h4 class="text-lg font-semibold tracking-tight">
                            {timezone}
                        </h4>
                    </div>

                    <div class="flex flex-col gap-3">
                        {#each items as item}
                            <div
                                class="flex items-center justify-between p-4 rounded-lg border bg-card hover:border-primary/50 transition-all group shadow-sm cursor-pointer"
                                onclick={() => openEdit(item)}
                                role="button"
                                tabindex="0"
                                onkeydown={(e) =>
                                    e.key === "Enter" && openEdit(item)}
                            >
                                <!-- Left: Icon + Info -->
                                <div class="flex items-center gap-4 shrink-0">
                                    <div
                                        class="p-2.5 bg-muted rounded-xl shrink-0"
                                    >
                                        <Globe
                                            class="w-5 h-5 text-foreground/70"
                                        />
                                    </div>
                                    <div class="min-w-[150px]">
                                        <h4 class="font-bold text-base">
                                            {item.code}
                                        </h4>
                                        <p
                                            class="text-sm text-muted-foreground"
                                        >
                                            {item.name}
                                        </p>
                                    </div>
                                    <!-- Trading Info -->
                                    <div
                                        class="ml-6 text-xs text-muted-foreground"
                                    >
                                        <div class="flex items-center gap-1">
                                            <Clock class="w-3 h-3" />
                                            {#if item.trading_sessions?.length}
                                                {item.trading_sessions
                                                    .map(
                                                        (s) =>
                                                            `${s.start_time}-${s.end_time}`,
                                                    )
                                                    .join(", ")}
                                            {:else}
                                                {$t(
                                                    "settings.markets.labels.notConfigured",
                                                )}
                                            {/if}
                                        </div>
                                        <div class="flex gap-1 mt-0.5">
                                            {#each weekdays as day}
                                                <span
                                                    class="w-5 h-5 flex items-center justify-center rounded text-[10px] {item.trading_days?.includes(
                                                        day.value,
                                                    )
                                                        ? 'bg-primary/20 text-primary'
                                                        : 'bg-muted/50 text-muted-foreground/50'}"
                                                >
                                                    {$t(
                                                        `general.weekdays.${day.label}`,
                                                    ).charAt(0)}
                                                </span>
                                            {/each}
                                        </div>
                                    </div>
                                </div>

                                <!-- Right: Actions -->
                                <div
                                    class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
                                >
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        onclick={(e) => {
                                            e.stopPropagation(); // Avoid triggering edit twice
                                            openEdit(item);
                                        }}
                                    >
                                        <Pencil class="w-4 h-4" />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        class="text-destructive hover:text-destructive hover:bg-destructive/10"
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            requestDelete(item.id);
                                        }}
                                    >
                                        <Trash2 class="w-4 h-4" />
                                    </Button>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        {:else}
            <div
                class="flex flex-col items-center justify-center p-8 border-2 border-dashed rounded-lg border-muted-foreground/25 text-muted-foreground h-[150px]"
            >
                <Globe class="w-8 h-8 mb-2 opacity-20" />
                <span class="text-sm">{$t("settings.markets.empty")}</span>
            </div>
        {/if}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[600px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingId
                    ? $t("settings.markets.edit")
                    : $t("settings.markets.new")}</Dialog.Title
            >
        </Dialog.Header>

        <div class="grid gap-6 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.markets.form.code")}</Label
                >
                <Input
                    bind:value={formMarket.code}
                    class="col-span-3 uppercase"
                    placeholder={$t("settings.markets.form.codePlaceholder")}
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.markets.form.name")}</Label
                >
                <Input
                    bind:value={formMarket.name}
                    class="col-span-3"
                    placeholder={$t("settings.markets.form.namePlaceholder")}
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <Label class="text-right"
                    >{$t("settings.markets.form.timezone")}</Label
                >
                <div class="col-span-3">
                    <Select.Root type="single" bind:value={formMarket.timezone}>
                        <Select.Trigger>
                            {timezones.find(
                                (t) => t.value === formMarket.timezone,
                            )?.label || formMarket.timezone}
                        </Select.Trigger>
                        <Select.Content>
                            {#each timezones as tz}
                                <Select.Item value={tz.value}
                                    >{tz.label}</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <Separator />

            <div class="space-y-4">
                <Label>{$t("settings.markets.form.tradingDays")}</Label>
                <div class="flex gap-2">
                    {#each weekdays as day}
                        <Button
                            variant={formMarket.trading_days.includes(day.value)
                                ? "default"
                                : "outline"}
                            class="flex-1"
                            onclick={() => toggleDay(day.value)}
                        >
                            {$t(`general.weekdays.${day.label}`).charAt(0)}
                        </Button>
                    {/each}
                </div>
            </div>

            <Separator />

            <div class="space-y-4">
                <div class="flex items-center justify-between">
                    <Label>{$t("settings.markets.form.tradingSessions")}</Label>
                    <Button
                        variant="ghost"
                        size="sm"
                        class="h-8 px-2"
                        onclick={addSession}
                    >
                        <Plus class="w-4 h-4 mr-1" />
                        {$t("settings.markets.form.addSession")}
                    </Button>
                </div>

                {#each formMarket.trading_sessions as session, idx}
                    <div class="grid grid-cols-7 items-center gap-2">
                        <Input
                            type="time"
                            bind:value={session.start_time}
                            class="col-span-3"
                        />
                        <span class="text-center text-muted-foreground"
                            >{$t("settings.markets.form.until")}</span
                        >
                        <Input
                            type="time"
                            bind:value={session.end_time}
                            class="col-span-3"
                        />
                        {#if formMarket.trading_sessions.length > 1}
                            <Button
                                variant="ghost"
                                size="icon"
                                class="text-destructive h-8 w-8"
                                onclick={() => removeSession(idx)}
                            >
                                <X class="w-4 h-4" />
                            </Button>
                        {/if}
                    </div>
                {/each}
            </div>
        </div>

        <Dialog.Footer>
            <Button onclick={save}>{$t("settings.markets.form.save")}</Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
