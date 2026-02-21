<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import {
        ChevronRight,
        ChevronLeft,
        User,
        Settings,
        Database,
        CheckCircle2,
        Rocket,
        Sparkles,
        LayoutDashboard,
        Globe,
        Coins,
        ChevronDown,
        ChevronUp,
    } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { fade, fly, slide } from "svelte/transition";
    import { t, locale } from "svelte-i18n";

    let { onComplete } = $props();

    let step = $state(1);
    let totalSteps = 7;
    let loading = $state(false);
    let progress = $state(0);
    let onboardingMeta = $state<any[]>([]);
    let expandedModules = $state<Record<string, boolean>>({});

    let formData = $state({
        name: "",
        main_currency: "BRL",
        language: "pt-BR",
        theme: "dark",
        setupMode: "standard", // "standard" | "custom"
        selectedItems: [] as string[], // IDs selecionados (ex: "markets:m1")
        selectedAccounts: [] as string[], // Contas selecionadas (ex: "account:simulador")
        generateDemo: true,
    });

    const coreAccounts = [
        {
            id: "account:real",
            label: "Conta Real",
            description: "Para suas operações reais",
        },
        {
            id: "account:simulador",
            label: "Conta Simulador",
            description: "Para treino e backtesting (Saldo: 100k)",
        },
        {
            id: "account:teste",
            label: "Conta Teste",
            description: "Sandbox para validação (Saldo: 100k)",
        },
    ];

    const marketAccountsMap: Record<string, { id: string; label: string }> = {
        "markets:m1": {
            id: "account:demo_b3_acoes",
            label: "Conta Demo B3 Ações",
        },
        // m1 also triggers futures, handle logic later or simplify
        "markets:m2": { id: "account:demo_forex", label: "Conta Demo Forex" },
        "markets:m3": { id: "account:demo_crypto", label: "Conta Demo Crypto" },
        "markets:m4": { id: "account:demo_nasdaq", label: "Conta Demo Nasdaq" },
        "markets:m5": { id: "account:demo_nasdaq", label: "Conta Demo Nasdaq" },
    };

    $inspect(formData).with((type, value) => {
        console.log(`[Wizard] formData ${type}:`, value);
    });

    onMount(async () => {
        console.log("[Wizard] Mounted. Current step:", step);
        try {
            const meta = (await invoke("get_onboarding_meta")) as any[];
            console.log("[Wizard] Meta loaded:", meta.length, "modules");
            onboardingMeta = meta;

            // Só inicializa o padrão se os itens selecionados estiverem vazios
            // Isso evita perda de estado se o componente remontar por qualquer motivo
            if (formData.selectedItems.length === 0) {
                console.log(
                    "[Wizard] Initializing default selections (Standard)",
                );
                formData.selectedItems = onboardingMeta.flatMap((m) => [
                    m.id,
                    ...m.items.map((i: any) => i.id),
                ]);
            } else {
                console.log(
                    "[Wizard] Preserving existing selections:",
                    formData.selectedItems.length,
                );
            }

            // Inicializa todos como não expandidos (apenas se ainda não estiverem)
            onboardingMeta.forEach((m) => {
                if (expandedModules[m.id] === undefined) {
                    expandedModules[m.id] = false;
                }
            });
        } catch (e) {
            console.error("Failed to load onboarding meta", e);
        }
    });

    const currencies = [
        { value: "BRL", label: "Real (R$)", symbol: "R$" },
        { value: "USD", label: "Dollar ($)", symbol: "$" },
        { value: "EUR", label: "Euro (€)", symbol: "€" },
    ];

    const languages = [
        { value: "pt-BR", label: "Português", flag: "🇧🇷" },
        { value: "en", label: "English", flag: "🇺🇸" },
    ];

    function toggleModuleExpansion(moduleId: string) {
        expandedModules[moduleId] = !expandedModules[moduleId];
    }

    function toggleModuleSelection(moduleId: string, items: any[]) {
        const itemIds = items.map((i) => i.id);
        const allSelected = itemIds.every((id) =>
            formData.selectedItems.includes(id),
        );

        if (allSelected) {
            // Deselecionar tudo do módulo
            formData.selectedItems = formData.selectedItems.filter(
                (id) => !itemIds.includes(id) && id !== moduleId,
            );
        } else {
            // Selecionar tudo do módulo
            formData.selectedItems = [
                ...new Set([...formData.selectedItems, moduleId, ...itemIds]),
            ];
        }
    }

    function toggleItemSelection(
        itemId: string,
        moduleId: string,
        items: any[],
    ) {
        if (formData.selectedItems.includes(itemId)) {
            formData.selectedItems = formData.selectedItems.filter(
                (id) => id !== itemId && id !== moduleId,
            );
        } else {
            formData.selectedItems = [...formData.selectedItems, itemId];
            // Se todos os itens do módulo foram selecionados, seleciona o módulo também
            const itemIds = items.map((i) => i.id);
            const allSelected = itemIds.every((id) =>
                formData.selectedItems.includes(id),
            );
            if (allSelected) {
                formData.selectedItems = [...formData.selectedItems, moduleId];
            }
        }
    }

    function toggleAccountSelection(accountId: string) {
        if (formData.selectedAccounts.includes(accountId)) {
            formData.selectedAccounts = formData.selectedAccounts.filter(
                (id) => id !== accountId,
            );
        } else {
            formData.selectedAccounts = [
                ...formData.selectedAccounts,
                accountId,
            ];
        }
    }

    // Valida e prepara as contas com base nos módulos selecionados
    function prepareAccountsStep() {
        if (step === 4) {
            // Coming from Module Selection
            // Pre-select core accounts if not already set (re-entering)
            if (formData.selectedAccounts.length === 0) {
                formData.selectedAccounts = coreAccounts.map((a) => a.id);
            }

            // Ensure market accounts are selected if modules are present
            // We don't remove them if module is unchecked to allow manual override?
            // No, let's sync them.
            for (const [marketId, acc] of Object.entries(marketAccountsMap)) {
                if (formData.selectedItems.includes(marketId)) {
                    if (!formData.selectedAccounts.includes(acc.id)) {
                        formData.selectedAccounts.push(acc.id);
                    }
                } else {
                    // Maybe remove if module unchecked? Let's leave it to user unless it was purely auto-added.
                    // For simplicity: If module is present, ensure account is selected.
                }
            }
        }
    }

    async function handleComplete() {
        loading = true;
        progress = 10;

        try {
            // 1. Salvar Perfil Básico
            await invoke("save_user_profile", {
                profile: {
                    ...settingsStore.userProfile,
                    name: formData.name || "Trader",
                    main_currency: formData.main_currency,
                    language: formData.language,
                    theme: formData.theme,
                },
            });
            progress = 30;

            // 2. Seeding (Dados configurais)
            if (formData.setupMode === "standard") {
                await invoke("ensure_defaults");
            } else {
                await invoke("seed_custom_data", {
                    modules: [
                        ...formData.selectedItems,
                        ...formData.selectedAccounts,
                    ], // Merge accounts into modules list for filtering
                });
            }
            progress = 70;

            // 3. Demo Data (opcional)
            if (formData.generateDemo) {
                progress = 70;
                console.log("[Wizard] Generating filtered demo data...");
                // Pass selected items as filter
                await invoke("seed_demo_data", {
                    modules: [
                        ...formData.selectedItems,
                        ...formData.selectedAccounts,
                    ],
                });
            }
            progress = 90;

            // 4. Marcar onboarding como concluído
            console.log("[Wizard] Completing onboarding in backend...");
            await invoke("complete_onboarding");
            progress = 100;

            console.log(
                "[Wizard] All steps finished. Triggering completion callback.",
            );
            toast.success("Configuração concluída!");

            // Proactively update local store flag to close UI immediately
            settingsStore.userProfile.onboarding_completed = true;

            setTimeout(() => {
                console.log("[Wizard] Calling onComplete...");
                loading = false;
                onComplete();
            }, 800);
        } catch (error) {
            console.error("Onboarding Error:", error);
            toast.error("Erro durante a configuração: " + (error as string));
            loading = false;
            progress = 0; // Reset progress to allow retry
        }
    }

    function nextStep() {
        console.log(`[Wizard] Next step: ${step} -> ${step + 1}`);
        if (step === 4) prepareAccountsStep();
        if (step < totalSteps) step++;
    }

    function prevStep() {
        console.log(`[Wizard] Prev step: ${step} -> ${step - 1}`);
        if (step > 1) step--;
    }
</script>

<div
    class="fixed inset-0 z-50 bg-background flex items-center justify-center p-4"
>
    <div class="max-w-2xl w-full">
        <!-- Progress Header -->
        <div class="mb-8 flex justify-between items-center">
            <div class="flex items-center gap-2">
                <div class="bg-primary/10 p-2 rounded-lg">
                    <Rocket class="w-6 h-6 text-primary" />
                </div>
                <div>
                    <h2 class="text-xl font-bold font-display tracking-tight">
                        TraderLog Pro
                    </h2>
                    <p
                        class="text-xs text-muted-foreground uppercase tracking-widest font-black"
                    >
                        Startup Wizard
                    </p>
                </div>
            </div>
            <div class="text-right">
                <span
                    class="text-xs font-bold text-muted-foreground uppercase tracking-widest"
                    >Passo {step} de {totalSteps}</span
                >
                <div
                    class="w-32 h-1.5 bg-muted rounded-full mt-1 overflow-hidden"
                >
                    <div
                        class="h-full bg-primary transition-all duration-500 ease-out"
                        style="width: {(step / totalSteps) * 100}%"
                    ></div>
                </div>
            </div>
        </div>

        <div class="relative min-h-[450px]">
            {#if step === 1}
                <div
                    in:fly={{ y: 20, duration: 500 }}
                    out:fade={{ duration: 200 }}
                    class="space-y-6"
                >
                    <div
                        class="flex flex-col items-center text-center space-y-4 py-8"
                    >
                        <div
                            class="w-20 h-20 bg-primary/20 rounded-full flex items-center justify-center mb-4"
                        >
                            <Sparkles
                                class="w-10 h-10 text-primary animate-pulse"
                            />
                        </div>
                        <h1 class="text-4xl font-black tracking-tighter">
                            Seja bem-vindo ao seu novo Hub de Performance.
                        </h1>
                        <p class="text-muted-foreground text-lg max-w-md">
                            Vamos configurar seu ambiente de trading em poucos
                            minutos. Você está pronto para subir de nível?
                        </p>
                    </div>
                </div>
            {:else if step === 2}
                <div
                    in:fly={{ y: 20, duration: 500 }}
                    out:fade={{ duration: 200 }}
                    class="space-y-6"
                >
                    <div class="space-y-2">
                        <h2
                            class="text-2xl font-black tracking-tight flex items-center gap-2"
                        >
                            <User class="w-6 h-6 text-primary" />
                            Quem é você?
                        </h2>
                        <p class="text-muted-foreground">
                            Como gostaria de ser chamado no app?
                        </p>
                    </div>

                    <div class="space-y-4 pt-4">
                        <div class="space-y-2">
                            <Label for="name" class="font-bold"
                                >Seu Nome ou Nickname</Label
                            >
                            <Input
                                id="name"
                                bind:value={formData.name}
                                placeholder="Ex: Trader Elite"
                                class="h-12 text-lg focus-visible:ring-primary"
                            />
                        </div>
                    </div>
                </div>
            {:else if step === 3}
                <div
                    in:fly={{ y: 20, duration: 500 }}
                    out:fade={{ duration: 200 }}
                    class="space-y-6"
                >
                    <div class="space-y-2">
                        <h2
                            class="text-2xl font-black tracking-tight flex items-center gap-2"
                        >
                            <Globe class="w-6 h-6 text-primary" />
                            Localização & Estilo
                        </h2>
                        <p class="text-muted-foreground">
                            Defina suas preferências visuais e regionais.
                        </p>
                    </div>

                    <div class="grid grid-cols-2 gap-4 pt-4">
                        <div class="space-y-2">
                            <Label class="font-bold">Moeda Principal</Label>
                            <select
                                bind:value={formData.main_currency}
                                class="flex h-12 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                            >
                                {#each currencies as curr}
                                    <option value={curr.value}
                                        >{curr.label}</option
                                    >
                                {/each}
                            </select>
                        </div>
                        <div class="space-y-2">
                            <Label class="font-bold">Idioma</Label>
                            <select
                                bind:value={formData.language}
                                class="flex h-12 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                            >
                                {#each languages as lang}
                                    <option value={lang.value}
                                        >{lang.flag} {lang.label}</option
                                    >
                                {/each}
                            </select>
                        </div>
                    </div>

                    <div class="space-y-2 pt-4">
                        <Label class="font-bold">Tema Visual</Label>
                        <div class="grid grid-cols-2 gap-4">
                            <button
                                class="p-4 border-2 rounded-xl transition-all flex flex-col items-center gap-2 {formData.theme ===
                                'dark'
                                    ? 'border-primary bg-primary/5'
                                    : 'border-muted hover:border-primary/50'}"
                                onclick={() => (formData.theme = "dark")}
                            >
                                <div
                                    class="w-full h-12 bg-zinc-900 rounded border border-zinc-700"
                                ></div>
                                <span
                                    class="text-sm font-bold uppercase tracking-widest"
                                    >Dark Mode</span
                                >
                            </button>
                            <button
                                class="p-4 border-2 rounded-xl transition-all flex flex-col items-center gap-2 {formData.theme ===
                                'light'
                                    ? 'border-primary bg-primary/5'
                                    : 'border-muted hover:border-primary/50'}"
                                onclick={() => (formData.theme = "light")}
                            >
                                <div
                                    class="w-full h-12 bg-zinc-100 rounded border border-zinc-300"
                                ></div>
                                <span
                                    class="text-sm font-bold uppercase tracking-widest"
                                    >Light Mode</span
                                >
                            </button>
                        </div>
                    </div>
                </div>
            {:else if step === 4}
                <div
                    in:fly={{ y: 20, duration: 500 }}
                    out:fade={{ duration: 200 }}
                    class="space-y-6"
                >
                    <div class="space-y-2">
                        <h2
                            class="text-2xl font-black tracking-tight flex items-center gap-2"
                        >
                            <Database class="w-6 h-6 text-primary" />
                            Instalação de Dados
                        </h2>
                        <p class="text-muted-foreground">
                            Como você deseja iniciar seu banco de dados?
                        </p>
                    </div>

                    <div class="grid gap-4 pt-4">
                        <button
                            class="p-5 border-2 rounded-xl transition-all text-left flex items-start gap-4 {formData.setupMode ===
                            'standard'
                                ? 'border-primary bg-primary/5'
                                : 'border-muted hover:border-primary/20'}"
                            onclick={() => (formData.setupMode = "standard")}
                        >
                            <div class="bg-primary/10 p-3 rounded-full">
                                <Rocket class="w-6 h-6 text-primary" />
                            </div>
                            <div class="flex-1">
                                <h3 class="font-black text-lg">
                                    Instalação Padrão (Recomendado)
                                </h3>
                                <p class="text-sm text-muted-foreground">
                                    Instala todos os mercados (B3, Forex,
                                    Crypto), estratégias e indicadores
                                    pré-configurados.
                                </p>
                            </div>
                            {#if formData.setupMode === "standard"}
                                <CheckCircle2 class="w-6 h-6 text-primary" />
                            {/if}
                        </button>

                        <button
                            class="p-5 border-2 rounded-xl transition-all text-left flex items-start gap-4 {formData.setupMode ===
                            'custom'
                                ? 'border-primary bg-primary/5'
                                : 'border-muted hover:border-primary/20'}"
                            onclick={() => (formData.setupMode = "custom")}
                        >
                            <div class="bg-primary/10 p-3 rounded-full">
                                <Settings class="w-6 h-6 text-primary" />
                            </div>
                            <div class="flex-1">
                                <h3 class="font-black text-lg">
                                    Instalação Personalizada
                                </h3>
                                <p class="text-sm text-muted-foreground">
                                    Escolha exatamente quais módulos e ativos
                                    você deseja importar agora.
                                </p>
                            </div>
                            {#if formData.setupMode === "custom"}
                                <CheckCircle2 class="w-6 h-6 text-primary" />
                            {/if}
                        </button>
                    </div>

                    {#if formData.setupMode === "custom"}
                        <div
                            class="space-y-3 max-h-[400px] overflow-y-auto pr-2 custom-scrollbar"
                            transition:fade
                        >
                            {#each onboardingMeta as module}
                                <div
                                    class="border rounded-xl bg-muted/20 overflow-hidden transition-all"
                                >
                                    <div
                                        class="flex items-center justify-between p-3 hover:bg-muted/40 transition-colors"
                                    >
                                        <div class="flex items-center gap-3">
                                            <Checkbox
                                                checked={module.items.every(
                                                    (i: any) =>
                                                        formData.selectedItems.includes(
                                                            i.id,
                                                        ),
                                                )}
                                                onCheckedChange={() =>
                                                    toggleModuleSelection(
                                                        module.id,
                                                        module.items,
                                                    )}
                                            />
                                            <button
                                                class="flex items-center gap-2 font-black text-sm uppercase tracking-wider"
                                                onclick={() =>
                                                    toggleModuleExpansion(
                                                        module.id,
                                                    )}
                                            >
                                                {module.label}
                                                {#if expandedModules[module.id]}
                                                    <ChevronUp
                                                        class="w-4 h-4 text-muted-foreground"
                                                    />
                                                {:else}
                                                    <ChevronDown
                                                        class="w-4 h-4 text-muted-foreground"
                                                    />
                                                {/if}
                                            </button>
                                        </div>
                                        <span
                                            class="text-[10px] bg-primary/10 text-primary px-2 py-0.5 rounded-full font-bold"
                                        >
                                            {module.items.filter((i: any) =>
                                                formData.selectedItems.includes(
                                                    i.id,
                                                ),
                                            ).length} / {module.items.length}
                                        </span>
                                    </div>

                                    {#if expandedModules[module.id]}
                                        <div
                                            class="p-3 pt-0 grid grid-cols-2 gap-2 border-t bg-background/50"
                                            transition:slide
                                        >
                                            {#each module.items as item}
                                                <label
                                                    class="flex items-center gap-2 p-2 rounded-lg hover:bg-muted/50 cursor-pointer transition-colors"
                                                >
                                                    <Checkbox
                                                        checked={formData.selectedItems.includes(
                                                            item.id,
                                                        )}
                                                        onCheckedChange={() =>
                                                            toggleItemSelection(
                                                                item.id,
                                                                module.id,
                                                                module.items,
                                                            )}
                                                    />
                                                    <span
                                                        class="text-xs font-bold leading-none"
                                                        >{item.label}</span
                                                    >
                                                </label>
                                            {/each}
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            {:else if step === 5}
                <div
                    in:fly={{ y: 20, duration: 500 }}
                    out:fade={{ duration: 200 }}
                    class="space-y-6"
                >
                    <div class="space-y-2">
                        <h2
                            class="text-2xl font-black tracking-tight flex items-center gap-2"
                        >
                            <User class="w-6 h-6 text-primary" />
                            Contas & Carteiras
                        </h2>
                        <p class="text-muted-foreground">
                            Selecione quais contas você deseja criar
                            inicialmente.
                        </p>
                    </div>

                    <div
                        class="space-y-4 pt-2 max-h-[350px] overflow-y-auto custom-scrollbar pr-2"
                    >
                        <!-- Core Accounts -->
                        <div>
                            <h3
                                class="text-sm font-bold uppercase text-muted-foreground mb-2"
                            >
                                Contas Padrão
                            </h3>
                            <div class="grid gap-2">
                                {#each coreAccounts as account}
                                    <label
                                        class="flex items-start gap-3 p-3 border rounded-xl hover:bg-muted/50 cursor-pointer transition-all {formData.selectedAccounts.includes(
                                            account.id,
                                        )
                                            ? 'border-primary bg-primary/5'
                                            : ''}"
                                    >
                                        <Checkbox
                                            checked={formData.selectedAccounts.includes(
                                                account.id,
                                            )}
                                            onCheckedChange={() =>
                                                toggleAccountSelection(
                                                    account.id,
                                                )}
                                        />
                                        <div>
                                            <div class="font-bold text-sm">
                                                {account.label}
                                            </div>
                                            <div
                                                class="text-xs text-muted-foreground"
                                            >
                                                {account.description}
                                            </div>
                                        </div>
                                    </label>
                                {/each}
                            </div>
                        </div>

                        <!-- Context Accounts -->
                        <div>
                            <h3
                                class="text-sm font-bold uppercase text-muted-foreground mb-2 mt-4"
                            >
                                Contas de Mercado (Sugeridas)
                            </h3>
                            <div class="grid gap-2">
                                {#each Object.entries(marketAccountsMap) as [marketId, acc]}
                                    {#if formData.selectedItems.includes(marketId)}
                                        <label
                                            class="flex items-start gap-3 p-3 border rounded-xl hover:bg-muted/50 cursor-pointer transition-all {formData.selectedAccounts.includes(
                                                acc.id,
                                            )
                                                ? 'border-primary bg-primary/5'
                                                : ''}"
                                        >
                                            <Checkbox
                                                checked={formData.selectedAccounts.includes(
                                                    acc.id,
                                                )}
                                                onCheckedChange={() =>
                                                    toggleAccountSelection(
                                                        acc.id,
                                                    )}
                                            />
                                            <div>
                                                <div class="font-bold text-sm">
                                                    {acc.label}
                                                </div>
                                                <div
                                                    class="text-xs text-muted-foreground"
                                                >
                                                    Recomendado para o módulo
                                                    selecionado.
                                                </div>
                                            </div>
                                        </label>
                                    {/if}
                                {/each}
                            </div>
                        </div>
                    </div>
                </div>
            {:else if step === 6}
                <div
                    in:fly={{ y: 20, duration: 500 }}
                    out:fade={{ duration: 200 }}
                    class="space-y-6"
                >
                    <div class="space-y-2">
                        <h2
                            class="text-2xl font-black tracking-tight flex items-center gap-2"
                        >
                            <LayoutDashboard class="w-6 h-6 text-primary" />
                            Dados de Demonstração
                        </h2>
                        <p class="text-muted-foreground">
                            Quer ver o app em ação imediatamente?
                        </p>
                    </div>

                    <div
                        class="p-6 border-2 rounded-2xl bg-muted/30 border-dashed flex flex-col items-center text-center space-y-4"
                    >
                        <div class="bg-primary/10 p-4 rounded-full">
                            <Coins class="w-10 h-10 text-primary" />
                        </div>
                        <div class="space-y-2">
                            <h3 class="font-black text-xl">
                                Deseja popular uma conta teste?
                            </h3>
                            <p class="text-muted-foreground">
                                Vamos gerar trades fictícios baseados na sua
                                personalização para que você possa explorar os
                                gráficos, relatórios e o Hub Financeiro agora
                                mesmo.
                            </p>
                        </div>

                        <label
                            class="flex items-center gap-3 p-4 bg-background rounded-full border shadow-sm cursor-pointer hover:border-primary transition-all"
                        >
                            <Checkbox bind:checked={formData.generateDemo} />
                            <span
                                class="font-black uppercase tracking-widest text-sm pr-4"
                                >Sim, gerar dados de exemplo</span
                            >
                        </label>

                        <p class="text-xs text-muted-foreground pt-4">
                            Você poderá remover esses dados a qualquer momento
                            nas configurações de banco de dados.
                        </p>
                    </div>
                </div>
            {:else if step === 7}
                <div
                    in:fly={{ y: 20, duration: 500 }}
                    out:fade={{ duration: 200 }}
                    class="space-y-8 flex flex-col items-center justify-center h-full"
                >
                    {#if !loading && progress === 0}
                        <div class="text-center space-y-4">
                            <div
                                class="w-24 h-24 bg-primary/20 rounded-full flex items-center justify-center mx-auto mb-6"
                            >
                                <CheckCircle2 class="w-12 h-12 text-primary" />
                            </div>
                            <h2 class="text-3xl font-black tracking-tight">
                                Tudo pronto!
                            </h2>
                            <p
                                class="text-muted-foreground text-lg max-w-sm mx-auto"
                            >
                                Clique no botão abaixo para aplicar as
                                configurações e iniciar sua jornada.
                            </p>
                            <Button
                                onclick={handleComplete}
                                class="h-16 px-12 text-xl font-bold rounded-full mt-8 shadow-xl shadow-primary/20"
                            >
                                Começar Agora
                            </Button>
                        </div>
                    {:else}
                        <div class="w-full space-y-8 text-center">
                            <div class="relative w-32 h-32 mx-auto">
                                <svg
                                    class="w-full h-full"
                                    viewBox="0 0 100 100"
                                >
                                    <circle
                                        cx="50"
                                        cy="50"
                                        r="45"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="8"
                                        class="text-muted"
                                    />
                                    <circle
                                        cx="50"
                                        cy="50"
                                        r="45"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="8"
                                        stroke-dasharray="283"
                                        stroke-dashoffset={283 -
                                            (283 * progress) / 100}
                                        class="text-primary transition-all duration-500 ease-out"
                                        transform="rotate(-90 50 50)"
                                    />
                                </svg>
                                <div
                                    class="absolute inset-0 flex items-center justify-center"
                                >
                                    <span class="text-2xl font-black"
                                        >{progress}%</span
                                    >
                                </div>
                            </div>

                            <div class="space-y-2">
                                <h3 class="text-xl font-bold animate-pulse">
                                    {#if progress < 30}
                                        Criando seu perfil...
                                    {:else if progress < 70}
                                        Instalando módulos...
                                    {:else if progress < 90}
                                        Gerando dados de teste...
                                    {:else}
                                        Finalizando tudo...
                                    {/if}
                                </h3>
                                <p class="text-muted-foreground">
                                    Quase lá! Preparando o motor de performance.
                                </p>
                            </div>
                        </div>
                    {/if}
                </div>
            {/if}
        </div>

        <!-- Navigation Footer -->
        {#if !loading && progress !== 100}
            <div class="mt-12 flex justify-between items-center border-t pt-6">
                <Button
                    variant="ghost"
                    onclick={prevStep}
                    disabled={step === 1}
                    class="font-bold text-muted-foreground uppercase tracking-widest text-xs"
                >
                    <ChevronLeft class="w-4 h-4 mr-2" />
                    Voltar
                </Button>

                {#if step < totalSteps}
                    <Button
                        onclick={nextStep}
                        class="px-8 font-black uppercase tracking-widest bg-primary text-primary-foreground hover:opacity-90 transition-opacity"
                    >
                        Próximo
                        <ChevronRight class="w-4 h-4 ml-2" />
                    </Button>
                {/if}
            </div>
        {/if}
    </div>
</div>

<style>
    @keyframes pulse {
        0%,
        100% {
            transform: scale(1);
        }
        50% {
            transform: scale(1.05);
        }
    }
    .animate-pulse {
        animation: pulse 2s infinite ease-in-out;
    }
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: hsl(var(--muted-foreground) / 0.2);
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: hsl(var(--muted-foreground) / 0.4);
    }
</style>
