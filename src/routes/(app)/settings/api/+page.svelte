<script lang="ts">
    import { Separator } from "$lib/components/ui/separator";
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import * as Card from "$lib/components/ui/card";
    import * as Select from "$lib/components/ui/select";
    import * as Dialog from "$lib/components/ui/dialog";
    import {
        Plus,
        Trash2,
        Pencil,
        Loader2,
        Key,
        Globe,
        ExternalLink,
    } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";

    type ApiConfig = {
        id: string;
        provider: string;
        api_key: string;
        enabled: boolean;
        daily_limit: number;
    };

    const providers = [
        { value: "exchangerate-api" },
        { value: "polygon" },
        { value: "openai" },
        { value: "google_gemini" },
        { value: "anthropic" },
    ];

    let apis: ApiConfig[] = $state([
        {
            id: "1",
            provider: "openai",
            api_key: "sk-proj-...",
            enabled: true,
            daily_limit: 1000,
        },
    ]);

    // Calendar Source Config
    let calendarSource = $state("https://br.investing.com/economic-calendar/");

    function saveCalendarSource() {
        localStorage.setItem(
            "traderlog:settings:calendar_source",
            calendarSource,
        );
        toast.success($t("settings.api.integrations.serviceMapping.success"));
    }

    $effect(() => {
        const saved = localStorage.getItem(
            "traderlog:settings:calendar_source",
        );
        if (saved) calendarSource = saved;
    });

    let isDialogOpen = $state(false);
    let editingId = $state<string | null>(null);
    let testing = $state(false);

    // Delete Modal State
    let isDeleteOpen = $state(false);
    let deleteId = $state<string | null>(null);

    let formData = $state<Omit<ApiConfig, "id">>({
        provider: "openai",
        api_key: "",
        enabled: true,
        daily_limit: 1500,
    });

    function openNew() {
        editingId = null;
        formData = {
            provider: "openai",
            api_key: "",
            enabled: true,
            daily_limit: 1500,
        };
        isDialogOpen = true;
    }

    function openEdit(api: ApiConfig) {
        editingId = api.id;
        formData = { ...api };
        isDialogOpen = true;
    }

    async function testConnection() {
        if (!formData.api_key) {
            toast.error($t("settings.api.form.errors.enterKeyFirst"));
            return;
        }
        testing = true;
        // Mock connection test
        await new Promise((resolve) => setTimeout(resolve, 1500));
        testing = false;
        toast.success($t("settings.api.form.messages.connectionSuccess"));
    }

    function saveApi() {
        if (!formData.api_key) {
            toast.error($t("settings.api.form.errors.keyRequired"));
            return;
        }

        if (editingId) {
            apis = apis.map((a) =>
                a.id === editingId ? { ...formData, id: editingId! } : a,
            );
        } else {
            apis = [...apis, { ...formData, id: crypto.randomUUID() }];
        }
        isDialogOpen = false;
    }

    function requestDelete(id: string) {
        deleteId = id;
        isDeleteOpen = true;
    }

    function confirmDelete() {
        if (deleteId) {
            apis = apis.filter((a) => a.id !== deleteId);
            toast.success($t("general.deleteSuccess"));
            deleteId = null;
        }
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">
                {$t("settings.nav.sections.integrations")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.api.form.description")}
            </p>
        </div>
        <Button onclick={openNew}>
            <Plus class="w-4 h-4 mr-2" />
            {$t("settings.api.form.newTitle")}
        </Button>
    </div>
    <Separator />

    <!-- Calendar Source Config -->
    <Card.Root>
        <Card.Header class="pb-3">
            <div class="flex items-center gap-2">
                <Globe class="w-5 h-5 text-blue-500" />
                <Card.Title class="text-base"
                    >{$t(
                        "settings.api.integrations.serviceMapping.calendarSource",
                    )}</Card.Title
                >
            </div>
            <Card.Description>
                {$t(
                    "settings.api.integrations.serviceMapping.calendarSourceHint",
                )}
            </Card.Description>
        </Card.Header>
        <Card.Content>
            <div class="flex gap-4">
                <div class="flex-1 relative">
                    <Input
                        bind:value={calendarSource}
                        placeholder="https://..."
                        class="pr-8"
                    />
                    <a
                        href={calendarSource}
                        target="_blank"
                        class="absolute right-3 top-2.5 text-muted-foreground hover:text-primary"
                    >
                        <ExternalLink class="w-4 h-4" />
                    </a>
                </div>
                <Button variant="secondary" onclick={saveCalendarSource}>
                    {$t(
                        "settings.api.integrations.serviceMapping.calendarSave",
                    )}
                </Button>
            </div>
        </Card.Content>
    </Card.Root>

    <div class="grid gap-4 md:grid-cols-1 lg:grid-cols-2">
        {#each apis as api}
            <Card.Root
                class="hover:border-primary/50 transition-all cursor-pointer"
                onclick={() => openEdit(api)}
                role="button"
                tabindex={0}
                onkeydown={(e) => e.key === "Enter" && openEdit(api)}
            >
                <Card.Header class="pb-2">
                    <div class="flex justify-between items-start">
                        <div class="flex items-center gap-3">
                            <div class="p-2 bg-muted rounded-lg border">
                                <Key class="w-5 h-5 text-primary" />
                            </div>
                            <div>
                                <Card.Title class="text-base"
                                    >{$t(
                                        `settings.api.form.providers.${api.provider}`,
                                    )}</Card.Title
                                >
                                <Card.Description
                                    class="font-mono text-xs mt-1 truncate max-w-[200px]"
                                >
                                    {api.api_key.substring(0, 8)}...
                                </Card.Description>
                            </div>
                        </div>
                        <div class="flex items-center gap-2">
                            <Button
                                variant="ghost"
                                size="icon"
                                class="h-8 w-8"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    openEdit(api);
                                }}
                            >
                                <Pencil class="w-4 h-4" />
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon"
                                class="h-8 w-8 text-destructive hover:text-destructive"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    requestDelete(api.id);
                                }}
                            >
                                <Trash2 class="w-4 h-4" />
                            </Button>
                        </div>
                    </div>
                </Card.Header>
                <Card.Content>
                    <div class="text-xs text-muted-foreground">
                        {$t("settings.api.form.dailyLimit")}: {api.daily_limit} reqs
                    </div>
                </Card.Content>
            </Card.Root>
        {/each}
    </div>
</div>

<DeleteConfirmationModal bind:open={isDeleteOpen} onConfirm={confirmDelete} />

<Dialog.Root bind:open={isDialogOpen}>
    <Dialog.Content class="sm:max-w-[500px]">
        <Dialog.Header>
            <Dialog.Title
                >{editingId
                    ? $t("settings.api.form.editTitle")
                    : $t("settings.api.form.newTitle")}</Dialog.Title
            >
        </Dialog.Header>

        <div class="grid gap-6 py-4">
            <div class="space-y-2">
                <Label>{$t("settings.api.form.provider")}</Label>
                <Select.Root type="single" bind:value={formData.provider}>
                    <Select.Trigger>
                        {$t(`settings.api.form.providers.${formData.provider}`)}
                    </Select.Trigger>
                    <Select.Content>
                        {#each providers as p}
                            <Select.Item value={p.value}
                                >{$t(
                                    `settings.api.form.providers.${p.value}`,
                                )}</Select.Item
                            >
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>

            <div class="space-y-2">
                <Label>{$t("settings.api.form.apiKey")}</Label>
                <div class="flex gap-2">
                    <Input
                        type="password"
                        bind:value={formData.api_key}
                        placeholder={$t("settings.api.form.apiKeyPlaceholder")}
                    />
                    <Button
                        variant="outline"
                        onclick={testConnection}
                        disabled={testing}
                    >
                        {#if testing}
                            <Loader2 class="w-4 h-4 animate-spin" />
                        {:else}
                            {$t("settings.api.form.test")}
                        {/if}
                    </Button>
                </div>
            </div>

            <div class="space-y-2">
                <Label>{$t("settings.api.form.dailyLimit")}</Label>
                <Input type="number" bind:value={formData.daily_limit} />
            </div>
        </div>

        <Dialog.Footer>
            <Button type="submit" onclick={saveApi}
                >{$t("settings.api.form.save")}</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
