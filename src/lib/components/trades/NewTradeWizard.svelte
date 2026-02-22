<script lang="ts">
    import { t, locale } from "svelte-i18n";
    import { invoke } from "@tauri-apps/api/core";
    import { toast } from "svelte-sonner";
    import { onMount, untrack } from "svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { rtdStore } from "$lib/stores/rtd.svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Textarea } from "$lib/components/ui/textarea";
    import * as Card from "$lib/components/ui/card";
    import * as Select from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";
    import AssetSelect from "./AssetSelect.svelte";
    import PartialExitsManager from "./PartialExitsManager.svelte";
    import ImageUploader from "./ImageUploader.svelte";
    import DailyChecklist from "./DailyChecklist.svelte";
    import {
        ChevronLeft,
        ChevronRight,
        Save,
        X,
        Brain,
        ShieldCheck,
        LayoutDashboard,
        Target,
        Camera,
        RotateCcw,
        TrendingUp,
        Calendar,
        Lock,
        RefreshCw,
        Plus,
        Coins,
    } from "lucide-svelte";

    let {
        trade = null,
        close = () => {},
        onsave = () => {},
    } = $props<{
        trade?: any;
        close: () => void;
        onsave?: () => void;
    }>();

    let currentStep = $state(1);
    let isSubmitting = $state(false);

    // Helper to get current time with user's configured offset
    const getNowWithOffset = () => {
        const now = new Date();
        // CRITICAL: Untrack profile to avoid reactive loop if settings update
        const offsetMinutes = untrack(
            () => settingsStore.userProfile?.utc_offset || 0,
        );
        const offsetMillis = offsetMinutes * 60000;
        return new Date(now.getTime() + offsetMillis)
            .toISOString()
            .slice(0, 16);
    };

    // Form Data - Modern Schema alignment with Historical refinements (d5398093)
    let formData = $state({
        account_id: "",
        strategy_id: "",
        timeframe: "",
        volatility: "",
        asset: "WIN",
        direction: "buy",
        entry_date: getNowWithOffset() as string,
        entry_price: 0,
        quantity: 1,
        modality_id: "",
        entry_emotional_state_id: "",
        stop_loss: null as number | null,
        take_profit: null as number | null,
        intensity: 10,
        fees: 0,
        partial_exits: [] as any[],
        status: "open",
        exit_price: null as number | null,
        exit_date: null as string | null, // Explicit type for lint
        exit_reason: "",
        exit_emotional_state_id: "",
        entry_rationale: "",
        confirmation_signals: "",
        market_context: "",
        relevant_news: "",
        psychology_analysis_during: "",
        followed_plan: true,
        what_worked: "",
        mistakes_improvements: "",
        lessons_learned: "",
        images: [] as string[],
        base_currency: "BRL",
    });

    let priceHasFocus = $state(false);

    let lastSyncedTradeId = $state<string | undefined>(undefined);
    let lastSyncedDraftKey = $state<string | undefined>(undefined);

    // Reactive synchronization when trade prop changes (CRITICAL for Edit flow)
    $effect(() => {
        // Track only trade prop and ID
        const currentTrade = trade;
        const tradeId = trade?.id;

        // Optimization: only re-sync if the trade ID has actually changed
        // This prevents re-running on every parent re-render if the prop is technically a new object
        if (tradeId && tradeId !== lastSyncedTradeId) {
            console.log(
                "[NewTradeWizard] Syncing formData with trade:",
                tradeId,
                "symbol:",
                currentTrade.asset_symbol,
            );

            lastSyncedTradeId = tradeId;

            // We use untrack to avoid formData dependency loop
            const baseData = untrack(() => formData);

            formData = {
                ...baseData,
                ...currentTrade,
                asset: currentTrade.asset_symbol,
                direction: (currentTrade.direction || "buy").toLowerCase(),
                // Robust date parsing for datetime-local (YYYY-MM-DDTHH:MM)
                entry_date: currentTrade.date
                    ? currentTrade.date.replace(" ", "T").slice(0, 16)
                    : getNowWithOffset(),
                exit_date: currentTrade.exit_date
                    ? currentTrade.exit_date.replace(" ", "T").slice(0, 16)
                    : null,

                // Numerical values with forced precision parsing
                entry_price: parseFloat(currentTrade.entry_price as any) || 0,
                exit_price: currentTrade.exit_price
                    ? parseFloat(currentTrade.exit_price as any)
                    : null,
                quantity: parseFloat(currentTrade.quantity as any) || 1,
                stop_loss: currentTrade.stop_loss
                    ? parseFloat(currentTrade.stop_loss as any)
                    : null,
                take_profit: currentTrade.take_profit
                    ? parseFloat(currentTrade.take_profit as any)
                    : null,
                intensity:
                    parseFloat(currentTrade.intensity as any) !== undefined
                        ? parseFloat(currentTrade.intensity as any)
                        : 10,
                fees: parseFloat(currentTrade.fee_total as any) || 0,

                images: currentTrade.images || [],
                partial_exits: (currentTrade.partial_exits || []).map(
                    (p: any) => ({
                        ...p,
                        price: parseFloat(p.price) || 0,
                        quantity: parseFloat(p.quantity) || 0,
                    }),
                ),
            };
        } else if (currentTrade) {
            // NEW: Support for DRAFT trades (from RTD detection pop-up)
            const draftKey = `${currentTrade.asset_symbol}-${currentTrade.entry_price}-${currentTrade.account_id}`;

            if (draftKey !== lastSyncedDraftKey) {
                console.log(
                    "[NewTradeWizard] Syncing formData with DRAFT trade from RTD:",
                    currentTrade.asset_symbol,
                    "Key:",
                    draftKey,
                );
                lastSyncedDraftKey = draftKey;

                const baseData = untrack(() => formData);
                formData = {
                    ...baseData,
                    asset: currentTrade.asset_symbol || baseData.asset,
                    entry_price:
                        parseFloat(currentTrade.entry_price as any) || 0,
                    account_id: currentTrade.account_id || baseData.account_id,
                    entry_date: currentTrade.date
                        ? currentTrade.date.replace(" ", "T").slice(0, 16)
                        : getNowWithOffset(),
                };
            }
        } else if (
            lastSyncedTradeId !== undefined ||
            lastSyncedDraftKey !== undefined
        ) {
            // Reset to defaults ONLY if we previously had a synced trade
            console.log(
                "[NewTradeWizard] Resetting form to defaults (transition to new trade)",
            );

            lastSyncedTradeId = undefined;
            lastSyncedDraftKey = undefined;

            formData = {
                account_id: "",
                strategy_id: "",
                timeframe: "",
                volatility: "",
                asset: "WIN",
                direction: "buy",
                entry_date: untrack(() => getNowWithOffset()),
                entry_price: 0,
                quantity: 1,
                modality_id: "",
                entry_emotional_state_id: "",
                stop_loss: null,
                take_profit: null,
                intensity: 10,
                fees: 0,
                partial_exits: [],
                status: "open",
                exit_price: null,
                exit_date: null,
                exit_reason: "",
                exit_emotional_state_id: "",
                entry_rationale: "",
                confirmation_signals: "",
                market_context: "",
                relevant_news: "",
                psychology_analysis_during: "",
                followed_plan: true,
                what_worked: "",
                mistakes_improvements: "",
                lessons_learned: "",
                images: [],
                base_currency: "BRL",
            };
        }
    });

    const steps = [
        {
            id: 1,
            label: $t("trades.wizard.steps.basic"),
            icon: LayoutDashboard,
        },
        { id: 2, label: $t("trades.wizard.steps.conduction"), icon: Target },
        { id: 3, label: $t("trades.wizard.steps.psychology"), icon: Brain },
        { id: 4, label: $t("trades.wizard.steps.media"), icon: Camera },
        { id: 5, label: $t("trades.wizard.steps.review"), icon: Save },
    ];

    // --- STABILITY LAYER (Svelte 5) ---
    // We create local snapshots of settings data to avoid "reactive noise"
    // caused by RTD updates triggering re-renders of the entire form.
    let accountsList = $state<any[]>([]);
    let strategiesList = $state<any[]>([]);
    let assetTypesList = $state<any[]>([]);
    let timeframeList = $state<any[]>([]);
    let assetsList = $state<any[]>([]);

    onMount(() => {
        // Capture initial snapshots from store
        accountsList = [...settingsStore.accounts];
        strategiesList = [...settingsStore.strategies];
        assetTypesList = [...settingsStore.assetTypes];
        assetsList = [...settingsStore.assets];
        timeframeList =
            settingsStore.timeframes.length > 0
                ? [...settingsStore.timeframes]
                : [
                      {
                          value: "1m",
                          name: $t("trades.wizard.timeframe_options.1m"),
                      },
                      {
                          value: "5m",
                          name: $t("trades.wizard.timeframe_options.5m"),
                      },
                      {
                          value: "15m",
                          name: $t("trades.wizard.timeframe_options.15m"),
                      },
                      {
                          value: "60m",
                          name: $t("trades.wizard.timeframe_options.60m"),
                      },
                      {
                          value: "D",
                          name: $t("trades.wizard.timeframe_options.d"),
                      },
                  ];

        console.log(
            "[NewTradeWizard] UI Snapshots captured for performance stability.",
        );
    });

    // Filtering logic (04/02/2026)
    let selectedAssetTypeId = $state("");
    let userManuallySelectedType = $state(false); // Track manual selection

    // Robust Initialization and auto-selection
    $effect(() => {
        // We need to wait for settingsStore.assets to be populated
        if (settingsStore.assets.length === 0) return;

        // Priority 1: Use trade.asset_type_id if editing and not already set
        if (trade && !selectedAssetTypeId) {
            console.log(
                "[NewTradeWizard] Attempting to initialize asset type for trade:",
                trade.id,
                "symbol:",
                trade.asset_symbol,
            );
            if (trade.asset_type_id) {
                const type = settingsStore.assetTypes.find(
                    (t) =>
                        t.id === trade.asset_type_id ||
                        t.id.replace(/^asset_type:/, "") ===
                            trade.asset_type_id.replace(/^asset_type:/, ""),
                );
                selectedAssetTypeId = type ? type.id : trade.asset_type_id;
                console.log(
                    "[NewTradeWizard] Initialized from trade.asset_type_id:",
                    selectedAssetTypeId,
                );
            }
            // Priority 2: Force symbol lookup for editing trades without asset_type_id
            else if (trade.asset_symbol) {
                const asset = assetsList.find(
                    (a) =>
                        a.symbol.toUpperCase() ===
                        trade.asset_symbol.toUpperCase(),
                );
                if (asset) {
                    const type = assetTypesList.find(
                        (t) =>
                            t.id === asset.asset_type_id ||
                            t.id.replace(/^asset_type:/, "") ===
                                asset.asset_type_id.replace(/^asset_type:/, ""),
                    );
                    selectedAssetTypeId = type ? type.id : asset.asset_type_id;
                    console.log(
                        "[NewTradeWizard] Initialized from symbol lookup (edit):",
                        selectedAssetTypeId,
                        "for",
                        trade.asset_symbol,
                    );
                } else {
                    console.warn(
                        "[NewTradeWizard] Symbol lookup failed for",
                        trade.asset_symbol,
                        "Total assets in store:",
                        settingsStore.assets.length,
                    );
                }
            }
        }

        // Priority 3: Auto-select based on formData.asset changes (for new trades or when changing symbol)
        // But don't override if user manually selected a type
        if (
            !userManuallySelectedType &&
            formData.asset &&
            assetsList.length > 0
        ) {
            const asset = assetsList.find(
                (a) => a.symbol.toUpperCase() === formData.asset.toUpperCase(),
            );
            if (asset) {
                const type = assetTypesList.find(
                    (t) =>
                        t.id === asset.asset_type_id ||
                        t.id.replace(/^asset_type:/, "") ===
                            asset.asset_type_id.replace(/^asset_type:/, ""),
                );
                if (type && type.id !== selectedAssetTypeId) {
                    selectedAssetTypeId = type.id;
                    console.log(
                        "[NewTradeWizard] Auto-syncing from asset symbol:",
                        selectedAssetTypeId,
                        "for",
                        formData.asset,
                    );
                }
            }
        }
    });

    // Stable list of extra assets from RTD
    let rtdAssets = $derived.by(() => {
        const symbols = rtdStore.symbols;
        if (symbols.length === 0) return [];

        // Cache external lookups for performance
        const assetSymbolsSet = new Set(
            settingsStore.assets.map((a) => a.symbol),
        );

        const futType =
            settingsStore.assetTypes.find(
                (t) =>
                    t.name.toLowerCase().includes("future") ||
                    t.name.toLowerCase().includes("futuro"),
            )?.id || "";
        const indType =
            settingsStore.assetTypes.find(
                (t) =>
                    t.name.toLowerCase().includes("index") ||
                    t.name.toLowerCase().includes("indice"),
            )?.id || "";
        const stockType =
            settingsStore.assetTypes.find(
                (t) =>
                    t.name.toLowerCase().includes("stock") ||
                    t.name.toLowerCase().includes("ação") ||
                    t.name.toLowerCase().includes("aç"),
            )?.id || "";

        return symbols
            .filter((sym) => !assetSymbolsSet.has(sym))
            .map((sym) => {
                let guessedTypeId = "rtd";
                if (/^(WIN|WDO|WSP|DI1|BGI|CCM|IND|DOL)/i.test(sym)) {
                    guessedTypeId = futType || indType || "rtd";
                } else if (/^[A-Z]{4}\d/i.test(sym)) {
                    guessedTypeId = stockType || "rtd";
                }

                return {
                    id: `rtd:${sym}`,
                    symbol: sym,
                    name: `Ativo Profit RTD (${sym})`,
                    asset_type_id: guessedTypeId,
                    point_value: sym.startsWith("WDO")
                        ? 10
                        : sym.startsWith("WIN")
                          ? 0.2
                          : 1.0,
                };
            });
    });

    let filteredAssets = $derived.by(() => {
        const assets = [...settingsStore.assets, ...rtdAssets];
        if (!selectedAssetTypeId) return assets;

        const typeId = selectedAssetTypeId.replace(/^asset_type:/, "");
        return assets.filter(
            (a) => a.asset_type_id.replace(/^asset_type:/, "") === typeId,
        );
    });

    // AUTO-SYNC Price: Removed as per user request to avoid lag
    // Price synchronization is now manual via the Refresh button.

    // Intermediate derivations with untracked isolation
    let selectedAsset = $derived.by(() => {
        const symbol = formData.asset;
        return untrack(() =>
            settingsStore.assets.find((a) => a.symbol === symbol),
        );
    });
    let selectedAccount = $derived.by(() => {
        const id = formData.account_id;
        return untrack(() => settingsStore.accounts.find((a) => a.id === id));
    });

    // Calculation Engine (c5a63810)
    let calculationResult = $derived.by(() => {
        const asset = selectedAsset;
        const assetTypes = assetTypesList;
        const assetType = assetTypes.find(
            (at) => at.id === asset?.asset_type_id,
        );
        const account = selectedAccount;
        const currencySymbol = account
            ? settingsStore.getCurrencySymbol(account.currency)
            : "R$";

        // Use untrack for ANY high-frequency data from rtdStore if accessed here
        // (Currently it only uses formData and snapshots, which is good)

        const pointValue = asset?.point_value || 1.0;
        const multiplier = formData.direction === "buy" ? 1 : -1;

        let grossTotal = 0;
        let runningQty = formData.quantity;
        let avgPrice = formData.entry_price;
        let memoryItems: any[] = [];

        // 1. Calculate Partials
        formData.partial_exits.forEach((p: any) => {
            const qty = p.quantity || 0;
            const price = p.price || 0;
            const isEntry = p.type === "entry";

            if (isEntry) {
                // Adjust Average Price
                const newQty = runningQty + qty;
                avgPrice = (avgPrice * runningQty + price * qty) / newQty;
                runningQty = newQty;
                memoryItems.push({
                    label: `${$t("trades.wizard.summary.addition")} (+${qty} ${assetType?.unit_label || "ctr"}) @ ${price}`,
                    result: 0,
                    type: "addition",
                });
            } else {
                const diff = price - avgPrice;
                const result = diff * qty * pointValue * multiplier;
                grossTotal += result;
                runningQty -= qty;
                memoryItems.push({
                    label: `${$t("trades.wizard.summary.partial_exit")} (-${qty} ${assetType?.unit_label || "ctr"})`,
                    result,
                    type: "exit",
                });
            }
        });

        // 2. Calculate Final Exit if we have an exit price (regardless of status)
        const remainingQty = runningQty; // Use runningQty for remaining
        const finalExitPrice =
            formData.exit_price !== null ? Number(formData.exit_price) : null;

        if (finalExitPrice !== null && remainingQty > 0) {
            const diff = finalExitPrice - avgPrice;
            const result = diff * remainingQty * pointValue * multiplier;
            grossTotal += result;
            memoryItems.push({
                label: `${$t("trades.wizard.summary.final_exit")} (-${remainingQty} ${assetType?.unit_label || "ctr"})`,
                result,
                type: "exit",
            });
        }

        // 3. Automatic Fee Calculation
        let calculatedFees = 0;
        const feeProfile = settingsStore.fees.find(
            (f) => f.id === asset?.default_fee_id,
        );

        if (feeProfile) {
            const entryValue =
                formData.entry_price * formData.quantity * pointValue;

            // Fixed fees (per contract/unit)
            if (feeProfile.fixed_fee > 0) {
                const fixed = feeProfile.fixed_fee * formData.quantity;
                calculatedFees += fixed;
                memoryItems.push({
                    label: $t("trades.wizard.summary.fixed_fee"),
                    result: -fixed,
                });
            }

            // Percentage fees (on total volume)
            if (feeProfile.percentage_fee > 0) {
                const perc = entryValue * (feeProfile.percentage_fee / 100);
                calculatedFees += perc;
                memoryItems.push({
                    label: `${$t("trades.wizard.summary.variable_fee")} (${feeProfile.percentage_fee}%)`,
                    result: -perc,
                });
            }

            // Exchange fees (Bovespa/Exchange)
            if (feeProfile.exchange_fee > 0) {
                const exch = entryValue * (feeProfile.exchange_fee / 100);
                calculatedFees += exch;
                memoryItems.push({
                    label: `Taxas de Bolsa (${feeProfile.exchange_fee}%)`,
                    result: -exch,
                });
            }

            // Withholding Tax (IRRF) - Only on positive results
            if (feeProfile.withholding_tax > 0 && grossTotal > 0) {
                const irrf = grossTotal * (feeProfile.withholding_tax / 100);
                calculatedFees += irrf;
                memoryItems.push({
                    label: `IRRF Estimado (${feeProfile.withholding_tax}%)`,
                    result: -irrf,
                });
            }
        }

        // Use manual fees if calculated is 0 and manual is provided
        const finalFees = calculatedFees || formData.fees || 0;

        return {
            gross: grossTotal,
            net: grossTotal - finalFees,
            fees: finalFees,
            remainingQty,
            memoryItems,
            assetType,
            currencySymbol,
        };
    });

    function handleNext() {
        if (currentStep === 1) {
            if (
                !formData.account_id ||
                !formData.asset ||
                !formData.strategy_id ||
                !formData.modality_id
            ) {
                toast.error($t("trades.wizard.messages.required_fields"));
                return;
            }
            if (formData.entry_price <= 0) {
                toast.error($t("trades.wizard.messages.entry_price_required"));
                return;
            }
        }
        if (currentStep < steps.length) currentStep++;
    }

    function handlePrev() {
        if (currentStep > 1) currentStep--;
    }

    async function handleSubmit() {
        const submissionId = trade?.id;
        console.log(
            "[NewTradeWizard] Submitting form. Mode:",
            submissionId ? "Edit" : "New",
            "Target ID:",
            submissionId,
        );
        console.log(
            "[NewTradeWizard] Form Data Snapshot:",
            $state.snapshot(formData),
        );

        // CRITICAL: Ensure asset_type_id is set before saving
        if (!selectedAssetTypeId && formData.asset) {
            const asset = assetsList.find(
                (a) => a.symbol.toUpperCase() === formData.asset.toUpperCase(),
            );
            if (asset) {
                const type = assetTypesList.find(
                    (t) =>
                        t.id === asset.asset_type_id ||
                        t.id.replace(/^asset_type:/, "") ===
                            asset.asset_type_id.replace(/^asset_type:/, ""),
                );
                selectedAssetTypeId = type ? type.id : asset.asset_type_id;
                console.log(
                    "[NewTradeWizard] Auto-filled asset_type_id from asset:",
                    selectedAssetTypeId,
                );
            } else {
                toast.error($t("trades.wizard.messages.valid_asset_type"));
                return;
            }
        }

        if (!selectedAssetTypeId) {
            toast.error($t("trades.wizard.messages.asset_type_required"));
            return;
        }

        isSubmitting = true;
        try {
            const tradeData: any = {
                // Save as ISO string to preserve time for editing later
                date: (formData.entry_date as string)?.includes("T")
                    ? formData.entry_date + ":00Z"
                    : formData.entry_date + "T00:00:00Z",
                asset_symbol: formData.asset,
                asset_type_id: selectedAssetTypeId,
                strategy_id: formData.strategy_id,
                account_id: formData.account_id,
                result: calculationResult.net,
                quantity: formData.quantity,
                direction: formData.direction === "buy" ? "Buy" : "Sell",
                entry_price: formData.entry_price,
                exit_price: formData.exit_price,
                exit_date: formData.exit_date
                    ? (formData.exit_date as string).includes("T")
                        ? formData.exit_date + ":00Z"
                        : formData.exit_date + "T00:00:00Z"
                    : formData.exit_price !== null
                      ? new Date().toISOString() // Fallback to now if closed but no date
                      : null,
                fee_total: formData.fees,
                notes: formData.entry_rationale,

                timeframe: formData.timeframe,
                volatility: formData.volatility,
                modality_id: formData.modality_id,
                stop_loss: formData.stop_loss,
                take_profit: formData.take_profit,
                intensity: formData.intensity,

                entry_emotional_state_id: formData.entry_emotional_state_id,

                exit_reason: formData.exit_reason,
                exit_emotional_state_id: formData.exit_emotional_state_id,

                entry_rationale: formData.entry_rationale,
                confirmation_signals: formData.confirmation_signals,
                market_context: formData.market_context,
                relevant_news: formData.relevant_news,

                followed_plan: !!formData.followed_plan,
                what_worked: formData.what_worked,
                mistakes_improvements: formData.mistakes_improvements,
                lessons_learned: formData.lessons_learned,

                images: formData.images,
                partial_exits: formData.partial_exits,
            };

            console.log(
                "[NewTradeWizard] Submission tradeData:",
                JSON.stringify(tradeData, null, 2),
            );

            if (trade?.id) {
                console.log(
                    "[NewTradeWizard] Calling updateTrade for ID:",
                    trade.id,
                );
                const result = await tradesStore.updateTrade(
                    trade.id,
                    tradeData,
                );
                if (result.success) {
                    toast.success($t("trades.wizard.messages.update_success"));
                    onsave();
                    close();
                } else {
                    console.error(
                        "[NewTradeWizard] Backend Update Error:",
                        result.error,
                    );
                    toast.error(
                        result.error ||
                            $t("trades.wizard.messages.update_error"),
                    );
                }
            } else {
                console.log("[NewTradeWizard] Calling addTrade");
                const result = await tradesStore.addTrade(tradeData);
                if (result.success) {
                    toast.success($t("trades.wizard.messages.save_success"));
                    onsave();
                    close();
                } else {
                    console.error(
                        "[NewTradeWizard] Backend Save Error:",
                        result.error,
                    );
                    toast.error(
                        result.error || $t("trades.wizard.messages.save_error"),
                    );
                }
            }
        } catch (e) {
            console.error("[NewTradeWizard] CRITICAL CLIENT CRASH:", e);
            toast.error($t("trades.wizard.messages.save_error"));
        } finally {
            isSubmitting = false;
        }
    }
</script>

<div class="flex flex-col h-full bg-background overflow-hidden">
    <!-- Header with Modern Stepper -->
    <div class="px-6 py-4 border-b border-border bg-card/30">
        <div class="flex items-center justify-between mb-4">
            <div>
                <h2 class="text-lg font-bold tracking-tight">
                    {trade?.id
                        ? $t("trades.wizard.title_edit")
                        : $t("trades.wizard.title_new")}
                </h2>
                <p
                    class="text-[10px] text-muted-foreground uppercase tracking-widest"
                >
                    {$t("trades.wizard.subtitle")}
                </p>
            </div>
        </div>

        <div class="relative flex justify-between max-w-2xl mx-auto">
            <div class="absolute top-4 left-0 w-full h-0.5 bg-muted -z-0"></div>
            {#each steps as step}
                {@const Icon = step.icon}
                {@const isCurrent = currentStep === step.id}
                {@const isPast = currentStep > step.id}
                <button
                    class="relative z-10 group flex flex-col items-center gap-1.5"
                    onclick={() => (currentStep = step.id)}
                >
                    <div
                        class="w-8 h-8 rounded-full flex items-center justify-center border-2 transition-all duration-300
                        {isCurrent
                            ? 'bg-primary border-primary text-primary-foreground shadow-[0_0_10px_rgba(var(--primary),0.25)]'
                            : isPast
                              ? 'bg-primary/20 border-primary text-primary'
                              : 'bg-background border-muted text-muted-foreground'}"
                    >
                        {#if isPast}
                            <ShieldCheck class="w-4 h-4" />
                        {:else}
                            <Icon class="w-4 h-4" />
                        {/if}
                    </div>
                    <span
                        class="text-[9px] font-bold uppercase tracking-widest {isCurrent
                            ? 'text-primary'
                            : 'text-muted-foreground'}">{step.label}</span
                    >
                </button>
            {/each}
        </div>
    </div>

    <!-- Main Content Area (Glassmorphism inspired) -->
    <div class="flex-1 overflow-y-auto p-4 bg-zinc-950/20">
        <div class="max-w-3xl mx-auto space-y-4">
            {#if currentStep === 1}
                <div
                    class="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300"
                >
                    <section class="space-y-4">
                        <h3
                            class="text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em]"
                        >
                            {$t("trades.wizard.sections.context")}
                        </h3>
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t("trades.wizard.fields.account")}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.account_id}
                                >
                                    <Select.Trigger
                                        class="h-8 bg-muted/30 border-0 focus:ring-1 focus:ring-primary/40 text-xs text-white"
                                    >
                                        {accountsList.find(
                                            (a) => a.id === formData.account_id,
                                        )?.nickname ||
                                            $t(
                                                "trades.wizard.placeholders.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each accountsList as acc}
                                            <Select.Item value={acc.id}
                                                >{acc.nickname}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.strategy",
                                    )}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.strategy_id}
                                >
                                    <Select.Trigger
                                        class="h-8 bg-muted/30 border-0 focus:ring-1 focus:ring-primary/40 text-xs text-white"
                                    >
                                        {strategiesList.find(
                                            (s) =>
                                                s.id === formData.strategy_id,
                                        )?.name ||
                                            $t(
                                                "trades.wizard.placeholders.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each strategiesList as strategy}
                                            <Select.Item value={strategy.id}
                                                >{strategy.name}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.timeframe",
                                    )}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.timeframe}
                                >
                                    <Select.Trigger
                                        class="h-8 bg-muted/30 border-0 focus:ring-1 focus:ring-primary/40 text-xs text-white"
                                    >
                                        {formData.timeframe ||
                                            $t(
                                                "trades.wizard.placeholders.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each timeframeList as tf}
                                            <Select.Item value={tf.value}
                                                >{tf.name}</Select.Item
                                            >
                                        {:else}
                                            <Select.Item value="5m"
                                                >{$t(
                                                    "trades.wizard.timeframe_options.5m",
                                                )}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.volatility",
                                    )}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.volatility}
                                >
                                    <Select.Trigger
                                        class="h-8 bg-muted/30 border-0 focus:ring-1 focus:ring-primary/40 text-xs text-white"
                                    >
                                        {formData.volatility ||
                                            $t(
                                                "trades.wizard.placeholders.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        <Select.Item value="baixa"
                                            >{$t(
                                                "trades.wizard.volatility_options.low",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="normal"
                                            >{$t(
                                                "trades.wizard.volatility_options.normal",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="alta"
                                            >{$t(
                                                "trades.wizard.volatility_options.high",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="extrema"
                                            >{$t(
                                                "trades.wizard.volatility_options.extreme",
                                            )}</Select.Item
                                        >
                                    </Select.Content>
                                </Select.Root>
                            </div>
                        </div>

                        <!-- Ativo e Direção -->
                        <!-- Main Grid: Assets (Left) and Trade Info (Right) -->
                        <div
                            class="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-muted/30"
                        >
                            <!-- Left Column: Assets -->
                            <div class="space-y-4">
                                <div class="flex flex-col gap-3">
                                    <!-- Asset Type Filter -->
                                    <div class="space-y-1">
                                        <Label
                                            class="text-[9px] uppercase font-bold text-muted-foreground tracking-tighter"
                                            >{$t(
                                                "trades.wizard.fields.asset_type",
                                            )}</Label
                                        >
                                        <Select.Root
                                            type="single"
                                            bind:value={selectedAssetTypeId}
                                            onValueChange={() => {
                                                userManuallySelectedType = true;
                                                console.log(
                                                    "[NewTradeWizard] User manually selected asset type:",
                                                    selectedAssetTypeId,
                                                );
                                            }}
                                        >
                                            <Select.Trigger
                                                class="h-8 bg-zinc-900 border-muted/20 text-[10px] w-full"
                                            >
                                                {assetTypesList.find(
                                                    (t) =>
                                                        t.id ===
                                                        selectedAssetTypeId,
                                                )?.name ||
                                                    $t(
                                                        "trades.wizard.placeholders.all_types",
                                                    )}
                                            </Select.Trigger>
                                            <Select.Content>
                                                <Select.Item value=""
                                                    >{$t(
                                                        "trades.wizard.placeholders.all_types",
                                                    )}</Select.Item
                                                >
                                                {#each assetTypesList as type}
                                                    <Select.Item value={type.id}
                                                        >{type.name}</Select.Item
                                                    >
                                                {/each}
                                            </Select.Content>
                                        </Select.Root>
                                    </div>

                                    <!-- Asset Selector -->
                                    <div class="space-y-1">
                                        <Label
                                            class="text-[9px] uppercase font-bold text-muted-foreground tracking-tighter"
                                            >{$t("trades.wizard.fields.asset")}
                                            <span class="text-red-500">*</span
                                            ></Label
                                        >
                                        <Select.Root
                                            type="single"
                                            bind:value={formData.asset}
                                        >
                                            <Select.Trigger
                                                class="h-10 bg-zinc-900 border-muted/40 text-xs font-bold w-full"
                                            >
                                                <div
                                                    class="flex items-center gap-2"
                                                >
                                                    {#if formData.asset}
                                                        <span
                                                            class="font-mono text-primary"
                                                            >{formData.asset}</span
                                                        >
                                                        {#if assetsList.find((a) => a.symbol === formData.asset)?.name}
                                                            <span
                                                                class="text-muted-foreground"
                                                                >- {assetsList.find(
                                                                    (a) =>
                                                                        a.symbol ===
                                                                        formData.asset,
                                                                )?.name}</span
                                                            >
                                                        {/if}
                                                    {:else}
                                                        <span
                                                            class="text-muted-foreground"
                                                            >{$t(
                                                                "trades.wizard.placeholders.select",
                                                            )}</span
                                                        >
                                                    {/if}
                                                </div>
                                            </Select.Trigger>
                                            <Select.Content
                                                class="max-h-[200px] overflow-y-auto"
                                            >
                                                {#each filteredAssets as asset}
                                                    <Select.Item
                                                        value={asset.symbol}
                                                    >
                                                        <span
                                                            class="font-mono font-bold mr-2"
                                                            >{asset.symbol}</span
                                                        >
                                                        <span
                                                            class="text-muted-foreground opacity-70"
                                                            >{asset.name}</span
                                                        >
                                                    </Select.Item>
                                                {/each}
                                            </Select.Content>
                                        </Select.Root>
                                    </div>

                                    <!-- Detail info -->
                                    {#if formData.asset}
                                        {@const asset =
                                            settingsStore.assets.find(
                                                (a) =>
                                                    a.symbol === formData.asset,
                                            )}
                                        <!-- Asset details hidden as per user request -->
                                    {/if}
                                </div>
                            </div>

                            <!-- Right Column: Trade Info -->
                            <div class="space-y-4">
                                <div class="space-y-1.5">
                                    <Label
                                        class="text-[9px] uppercase font-bold text-muted-foreground tracking-tighter"
                                        >{$t("trades.wizard.fields.direction")}
                                        <span class="text-red-500">*</span
                                        ></Label
                                    >
                                    <div
                                        class="grid grid-cols-2 gap-3 h-[42px]"
                                    >
                                        <button
                                            type="button"
                                            class="rounded-md font-bold text-xs transition-all flex items-center justify-center {formData.direction ===
                                            'buy'
                                                ? 'bg-emerald-500 text-white shadow-[0_0_15px_rgba(16,185,129,0.3)]'
                                                : 'bg-muted/20 text-muted-foreground hover:bg-muted/30'}"
                                            onclick={() =>
                                                (formData.direction = "buy")}
                                        >
                                            {$t("trades.wizard.fields.buy")}
                                        </button>
                                        <button
                                            type="button"
                                            class="rounded-md font-bold text-xs transition-all flex items-center justify-center {formData.direction ===
                                            'sell'
                                                ? 'bg-red-500 text-white shadow-[0_0_15px_rgba(239,68,68,0.3)]'
                                                : 'bg-muted/20 text-muted-foreground hover:bg-muted/30'}"
                                            onclick={() =>
                                                (formData.direction = "sell")}
                                        >
                                            {$t("trades.wizard.fields.sell")}
                                        </button>
                                    </div>
                                </div>

                                <div class="grid grid-cols-2 gap-3">
                                    <div class="space-y-1.5">
                                        <Label
                                            class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                            >{$t(
                                                "trades.wizard.fields.date_time",
                                            )}</Label
                                        >
                                        <div class="relative">
                                            <Input
                                                type="datetime-local"
                                                bind:value={formData.entry_date}
                                                class="h-10 bg-zinc-900 border-muted/40 text-xs text-white pl-3 pr-8 w-full"
                                            />
                                            <Calendar
                                                class="w-4 h-4 text-muted-foreground absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none"
                                            />
                                        </div>
                                    </div>
                                    <div class="space-y-1.5">
                                        <Label
                                            class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                            >{$t(
                                                "trades.wizard.fields.entry_price",
                                            )}
                                            <span class="text-red-500">*</span
                                            ></Label
                                        >
                                        <div class="flex gap-2">
                                            <Input
                                                onfocus={() =>
                                                    (priceHasFocus = true)}
                                                onblur={() =>
                                                    (priceHasFocus = false)}
                                                type="number"
                                                step="0.00001"
                                                bind:value={
                                                    formData.entry_price
                                                }
                                                class="h-10 bg-zinc-900 border-muted/40 text-sm font-mono text-white flex-1"
                                            />
                                            {#if true || settingsStore.rtdEnabled || rtdStore.symbols.length > 0}
                                                <Button
                                                    variant="secondary"
                                                    size="icon"
                                                    class="h-10 w-10 shrink-0"
                                                    onclick={() => {
                                                        const asset =
                                                            formData.asset.toUpperCase();
                                                        // Try exact, then common aliases
                                                        const quote =
                                                            rtdStore.quotes[
                                                                asset
                                                            ] ||
                                                            rtdStore.quotes[
                                                                asset.replace(
                                                                    "FUT",
                                                                    "",
                                                                )
                                                            ] ||
                                                            rtdStore.quotes[
                                                                asset + "FUT"
                                                            ] ||
                                                            Object.values(
                                                                rtdStore.quotes,
                                                            ).find((q) =>
                                                                q.symbol.startsWith(
                                                                    asset.replace(
                                                                        "FUT",
                                                                        "",
                                                                    ),
                                                                ),
                                                            );

                                                        if (
                                                            quote &&
                                                            quote.last > 0
                                                        ) {
                                                            formData.entry_price =
                                                                quote.last;
                                                            // Also sync date and time to NOW applying configured offset
                                                            formData.entry_date =
                                                                getNowWithOffset();

                                                            toast.success(
                                                                $t(
                                                                    "trades.wizard.messages.rtd_sync_success",
                                                                    {
                                                                        values: {
                                                                            symbol: quote.symbol,
                                                                            price: quote.last,
                                                                        },
                                                                    },
                                                                ),
                                                            );
                                                        } else {
                                                            toast.error(
                                                                $t(
                                                                    "trades.wizard.messages.rtd_not_available",
                                                                ),
                                                            );
                                                        }
                                                    }}
                                                >
                                                    <RefreshCw
                                                        class="w-4 h-4 text-muted-foreground"
                                                    />
                                                </Button>
                                            {/if}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- Bottom Row: Qty, Emotional, Stop Loss, Take Profit -->
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 pt-3">
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t("trades.wizard.fields.quantity")}
                                    <span class="text-red-500">*</span></Label
                                >
                                <Input
                                    type="number"
                                    step="any"
                                    bind:value={formData.quantity}
                                    class="h-10 bg-zinc-900 border-muted/40 text-sm text-white pl-4"
                                />
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight flex items-center gap-1"
                                >
                                    <Brain class="w-3 h-3 text-pink-400" />
                                    {$t("trades.wizard.fields.emotional_state")}
                                </Label>
                                <Select.Root
                                    type="single"
                                    bind:value={
                                        formData.entry_emotional_state_id
                                    }
                                >
                                    <Select.Trigger
                                        class="h-10 bg-zinc-900 border-muted/40 text-xs text-white"
                                    >
                                        {settingsStore.emotionalStates.find(
                                            (e) =>
                                                e.id ===
                                                formData.entry_emotional_state_id,
                                        )?.name ||
                                            $t(
                                                "trades.wizard.placeholders.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each settingsStore.emotionalStates as state}
                                            <Select.Item value={state.id}
                                                >{state.name}</Select.Item
                                            >
                                        {:else}
                                            <Select.Item value="" disabled
                                                >{$t(
                                                    "trades.wizard.placeholders.no_states",
                                                )}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>

                            <!-- Modality -->
                            <div class="space-y-1.5 lg:col-span-2">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t("trades.filters.modality")}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.modality_id}
                                >
                                    <Select.Trigger
                                        class="h-10 bg-zinc-900 border-muted/40 text-xs text-white"
                                    >
                                        {settingsStore.modalities.find(
                                            (m) =>
                                                m.id === formData.modality_id,
                                        )?.name ||
                                            $t(
                                                "trades.wizard.placeholders.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each settingsStore.modalities as modality}
                                            <Select.Item value={modality.id}
                                                >{modality.name}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.stop_loss",
                                    )}</Label
                                >
                                <Input
                                    type="number"
                                    step="0.00001"
                                    bind:value={formData.stop_loss}
                                    class="h-10 bg-zinc-900 border-muted/40 text-xs font-mono text-white"
                                    placeholder="0.00"
                                />
                                {#if formData.stop_loss && formData.entry_price && formData.asset}
                                    {@const asset = settingsStore.assets.find(
                                        (a) => a.symbol === formData.asset,
                                    )}
                                    <div
                                        class="text-[9px] text-red-500 font-bold uppercase tracking-tighter"
                                    >
                                        {$t("trades.wizard.tooltips.risk")}: R$ {Math.abs(
                                            (formData.entry_price -
                                                formData.stop_loss) *
                                                formData.quantity *
                                                (asset?.point_value || 1),
                                        ).toLocaleString($locale || "pt-BR", {
                                            minimumFractionDigits: 2,
                                        })}
                                    </div>
                                {/if}
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.take_profit",
                                    )}</Label
                                >
                                <Input
                                    type="number"
                                    step="0.00001"
                                    bind:value={formData.take_profit}
                                    class="h-10 bg-zinc-900 border-muted/40 text-xs font-mono text-white"
                                    placeholder="0.00"
                                />
                                {#if formData.take_profit && formData.entry_price && formData.asset}
                                    {@const asset = settingsStore.assets.find(
                                        (a) => a.symbol === formData.asset,
                                    )}
                                    <div
                                        class="text-[9px] text-emerald-500 font-bold uppercase tracking-tighter"
                                    >
                                        {$t("trades.wizard.tooltips.target")}:
                                        R$ {Math.abs(
                                            (formData.take_profit -
                                                formData.entry_price) *
                                                formData.quantity *
                                                (asset?.point_value || 1),
                                        ).toLocaleString($locale || "pt-BR", {
                                            minimumFractionDigits: 2,
                                        })}
                                    </div>
                                {/if}
                            </div>
                        </div>
                    </section>
                </div>
            {:else if currentStep === 2}
                <div
                    class="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300"
                >
                    <section class="space-y-4">
                        <h3
                            class="text-sm font-medium text-muted-foreground uppercase tracking-wider"
                        >
                            {$t(
                                "trades.wizard.sections.partial_management.title",
                            )}
                        </h3>
                        {#if calculationResult.assetType}
                            <PartialExitsManager
                                bind:partials={formData.partial_exits}
                                entryPrice={formData.entry_price}
                                totalQuantity={formData.quantity}
                                unitLabel={calculationResult.assetType
                                    .unit_label ||
                                    $t("trades.wizard.unit_labels.contracts")}
                                resultSuffix={calculationResult.assetType
                                    .result_type === "currency"
                                    ? ""
                                    : "pts"}
                                resultPrefix={calculationResult.assetType
                                    .result_type === "currency"
                                    ? calculationResult.currencySymbol
                                    : ""}
                            />
                        {/if}
                    </section>

                    <section class="space-y-4 pt-4 border-t border-muted/30">
                        <Label
                            class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight flex items-center gap-1"
                        >
                            <Lock class="w-3 h-3" />
                            {$t("trades.wizard.sections.closing_data")}
                        </Label>
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.exit_price",
                                    )}</Label
                                >
                                <Input
                                    type="number"
                                    step="any"
                                    bind:value={formData.exit_price}
                                    class="bg-muted/30 border-0 h-8 text-xs font-mono"
                                />
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.exit_date",
                                    )}</Label
                                >
                                <div class="relative">
                                    <Input
                                        type="datetime-local"
                                        bind:value={formData.exit_date}
                                        class="bg-muted/30 border-0 pr-8 h-10 text-xs w-full"
                                    />
                                    <Calendar
                                        class="w-4 h-4 text-muted-foreground absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none"
                                    />
                                </div>
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.exit_reason",
                                    )}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.exit_reason}
                                >
                                    <Select.Trigger
                                        class="h-8 bg-muted/30 border-0 focus:ring-1 focus:ring-primary/40 text-xs text-left"
                                    >
                                        {formData.exit_reason ||
                                            $t(
                                                "trades.wizard.placeholders.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        <Select.Item value="Take Profit"
                                            >{$t(
                                                "trades.wizard.exit_reasons.take_profit",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="Stop Loss"
                                            >{$t(
                                                "trades.wizard.exit_reasons.stop_loss",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="Manual"
                                            >{$t(
                                                "trades.wizard.exit_reasons.manual",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="Time"
                                            >{$t(
                                                "trades.wizard.exit_reasons.time",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="Strategy"
                                            >{$t(
                                                "trades.wizard.exit_reasons.strategy",
                                            )}</Select.Item
                                        >
                                    </Select.Content>
                                </Select.Root>
                            </div>
                            <div class="space-y-1.5">
                                <Label
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight flex items-center gap-1"
                                >
                                    <Brain class="w-3 h-3 text-pink-400" />
                                    {$t("trades.wizard.fields.emotional_state")}
                                </Label>
                                <Select.Root
                                    type="single"
                                    bind:value={
                                        formData.exit_emotional_state_id
                                    }
                                >
                                    <Select.Trigger
                                        class="h-8 bg-muted/30 border-0 focus:ring-1 focus:ring-primary/40 text-xs"
                                    >
                                        {settingsStore.emotionalStates.find(
                                            (e) =>
                                                e.id ===
                                                formData.exit_emotional_state_id,
                                        )?.name ||
                                            $t(
                                                "trades.wizard.placeholders.select",
                                            )}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each settingsStore.emotionalStates as state}
                                            <Select.Item value={state.id}
                                                >{state.name}</Select.Item
                                            >
                                        {:else}
                                            <Select.Item value="" disabled
                                                >{$t(
                                                    "trades.wizard.placeholders.no_states",
                                                )}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                            </div>
                        </div>
                    </section>

                    <!-- Financial Summary Display -->
                    <div
                        class="mt-4 p-4 rounded-xl bg-zinc-900 border border-white/5 shadow-lg overflow-hidden relative group"
                    >
                        <div
                            class="relative flex flex-col md:flex-row justify-between items-start md:items-center gap-4"
                        >
                            <div class="flex items-center gap-3">
                                <div
                                    class="p-3 rounded-lg bg-primary/10 border border-primary/20"
                                >
                                    <TrendingUp class="w-6 h-6 text-primary" />
                                </div>
                                <div>
                                    <div class="flex items-center gap-2">
                                        <h4
                                            class="text-base font-bold tracking-tight"
                                        >
                                            {formData.asset}
                                        </h4>
                                        <span
                                            class="px-1.5 py-0.5 rounded text-[9px] font-bold {formData.direction ===
                                            'buy'
                                                ? 'bg-emerald-500/10 text-emerald-500'
                                                : 'bg-red-500/10 text-red-500'} uppercase"
                                        >
                                            {formData.direction === "buy"
                                                ? $t("trades.wizard.fields.buy")
                                                : $t(
                                                      "trades.wizard.fields.sell",
                                                  )}
                                        </span>
                                    </div>
                                    <p
                                        class="text-[9px] text-primary/80 font-bold uppercase tracking-tighter"
                                    >
                                        {$t("trades.wizard.fields.entry")}:
                                        <span class="text-white font-mono"
                                            >{formData.entry_price}</span
                                        >
                                        |
                                        <span class="text-primary"
                                            >{formData.quantity}
                                            {$t(
                                                "trades.wizard.unit_labels.contracts",
                                            )}</span
                                        >
                                    </p>
                                </div>
                            </div>
                            <div class="w-full md:w-auto text-right">
                                <p
                                    class="text-[9px] text-muted-foreground uppercase font-bold tracking-widest"
                                >
                                    {$t("trades.details.net_result")}
                                </p>
                                <h3
                                    class="text-2xl font-black {calculationResult.net >=
                                    0
                                        ? 'text-emerald-400'
                                        : 'text-red-400'}"
                                >
                                    R$ {calculationResult.net.toLocaleString(
                                        $locale || "pt-BR",
                                        { minimumFractionDigits: 2 },
                                    )}
                                </h3>
                            </div>
                        </div>
                    </div>
                </div>
            {:else if currentStep === 3}
                <div
                    class="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300"
                >
                    <section class="space-y-4">
                        <Label
                            class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                            >{$t(
                                "trades.wizard.sections.psychology_analysis",
                            )}</Label
                        >
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div class="space-y-2">
                                <Label class="text-xs"
                                    >{$t(
                                        "trades.wizard.fields.entry_rationale",
                                    )}</Label
                                >
                                <Textarea
                                    bind:value={formData.entry_rationale}
                                    placeholder={$t(
                                        "trades.wizard.placeholders.rationale",
                                    )}
                                    class="bg-muted/30 border-0 h-24 text-sm resize-none"
                                />
                            </div>
                            <div class="space-y-2">
                                <Label class="text-xs"
                                    >{$t(
                                        "trades.wizard.fields.confirmation_signals",
                                    )}</Label
                                >
                                <Textarea
                                    bind:value={formData.confirmation_signals}
                                    placeholder={$t(
                                        "trades.wizard.placeholders.signals",
                                    )}
                                    class="bg-muted/30 border-0 h-24 text-sm resize-none"
                                />
                            </div>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div class="space-y-2">
                                <Label class="text-xs"
                                    >{$t(
                                        "trades.wizard.fields.market_context",
                                    )}</Label
                                >
                                <Textarea
                                    bind:value={formData.market_context}
                                    placeholder={$t(
                                        "trades.wizard.placeholders.context",
                                    )}
                                    class="bg-muted/30 border-0 h-24 text-sm resize-none"
                                />
                            </div>
                            <div class="space-y-2">
                                <Label class="text-xs"
                                    >{$t(
                                        "trades.wizard.fields.improvements",
                                    )}</Label
                                >
                                <Textarea
                                    bind:value={formData.mistakes_improvements}
                                    placeholder={$t(
                                        "trades.wizard.placeholders.improvements",
                                    )}
                                    class="bg-muted/30 border-0 h-24 text-sm resize-none"
                                />
                            </div>
                        </div>

                        <div class="pt-4 border-t border-white/5 space-y-4">
                            <div class="flex items-center justify-between">
                                <div class="space-y-0.5">
                                    <Label
                                        class="text-xs font-bold text-white uppercase tracking-tight flex items-center gap-2"
                                    >
                                        <Brain
                                            class="w-3.5 h-3.5 text-primary"
                                        />
                                        {$t(
                                            "trades.wizard.emotions.intensity_label",
                                        )}
                                    </Label>
                                    <p
                                        class="text-[10px] text-muted-foreground uppercase"
                                    >
                                        {$t(
                                            "trades.wizard.emotions.intensity_hint",
                                        )}
                                    </p>
                                </div>
                                <span
                                    class="text-xl font-black text-primary font-mono"
                                    >{formData.intensity}</span
                                >
                            </div>
                            <input
                                type="range"
                                min="0"
                                max="10"
                                step="1"
                                bind:value={formData.intensity}
                                class="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-primary"
                            />
                            <div
                                class="flex justify-between text-[8px] text-zinc-500 font-bold uppercase tracking-widest px-1"
                            >
                                <span>{$t("trades.wizard.emotions.light")}</span
                                >
                                <span
                                    >{$t(
                                        "trades.wizard.emotions.moderate",
                                    )}</span
                                >
                                <span
                                    >{$t(
                                        "trades.wizard.emotions.extreme",
                                    )}</span
                                >
                            </div>
                        </div>
                    </section>
                </div>
            {:else if currentStep === 4}
                <div
                    class="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300"
                >
                    <section class="space-y-4 text-center py-12">
                        <h3
                            class="text-sm font-bold text-white tracking-tight uppercase"
                        >
                            {$t("trades.wizard.sections.visual_evidence.title")}
                        </h3>
                        <p
                            class="text-[10px] text-muted-foreground max-w-xs mx-auto uppercase"
                        >
                            {$t("trades.wizard.placeholders.visual_desc")}
                        </p>

                        <div
                            class="mt-4 text-left bg-zinc-950/30 rounded-xl p-4 border border-white/5"
                        >
                            <ImageUploader bind:images={formData.images} />
                        </div>
                    </section>
                </div>
            {:else if currentStep === 5}
                <div
                    class="space-y-8 animate-in fade-in slide-in-from-right-4 duration-300"
                >
                    <div
                        class="group relative overflow-hidden rounded-2xl bg-zinc-950 p-4 border border-white/5 shadow-xl"
                    >
                        <div
                            class="absolute inset-x-0 -top-px h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent"
                        ></div>
                        <div class="relative flex flex-col md:flex-row gap-6">
                            <div class="flex-1 space-y-6">
                                <div class="space-y-2">
                                    <h3
                                        class="text-lg font-black tracking-tighter text-white uppercase italic flex items-center gap-2"
                                    >
                                        <ShieldCheck
                                            class="w-5 h-5 text-primary"
                                        />
                                        {$t(
                                            "trades.wizard.sections.final_summary",
                                        )}
                                    </h3>
                                    <div
                                        class="h-0.5 w-12 bg-primary rounded-full"
                                    ></div>
                                </div>

                                <div class="grid grid-cols-2 gap-6 pt-2">
                                    <div class="space-y-0.5">
                                        <p
                                            class="text-[9px] text-muted-foreground uppercase font-bold tracking-widest"
                                        >
                                            {$t(
                                                "trades.wizard.fields.asset_direction",
                                            )}
                                        </p>
                                        <div class="flex items-center gap-2">
                                            <span
                                                class="text-base font-mono font-bold text-white"
                                                >{formData.asset}</span
                                            >
                                            <span
                                                class="px-2 py-0.5 rounded text-[8px] font-black {formData.direction ===
                                                'buy'
                                                    ? 'bg-emerald-500/20 text-emerald-400'
                                                    : 'bg-red-500/20 text-red-400'}"
                                            >
                                                {formData.direction === "buy"
                                                    ? $t(
                                                          "trades.wizard.fields.buy",
                                                      ).toUpperCase()
                                                    : $t(
                                                          "trades.wizard.fields.sell",
                                                      ).toUpperCase()}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="space-y-0.5">
                                        <p
                                            class="text-[9px] text-muted-foreground uppercase font-bold tracking-widest"
                                        >
                                            {$t("trades.table.pl")}
                                        </p>
                                        <p
                                            class="text-xl font-black {calculationResult.net >=
                                            0
                                                ? 'text-emerald-400'
                                                : 'text-red-400'}"
                                        >
                                            {calculationResult.assetType
                                                ?.result_type === "currency"
                                                ? calculationResult.currencySymbol +
                                                  " "
                                                : ""}
                                            {calculationResult.net.toLocaleString(
                                                $locale || "pt-BR",
                                                { minimumFractionDigits: 2 },
                                            )}
                                            {calculationResult.assetType
                                                ?.result_type === "points"
                                                ? " pts"
                                                : ""}
                                        </p>
                                    </div>
                                    <div class="space-y-0.5">
                                        <p
                                            class="text-[9px] text-muted-foreground uppercase font-bold tracking-widest"
                                        >
                                            {$t("trades.table.date")}
                                        </p>
                                        <p
                                            class="text-xs font-medium text-white/80"
                                        >
                                            {new Date(
                                                formData.entry_date,
                                            ).toLocaleTimeString(
                                                $locale || "pt-BR",
                                                {
                                                    hour: "2-digit",
                                                    minute: "2-digit",
                                                },
                                            )}
                                        </p>
                                    </div>
                                    <div class="space-y-0.5">
                                        <p
                                            class="text-[9px] text-muted-foreground uppercase font-bold tracking-widest"
                                        >
                                            {$t("trades.table.strategy")}
                                        </p>
                                        <p
                                            class="text-xs font-medium text-white/80 truncate"
                                        >
                                            {settingsStore.strategies.find(
                                                (s) =>
                                                    s.id ===
                                                    formData.strategy_id,
                                            )?.name ||
                                                $t("trades.wizard.summary.na")}
                                        </p>
                                    </div>
                                </div>

                                <!-- Memória de Cálculo -->
                                <div
                                    class="space-y-3 pt-6 border-t border-white/5"
                                >
                                    <h4
                                        class="text-[10px] font-black tracking-widest text-primary uppercase flex items-center gap-2"
                                    >
                                        <div
                                            class="w-1 h-1 bg-primary rounded-full"
                                        ></div>
                                        {$t(
                                            "trades.wizard.sections.calc_memory",
                                        )}
                                    </h4>

                                    <div class="space-y-2">
                                        <div
                                            class="flex justify-between items-center text-[10px] text-muted-foreground/60 font-bold uppercase tracking-tighter"
                                        >
                                            <span
                                                >{$t(
                                                    "trades.wizard.fields.notes",
                                                )}</span
                                            >
                                            <span
                                                >{$t(
                                                    "settings.api.integrations.keys.table.usage",
                                                )}</span
                                            >
                                        </div>

                                        <div class="space-y-1">
                                            {#each calculationResult.memoryItems as item}
                                                <div
                                                    class="flex justify-between items-center bg-white/5 rounded p-2 border border-white/5 transition-colors hover:bg-white/10"
                                                >
                                                    <div
                                                        class="flex items-center gap-2"
                                                    >
                                                        {#if item.type === "addition"}
                                                            <span
                                                                class="px-1 py-0.5 rounded-[4px] bg-emerald-500/10 text-emerald-400 text-[8px] font-black border border-emerald-500/20 uppercase tracking-tighter"
                                                                >{$t(
                                                                    "trades.table.entry",
                                                                )}</span
                                                            >
                                                        {:else if item.type === "exit"}
                                                            <span
                                                                class="px-1 py-0.5 rounded-[4px] bg-blue-500/10 text-blue-400 text-[8px] font-black border border-blue-500/20 uppercase tracking-tighter"
                                                                >{$t(
                                                                    "trades.table.exit",
                                                                )}</span
                                                            >
                                                        {/if}
                                                        <span
                                                            class="text-[10px] font-medium text-white/70"
                                                            >{item.label}</span
                                                        >
                                                    </div>
                                                    <span
                                                        class="text-[10px] font-mono font-bold {item.result >=
                                                        0
                                                            ? 'text-emerald-400'
                                                            : 'text-red-400'}"
                                                    >
                                                        {item.result >= 0
                                                            ? "+"
                                                            : ""}
                                                        {calculationResult
                                                            .assetType
                                                            ?.result_type ===
                                                        "currency"
                                                            ? calculationResult.currencySymbol +
                                                              " "
                                                            : ""}
                                                        {(
                                                            item.result || 0
                                                        ).toLocaleString(
                                                            $locale || "pt-BR",
                                                            {
                                                                minimumFractionDigits: 2,
                                                            },
                                                        )}
                                                        {calculationResult
                                                            .assetType
                                                            ?.result_type ===
                                                            "points" &&
                                                        item.result !== 0
                                                            ? "pts"
                                                            : ""}
                                                    </span>
                                                </div>
                                            {/each}
                                        </div>

                                        <div
                                            class="flex justify-between items-center pt-2 px-2"
                                        >
                                            <span
                                                >{$t(
                                                    "trades.details.gross_result",
                                                )}</span
                                            >
                                            <span
                                                class="text-xs font-mono font-black text-white"
                                            >
                                                {calculationResult.assetType
                                                    ?.result_type === "currency"
                                                    ? calculationResult.currencySymbol +
                                                      " "
                                                    : ""}
                                                {(
                                                    calculationResult.gross || 0
                                                ).toLocaleString(
                                                    $locale || "pt-BR",
                                                    {
                                                        minimumFractionDigits: 2,
                                                    },
                                                )}
                                                {calculationResult.assetType
                                                    ?.result_type === "points"
                                                    ? "pts"
                                                    : ""}
                                            </span>
                                        </div>
                                        <div
                                            class="flex justify-between items-center px-2"
                                        >
                                            <span
                                                >{$t(
                                                    "trades.details.fees",
                                                )}</span
                                            >
                                            <span
                                                class="text-xs font-mono font-black text-red-400"
                                            >
                                                - {calculationResult.currencySymbol}
                                                {(
                                                    calculationResult.fees || 0
                                                ).toLocaleString(
                                                    $locale || "pt-BR",
                                                    {
                                                        minimumFractionDigits: 2,
                                                    },
                                                )}
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="w-full md:w-56 space-y-4">
                                <div
                                    class="p-4 rounded-xl bg-white/5 border border-white/5 space-y-4 shadow-inner"
                                >
                                    <div
                                        class="flex justify-between items-center"
                                    >
                                        <span
                                            class="text-[9px] text-muted-foreground uppercase font-bold"
                                            >{$t(
                                                "trades.wizard.summary.images",
                                            )}</span
                                        >
                                        <span
                                            class="text-lg font-black text-white"
                                            >{formData.images.length}</span
                                        >
                                    </div>
                                    <div
                                        class="flex justify-between items-center pt-2 border-t border-white/5"
                                    >
                                        <span
                                            class="text-[9px] text-muted-foreground uppercase font-bold"
                                            >{$t(
                                                "trades.wizard.fields.followed_plan",
                                            )}</span
                                        >
                                        <span
                                            class="px-2 py-0.5 rounded-full text-[8px] font-black {formData.followed_plan
                                                ? 'bg-emerald-500/10 text-emerald-400'
                                                : 'bg-red-500/10 text-red-400'} border border-white/5"
                                        >
                                            {formData.followed_plan
                                                ? $t("trades.wizard.summary.s")
                                                : $t("trades.wizard.summary.n")}
                                        </span>
                                    </div>
                                    <button
                                        class="w-full h-12 bg-primary hover:bg-primary/90 text-primary-foreground font-black text-xs rounded-xl transition-all shadow-lg active:scale-[0.98] flex items-center justify-center gap-2 group disabled:opacity-50"
                                        onclick={handleSubmit}
                                        disabled={isSubmitting}
                                    >
                                        {#if isSubmitting}
                                            <RotateCcw
                                                class="w-4 h-4 animate-spin"
                                            />
                                        {:else}
                                            {$t("general.save").toUpperCase()}
                                            <Save
                                                class="w-4 h-4 transition-transform group-hover:scale-110"
                                            />
                                        {/if}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            {/if}
        </div>
    </div>

    <!-- Navigation Footer (Flat Design) -->
    <div
        class="px-6 py-4 border-t border-border bg-card/60 flex items-center justify-between"
    >
        <Button
            variant="outline"
            onclick={close}
            class="border-muted text-muted-foreground hover:bg-muted/10"
        >
            <X class="w-4 h-4 mr-2" />
            {$t("general.cancel")}
        </Button>

        <div class="flex gap-3">
            {#if currentStep > 1}
                <Button variant="ghost" onclick={handlePrev}>
                    <ChevronLeft class="w-4 h-4 mr-2" />
                    {$t("general.back")}
                </Button>
            {/if}

            {#if currentStep < steps.length}
                <Button onclick={handleNext} class="min-w-[120px]">
                    {$t("general.next")}
                    <ChevronRight class="w-4 h-4 ml-2" />
                </Button>
            {:else}
                <Button
                    onclick={handleSubmit}
                    disabled={isSubmitting}
                    class="bg-primary text-primary-foreground min-w-[150px] font-bold shadow-[0_0_20px_rgba(var(--primary),0.4)]"
                >
                    <Save class="w-4 h-4 mr-2" />
                    {isSubmitting ? $t("general.saving") : $t("general.finish")}
                </Button>
            {/if}
        </div>
    </div>
</div>

<style>
    /* Make native calendar picker transparent and fill the input area to ensure clickability */
    :global(input[type="datetime-local"]::-webkit-calendar-picker-indicator) {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        width: 100%;
        height: 100%;
        margin: 0;
        padding: 0;
        cursor: pointer;
        background: transparent !important;
        color: transparent !important;
        appearance: none !important;
    }
</style>
