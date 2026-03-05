<script lang="ts">
    import { t, locale } from "svelte-i18n";
    import { invoke } from "@tauri-apps/api/core";
    import { emit } from "@tauri-apps/api/event";
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
    import { riskEngine } from "$lib/logic/riskEngine.svelte";
    import { AlertCircle } from "lucide-svelte";
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
        ExternalLink,
        Maximize2,
    } from "lucide-svelte";

    let {
        trade = null,
        editTradeId = undefined,
        close = () => {},
        onsave = () => {},
        detached = false,
    } = $props<{
        trade?: any;
        editTradeId?: string;
        close: () => void;
        onsave?: () => void;
        detached?: boolean;
    }>();

    // Multi-window utility
    const detach = async () => {
        try {
            // Save current state to localStorage for the new window to pick up
            const stateToPass = $state.snapshot(formData);
            localStorage.setItem(
                "pending_trade_wizard_state",
                JSON.stringify(stateToPass),
            );

            await invoke("open_detached_trade_window", {
                theme: settingsStore.userProfile.theme,
            });
            close(); // Close the modal in the main window
        } catch (e) {
            console.error("Failed to detach window:", e);
        }
    };

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

    let selectedAsset = $derived.by(() => {
        const symbol = formData.asset;
        return untrack(() => {
            const allAssets = [...settingsStore.assets, ...rtdAssets];
            return allAssets.find((a) => a.symbol === symbol);
        });
    });

    let selectedAccount = $derived.by(() => {
        const id = formData.account_id;
        return untrack(() => settingsStore.accounts.find((a) => a.id === id));
    });

    let activeRiskProfile = $derived.by(() => {
        const account = selectedAccount;
        if (!account) return settingsStore.riskProfiles[0] || null;

        // Find profile for specific account type or "All"
        return (
            settingsStore.riskProfiles.find(
                (p) =>
                    p.account_type_applicability === account.account_type ||
                    p.account_type_applicability === "All",
            ) ||
            settingsStore.riskProfiles[0] ||
            null
        );
    });

    const suggestedLotMultiplier = $derived(
        riskEngine.getSuggestedLotMultiplier(activeRiskProfile),
    );
    const riskWarnings = $derived(
        riskEngine.getProactiveWarnings(activeRiskProfile),
    );

    // Auto-apply lot adjustment if user clicks
    function applyLotAdjustment() {
        formData.quantity = Math.round(
            formData.quantity * suggestedLotMultiplier,
        );
        toast.success($t("trades.wizard.risk.toast_lot_adjusted"));
    }

    let priceHasFocus = $state(false);

    let lastSyncedTradeId = $state<string | undefined>(undefined);
    let lastSyncedDraftKey = $state<string | undefined>(undefined);
    let closureAlreadyExists = $state(false);

    // Track original result for DARF increase warning (Fiscal Guard)
    let originalResult = $state<number | null>(null);

    // Check if a daily closure already exists for data/account to warn the user
    $effect(() => {
        const dateStr = formData.entry_date;
        const accId = formData.account_id;
        if (dateStr && accId) {
            settingsStore
                .hasClosureForDate(dateStr, accId)
                .then((exists) => (closureAlreadyExists = exists));
        } else {
            closureAlreadyExists = false;
        }
    });

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
                status:
                    currentTrade.exit_price !== null &&
                    currentTrade.exit_price !== undefined
                        ? "closed"
                        : "open",

                images: currentTrade.images || [],
                partial_exits: (currentTrade.partial_exits || []).map(
                    (p: any) => ({
                        ...p,
                        price: parseFloat(p.price) || 0,
                        quantity: parseFloat(p.quantity) || 0,
                    }),
                ),
            };

            // Capture original result for Fiscal Guard comparison
            originalResult = parseFloat(currentTrade.result as any) || 0;
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

            originalResult = null;
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

    // Reactive sync for snapshots (for detached windows or slow loads)
    $effect(() => {
        if (!settingsStore.isLoadingData) {
            if (
                accountsList.length === 0 &&
                settingsStore.accounts.length > 0
            ) {
                accountsList = [...settingsStore.accounts];
            }
            if (
                strategiesList.length === 0 &&
                settingsStore.strategies.length > 0
            ) {
                strategiesList = [...settingsStore.strategies];
            }
            if (
                assetTypesList.length === 0 &&
                settingsStore.assetTypes.length > 0
            ) {
                assetTypesList = [...settingsStore.assetTypes];
            }
            if (assetsList.length === 0 && settingsStore.assets.length > 0) {
                assetsList = [...settingsStore.assets];
            }
            if (
                timeframeList.length === 0 &&
                settingsStore.timeframes.length > 0
            ) {
                timeframeList = [...settingsStore.timeframes];
            }
        }
    });

    onMount(() => {
        // --- RESTORE DETACHED STATE ---
        if (detached) {
            const savedState = localStorage.getItem(
                "pending_trade_wizard_state",
            );
            if (savedState) {
                try {
                    const parsed = JSON.parse(savedState);
                    formData = { ...formData, ...parsed };
                    console.log(
                        "[NewTradeWizard] Restored state in detached window.",
                    );
                    // Clear it so it doesn't leak to future windows
                    localStorage.removeItem("pending_trade_wizard_state");
                } catch (e) {
                    console.error("Failed to restore detached state:", e);
                }
            }
        }

        // Capture initial snapshots from store
        if (!settingsStore.isLoadingData) {
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
        }

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
        if (!userManuallySelectedType && formData.asset) {
            const allAssets = [...settingsStore.assets, ...rtdAssets];
            const asset = allAssets.find(
                (a) => a.symbol.toUpperCase() === formData.asset.toUpperCase(),
            );
            if (asset) {
                const type = settingsStore.assetTypes.find(
                    (t) =>
                        t.id === asset.asset_type_id ||
                        t.id.replace(/^asset_type:/, "") ===
                            asset.asset_type_id.replace(/^asset_type:/, ""),
                );
                if (type && type.id !== selectedAssetTypeId) {
                    selectedAssetTypeId = type.id;
                    console.log(
                        "[NewTradeWizard] Auto-syncing from asset symbol (direct store access):",
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
                    default_fee_id: undefined,
                    tax_profile_id: undefined,
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

    function sanitize(val: any): any {
        if (typeof val === "string") return val.trim().slice(0, 1000); // Limit long strings
        return val;
    }

    async function handleSubmit() {
        // CRITICAL: Use editTradeId PROP (passed from parent at mount time via {#key}) as the
        // submission mode indicator. This is immune to Svelte $effect re-evaluation during async saves.
        // lastSyncedTradeId can be reset mid-save if the trade prop changes reactively.
        const submissionId = editTradeId;
        console.log(
            "[NewTradeWizard] Submitting form. Mode:",
            submissionId ? "Edit" : "New",
            "Target ID (editTradeId prop):",
            submissionId,
            "lastSyncedTradeId at submit time:",
            lastSyncedTradeId,
            "trade?.id at submit time:",
            trade?.id,
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

        // --- SECURITY & VALIDATION LAYER ---
        const cleanAsset = sanitize(formData.asset).toUpperCase();
        if (cleanAsset.length < 2 || cleanAsset.length > 20) {
            toast.error($t("trades.wizard.messages.invalid_asset_symbol"));
            return;
        }

        const qty = Number(formData.quantity);
        if (isNaN(qty) || qty <= 0 || qty > 1000000000) {
            toast.error($t("trades.wizard.messages.invalid_quantity"));
            return;
        }

        const ePrice = Number(formData.entry_price);
        if (isNaN(ePrice) || ePrice <= 0) {
            toast.error($t("trades.wizard.messages.invalid_entry_price"));
            return;
        }

        if (formData.status === "closed" || formData.exit_price !== null) {
            const exPrice = Number(formData.exit_price);
            if (isNaN(exPrice) || exPrice <= 0) {
                toast.error($t("trades.wizard.messages.invalid_exit_price"));
                return;
            }
        }

        isSubmitting = true;
        try {
            const tradeData: any = {
                // Save as ISO string to preserve time for editing later
                date: (formData.entry_date as string)?.includes("T")
                    ? formData.entry_date + ":00Z"
                    : formData.entry_date + "T00:00:00Z",
                asset_symbol: cleanAsset,
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

            if (submissionId) {
                // FISCAL GUARD (d5398093): Warn if profit increase might require complementary DARF
                const currentNetResult = calculationResult.net;
                if (
                    originalResult !== null &&
                    activeRiskProfile?.id !== "demo"
                ) {
                    const month = formData.entry_date.substring(0, 7);
                    const nowMonth = new Date().toISOString().substring(0, 7);

                    // Significant increase (> R$ 10.0 or 20% relative to month total?)
                    // Simple threshold: if new result > original + 10.0 (minimum DARF trigger)
                    const profitIncrease = currentNetResult - originalResult;

                    if (profitIncrease > 10.0 && month < nowMonth) {
                        const confirmed = confirm(
                            $t(
                                "trades.wizard.messages.complementary_darf_warning",
                            ),
                        );
                        if (!confirmed) {
                            isSubmitting = false;
                            return;
                        }
                    }
                }

                console.log(
                    "[NewTradeWizard] Calling updateTrade for ID:",
                    submissionId,
                );
                const result = await tradesStore.updateTrade(
                    submissionId,
                    tradeData,
                );
                if (result.success) {
                    toast.success($t("trades.wizard.messages.update_success"));
                    currentStep = 1;
                    emit("trade-saved", { mode: "update" }).catch(() => {});
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
                    if (closureAlreadyExists && !submissionId) {
                        toast.success(
                            $t("trades.wizard.messages.save_success_with_sync"),
                        );
                    } else {
                        toast.success(
                            $t("trades.wizard.messages.save_success"),
                        );
                    }
                    currentStep = 1;
                    emit("trade-saved", { mode: "new" }).catch(() => {});
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

            <div class="flex items-center gap-2">
                {#if !detached}
                    <Button
                        variant="ghost"
                        size="icon"
                        class="h-8 w-8 text-muted-foreground hover:text-primary"
                        onclick={detach}
                        title="Abrir em janela independente"
                    >
                        <ExternalLink class="w-4 h-4" />
                    </Button>
                {/if}
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
    <div class="flex-1 overflow-y-auto p-4 bg-muted/40 backdrop-blur-md">
        <div class="max-w-3xl mx-auto space-y-4">
            {#if currentStep === 1}
                <div
                    class="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300"
                >
                    <section class="space-y-4">
                        {#if closureAlreadyExists && !trade?.id}
                            <div
                                class="p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg space-y-2 mb-4 animate-in fade-in zoom-in duration-300"
                            >
                                <div
                                    class="flex items-center gap-2 text-blue-600 dark:text-blue-400"
                                >
                                    <ShieldCheck class="w-4 h-4" />
                                    <span
                                        class="text-xs font-bold uppercase tracking-wider"
                                        >Aviso de Sincronização</span
                                    >
                                </div>
                                <p
                                    class="text-[10px] text-blue-800 dark:text-blue-200/80 leading-tight"
                                >
                                    {$t(
                                        "trades.wizard.messages.closure_exists_warning",
                                    )}
                                </p>
                            </div>
                        {/if}

                        {#if riskWarnings.length > 0}
                            <div
                                class="p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg space-y-2 animate-in fade-in zoom-in duration-300"
                            >
                                <div
                                    class="flex items-center gap-2 text-amber-500"
                                >
                                    <AlertCircle class="w-4 h-4" />
                                    <span
                                        class="text-xs font-bold uppercase tracking-wider"
                                        >{$t(
                                            "trades.wizard.risk.alerts_title",
                                        )}</span
                                    >
                                </div>
                                <ul class="space-y-1">
                                    {#each riskWarnings as warning}
                                        <li
                                            class="text-[10px] text-amber-800 dark:text-amber-200/80 leading-tight"
                                        >
                                            • {$t(warning.key, warning.params)}
                                        </li>
                                    {/each}
                                </ul>

                                {#if suggestedLotMultiplier < 1.0}
                                    <Button
                                        variant="outline"
                                        size="sm"
                                        class="w-full h-7 text-[9px] border-amber-500/40 bg-amber-500/5 hover:bg-amber-500/20 text-amber-700 dark:text-amber-200 uppercase font-bold"
                                        onclick={applyLotAdjustment}
                                    >
                                        {$t(
                                            "trades.wizard.risk.reduce_lot_button",
                                            {
                                                percent: Math.round(
                                                    (1 -
                                                        suggestedLotMultiplier) *
                                                        100,
                                                ),
                                            } as any,
                                        )}
                                    </Button>
                                {/if}
                            </div>
                        {/if}

                        <h3
                            class="text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em]"
                        >
                            {$t("trades.wizard.sections.context")}
                        </h3>
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
                            <div class="space-y-1.5">
                                <Label
                                    for="account-select"
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t("trades.wizard.fields.account")}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.account_id}
                                >
                                    <Select.Trigger
                                        id="account-select"
                                        class="h-8 bg-muted/20 border-border/40 focus:ring-1 focus:ring-primary/40 text-xs text-foreground"
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
                                    for="strategy-select"
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
                                        id="strategy-select"
                                        class="h-8 bg-muted/20 border-border/40 focus:ring-1 focus:ring-primary/40 text-xs text-foreground"
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
                                    for="timeframe-select"
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
                                        id="timeframe-select"
                                        class="h-8 bg-muted/20 border-border/40 focus:ring-1 focus:ring-primary/40 text-xs text-foreground"
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
                                    for="volatility-select"
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
                                        id="volatility-select"
                                        class="h-8 bg-muted/20 border-border/40 focus:ring-1 focus:ring-primary/40 text-xs text-foreground"
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
                                            for="asset-type-select"
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
                                                id="asset-type-select"
                                                class="h-8 bg-muted/40 border-border/20 text-[10px] w-full"
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
                                            for="asset-select-main"
                                            class="text-[9px] uppercase font-bold text-muted-foreground tracking-tighter"
                                            >{$t("trades.wizard.fields.asset")}
                                            <span class="text-rose-500">*</span
                                            ></Label
                                        >
                                        <Select.Root
                                            type="single"
                                            bind:value={formData.asset}
                                        >
                                            <Select.Trigger
                                                id="asset-select-main"
                                                class="h-10 bg-muted/40 border-border/20 text-xs font-bold w-full"
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
                                        <span class="text-rose-500">*</span
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
                                                : 'bg-muted/40 text-muted-foreground hover:bg-muted/60'}"
                                            onclick={() =>
                                                (formData.direction = "buy")}
                                        >
                                            {$t("trades.wizard.fields.buy")}
                                        </button>
                                        <button
                                            type="button"
                                            class="rounded-md font-bold text-xs transition-all flex items-center justify-center {formData.direction ===
                                            'sell'
                                                ? 'bg-rose-500 text-white shadow-[0_0_15px_rgba(239,68,68,0.3)]'
                                                : 'bg-muted/40 text-muted-foreground hover:bg-muted/60'}"
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
                                                class="h-10 bg-muted/40 border-border/20 text-xs text-foreground pl-3 pr-8 w-full"
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
                                            <span class="text-rose-500">*</span
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
                                                class="h-10 bg-muted/40 border-border/40 text-sm font-mono font-bold text-foreground flex-1"
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
                                    for="quantity-input"
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t("trades.wizard.fields.quantity")}
                                    <span class="text-red-500">*</span></Label
                                >
                                <Input
                                    id="quantity-input"
                                    type="number"
                                    step="any"
                                    bind:value={formData.quantity}
                                    class="h-10 bg-muted/40 border-border/40 text-sm font-mono font-bold text-foreground pl-4"
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
                                        class="h-10 bg-muted/40 border-border/40 text-xs text-foreground"
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
                                    for="modality-select"
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t("trades.filters.modality")}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.modality_id}
                                >
                                    <Select.Trigger
                                        id="modality-select"
                                        class="h-10 bg-muted/40 border-border/40 text-xs text-foreground"
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
                                    for="stop-loss-input"
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.stop_loss",
                                    )}</Label
                                >
                                <Input
                                    id="stop-loss-input"
                                    type="number"
                                    step="0.00001"
                                    bind:value={formData.stop_loss}
                                    class="h-10 bg-muted/40 border-border/40 text-xs font-mono font-bold text-foreground"
                                    placeholder="0.00"
                                />
                                {#if formData.stop_loss && formData.entry_price && formData.asset}
                                    {@const asset = settingsStore.assets.find(
                                        (a) => a.symbol === formData.asset,
                                    )}
                                    <div
                                        class="text-[9px] font-mono text-red-500 font-bold uppercase tracking-tighter"
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
                                    for="take-profit-input"
                                    class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight"
                                    >{$t(
                                        "trades.wizard.fields.take_profit",
                                    )}</Label
                                >
                                <Input
                                    id="take-profit-input"
                                    type="number"
                                    step="0.00001"
                                    bind:value={formData.take_profit}
                                    class="h-10 bg-input/30 border-border/20 text-xs font-mono font-bold text-foreground"
                                    placeholder="0.00"
                                />
                                {#if formData.take_profit && formData.entry_price && formData.asset}
                                    {@const asset = settingsStore.assets.find(
                                        (a) => a.symbol === formData.asset,
                                    )}
                                    <div
                                        class="text-[9px] font-mono text-emerald-500 font-bold uppercase tracking-tighter"
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
                        <div class="flex items-center justify-between">
                            <Label
                                class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight flex items-center gap-1"
                            >
                                <Lock class="w-3 h-3" />
                                {$t("trades.wizard.sections.closing_data")}
                            </Label>
                            <div class="flex items-center gap-2">
                                <span
                                    class="text-[10px] font-bold uppercase tracking-tighter {formData.status ===
                                    'open'
                                        ? 'text-primary'
                                        : 'text-muted-foreground'}">Aberto</span
                                >
                                <button
                                    class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center justify-center rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 transition-colors bg-muted/40 border border-border/40"
                                    type="button"
                                    role="switch"
                                    aria-label="Toggle trade status"
                                    aria-checked={formData.status === "closed"}
                                    onclick={(e) => {
                                        e.preventDefault();
                                        formData.status =
                                            formData.status === "open"
                                                ? "closed"
                                                : "open";
                                        if (formData.status === "open") {
                                            formData.exit_price = null;
                                            formData.exit_date = null;
                                            formData.exit_reason = "";
                                            formData.exit_emotional_state_id =
                                                "";
                                        }
                                    }}
                                >
                                    <span
                                        class="pointer-events-none block h-4 w-4 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0 {formData.status ===
                                        'closed'
                                            ? 'translate-x-4 bg-emerald-500'
                                            : 'translate-x-0'}"
                                    ></span>
                                </button>
                                <span
                                    class="text-[10px] font-bold uppercase tracking-tighter {formData.status ===
                                    'closed'
                                        ? 'text-emerald-500'
                                        : 'text-muted-foreground'}"
                                    >Fechado</span
                                >
                            </div>
                        </div>

                        {#if formData.status === "closed"}
                            <div
                                class="grid grid-cols-2 lg:grid-cols-4 gap-3 animate-in fade-in slide-in-from-top-2 duration-200"
                            >
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
                                        class="bg-muted/30 border-border/40 h-8 text-xs font-mono font-bold text-foreground"
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
                                            class="bg-muted/30 border-border/40 pr-8 h-10 text-xs w-full text-foreground"
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
                                            class="h-8 bg-muted/30 border-border/40 focus:ring-1 focus:ring-primary/40 text-xs text-left"
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
                                        {$t(
                                            "trades.wizard.fields.emotional_state",
                                        )}
                                    </Label>
                                    <Select.Root
                                        type="single"
                                        bind:value={
                                            formData.exit_emotional_state_id
                                        }
                                    >
                                        <Select.Trigger
                                            class="h-8 bg-input/20 border-border/20 focus:ring-1 focus:ring-primary/40 text-xs"
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
                        {/if}
                    </section>

                    <!-- Financial Summary Display -->
                    <div
                        class="mt-4 p-4 rounded-xl bg-card border border-border/40 shadow-lg overflow-hidden relative group"
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
                                        <span
                                            class="text-foreground font-mono font-bold"
                                            >{formData.entry_price}</span
                                        >
                                        |
                                        <span
                                            class="text-primary font-mono font-bold"
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
                                <Label
                                    for="entry-rationale-text"
                                    class="text-xs"
                                    >{$t(
                                        "trades.wizard.fields.entry_rationale",
                                    )}</Label
                                >
                                <Textarea
                                    id="entry-rationale-text"
                                    bind:value={formData.entry_rationale}
                                    placeholder={$t(
                                        "trades.wizard.placeholders.rationale",
                                    )}
                                    class="bg-muted/30 border-border/40 h-24 text-sm resize-none"
                                />
                            </div>
                            <div class="space-y-2">
                                <Label
                                    for="confirmation-signals-text"
                                    class="text-xs"
                                    >{$t(
                                        "trades.wizard.fields.confirmation_signals",
                                    )}</Label
                                >
                                <Textarea
                                    id="confirmation-signals-text"
                                    bind:value={formData.confirmation_signals}
                                    placeholder={$t(
                                        "trades.wizard.placeholders.signals",
                                    )}
                                    class="bg-input/20 border-border/20 h-24 text-sm resize-none"
                                />
                            </div>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div class="space-y-2">
                                <Label for="market-context-text" class="text-xs"
                                    >{$t(
                                        "trades.wizard.fields.market_context",
                                    )}</Label
                                >
                                <Textarea
                                    id="market-context-text"
                                    bind:value={formData.market_context}
                                    placeholder={$t(
                                        "trades.wizard.placeholders.context",
                                    )}
                                    class="bg-input/20 border-border/20 h-24 text-sm resize-none"
                                />
                            </div>
                            <div class="space-y-2">
                                <Label for="improvements-text" class="text-xs"
                                    >{$t(
                                        "trades.wizard.fields.improvements",
                                    )}</Label
                                >
                                <Textarea
                                    id="improvements-text"
                                    bind:value={formData.mistakes_improvements}
                                    placeholder={$t(
                                        "trades.wizard.placeholders.improvements",
                                    )}
                                    class="bg-muted/30 border-border/40 h-24 text-sm resize-none"
                                />
                            </div>
                        </div>

                        <div class="pt-4 border-t border-white/5 space-y-4">
                            <div class="flex items-center justify-between">
                                <div class="space-y-0.5">
                                    <Label
                                        for="emotion-intensity-range"
                                        class="text-xs font-bold text-foreground uppercase tracking-tight flex items-center gap-2"
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
                                id="emotion-intensity-range"
                                type="range"
                                min="0"
                                max="10"
                                step="1"
                                bind:value={formData.intensity}
                                class="w-full h-1.5 bg-muted/40 rounded-lg appearance-none cursor-pointer accent-primary"
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
                            class="mt-4 text-left bg-muted/30 rounded-xl p-4 border border-border/40"
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
                        class="group relative overflow-hidden rounded-2xl bg-card p-4 border border-border/40 shadow-xl"
                    >
                        <div
                            class="absolute inset-x-0 -top-px h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent"
                        ></div>
                        <div class="relative flex flex-col md:flex-row gap-6">
                            <div class="flex-1 space-y-6">
                                <div class="space-y-2">
                                    <h3
                                        class="text-[10px] font-bold text-muted-foreground uppercase tracking-[0.2em] flex items-center gap-2"
                                    >
                                        <ShieldCheck
                                            class="w-4 h-4 text-primary"
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
                                                class="text-base font-mono font-bold text-foreground"
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
                                            class="text-xs font-medium text-foreground/80"
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
                                            class="text-xs font-medium text-foreground/80 truncate"
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
                                    class="space-y-3 pt-6 border-t border-border/40"
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
                                                    class="flex justify-between items-center bg-muted/30 rounded p-2 border border-border/40 transition-colors hover:bg-muted/40"
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
                                                            class="text-[10px] font-medium text-foreground/70"
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
                                                class="text-xs font-mono font-black text-foreground"
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
                                    class="p-4 rounded-xl bg-muted/20 border border-border/40 space-y-4 shadow-inner"
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
                                            class="text-lg font-black text-foreground"
                                            >{formData.images.length}</span
                                        >
                                    </div>
                                    <div
                                        class="flex justify-between items-center pt-2 border-t border-border/20"
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
                                                : 'bg-red-500/10 text-red-400'} border border-border/20"
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
