<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { rtdStore } from "$lib/stores/rtd.svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import {
        ChevronRight,
        ChevronLeft,
        User,
        CheckCircle2,
        Rocket,
        Sparkles,
        Key,
        Upload,
        AlertCircle,
        Wallet,
        Database,
        Globe,
        LineChart,
        Moon,
        Sun,
    } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { fade, fly, slide, scale } from "svelte/transition";
    import { t, locale } from "svelte-i18n";
    import * as Select from "$lib/components/ui/select";
    import { open } from "@tauri-apps/plugin-dialog";
    import { readTextFile } from "@tauri-apps/plugin-fs";
    import { validateLicenseKey } from "$lib/utils/license";
    import RTDImportDialog from "$lib/components/settings/RTDImportDialog.svelte";
    import { setMode, mode } from "mode-watcher";

    let { onComplete } = $props();

    let step = $state(
        typeof window !== "undefined"
            ? Number(sessionStorage.getItem("wizard_step") || 1)
            : 1,
    );
    $effect(() => {
        if (typeof window !== "undefined") {
            sessionStorage.setItem("wizard_step", step.toString());
        }
    });

    let totalSteps = 9;
    let isImportOpen = $state(false);

    const CURRENCIES = [
        { value: "BRL", label: "Real (R$)" },
        { value: "USD", label: "Dólar ($)" },
        { value: "USDT", label: "Tether" },
    ];

    const LANGUAGES = [
        { value: "pt-BR", label: "🇧🇷 Português (BR)" },
        { value: "en", label: "🇺🇸 English" },
        { value: "es-ES", label: "🇪🇸 Español" },
        { value: "fr-FR", label: "🇫🇷 Français" },
    ];

    const MARKETS = [
        { id: "M1", name: "B3 (Brasil)", timezone: "America/Sao_Paulo" },
        { id: "M2", name: "NYSE (EUA)", timezone: "America/New_York" },
        { id: "M3", name: "NASDAQ (EUA)", timezone: "America/New_York" },
        { id: "M4", name: "CME Group", timezone: "America/Chicago" },
        { id: "M5", name: "Forex", timezone: "UTC" },
        { id: "M6", name: "Cripto", timezone: "UTC" },
    ];

    const ASSET_TYPES = [
        { id: "T1", name: "Ações BR", market_id: "M1" },
        { id: "T2", name: "Opções BR", market_id: "M1" },
        { id: "T3", name: "Futuros BR", market_id: "M1" },
        { id: "T4", name: "FIIs", market_id: "M1" },
        { id: "T5", name: "Stocks US (NYSE)", market_id: "M2" },
        { id: "T5B", name: "Stocks US (NASDAQ)", market_id: "M3" },
        { id: "T6", name: "Forex Pairs", market_id: "M5" },
        { id: "T7", name: "Cryptocurrencies", market_id: "M6" },
    ];

    let selectedLanguage = $state($locale || "pt-BR");
    let selectedTheme = $state(mode.current || "dark");

    function applyLanguage(lang: string) {
        selectedLanguage = lang;
        locale.set(lang);
        localStorage.setItem("locale", lang);
    }

    function applyTheme(t: "dark" | "light") {
        selectedTheme = t;
        // Update store immediately so layout global syncs correctly
        settingsStore.userProfile.theme = t;
        setMode(t);
    }

    async function pickAndConnectRTD() {
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
                console.log("[Wizard] Invoking start_rtd_monitor_cmd...");
                await invoke("start_rtd_monitor_cmd", {
                    excelPath: settingsStore.rtdExcelPath || null,
                });
                console.log("[Wizard] Monitor cmd invoked successfully.");
                settingsStore.setRtdEnabled(true);
                toast.success("RTD Conectado! Lendo ativos...", {
                    id: "rtd-load",
                });

                // Increase delay and log
                console.log(
                    "[Wizard] Waiting for first quotes to open dialog...",
                );
                setTimeout(() => {
                    console.log(
                        "[Wizard] Opening RTD Import Dialog (isImportOpen = true)",
                    );
                    isImportOpen = true;
                }, 1000);
            }
        } catch (e) {
            console.error("[Wizard] RTD Error:", e);
            toast.error("Erro ao configurar arquivo RTD.", { id: "rtd-load" });
        }
    }

    let loading = $state(false);
    let progress = $state(0);

    let formData = $state({
        name: settingsStore.userProfile.name || "",
        email: settingsStore.userProfile.email || "",
        password: "",
        confirmPassword: "",
        licenseKey: "",
        mainCurrency: "BRL",
        selectedCurrencies: ["BRL", "USD", "USDT"],
        selectedMarkets: ["M1"],
        selectedAssetTypes: ["T1", "T3"],
    });

    let recoveryKey = $state("");

    function generateKey() {
        const chars = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let result = "";
        for (let i = 0; i < 16; i++) {
            if (i > 0 && i % 4 === 0) result += "-";
            result += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        return result;
    }

    let isTestAccount = $derived(
        formData.email.trim().toLowerCase() === "teste@traderlog.com" ||
            formData.email.trim().toLowerCase() === "test@traderlog.com",
    );

    let validatingLicense = $state(false);
    let licenseUploaded = $state(false);
    let licensePlanName = $state("");

    async function handleLicenseUpload() {
        try {
            const selected = await open({
                multiple: false,
                filters: [
                    { name: "Licença TraderLog Pro", extensions: ["lic"] },
                ],
            });

            if (selected && typeof selected === "string") {
                validatingLicense = true;
                const text = await readTextFile(selected);
                const rawKey = text.trim();
                const result = await validateLicenseKey(
                    rawKey,
                    settingsStore.hardwareId || "",
                );

                if (result.valid) {
                    formData.licenseKey = rawKey;
                    licenseUploaded = true;
                    licensePlanName = result.plan;
                    toast.success("Licença verificada com sucesso!");
                } else {
                    toast.error(result.error || "Licença inválida.");
                }
            }
        } catch (e) {
            console.error("License upload error:", e);
            toast.error("Erro ao ler arquivo de licença.");
        } finally {
            validatingLicense = false;
        }
    }

    async function handleComplete() {
        loading = true;
        progress = 10;

        try {
            // 1. Prepare Store with all wizard data
            settingsStore.userProfile.onboarding_completed = true;
            settingsStore.userProfile.name = formData.name || "Trader";
            settingsStore.userProfile.email = formData.email;
            settingsStore.userProfile.main_currency = formData.mainCurrency;
            settingsStore.userProfile.language = selectedLanguage;
            settingsStore.userProfile.theme = selectedTheme;
            settingsStore.userProfile.password_hash = formData.password;
            settingsStore.userProfile.recovery_hash = recoveryKey;

            if (formData.licenseKey) {
                settingsStore.userProfile.license_key = formData.licenseKey;
            }

            // 2. Local State for UI immediate feedback
            localStorage.setItem("isLoggedIn", "true");
            localStorage.setItem("onboarding_done", "true");
            settingsStore.isLoggedIn = true;

            progress = 30;

            // 3. Process Relations (Currencies, Markets, Asset Types)
            const formattedMarkets = formData.selectedMarkets.map(
                (m) => `markets:${m.toLowerCase()}`,
            );
            const formattedAssetTypes = formData.selectedAssetTypes.map(
                (t) => `asset_types:${t.toLowerCase()}`,
            );

            await invoke("finish_custom_onboarding", {
                currencies: formData.selectedCurrencies,
                markets: formattedMarkets,
                assetTypes: formattedAssetTypes,
            });

            progress = 70;

            // 4. Final Atomic Save of User Profile
            await invoke("save_user_profile", {
                profile: {
                    ...$state.snapshot(settingsStore.userProfile),
                    onboarding_completed: true, // Triple ensure
                },
            });

            if (formData.licenseKey) {
                await settingsStore.refreshLicenseStatus();
            }

            progress = 100;
            toast.success("Configuração concluída!");
            progress = 100;
            toast.success("Configuração concluída!");

            // RE-FETCH MUST BE AWAITED before completion
            await Promise.all([
                settingsStore.loadData(),
                tradesStore.loadTrades(),
            ]);

            setTimeout(() => {
                loading = false;
                sessionStorage.removeItem("wizard_step");
                onComplete();
            }, 800);
        } catch (error) {
            console.error("Onboarding Error:", error);
            toast.error("Erro durante a configuração.");
            // Rollback if critical failure (optional, but safer to let user retry)
            settingsStore.userProfile.onboarding_completed = false;
            loading = false;
            progress = 0;
        }
    }

    function nextStep() {
        if (step === 2) {
            if (!formData.name.trim()) {
                toast.error("Informe seu nome.");
                return;
            }
            if (!isTestAccount) {
                if (!formData.password) {
                    toast.error("Defina uma senha.");
                    return;
                }
                if (formData.password !== formData.confirmPassword) {
                    toast.error("Senhas não coincidem.");
                    return;
                }
                if (!recoveryKey) recoveryKey = generateKey();
            }
        }
        if (step < totalSteps) step++;
    }

    function prevStep() {
        if (step > 1) step--;
    }
</script>

<div
    class="fixed inset-0 z-[40] bg-background/80 backdrop-blur-md flex items-center justify-center p-4 selection:bg-primary/30"
    in:fade={{ duration: 400 }}
>
    <!-- Modal Container -->
    <div
        class="bg-card w-full max-w-5xl max-h-[95vh] overflow-hidden rounded-[2.5rem] shadow-2xl border border-border/50 flex flex-col relative"
        in:scale={{ start: 0.95, duration: 400, delay: 100 }}
    >
        <!-- Navbar Superior Interna -->
        <header
            class="border-b border-border bg-card/50 backdrop-blur-md flex-shrink-0"
        >
            <div
                class="max-w-5xl mx-auto px-6 h-16 flex items-center justify-between"
            >
                <div class="flex items-center gap-3">
                    <div class="bg-primary/10 p-2 rounded-lg">
                        <img src="/branding/navbar-logo.png" alt="Logo" class="w-5 h-5 object-contain" />
                    </div>
                    <div>
                        <h2
                            class="text-base font-black font-display tracking-tight leading-none uppercase"
                        >
                            TraderLog Pro
                        </h2>
                        <p
                            class="text-[9px] text-muted-foreground uppercase tracking-widest font-black opacity-70"
                        >
                            Setup Engine v1.0
                        </p>
                    </div>
                </div>

                <div class="flex items-center gap-4">
                    <div class="text-right hidden sm:block">
                        <p
                            class="text-[9px] font-black text-muted-foreground uppercase tracking-widest"
                        >
                            {$t("general.progress")}
                        </p>
                        <p class="text-xs font-bold">
                            {step}
                            {$t("calendar.of")}
                            {totalSteps}
                        </p>
                    </div>
                    <div
                        class="w-20 h-1.5 bg-muted rounded-full overflow-hidden"
                    >
                        <div
                            class="h-full bg-primary transition-all duration-700 ease-out"
                            style="width: {(step / totalSteps) * 100}%"
                        ></div>
                    </div>
                </div>
            </div>
        </header>

        <main class="flex-1 overflow-y-auto bg-card/10 custom-scrollbar">
            <!-- Main Container Content -->
            <div
                class="max-w-4xl mx-auto px-4 py-8 md:py-12 min-h-full flex flex-col"
            >
                <!-- Step Content -->
                <div
                    class="flex-1 bg-card border border-border/50 rounded-3xl shadow-sm overflow-hidden flex flex-col"
                >
                    <!-- Step Content -->
                    <div class="p-8 md:p-12 flex-1">
                        {#if step === 1}
                            <div
                                in:fly={{ y: 20, duration: 400 }}
                                out:fade={{ duration: 200 }}
                                class="space-y-10"
                            >
                                <div
                                    class="flex flex-col items-center text-center space-y-4"
                                >
                                    <div
                                        class="w-16 h-16 bg-primary/10 rounded-2xl flex items-center justify-center rotate-3"
                                    >
                                        <Sparkles
                                            class="w-8 h-8 text-primary animate-pulse"
                                        />
                                    </div>
                                    <div class="space-y-2">
                                        <h1
                                            class="text-3xl font-black tracking-tighter sm:text-4xl text-gradient"
                                        >
                                            {$t("setup.wizard.welcome.title")}
                                        </h1>
                                        <p
                                            class="text-muted-foreground text-sm max-w-md mx-auto leading-relaxed"
                                        >
                                            {$t(
                                                "setup.wizard.welcome.description",
                                            )}
                                        </p>
                                    </div>
                                </div>

                                <div
                                    class="w-full max-w-lg mx-auto pt-4 border-t border-border/50"
                                >
                                    <p
                                        class="text-[10px] font-black uppercase tracking-[0.2em] text-center text-muted-foreground mb-4"
                                    >
                                        {$t(
                                            "setup.wizard.welcome.select_language",
                                        )}
                                    </p>
                                    <div class="grid grid-cols-2 gap-2">
                                        {#each LANGUAGES as lang}
                                            <button
                                                type="button"
                                                onclick={() =>
                                                    applyLanguage(lang.value)}
                                                class="px-4 py-3 rounded-xl border text-xs font-bold transition-all {selectedLanguage ===
                                                lang.value
                                                    ? 'border-primary bg-primary/10 text-primary shadow-sm'
                                                    : 'border-border bg-muted/20 text-muted-foreground hover:border-primary/50'}"
                                            >
                                                {lang.label}
                                            </button>
                                        {/each}
                                    </div>
                                </div>

                                <div
                                    class="w-full max-w-lg mx-auto pt-4 border-t border-border/50"
                                >
                                    <p
                                        class="text-[10px] font-black uppercase tracking-[0.2em] text-center text-muted-foreground mb-4"
                                    >
                                        SELECIONE O TEMA VISUAL
                                    </p>
                                    <div class="grid grid-cols-2 gap-4">
                                        <button
                                            type="button"
                                            onclick={() => applyTheme("dark")}
                                            class="p-4 rounded-xl border flex flex-col items-center gap-2 transition-all {selectedTheme ===
                                            'dark'
                                                ? 'border-primary bg-primary/10 text-primary shadow-sm scale-[1.02]'
                                                : 'border-border bg-muted/20 text-muted-foreground hover:border-primary/50'}"
                                        >
                                            <Moon class="w-5 h-5" />
                                            <span
                                                class="text-[10px] font-black uppercase"
                                                >{$t(
                                                    "setup.wizard.welcome.dark",
                                                )}</span
                                            >
                                        </button>
                                        <button
                                            type="button"
                                            onclick={() => applyTheme("light")}
                                            class="p-4 rounded-xl border flex flex-col items-center gap-2 transition-all {selectedTheme ===
                                            'light'
                                                ? 'border-primary bg-primary/10 text-primary shadow-sm scale-[1.02]'
                                                : 'border-border bg-muted/20 text-muted-foreground hover:border-primary/50'}"
                                        >
                                            <Sun class="w-5 h-5" />
                                            <span
                                                class="text-[10px] font-black uppercase"
                                                >{$t(
                                                    "setup.wizard.welcome.light",
                                                )}</span
                                            >
                                        </button>
                                    </div>
                                </div>
                            </div>
                        {:else if step === 2}
                            <div
                                in:fly={{ y: 20, duration: 400 }}
                                out:fade={{ duration: 200 }}
                                class="space-y-8 max-w-2xl mx-auto"
                            >
                                <div class="space-y-1 text-center sm:text-left">
                                    <h2
                                        class="text-2xl font-black tracking-tight flex items-center justify-center sm:justify-start gap-3"
                                    >
                                        <User class="w-6 h-6 text-primary" />
                                        {$t("setup.wizard.profile.title")}
                                    </h2>
                                    <p class="text-sm text-muted-foreground">
                                        {$t("setup.wizard.profile.description")}
                                    </p>
                                </div>

                                <div class="grid gap-5 pt-2">
                                    <div class="space-y-2">
                                        <Label
                                            class="text-[10px] font-black uppercase tracking-widest"
                                            >{$t(
                                                "setup.wizard.common.name",
                                            )}</Label
                                        >
                                        <Input
                                            bind:value={formData.name}
                                            placeholder={$t(
                                                "setup.wizard.common.name_placeholder",
                                            )}
                                            class="h-11"
                                        />
                                    </div>
                                    <div class="space-y-2">
                                        <Label
                                            class="text-[10px] font-black uppercase tracking-widest"
                                            >E-mail</Label
                                        >
                                        <Input
                                            type="email"
                                            bind:value={formData.email}
                                            placeholder="seu@email.com"
                                            class="h-11"
                                        />
                                    </div>

                                    <div
                                        class="pt-6 grid sm:grid-cols-2 gap-4 border-t border-border mt-2"
                                    >
                                        <div class="space-y-2">
                                            <Label
                                                class="text-[10px] font-black uppercase tracking-widest text-primary"
                                                >{$t(
                                                    "setup.wizard.common.password",
                                                )}</Label
                                            >
                                            <Input
                                                type="password"
                                                bind:value={formData.password}
                                                class="h-11"
                                            />
                                        </div>
                                        <div class="space-y-2">
                                            <Label
                                                class="text-[10px] font-black uppercase tracking-widest text-primary"
                                                >Confirmar Senha</Label
                                            >
                                            <Input
                                                type="password"
                                                bind:value={
                                                    formData.confirmPassword
                                                }
                                                class="h-11"
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>
                        {:else if step === 3}
                            <div
                                in:fly={{ y: 20, duration: 400 }}
                                out:fade={{ duration: 200 }}
                                class="space-y-8 max-w-xl mx-auto text-center"
                            >
                                <div class="space-y-2">
                                    <div
                                        class="mx-auto w-14 h-14 bg-rose-500/10 rounded-2xl flex items-center justify-center"
                                    >
                                        <AlertCircle
                                            class="w-7 h-7 text-rose-500"
                                        />
                                    </div>
                                    <h2
                                        class="text-2xl font-black tracking-tight text-rose-500 uppercase"
                                    >
                                        {$t(
                                            "setup.wizard.common.recovery_key_title",
                                        )}
                                    </h2>
                                    <p class="text-sm text-muted-foreground">
                                        {$t(
                                            "setup.wizard.common.recovery_key_desc",
                                        )}
                                    </p>
                                </div>

                                <div
                                    role="button"
                                    tabindex="0"
                                    class="p-10 border border-rose-500/30 rounded-3xl bg-rose-500/5 space-y-4 cursor-pointer hover:bg-rose-500/10 transition-all group relative overflow-hidden"
                                    onclick={() => {
                                        navigator.clipboard.writeText(
                                            recoveryKey,
                                        );
                                        toast.success("Copiado!");
                                    }}
                                    onkeydown={(e) => {
                                        if (
                                            e.key === "Enter" ||
                                            e.key === " "
                                        ) {
                                            navigator.clipboard.writeText(
                                                recoveryKey,
                                            );
                                            toast.success("Copiado!");
                                        }
                                    }}
                                >
                                    <div
                                        class="absolute inset-0 bg-gradient-to-br from-rose-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"
                                    ></div>
                                    <p
                                        class="text-[10px] font-black uppercase tracking-[0.3em] text-rose-400"
                                    >
                                        {$t("setup.wizard.common.master_key")}
                                    </p>
                                    <div
                                        class="text-2xl sm:text-3xl font-mono font-black tracking-[0.2em] text-rose-950 dark:text-white tabular-nums relative z-10"
                                    >
                                        {recoveryKey}
                                    </div>
                                    <p
                                        class="text-[10px] text-rose-400 font-bold uppercase tracking-widest opacity-60"
                                    >
                                        {$t(
                                            "setup.wizard.common.copy_to_clipboard",
                                        )}
                                    </p>
                                </div>

                                <div
                                    class="bg-destructive/10 border border-destructive/20 p-4 rounded-xl text-left"
                                >
                                    <p
                                        class="text-xs text-destructive font-black leading-relaxed"
                                    >
                                        {$t("setup.wizard.common.never_share")}
                                    </p>
                                </div>
                            </div>
                        {:else if step === 4}
                            <div
                                in:fly={{ y: 20, duration: 400 }}
                                out:fade={{ duration: 200 }}
                                class="space-y-10 max-w-2xl mx-auto text-center"
                            >
                                <div class="space-y-2">
                                    <div
                                        class="mx-auto w-14 h-14 bg-primary/10 rounded-2xl flex items-center justify-center"
                                    >
                                        <Key class="w-7 h-7 text-primary" />
                                    </div>
                                    <h2
                                        class="text-2xl font-black tracking-tight"
                                    >
                                        {$t(
                                            "setup.wizard.common.activation_title",
                                        )}
                                    </h2>
                                    <p
                                        class="text-sm text-muted-foreground whitespace-pre-line leading-relaxed"
                                    >
                                        {$t(
                                            "setup.wizard.common.activation_desc",
                                        )}
                                    </p>
                                </div>

                                <div
                                    class="p-12 border-2 rounded-3xl bg-muted/5 border-dashed border-border flex flex-col items-center space-y-8"
                                >
                                    {#if licenseUploaded}
                                        <div
                                            class="bg-green-500/20 p-5 rounded-3xl"
                                        >
                                            <CheckCircle2
                                                class="w-14 h-14 text-green-500"
                                            />
                                        </div>
                                        <div class="space-y-2">
                                            <h3
                                                class="font-black text-2xl text-green-500 uppercase"
                                            >
                                                {$t(
                                                    "setup.wizard.common.active_license",
                                                )}
                                            </h3>
                                            <p
                                                class="text-xs font-black text-muted-foreground uppercase tracking-widest bg-muted/50 px-4 py-1.5 rounded-full inline-block"
                                            >
                                                {$t(
                                                    "setup.wizard.common.plan",
                                                )}: {licensePlanName}
                                            </p>
                                        </div>
                                    {:else}
                                        <Button
                                            variant="outline"
                                            class="h-16 px-12 rounded-2xl border-primary bg-primary/5 hover:bg-primary/10 font-black text-xs uppercase tracking-widest transition-all shadow-lg shadow-primary/5"
                                            disabled={validatingLicense}
                                            onclick={handleLicenseUpload}
                                        >
                                            {#if validatingLicense}
                                                <span class="animate-pulse"
                                                    >{$t(
                                                        "setup.wizard.common.configuring",
                                                    )}</span
                                                >
                                            {:else}
                                                <Upload class="w-5 h-5 mr-3" />
                                                {$t(
                                                    "setup.wizard.common.import_lic",
                                                )}
                                            {/if}
                                        </Button>
                                        <div class="space-y-1">
                                            <p
                                                class="text-[10px] text-muted-foreground font-black uppercase tracking-widest"
                                            >
                                                {$t(
                                                    "setup.wizard.common.no_license_title",
                                                )}
                                            </p>
                                            <p
                                                class="text-[9px] text-muted-foreground/60 uppercase"
                                            >
                                                {$t(
                                                    "setup.wizard.common.no_license_desc",
                                                )}
                                            </p>
                                        </div>
                                    {/if}
                                </div>
                            </div>
                        {:else if step === 5}
                            <div
                                in:fly={{ y: 20, duration: 400 }}
                                out:fade={{ duration: 200 }}
                                class="space-y-8 max-w-2xl mx-auto"
                            >
                                <div class="space-y-1">
                                    <h2
                                        class="text-2xl font-black tracking-tight flex items-center gap-3"
                                    >
                                        <Wallet class="w-7 h-7 text-primary" />
                                        {$t(
                                            "setup.wizard.common.currency_title",
                                        )}
                                    </h2>
                                    <p
                                        class="text-sm text-muted-foreground leading-relaxed"
                                    >
                                        {$t(
                                            "setup.wizard.common.currency_desc",
                                        )}
                                    </p>
                                </div>

                                <div
                                    class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-4"
                                >
                                    {#each CURRENCIES as curr}
                                        <button
                                            type="button"
                                            onclick={() => {
                                                formData.mainCurrency =
                                                    curr.value;
                                                if (
                                                    !formData.selectedCurrencies.includes(
                                                        curr.value,
                                                    )
                                                )
                                                    formData.selectedCurrencies.push(
                                                        curr.value,
                                                    );
                                            }}
                                            class="p-6 border-2 rounded-2xl flex flex-col items-center gap-3 transition-all {formData.mainCurrency ===
                                            curr.value
                                                ? 'border-primary bg-primary/10 text-primary scale-[1.03] shadow-lg shadow-primary/5'
                                                : 'border-border bg-muted/10 text-muted-foreground hover:bg-muted/20'}"
                                        >
                                            <span class="text-2xl font-black"
                                                >{curr.value}</span
                                            >
                                            <span
                                                class="text-[10px] font-black uppercase tracking-widest opacity-60"
                                                >{curr.label}</span
                                            >
                                        </button>
                                    {/each}
                                </div>
                            </div>
                        {:else if step === 6}
                            <div
                                in:fly={{ y: 20, duration: 400 }}
                                out:fade={{ duration: 200 }}
                                class="space-y-8"
                            >
                                <div class="text-center space-y-2">
                                    <h2
                                        class="text-2xl font-black tracking-tight"
                                    >
                                        {$t(
                                            "setup.wizard.common.markets_title",
                                        )}
                                    </h2>
                                    <p class="text-sm text-muted-foreground">
                                        {$t("setup.wizard.common.markets_desc")}
                                    </p>
                                </div>

                                <div
                                    class="grid grid-cols-2 md:grid-cols-3 gap-3"
                                >
                                    {#each MARKETS as mkt}
                                        <button
                                            type="button"
                                            onclick={() => {
                                                if (
                                                    formData.selectedMarkets.includes(
                                                        mkt.id,
                                                    )
                                                ) {
                                                    if (
                                                        formData.selectedMarkets
                                                            .length > 1
                                                    )
                                                        formData.selectedMarkets =
                                                            formData.selectedMarkets.filter(
                                                                (m) =>
                                                                    m !==
                                                                    mkt.id,
                                                            );
                                                } else {
                                                    formData.selectedMarkets = [
                                                        ...formData.selectedMarkets,
                                                        mkt.id,
                                                    ];
                                                }
                                            }}
                                            class="p-5 border rounded-2xl flex flex-col gap-2 transition-all {formData.selectedMarkets.includes(
                                                mkt.id,
                                            )
                                                ? 'bg-primary/10 border-primary text-primary shadow-sm'
                                                : 'bg-card border-border text-muted-foreground hover:bg-muted/20'}"
                                        >
                                            <span
                                                class="text-sm font-black uppercase tracking-tight"
                                                >{$t(
                                                    `setup.wizard.common.markets.${mkt.id}`,
                                                )}</span
                                            >
                                            <span
                                                class="text-[10px] font-mono opacity-50"
                                                >{mkt.timezone}</span
                                            >
                                        </button>
                                    {/each}
                                </div>
                            </div>
                        {:else if step === 7}
                            <div
                                in:fly={{ y: 20, duration: 400 }}
                                out:fade={{ duration: 200 }}
                                class="space-y-8"
                            >
                                <div class="space-y-1">
                                    <h2
                                        class="text-2xl font-black tracking-tight"
                                    >
                                        {$t(
                                            "setup.wizard.common.asset_types_title",
                                        )}
                                    </h2>
                                    <p class="text-sm text-muted-foreground">
                                        {$t(
                                            "setup.wizard.common.asset_types_desc",
                                        )}
                                    </p>
                                </div>

                                <div
                                    class="grid grid-cols-2 sm:grid-cols-4 gap-2"
                                >
                                    {#each ASSET_TYPES as type}
                                        {#if formData.selectedMarkets.some((m) => type.market_id === m)}
                                            <button
                                                type="button"
                                                onclick={() => {
                                                    if (
                                                        formData.selectedAssetTypes.includes(
                                                            type.id,
                                                        )
                                                    ) {
                                                        formData.selectedAssetTypes =
                                                            formData.selectedAssetTypes.filter(
                                                                (i) =>
                                                                    i !==
                                                                    type.id,
                                                            );
                                                    } else {
                                                        formData.selectedAssetTypes =
                                                            [
                                                                ...formData.selectedAssetTypes,
                                                                type.id,
                                                            ];
                                                    }
                                                }}
                                                class="px-4 py-4 border rounded-2xl text-xs font-black transition-all uppercase tracking-tight text-center {formData.selectedAssetTypes.includes(
                                                    type.id,
                                                )
                                                    ? 'bg-primary text-primary-foreground border-primary shadow-md'
                                                    : 'bg-card border-border text-muted-foreground hover:bg-muted/50'}"
                                            >
                                                {$t(
                                                    `setup.wizard.common.assets.${type.id}`,
                                                )}
                                            </button>
                                        {/if}
                                    {/each}
                                </div>
                            </div>
                        {:else if step === 8}
                            <div
                                in:fly={{ y: 20, duration: 400 }}
                                out:fade={{ duration: 200 }}
                                class="space-y-10 max-w-2xl mx-auto text-center"
                            >
                                <div class="space-y-2">
                                    <h2
                                        class="text-2xl font-black tracking-tight flex items-center justify-center gap-3"
                                    >
                                        <LineChart
                                            class="w-7 h-7 text-emerald-500"
                                        />
                                        {$t("setup.wizard.common.rtd_title")}
                                    </h2>
                                    <p class="text-sm text-muted-foreground">
                                        {$t("setup.wizard.common.rtd_desc")}
                                    </p>
                                </div>

                                <div
                                    class="grid sm:grid-cols-2 gap-6 items-stretch"
                                >
                                    <div
                                        class="p-8 border-2 rounded-3xl bg-emerald-500/5 border-emerald-500/20 flex flex-col justify-between space-y-6"
                                    >
                                        <div class="space-y-4">
                                            <div
                                                class="w-12 h-12 bg-emerald-500/20 rounded-2xl flex items-center justify-center mx-auto"
                                            >
                                                <Database
                                                    class="w-6 h-6 text-emerald-500"
                                                />
                                            </div>
                                            <h3
                                                class="font-black text-xs uppercase tracking-widest"
                                            >
                                                {$t(
                                                    "setup.wizard.common.rtd_active_now",
                                                )}
                                            </h3>
                                            <p
                                                class="text-[10px] text-muted-foreground leading-relaxed"
                                            >
                                                {$t(
                                                    "setup.wizard.common.rtd_active_desc",
                                                )}
                                            </p>
                                        </div>
                                        <Button
                                            class="w-full bg-emerald-600 hover:bg-emerald-700 font-black text-[10px] h-11"
                                            onclick={pickAndConnectRTD}
                                        >
                                            {$t(
                                                "setup.wizard.common.rtd_select_sheet",
                                            )}
                                        </Button>
                                    </div>

                                    <div
                                        class="p-8 border-2 rounded-3xl bg-muted/10 border-border flex flex-col justify-between space-y-6"
                                    >
                                        <div class="space-y-4">
                                            <div
                                                class="w-12 h-12 bg-muted rounded-2xl flex items-center justify-center mx-auto"
                                            >
                                                <Sparkles
                                                    class="w-6 h-6 text-muted-foreground"
                                                />
                                            </div>
                                            <h3
                                                class="font-black text-xs uppercase tracking-widest text-muted-foreground"
                                            >
                                                {$t(
                                                    "setup.wizard.common.rtd_later_title",
                                                )}
                                            </h3>
                                            <p
                                                class="text-[10px] text-muted-foreground leading-relaxed"
                                            >
                                                {$t(
                                                    "setup.wizard.common.rtd_later_desc",
                                                )}
                                            </p>
                                        </div>
                                        <Button
                                            variant="outline"
                                            class="w-full font-black text-[10px] h-11"
                                            onclick={() => nextStep()}
                                        >
                                            {$t("setup.wizard.common.rtd_skip")}
                                        </Button>
                                    </div>
                                </div>
                            </div>
                        {:else if step === 9}
                            <div
                                in:scale={{ start: 0.9, duration: 500 }}
                                class="flex flex-col items-center text-center space-y-8 py-12"
                            >
                                <div
                                    class="w-24 h-24 bg-primary/10 rounded-full flex items-center justify-center relative"
                                >
                                    <img src="/branding/logo.png" alt="Logo" class="w-12 h-12 object-contain animate-bounce" />
                                    <div
                                        class="absolute inset-0 rounded-full border-4 border-primary/20 border-t-primary animate-spin"
                                    ></div>
                                </div>
                                <div class="space-y-2">
                                    <h2
                                        class="text-4xl font-black tracking-tighter uppercase"
                                    >
                                        {$t(
                                            "setup.wizard.common.mission_ready",
                                        )}
                                    </h2>
                                    <p
                                        class="text-muted-foreground text-sm max-w-xs mx-auto"
                                    >
                                        {$t(
                                            "setup.wizard.common.mission_ready_desc",
                                        )}
                                    </p>
                                </div>
                                <Button
                                    onclick={handleComplete}
                                    disabled={loading}
                                    class="h-16 px-16 rounded-2xl bg-primary font-black text-sm uppercase tracking-widest shadow-xl shadow-primary/20 hover:scale-105 transition-all"
                                >
                                    {#if loading}
                                        <span class="animate-pulse"
                                            >{$t(
                                                "setup.wizard.common.configuring",
                                            )}
                                            {progress}%</span
                                        >
                                    {:else}
                                        {$t(
                                            "setup.wizard.common.start_journey",
                                        )}
                                        <ChevronRight class="w-5 h-5 ml-3" />
                                    {/if}
                                </Button>
                            </div>
                        {/if}
                    </div>

                    <!-- Footer do Card -->
                    {#if step < totalSteps && !loading}
                        <div
                            class="border-t border-border bg-card/40 px-8 py-6 flex justify-between items-center"
                        >
                            <Button
                                variant="ghost"
                                onclick={prevStep}
                                disabled={step === 1}
                                class="h-10 px-8 font-black text-[10px] uppercase tracking-[0.2em] gap-2 transition-all {step ===
                                1
                                    ? 'opacity-0'
                                    : ''}"
                            >
                                <ChevronLeft class="w-4 h-4" />
                                {$t("setup.wizard.nav.back")}
                            </Button>

                            <Button
                                onclick={nextStep}
                                class="h-10 px-10 rounded-xl font-black text-[10px] uppercase tracking-[0.2em] gap-2 shadow-premium hover:shadow-primary/20 transition-all"
                            >
                                {#if step === 4 && !licenseUploaded}
                                    {$t("setup.wizard.nav.skipTrial")}
                                {:else}
                                    {$t("calendar.next") || "PRÓXIMO"}
                                {/if}
                                <ChevronRight class="w-4 h-4 ml-1" />
                            </Button>
                        </div>
                    {/if}
                </div>
            </div>
        </main>

        <footer
            class="py-4 text-center bg-background/50 border-t border-border/30"
        >
            <p
                class="text-[8px] font-black text-muted-foreground uppercase tracking-[0.4em] opacity-40"
            >
                © 2026 TRADERLOG PRO · JOURNAL ENGINE
            </p>
        </footer>
    </div>
</div>

<RTDImportDialog bind:open={isImportOpen} />

<style>
    .text-gradient {
        background: linear-gradient(
            to bottom right,
            hsl(var(--foreground)),
            hsl(var(--foreground) / 0.6)
        );
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
    }
</style>
