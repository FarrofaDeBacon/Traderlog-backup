<script lang="ts">
    import { t } from "svelte-i18n";
    import * as Card from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import { Separator } from "$lib/components/ui/separator";
    import { Badge } from "$lib/components/ui/badge";
    import { toast } from "svelte-sonner";
    import {
        Crown,
        CheckCircle2,
        AlertCircle,
        Calendar,
        Key,
        ShieldCheck,
        FileCode,
        Download,
        Upload,
        Trash2,
    } from "lucide-svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import {
        validateLicenseKey,
        computeCustomerId,
        type LicenseData,
    } from "$lib/utils/license";
    import { open } from "@tauri-apps/plugin-dialog";
    import { readFile } from "@tauri-apps/plugin-fs";

    let isActivating = $state(false);
    let customerPin = $state("");

    // Calculate PIN based on profile and hardware
    $effect(() => {
        const profile = settingsStore.userProfile;
        const hwid = settingsStore.hardwareId;
        if (profile && profile.name && profile.cpf) {
            computeCustomerId({
                name: profile.name,
                cpf: profile.cpf,
                birthDate: profile.birth_date || "",
                hardwareId: hwid,
            }).then((id) => {
                customerPin = id;
            });
        }
    });

    // Diagnostic trace for license state
    $effect(() => {
        console.log("[License UI] Current License Details:", {
            details: settingsStore.licenseDetails,
            planName: settingsStore.licensePlanName,
            totalDays: settingsStore.licenseTotalDays,
            daysRemaining: settingsStore.licenseDaysRemaining,
        });
    });

    async function handleFileUpload() {
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: "Licença", extensions: ["lic"] }],
            });

            if (selected && !Array.isArray(selected)) {
                console.log("[License] Selected file:", selected);
                isActivating = true;
                const fileContent = await readFile(selected);
                const text = new TextDecoder().decode(fileContent);

                // Validate
                const result = await validateLicenseKey(
                    text.trim(),
                    customerPin,
                );

                if (result.valid) {
                    settingsStore.updateUserProfile({
                        license_key: text.trim(),
                    });
                    toast.success(
                        `Licença ${result.plan} ativada com sucesso!`,
                    );
                } else {
                    console.warn("Validation failed:", result.error);
                    toast.error(
                        result.error ||
                            "Arquivo de licença inválido para esta máquina.",
                    );
                }
            }
        } catch (e) {
            console.error(e);
            toast.error("Erro ao carregar arquivo de licença.");
        } finally {
            isActivating = false;
        }
    }
    async function handleDeactivate() {
        const confirmed = confirm(
            "Tem certeza que deseja remover esta licença? Você voltará para o período trial ou ficará sem acesso se ele já tiver expirado.",
        );
        if (confirmed) {
            await settingsStore.deactivateLicense();
            toast.success("Licença removida com sucesso.");
        }
    }
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
    <div class="flex items-center justify-between">
        <div>
            <h3 class="text-lg font-medium text-white flex items-center gap-2">
                <Crown class="w-5 h-5 text-yellow-500" />
                {$t("settings.license.management")}
            </h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.license.subtitle")}
            </p>
        </div>
        <div class="flex items-center gap-3">
            {#if settingsStore.licenseStatus === "active"}
                <Badge
                    variant="default"
                    class="bg-green-500/10 text-green-500 border-green-500/20 px-3 py-1 text-xs uppercase tracking-widest font-bold"
                >
                    Assinatura {settingsStore.licensePlanName} Ativa
                </Badge>
            {:else if settingsStore.licenseStatus === "trial"}
                <Badge
                    variant="outline"
                    class="bg-blue-500/10 text-blue-500 border-blue-500/20 px-3 py-1 text-xs uppercase tracking-widest font-bold"
                >
                    {$t("settings.license.trialBadge")} ({settingsStore.trialDaysLeft}
                    dias)
                </Badge>
            {:else}
                <Badge
                    variant="destructive"
                    class="px-3 py-1 text-xs uppercase tracking-widest font-bold"
                >
                    Expirada
                </Badge>
            {/if}

            {#if settingsStore.userProfile.license_key}
                <Button
                    variant="destructive"
                    size="sm"
                    class="h-8 bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white border-red-500/20 px-3"
                    onclick={handleDeactivate}
                >
                    <Trash2 class="w-4 h-4 mr-2" />
                    Remover Licença
                </Button>
            {/if}
        </div>
    </div>

    {#if settingsStore.licenseStatus === "active" && settingsStore.licenseDetails}
        <div class="grid gap-4 md:grid-cols-3">
            <div
                class="bg-zinc-900/60 border border-zinc-800 rounded-xl p-4 flex items-center gap-4"
            >
                <div
                    class="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center"
                >
                    <Crown class="w-5 h-5 text-primary" />
                </div>
                <div>
                    <p
                        class="text-[10px] text-zinc-500 uppercase font-bold tracking-tight"
                    >
                        Tipo de Licença
                    </p>
                    <p class="text-sm font-semibold text-white">
                        {settingsStore.licensePlanName}
                    </p>
                </div>
            </div>

            {#if settingsStore.licenseTotalDays}
                <div
                    class="bg-zinc-900/60 border border-zinc-800 rounded-xl p-4 flex items-center gap-4"
                >
                    <div
                        class="w-10 h-10 rounded-full bg-blue-500/10 flex items-center justify-center"
                    >
                        <Calendar class="w-5 h-5 text-blue-500" />
                    </div>
                    <div>
                        <p
                            class="text-[10px] text-zinc-500 uppercase font-bold tracking-tight"
                        >
                            Duração Contratada
                        </p>
                        <p class="text-sm font-semibold text-white">
                            {settingsStore.licenseTotalDays} dias
                        </p>
                    </div>
                </div>
            {:else if settingsStore.licenseDetails?.plan === "Lifetime"}
                <div
                    class="bg-zinc-900/60 border border-zinc-800 rounded-xl p-4 flex items-center gap-4"
                >
                    <div
                        class="w-10 h-10 rounded-full bg-blue-500/10 flex items-center justify-center"
                    >
                        <Calendar class="w-5 h-5 text-blue-500" />
                    </div>
                    <div>
                        <p
                            class="text-[10px] text-zinc-500 uppercase font-bold tracking-tight"
                        >
                            Duração Contratada
                        </p>
                        <p class="text-sm font-semibold text-white">
                            Vitalícia
                        </p>
                    </div>
                </div>
            {/if}

            {#if settingsStore.licenseDaysRemaining !== null}
                <div
                    class="bg-zinc-900/60 border border-zinc-800 rounded-xl p-4 flex items-center gap-4"
                >
                    <div
                        class="w-10 h-10 rounded-full bg-green-500/10 flex items-center justify-center"
                    >
                        <ShieldCheck class="w-5 h-5 text-green-500" />
                    </div>
                    <div>
                        <p
                            class="text-[10px] text-zinc-500 uppercase font-bold tracking-tight"
                        >
                            Restante
                        </p>
                        <p class="text-sm font-semibold text-white">
                            {settingsStore.licenseDaysRemaining} dias
                        </p>
                    </div>
                </div>
            {/if}
        </div>
    {/if}

    <Separator class="bg-border/40" />

    <div class="grid gap-6 md:grid-cols-2">
        <!-- Passo 1: PIN -->
        <Card.Root class="bg-zinc-900/40 border-zinc-800">
            <Card.Header>
                <Card.Title class="text-base flex items-center gap-2">
                    <span
                        class="w-6 h-6 rounded-full bg-primary/20 text-primary flex items-center justify-center text-xs"
                        >1</span
                    >
                    {$t("settings.license.step1")}
                </Card.Title>
                <Card.Description>
                    {$t("settings.license.step1Desc")}
                </Card.Description>
            </Card.Header>
            <Card.Content class="space-y-4">
                <div
                    class="p-4 bg-zinc-950/50 rounded-lg border border-zinc-800 space-y-3"
                >
                    <div class="space-y-1">
                        <span
                            class="text-[10px] text-zinc-500 uppercase font-bold tracking-tighter"
                            >{$t("settings.license.pinLabel")}</span
                        >
                        <div class="flex items-center justify-between gap-2">
                            <code
                                class="text-xl font-mono text-primary font-bold tracking-widest bg-primary/5 px-2 rounded"
                            >
                                {customerPin ||
                                    $t("settings.license.incompleteData")}
                            </code>
                            <Button
                                variant="outline"
                                size="sm"
                                class="h-8"
                                disabled={!customerPin}
                                onclick={() => {
                                    navigator.clipboard.writeText(customerPin);
                                    toast.success("PIN copiado!");
                                }}
                            >
                                <Download class="w-4 h-4 mr-2" />
                                {$t("settings.license.copy")}
                            </Button>
                        </div>
                    </div>
                    <p class="text-xs text-zinc-400 leading-relaxed">
                        {$t("settings.license.sendPinInfo")}
                    </p>
                </div>

                <div class="pt-2 text-xs text-zinc-500 italic">
                    {$t("settings.license.pinDisclaimer")}
                </div>
            </Card.Content>
        </Card.Root>

        <!-- Passo 2: Upload -->
        <Card.Root class="bg-zinc-900/40 border-zinc-800">
            <Card.Header>
                <Card.Title class="text-base flex items-center gap-2">
                    <span
                        class="w-6 h-6 rounded-full bg-primary/20 text-primary flex items-center justify-center text-xs"
                        >2</span
                    >
                    {$t("settings.license.step2")}
                </Card.Title>
                <Card.Description>
                    {$t("settings.license.step2Desc")}
                </Card.Description>
            </Card.Header>
            <Card.Content
                class="flex flex-col items-center justify-center py-8 space-y-4 border-2 border-dashed border-zinc-800 rounded-lg mx-6 mb-6"
            >
                <div
                    class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center"
                >
                    <FileCode class="w-6 h-6 text-zinc-400" />
                </div>
                <div class="text-center">
                    <p class="text-sm font-medium text-zinc-300">
                        {$t("settings.license.noFile")}
                    </p>
                    <p class="text-xs text-zinc-500">
                        {$t("settings.license.formatAccepted")}
                    </p>
                </div>
                <Button
                    class="font-bold px-8"
                    onclick={handleFileUpload}
                    disabled={isActivating || !customerPin}
                >
                    {#if isActivating}
                        Ativando...
                    {:else}
                        <Upload class="w-4 h-4 mr-2" />
                        {$t("settings.license.loadFile")}
                    {/if}
                </Button>
            </Card.Content>
        </Card.Root>
    </div>

    <div class="bg-primary/5 rounded-lg p-6 border border-primary/20 space-y-4">
        <h4 class="font-bold text-white flex items-center gap-2">
            <ShieldCheck class="w-5 h-5 text-primary" />
            {$t("settings.license.perksTitle")}
        </h4>
        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <div class="flex gap-3">
                <CheckCircle2 class="w-4 h-4 text-green-500 mt-0.5 shrink-0" />
                <span class="text-sm text-zinc-300"
                    >{$t("settings.license.perk1")}</span
                >
            </div>
            <div class="flex gap-3">
                <CheckCircle2 class="w-4 h-4 text-green-500 mt-0.5 shrink-0" />
                <span class="text-sm text-zinc-300"
                    >{$t("settings.license.perk2")}</span
                >
            </div>
            <div class="flex gap-3">
                <CheckCircle2 class="w-4 h-4 text-green-500 mt-0.5 shrink-0" />
                <span class="text-sm text-zinc-300"
                    >{$t("settings.license.perk3")}</span
                >
            </div>
        </div>
    </div>
</div>
