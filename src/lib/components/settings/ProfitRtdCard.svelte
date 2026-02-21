<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Input } from "$lib/components/ui/input";
    import {
        Activity,
        RefreshCw,
        FileText,
        AlertCircle,
        FolderOpen,
        Link2,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { rtdStore } from "$lib/stores/rtd.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { toast } from "svelte-sonner";

    let isConnecting = $state(false);

    async function pickFile() {
        try {
            const selected = await open({
                multiple: false,
                filters: [
                    {
                        name: "Excel",
                        extensions: ["xlsx", "xls", "xlsm", "xlsb"],
                    },
                ],
            });
            if (selected) {
                settingsStore.setRtdExcelPath(selected as string);
                toast.success(
                    $t("settings.api.integrations.profit.fileSelected"),
                );
            }
        } catch (e) {
            console.error(e);
            toast.error("Erro ao selecionar arquivo");
        }
    }

    async function connect() {
        isConnecting = true;
        try {
            await invoke("start_rtd_monitor_cmd", {
                excelPath: settingsStore.rtdExcelPath || null,
            });
            settingsStore.setRtdEnabled(true);
            toast.success($t("settings.api.integrations.profit.connected"));
        } catch (e) {
            console.error(e);
            toast.error($t("settings.api.integrations.profit.error"));
        } finally {
            isConnecting = false;
        }
    }

    function disconnect() {
        settingsStore.setRtdEnabled(false);
        toast.info($t("settings.api.integrations.profit.disconnected"));
    }

    let status = $derived(
        settingsStore.rtdEnabled ? "connected" : "disconnected",
    );
</script>

<Card.Root class="border-l-4 border-l-green-500 bg-card/50">
    <Card.Header>
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <div class="p-2 bg-green-500/10 rounded-lg">
                    <Activity class="w-6 h-6 text-green-500" />
                </div>
                <div>
                    <Card.Title
                        >{$t(
                            "settings.api.integrations.profit.title",
                        )}</Card.Title
                    >
                    <Card.Description>
                        {$t("settings.api.integrations.profit.description")}
                    </Card.Description>
                </div>
            </div>
            <div class="flex flex-col items-end gap-2">
                <Badge
                    variant={settingsStore.rtdEnabled ? "default" : "secondary"}
                    class={settingsStore.rtdEnabled ? "bg-green-500" : ""}
                >
                    {settingsStore.rtdEnabled
                        ? $t("settings.api.integrations.profit.connected")
                        : $t("settings.api.integrations.profit.disconnected")}
                </Badge>
                {#if rtdStore.lastUpdate}
                    <span
                        class="text-[10px] text-muted-foreground flex items-center gap-1"
                    >
                        <RefreshCw class="w-2 h-2" />
                        {rtdStore.lastUpdate.toLocaleTimeString()}
                    </span>
                {/if}
            </div>
        </div>
    </Card.Header>
    <Card.Content>
        <div class="grid md:grid-cols-2 gap-8">
            <div class="space-y-6">
                <!-- File Picker Section -->
                <div class="space-y-2">
                    <label
                        class="text-xs font-medium text-muted-foreground flex items-center gap-2"
                    >
                        <Link2 class="w-3 h-3" />
                        {$t("settings.api.integrations.profit.excelFile")}
                    </label>
                    <div class="flex gap-2">
                        <Input
                            readonly
                            value={settingsStore.rtdExcelPath ||
                                $t(
                                    "settings.api.integrations.profit.noFileSelected",
                                )}
                            class="bg-muted/30 text-xs h-9"
                        />
                        <Button
                            variant="outline"
                            size="sm"
                            class="h-9 whitespace-nowrap"
                            onclick={pickFile}
                        >
                            <FolderOpen class="w-4 h-4 mr-2" />
                            {$t("settings.api.integrations.profit.selectFile")}
                        </Button>
                    </div>
                </div>

                <div
                    class="flex items-start gap-3 p-4 bg-blue-500/5 rounded-lg border border-blue-500/20"
                >
                    <div class="p-1.5 bg-blue-500/10 rounded-md shrink-0">
                        <FileText class="w-4 h-4 text-blue-400" />
                    </div>
                    <div class="text-[11px] space-y-1.5">
                        <p class="font-bold text-blue-300">
                            {$t(
                                "settings.api.integrations.profit.requirementTitle",
                            )}
                        </p>
                        <p class="text-muted-foreground leading-relaxed">
                            {$t(
                                "settings.api.integrations.profit.requirementDesc",
                            )}
                        </p>
                        <p class="text-emerald-400 font-medium">
                            ✓ {$t(
                                "settings.api.integrations.profit.allAssetsSupport",
                            )}
                        </p>
                    </div>
                </div>
            </div>

            <div class="flex flex-col justify-center gap-4">
                <div class="grid gap-3">
                    {#if !settingsStore.rtdEnabled}
                        <Button
                            size="lg"
                            class="w-full bg-green-600 hover:bg-green-700 text-white shadow-lg shadow-green-900/20 h-12"
                            onclick={connect}
                            disabled={isConnecting}
                        >
                            {#if isConnecting}
                                <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                                {$t(
                                    "settings.api.integrations.profit.connecting",
                                )}
                            {:else}
                                <Activity class="w-4 h-4 mr-2" />
                                {$t("settings.api.integrations.profit.connect")}
                            {/if}
                        </Button>
                    {:else}
                        <Button
                            variant="outline"
                            size="lg"
                            class="w-full border-destructive text-destructive hover:bg-destructive hover:text-destructive-foreground h-12"
                            onclick={disconnect}
                        >
                            <AlertCircle class="w-4 h-4 mr-2" />
                            {$t("settings.api.integrations.profit.disconnect")}
                        </Button>
                    {/if}
                </div>

                <div class="flex flex-col gap-2 pt-2">
                    <Button
                        variant="link"
                        size="sm"
                        class="text-xs text-muted-foreground hover:text-primary underline h-auto p-0"
                    >
                        {$t("settings.api.integrations.profit.setupGuide")}
                    </Button>
                </div>
            </div>
        </div>
    </Card.Content>
</Card.Root>
