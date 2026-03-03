<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { save, open } from "@tauri-apps/plugin-dialog";
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import {
        RefreshCw,
        Search,
        Database,
        Trash2,
        AlertCircle,
        Download,
        Upload,
        HardDrive,
    } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";
    import DeleteConfirmationModal from "$lib/components/settings/DeleteConfirmationModal.svelte";

    let isChecking = $state(false);
    let statusMessage = $state("");

    async function checkDatabase() {
        isChecking = true;
        statusMessage = "";
        try {
            const status = await invoke<string>("check_database_status");
            statusMessage = status;
            toast.success($t("settings.database.status.success"));
        } catch (e) {
            const errorMsg = typeof e === "string" ? e : (e as Error).message;
            statusMessage = "ERRO: " + errorMsg;
            toast.error($t("settings.database.status.error") + ": " + errorMsg);
        } finally {
            isChecking = false;
        }
    }

    let isRestoring = $state(false);
    let isRestoreModalOpen = $state(false);

    function triggerRestoreDefaults() {
        isRestoreModalOpen = true;
    }

    async function executeRestoreDefaults() {
        isRestoring = true;
        try {
            await invoke("ensure_defaults");
            toast.success("OK");
            setTimeout(() => window.location.reload(), 1000);
        } catch (e) {
            toast.error((e as Error).message);
        } finally {
            isRestoring = false;
        }
    }

    // --- Backup & Restore ---
    let isBackingUp = $state(false);
    let isRestoringBackup = $state(false);

    async function handleBackup() {
        isBackingUp = true;
        try {
            const now = new Date();
            const ts = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}_${String(now.getHours()).padStart(2, "0")}${String(now.getMinutes()).padStart(2, "0")}`;
            const defaultName = `traderlog_backup_${ts}.json`;

            const path = await save({
                title: $t("settings.database.backup.saveTitle"),
                defaultPath: defaultName,
                filters: [{ name: "JSON Backup", extensions: ["json"] }],
            });

            if (!path) {
                isBackingUp = false;
                return; // user cancelled
            }

            await invoke("backup_database", { path });
            toast.success($t("settings.database.backup.successExport"));
        } catch (e) {
            const errorMsg = typeof e === "string" ? e : String(e);
            console.error("Backup failed:", e);
            toast.error(
                $t("settings.database.backup.errorExport") + ": " + errorMsg,
            );
        } finally {
            isBackingUp = false;
        }
    }

    async function handleRestore() {
        isRestoringBackup = true;
        try {
            const path = await open({
                title: $t("settings.database.backup.openTitle"),
                filters: [{ name: "JSON Backup", extensions: ["json"] }],
                multiple: false,
                directory: false,
            });

            if (!path || Array.isArray(path)) {
                isRestoringBackup = false;
                return; // user cancelled
            }

            const count = await invoke<number>("restore_database", { path });
            toast.success(
                $t("settings.database.backup.successImport").replace(
                    "{count}",
                    String(count),
                ),
            );
            setTimeout(() => window.location.reload(), 1500);
        } catch (e) {
            const errorMsg = typeof e === "string" ? e : String(e);
            console.error("Restore failed:", e);
            toast.error(
                $t("settings.database.backup.errorImport") + ": " + errorMsg,
            );
        } finally {
            isRestoringBackup = false;
        }
    }

    // Simplified Demo Data Management
    const DEMO_ACCOUNTS = [
        "account:demo_forex",
        "account:demo_b3_acoes",
        "account:demo_b3_futuros",
        "account:demo_nasdaq",
        "account:demo_crypto",
    ];

    let isGenerating = $state(false);
    let isCleaning = $state(false);

    let isCleanModalOpen = $state(false);

    function triggerCleanAll() {
        isCleanModalOpen = true;
    }

    async function executeCleanAll() {
        isCleaning = true;
        try {
            await invoke("delete_all_demo_trades");
            toast.success($t("settings.database.demo.successClean"));
            setTimeout(() => window.location.reload(), 1000);
        } catch (e) {
            toast.error((e as Error).message);
        } finally {
            isCleaning = false;
        }
    }

    async function handleGenerateDemo() {
        isGenerating = true;
        try {
            for (const accountId of DEMO_ACCOUNTS) {
                await invoke("seed_demo_account", { accountId });
            }
            toast.success($t("settings.database.demo.successGenerate"));
            setTimeout(() => window.location.reload(), 1000);
        } catch (e) {
            toast.error((e as Error).message);
        } finally {
            isGenerating = false;
        }
    }

    let isResetReseed = $state(false);
    let isResetModalOpen = $state(false);

    function triggerForceReseed() {
        isResetModalOpen = true;
    }

    async function executeForceReseed() {
        isResetReseed = true;
        try {
            await invoke("force_reseed");
            toast.success("Reset complete");
            setTimeout(() => window.location.reload(), 1000);
        } catch (e) {
            toast.error((e as Error).message);
        } finally {
            isResetReseed = false;
        }
    }
</script>

<DeleteConfirmationModal
    bind:open={isRestoreModalOpen}
    onConfirm={executeRestoreDefaults}
    title={$t("settings.database.defaults.title")}
    description={$t("settings.database.defaults.confirm")}
/>

<DeleteConfirmationModal
    bind:open={isCleanModalOpen}
    onConfirm={executeCleanAll}
    title={$t("settings.database.demo.confirmDeleteTitle")}
    description={$t("settings.database.demo.confirmDeleteDescription")}
/>

<DeleteConfirmationModal
    bind:open={isResetModalOpen}
    onConfirm={executeForceReseed}
    title={$t("settings.database.danger.title")}
    description={$t("settings.database.danger.confirm")}
/>

<div
    class="container max-w-4xl mx-auto p-4 md:p-8 space-y-8 animate-in fade-in duration-500"
>
    <div>
        <h1 class="text-3xl font-black tracking-tight flex items-center gap-3">
            <Database class="w-8 h-8 text-primary" />
            {$t("settings.database.title")}
        </h1>
        <p class="text-muted-foreground mt-2 font-medium">
            {$t("settings.database.description")}
        </p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Status Card -->
        <Card.Root>
            <Card.Header>
                <div class="flex items-center gap-2">
                    <Search class="w-5 h-5 text-blue-500" />
                    <Card.Title
                        >{$t("settings.database.status.title")}</Card.Title
                    >
                </div>
                <Card.Description>
                    {$t("settings.database.status.description")}
                </Card.Description>
            </Card.Header>
            <Card.Content class="space-y-4">
                <Button
                    onclick={checkDatabase}
                    disabled={isChecking}
                    variant="outline"
                    class="w-full"
                >
                    {#if isChecking}
                        <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                        {$t("settings.database.status.checking")}
                    {:else}
                        <Search class="w-4 h-4 mr-2" />
                        {$t("settings.database.status.button")}
                    {/if}
                </Button>

                {#if statusMessage}
                    <div
                        class="rounded-lg border border-border bg-muted/30 p-4 overflow-auto max-h-40"
                    >
                        <pre
                            class="text-[10px] font-mono whitespace-pre-wrap">{statusMessage}</pre>
                    </div>
                {/if}
            </Card.Content>
        </Card.Root>

        <!-- Defaults Card -->
        <Card.Root>
            <Card.Header>
                <div class="flex items-center gap-2">
                    <RefreshCw class="w-5 h-5 text-emerald-500" />
                    <Card.Title
                        >{$t("settings.database.defaults.title")}</Card.Title
                    >
                </div>
                <Card.Description>
                    {$t("settings.database.defaults.description")}
                </Card.Description>
            </Card.Header>
            <Card.Content class="space-y-4">
                <div
                    class="bg-emerald-500/10 border border-emerald-500/20 rounded p-3 text-[11px] text-emerald-600 dark:text-emerald-400 font-medium"
                >
                    {$t("settings.database.defaults.warning")}
                </div>
                <Button
                    onclick={triggerRestoreDefaults}
                    disabled={isRestoring}
                    class="w-full bg-emerald-600 hover:bg-emerald-700 text-white"
                >
                    {#if isRestoring}
                        <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                        {$t("settings.database.defaults.restoring")}
                    {:else}
                        <RefreshCw class="w-4 h-4 mr-2" />
                        {$t("settings.database.defaults.button")}
                    {/if}
                </Button>
            </Card.Content>
        </Card.Root>
    </div>

    <!-- Demo Section (Simplified) -->
    <Card.Root>
        <Card.Header>
            <div class="flex items-center gap-2">
                <Database class="w-5 h-5 text-indigo-500" />
                <Card.Title>{$t("settings.database.demo.title")}</Card.Title>
            </div>
            <Card.Description>
                {$t("settings.database.demo.description")}
            </Card.Description>
        </Card.Header>
        <Card.Content>
            <div class="flex flex-col sm:flex-row gap-4">
                <Button
                    onclick={handleGenerateDemo}
                    disabled={isGenerating}
                    class="flex-1 bg-indigo-600 hover:bg-indigo-700 text-white"
                >
                    {#if isGenerating}
                        <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                        {$t("settings.database.demo.generating")}
                    {:else}
                        <Database class="w-4 h-4 mr-2" />
                        {$t("settings.database.demo.generate")}
                    {/if}
                </Button>

                <Button
                    onclick={triggerCleanAll}
                    disabled={isCleaning}
                    variant="outline"
                    class="flex-1 border-indigo-500/30 text-indigo-500 hover:bg-indigo-500/10"
                >
                    {#if isCleaning}
                        <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                        {$t("settings.database.demo.cleaning")}
                    {:else}
                        <Trash2 class="w-4 h-4 mr-2" />
                        {$t("settings.database.demo.cleanAll")}
                    {/if}
                </Button>
            </div>
        </Card.Content>
    </Card.Root>

    <!-- Backup & Restore -->
    <Card.Root class="border-emerald-500/20">
        <Card.Header>
            <div class="flex items-center gap-2">
                <HardDrive class="w-5 h-5 text-emerald-500" />
                <Card.Title>{$t("settings.database.backup.title")}</Card.Title>
            </div>
            <Card.Description>
                {$t("settings.database.backup.description")}
            </Card.Description>
        </Card.Header>
        <Card.Content>
            <div class="flex flex-col sm:flex-row gap-4">
                <!-- Export -->
                <Button
                    onclick={handleBackup}
                    disabled={isBackingUp}
                    class="flex-1 bg-emerald-600 hover:bg-emerald-700 text-white"
                >
                    {#if isBackingUp}
                        <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                        {$t("settings.database.backup.exporting")}
                    {:else}
                        <Download class="w-4 h-4 mr-2" />
                        {$t("settings.database.backup.export")}
                    {/if}
                </Button>

                <!-- Import/Restore -->
                <Button
                    onclick={handleRestore}
                    disabled={isRestoringBackup}
                    variant="outline"
                    class="flex-1 border-emerald-500/40 text-emerald-600 hover:bg-emerald-500/10"
                >
                    {#if isRestoringBackup}
                        <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                        {$t("settings.database.backup.importing")}
                    {:else}
                        <Upload class="w-4 h-4 mr-2" />
                        {$t("settings.database.backup.import")}
                    {/if}
                </Button>
            </div>
            <p class="text-[11px] text-muted-foreground mt-3">
                {$t("settings.database.backup.hint")}
            </p>
        </Card.Content>
    </Card.Root>

    <!-- Danger Zone -->
    <Card.Root class="border-destructive/30 bg-destructive/5 backdrop-blur-sm">
        <Card.Header>
            <div class="flex items-center gap-2">
                <AlertCircle class="w-5 h-5 text-destructive" />
                <Card.Title class="text-destructive"
                    >{$t("settings.database.danger.title")}</Card.Title
                >
            </div>
            <Card.Description>
                {$t("settings.database.danger.description")}
            </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-4">
            <p
                class="text-xs text-destructive font-black uppercase tracking-widest"
            >
                {$t("settings.database.danger.warning")}
            </p>
            <Button
                onclick={triggerForceReseed}
                disabled={isResetReseed}
                variant="destructive"
                class="w-full font-bold"
            >
                {#if isResetReseed}
                    <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                    {$t("settings.database.danger.resetting")}
                {:else}
                    <Trash2 class="w-4 h-4 mr-2" />
                    {$t("settings.database.danger.button")}
                {/if}
            </Button>
        </Card.Content>
    </Card.Root>
</div>
