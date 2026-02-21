import { invoke } from "@tauri-apps/api/core";
import { fetch } from "@tauri-apps/plugin-http";
import { getLocalDatePart } from "$lib/utils";
import { validateLicenseKey, computeCustomerId, type LicenseData } from "$lib/utils/license";

// Mock Global Store for Settings (Simulating Backend DB)
// Using Svelte 5 Runes for reactivity outside components

import type {
    TradingSession, Market, AssetType, Asset, Currency, Account,
    JournalEntry, Trade, EmotionalState, Strategy, UserProfile,
    FeeProfile, RiskProfile, Modality, Tag, Indicator, Timeframe,
    ChartType, ApiConfig, CashTransaction, TaxRule, TaxMapping, TaxProfile, TaxProfileEntry
} from "$lib/types";

export type {
    TradingSession, Market, AssetType, Asset, Currency, Account,
    JournalEntry, Trade, EmotionalState, Strategy, UserProfile,
    FeeProfile, RiskProfile, Modality, Tag, Indicator, Timeframe,
    ChartType, ApiConfig, CashTransaction, TaxRule, TaxMapping, TaxProfile, TaxProfileEntry
} from "$lib/types";

class SettingsStore {
    markets = $state<Market[]>([]);
    assetTypes = $state<AssetType[]>([]);
    assets = $state<Asset[]>([]);
    riskProfiles = $state<RiskProfile[]>([]);
    modalities = $state<Modality[]>([]);
    userProfile = $state<UserProfile>({
        id: "main",
        name: "",
        email: "",
        phone: "",
        cpf: "",
        theme: "dark",
        language: "pt-BR",
        timezone: "America/Sao_Paulo",
        main_currency: "BRL",
        avatar: null,
        convert_all_to_main: false,
        onboarding_completed: false,
        currency_api_url: "https://economia.awesomeapi.com.br/last/",
        birth_date: null,
        trial_start_date: null,
        license_key: null,
        utc_offset: -180, // Default to Brasilia
    });

    psychologyApiId = $state<string>("none");
    marketDataApiId = $state<string>("none");
    rtdEnabled = $state<boolean>(false);
    rtdExcelPath = $state<string>("");

    emotionalStates = $state<EmotionalState[]>([]);
    tags = $state<Tag[]>([]);
    indicators = $state<Indicator[]>([]);
    timeframes = $state<Timeframe[]>([]);
    chartTypes = $state<ChartType[]>([]);
    cashTransactions = $state<CashTransaction[]>([]);
    journalEntries = $state<JournalEntry[]>([]);
    apiConfigs = $state<ApiConfig[]>([]);
    currencies = $state<Currency[]>([]);
    accounts = $state<Account[]>([]);
    fees = $state<FeeProfile[]>([]);
    taxRules = $state<TaxRule[]>([]);
    taxMappings = $state<TaxMapping[]>([]);
    taxProfiles = $state<TaxProfile[]>([]);
    taxProfileEntries = $state<TaxProfileEntry[]>([]);
    strategies = $state<Strategy[]>([]);
    hardwareId = $state<string>("");
    licenseDetails = $state<LicenseData | null>(null);
    isLoadingData = $state<boolean>(false);


    // Computed license info
    licenseStatus = $derived.by(() => {
        if (this.licenseDetails?.valid) {
            return "active";
        }

        if (!this.userProfile.trial_start_date) return "trial";

        const start = new Date(this.userProfile.trial_start_date);
        const now = new Date();
        const diffTime = Math.abs(now.getTime() - start.getTime());
        const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

        return diffDays > 7 ? "expired" : "trial";
    });

    trialDaysLeft = $derived.by(() => {
        if (!this.userProfile.trial_start_date) return 7;
        const start = new Date(this.userProfile.trial_start_date);
        const now = new Date();
        const diffTime = now.getTime() - start.getTime();
        const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));
        return Math.max(0, 7 - diffDays);
    });

    licensePlanName = $derived(this.licenseDetails?.plan || "Trial");

    licenseTotalDays = $derived.by(() => {
        if (this.licensePlanName === "Lifetime") return null;
        if (!this.licenseDetails?.expiration || !this.licenseDetails?.createdAt) return null;
        const start = new Date(this.licenseDetails.createdAt);
        const end = new Date(this.licenseDetails.expiration);
        const diffTime = end.getTime() - start.getTime();
        return Math.round(diffTime / (1000 * 60 * 60 * 24));
    });

    licenseDaysRemaining = $derived.by(() => {
        if (this.licensePlanName === "Lifetime") return null;
        if (!this.licenseDetails?.expiration) return null;
        const end = new Date(this.licenseDetails.expiration);
        const now = new Date();
        const diffTime = end.getTime() - now.getTime();
        return Math.max(0, Math.ceil(diffTime / (1000 * 60 * 60 * 24)));
    });


    constructor() {
        if (typeof window !== "undefined") {
            this.loadData();
        }
    }

    async refreshLicenseStatus() {
        if (!this.userProfile.license_key) {
            console.log("[SettingsStore] No license key found, skipping refresh.");
            return;
        }

        console.log("[SettingsStore] Refreshing license status for key:", this.userProfile.license_key.substring(0, 10) + "...");

        try {
            const customerId = await computeCustomerId({
                name: this.userProfile.name,
                cpf: this.userProfile.cpf,
                birthDate: this.userProfile.birth_date || "",
                hardwareId: this.hardwareId
            });

            console.log("[SettingsStore] Computed Customer ID:", customerId);

            const result = await validateLicenseKey(this.userProfile.license_key, customerId);
            console.log("[SettingsStore] License Validation Result:", result);
            this.licenseDetails = result;
        } catch (e) {
            console.error("[SettingsStore] Error refreshing license:", e);
        }
    }

    async loadData() {
        if (this.isLoadingData) {
            console.log("[SettingsStore] loadData already in progress, skipping.");
            return;
        }
        this.isLoadingData = true;
        console.log("[SettingsStore] Starting loadData...");

        const safeInvoke = async <T>(command: string, label: string): Promise<T | null> => {
            try {
                console.log(`[SettingsStore] Fetching ${label}...`);
                const result = await invoke(command) as T;
                if (Array.isArray(result)) {
                    console.log(`[SettingsStore] ${label} loaded: ${result.length} items`);
                } else {
                    console.log(`[SettingsStore] ${label} loaded:`, result ? "success" : "null");
                }
                return result;
            } catch (e) {
                console.error(`[SettingsStore] ERROR loading ${label}:`, e);
                return null;
            }
        };

        try {
            // User Profile
            const profile = await safeInvoke<UserProfile>("get_user_profile", "User Profile");
            console.log("[SettingsStore] Received profile from backend:", profile);
            if (profile) {
                this.userProfile = { ...this.userProfile, ...profile };
                console.log("[SettingsStore] userProfile updated to:", $state.snapshot(this.userProfile));

                // Initialize trial if first run
                if (!this.userProfile.trial_start_date && this.userProfile.onboarding_completed) {
                    this.userProfile.trial_start_date = new Date().toISOString();
                    this.saveUserProfile();
                }
            }

            // Hardware ID
            const hwid = await safeInvoke<string>("get_machine_id_cmd", "Hardware ID");
            if (hwid) this.hardwareId = hwid;

            // API Configs
            const apiConfigs = await safeInvoke<ApiConfig[]>("get_api_configs", "API Configs");
            if (apiConfigs && apiConfigs.length > 0) this.apiConfigs = apiConfigs;

            // Accounts
            const accounts = await safeInvoke<Account[]>("get_accounts", "Accounts");
            if (accounts && accounts.length > 0) this.accounts = accounts;

            // Currencies
            const currencies = await safeInvoke<Currency[]>("get_currencies", "Currencies");
            if (currencies && currencies.length > 0) this.currencies = currencies;

            // Markets
            const markets = await safeInvoke<Market[]>("get_markets", "Markets");
            if (markets && markets.length > 0) this.markets = markets;

            // Asset Types
            const assetTypes = await safeInvoke<AssetType[]>("get_asset_types", "Asset Types");
            if (assetTypes && assetTypes.length > 0) this.assetTypes = assetTypes;

            // Assets
            const assets = await safeInvoke<Asset[]>("get_assets", "Assets");
            if (assets && assets.length > 0) this.assets = assets;

            // Emotional States
            const emotionalStates = await safeInvoke<EmotionalState[]>("get_emotional_states", "Emotional States");
            if (emotionalStates && emotionalStates.length > 0) this.emotionalStates = emotionalStates;

            // Strategies
            const strategies = await safeInvoke<Strategy[]>("get_strategies", "Strategies");
            if (strategies && strategies.length > 0) this.strategies = strategies;

            // Trades loading is now handled by TradesStore


            // Cash Transactions
            const transactions = await safeInvoke<CashTransaction[]>("get_cash_transactions", "Cash Transactions");
            if (transactions && transactions.length > 0) this.cashTransactions = transactions;

            // Journal Entries
            const journalEntries = await safeInvoke<JournalEntry[]>("get_journal_entries", "Journal Entries");
            if (journalEntries && journalEntries.length > 0) this.journalEntries = journalEntries;

            // Fees
            const fees = await safeInvoke<FeeProfile[]>("get_fees", "Fees");
            if (fees && fees.length > 0) this.fees = fees;

            // Risk Profiles
            const riskProfiles = await safeInvoke<RiskProfile[]>("get_risk_profiles", "Risk Profiles");
            if (riskProfiles && riskProfiles.length > 0) this.riskProfiles = riskProfiles;

            // Modalities
            const modalities = await safeInvoke<Modality[]>("get_modalities", "Modalities");
            if (modalities && modalities.length > 0) this.modalities = modalities;

            // Tags
            const tags = await safeInvoke<Tag[]>("get_tags", "Tags");
            if (tags && tags.length > 0) this.tags = tags;

            // Indicators
            const indicators = await safeInvoke<Indicator[]>("get_indicators", "Indicators");
            if (indicators && indicators.length > 0) this.indicators = indicators;

            // Timeframes
            const timeframes = await safeInvoke<Timeframe[]>("get_timeframes", "Timeframes");
            if (timeframes && timeframes.length > 0) this.timeframes = timeframes;

            // Chart Types
            const chartTypes = await safeInvoke<ChartType[]>("get_chart_types", "Chart Types");
            if (chartTypes && chartTypes.length > 0) this.chartTypes = chartTypes;

            // Tax Rules
            const taxRules = await safeInvoke<TaxRule[]>("get_tax_rules", "Tax Rules");
            if (taxRules && taxRules.length > 0) {
                const validRules = taxRules.filter(r => r.name && r.name.trim() !== "");
                const ghostRules = taxRules.filter(r => !r.name || r.name.trim() === "");
                for (const ghost of ghostRules) {
                    await invoke("delete_tax_rule", { id: ghost.id }).catch(e => console.error(e));
                }
                this.taxRules = validRules;
            }

            // Tax Mappings
            const taxMappings = await safeInvoke<TaxMapping[]>("get_tax_mappings", "Tax Mappings");
            if (taxMappings && taxMappings.length > 0) this.taxMappings = taxMappings;

            // Tax Profiles
            const taxProfiles = await safeInvoke<TaxProfile[]>("get_tax_profiles", "Tax Profiles");
            if (taxProfiles && taxProfiles.length > 0) {
                const validProfiles = taxProfiles.filter(p => p.name && p.name.trim() !== "");
                const ghostProfiles = taxProfiles.filter(p => !p.name || p.name.trim() === "");
                for (const ghost of ghostProfiles) {
                    await invoke("delete_tax_profile", { id: ghost.id }).catch(e => console.error(e));
                }
                this.taxProfiles = validProfiles;
            }

            // Tax Profile Entries
            const taxProfileEntries = await safeInvoke<TaxProfileEntry[]>("get_tax_profile_entries", "Tax Profile Entries"); // Fetch all
            if (taxProfileEntries && taxProfileEntries.length > 0) {
                const validEntries = taxProfileEntries.filter(e => e.tax_profile_id && e.tax_rule_id && e.modality_id);
                const ghostEntries = taxProfileEntries.filter(e => !e.tax_profile_id || !e.tax_rule_id || !e.modality_id);
                for (const ghost of ghostEntries) {
                    await invoke("delete_tax_profile_entry", { id: ghost.id }).catch(e => console.error(e));
                }
                this.taxProfileEntries = validEntries;
            }

            // Other items (localStorage fallback for shared/UI state only)
            const savedBindings = localStorage.getItem("service_api_bindings");
            if (savedBindings) {
                const parsed = JSON.parse(savedBindings);
                this.psychologyApiId = parsed.psychology || "mock";
                this.marketDataApiId = parsed.market_data || "mock";
            }

            const savedRtd = localStorage.getItem("rtd_enabled");
            if (savedRtd) this.rtdEnabled = savedRtd === "true";

            const savedRtdPath = localStorage.getItem("rtd_excel_path");
            if (savedRtdPath) this.rtdExcelPath = savedRtdPath;

            // Initial license verification
            if (this.userProfile.license_key && this.hardwareId) {
                this.refreshLicenseStatus();
            }

            // --- AUTO-START RTD MONITOR ---
            // If it was enabled in a previous session, we must re-start the bridge
            if (this.rtdEnabled) {
                console.log("[SettingsStore] RTD was enabled, restarting monitor...");
                invoke("start_rtd_monitor_cmd", {
                    excelPath: this.rtdExcelPath || null,
                }).catch(e => console.error("[SettingsStore] Failed to auto-start RTD monitor:", e));
            }

            console.log("[SettingsStore] loadData completed.");
        } catch (e) {
            console.error("Failed to load data from SurrealDB (Critical)", e);
        } finally {
            this.isLoadingData = false;
        }
    }

    async saveUserProfile() {
        try {
            await invoke("save_user_profile", { profile: $state.snapshot(this.userProfile) });
        } catch (e) {
            console.error("[SettingsStore] Error saving user profile:", e);
        }
    }

    async deactivateLicense() {
        this.userProfile.license_key = null;
        this.licenseDetails = null;
        await this.saveUserProfile();
        console.log("[SettingsStore] License deactivated.");
    }

    private async saveApiConfigs() {
        for (const config of this.apiConfigs) {
            try {
                await invoke("save_api_config", { config: $state.snapshot(config) });
            } catch (e) {
                console.error("[SettingsStore] Error saving api config:", e);
            }
        }
    }

    private async saveAccounts() {
        for (const account of this.accounts) {
            try {
                await invoke("save_account", { account: $state.snapshot(account) });
            } catch (e) {
                console.error("[SettingsStore] Error saving account:", e);
            }
        }
    }

    private async saveCurrencies() {
        for (const currency of this.currencies) {
            try {
                await invoke("save_currency", { currency: $state.snapshot(currency) });
            } catch (e) {
                console.error("[SettingsStore] Error saving currency:", e);
            }
        }
    }

    private async saveMarkets() {
        for (const market of this.markets) {
            try {
                await invoke("save_market", { market: $state.snapshot(market) });
            } catch (e) {
                console.error("[SettingsStore] Error saving market:", e);
            }
        }
    }

    private async saveAssetTypes() {
        for (const assetType of this.assetTypes) {
            try {
                await invoke("save_asset_type", { assetType: $state.snapshot(assetType) });
            } catch (e) {
                console.error("[SettingsStore] Error saving asset type:", e);
            }
        }
    }

    private async saveAssets() {
        for (const asset of this.assets) {
            try {
                await invoke("save_asset", { asset: $state.snapshot(asset) });
            } catch (e) {
                console.error("[SettingsStore] Error saving asset:", e);
            }
        }
    }

    private async saveEmotionalStates() {
        for (const state of this.emotionalStates) {
            try {
                await invoke("save_emotional_state", { state: $state.snapshot(state) });
            } catch (e) {
                console.error("[SettingsStore] Error saving emotional state:", e);
            }
        }
    }

    private async saveStrategies() {
        for (const strategy of this.strategies) {
            try {
                await invoke("save_strategy", { strategy: $state.snapshot(strategy) });
            } catch (e) {
                console.error("[SettingsStore] Error saving strategy:", e);
            }
        }
    }

    private async saveModalities() {
        for (const modality of this.modalities) {
            try {
                await invoke("save_modality", { modality: $state.snapshot(modality) });
            } catch (e) {
                console.error("[SettingsStore] Error saving modality:", e);
            }
        }
    }

    private async saveTags() {
        for (const tag of this.tags) {
            try {
                await invoke("save_tag", { tag: $state.snapshot(tag) });
            } catch (e) {
                console.error("[SettingsStore] Error saving tag:", e);
            }
        }
    }

    private async saveIndicators() {
        for (const indicator of this.indicators) {
            try {
                await invoke("save_indicator", { indicator: $state.snapshot(indicator) });
            } catch (e) {
                console.error("[SettingsStore] Error saving indicator:", e);
            }
        }
    }

    private async saveTimeframes() {
        for (const timeframe of this.timeframes) {
            try {
                await invoke("save_timeframe", { timeframe: $state.snapshot(timeframe) });
            } catch (e) {
                console.error("[SettingsStore] Error saving timeframe:", e);
            }
        }
    }

    private async saveChartTypes() {
        for (const chartType of this.chartTypes) {
            try {
                await invoke("save_chart_type", { chartType: $state.snapshot(chartType) });
            } catch (e) {
                console.error("[SettingsStore] Error saving chart type:", e);
            }
        }
    }

    private async saveJournal() {
        for (const entry of this.journalEntries) {
            await this.saveSingleJournalEntry(entry);
        }
    }

    async saveSingleJournalEntry(entry: JournalEntry) {
        try {
            await invoke("save_journal_entry", { entry: $state.snapshot(entry) });
        } catch (e) {
            console.error("[SettingsStore] Error saving journal entry:", e);
        }
    }

    private async saveTransactions() {
        for (const transaction of this.cashTransactions) {
            await this.saveSingleTransaction(transaction);
        }
    }

    private async saveSingleTransaction(transaction: CashTransaction) {
        try {
            await invoke("save_cash_transaction", { transaction: $state.snapshot(transaction) });
        } catch (e) {
            console.error("[SettingsStore] Error saving transaction:", e);
        }
    }

    private async saveFees() {
        for (const fee of this.fees) {
            try {
                await invoke("save_fee", { fee: $state.snapshot(fee) });
            } catch (e) {
                console.error("[SettingsStore] Error saving fee:", e);
            }
        }
    }

    private async saveRiskProfiles() {
        for (const profile of this.riskProfiles) {
            try {
                await invoke("save_risk_profile", { profile: $state.snapshot(profile) });
            } catch (e) {
                console.error("[SettingsStore] Error saving risk profile:", e);
            }
        }
    }

    // --- Public Logic Methods ---

    getJournalEntryByDate(date: string) {
        return this.journalEntries.find(e => e.date === date);
    }

    // Markets
    addMarket(item: Omit<Market, "id">) {
        const id = crypto.randomUUID();
        this.markets.push({ ...item, id });
        this.saveMarkets();
    }
    updateMarket(id: string, item: Partial<Market>) {
        this.markets = this.markets.map(m => m.id === id ? { ...m, ...item } : m);
        this.saveMarkets();
    }
    async deleteMarket(id: string): Promise<{ success: boolean; error?: string }> {
        if (this.assetTypes.some(at => at.market_id === id)) {
            return { success: false, error: "Este Mercado está associado a Tipos de Ativos existentes." };
        }
        try {
            await invoke("delete_market", { id });
            this.markets = this.markets.filter(m => m.id !== id);
            return { success: true };
        } catch (e) {
            return { success: false, error: String(e) };
        }
    }
    getMarketCode(id: string): string {
        const item = this.markets.find(m => m.id === id);
        return item?.code || "";
    }

    // Asset Types
    addAssetType(item: Omit<AssetType, "id">) {
        const id = crypto.randomUUID();
        this.assetTypes.push({ ...item, id });
        this.saveAssetTypes();
    }
    updateAssetType(id: string, item: Partial<AssetType>) {
        this.assetTypes = this.assetTypes.map(at => at.id === id ? { ...at, ...item } : at);
        this.saveAssetTypes();
    }
    async deleteAssetType(id: string): Promise<{ success: boolean; error?: string }> {
        if (this.assets.some(a => a.asset_type_id === id)) {
            return { success: false, error: "Este Tipo de Ativo está associado a Ativos existentes." };
        }
        try {
            await invoke("delete_asset_type", { id });
            this.assetTypes = this.assetTypes.filter(at => at.id !== id);
            return { success: true };
        } catch (e) {
            return { success: false, error: String(e) };
        }
    }
    getAssetTypeCode(id: string): string {
        const item = this.assetTypes.find(at => at.id === id);
        return item?.code || "";
    }

    // Assets
    addAsset(item: Omit<Asset, "id">) {
        this.assets.push({ ...item, id: crypto.randomUUID() });
        this.saveAssets();
    }
    updateAsset(id: string, item: Partial<Asset>) {
        this.assets = this.assets.map(a => a.id === id ? { ...a, ...item } : a);
        this.saveAssets();
    }
    async deleteAsset(id: string): Promise<{ success: boolean; error?: string }> {
        const asset = this.assets.find(a => a.id === id);
        if (asset && this.strategies.some(s => s.specific_assets.includes(asset.symbol))) {
            return { success: false, error: "Este Ativo é referenciado especificamente em Estratégias." };
        }
        await invoke("delete_asset", { id });
        this.assets = this.assets.filter(a => a.id !== id);
        return { success: true };
    }

    ensureAssetExists(symbol: string, forceTypeId?: string) {
        if (!symbol) return;
        const sym = symbol.toUpperCase();
        const existing = this.assets.find(a => a.symbol === sym);
        if (existing) {
            if (forceTypeId && existing.asset_type_id !== forceTypeId) {
                this.updateAsset(existing.id, { asset_type_id: forceTypeId });
            }
            return;
        }

        let typeId = forceTypeId || "";
        let name = sym;

        if (sym.startsWith("WIN") || sym.startsWith("WDO") || sym.startsWith("IND") || sym.startsWith("DOL") || sym.startsWith("BIT")) {
            const type = this.assetTypes.find(at => at.name.toLowerCase().includes("futuro") || at.code.toLowerCase().includes("index"));
            typeId = type?.id || this.assetTypes[0]?.id || "";
            name = sym.startsWith("WIN") ? "Mini Índice" :
                sym.startsWith("WDO") ? "Mini Dólar" :
                    sym.startsWith("IND") ? "Índice Bovespa" :
                        sym.startsWith("DOL") ? "Dólar Cheio" :
                            sym.startsWith("BIT") ? "Mini Bitcoin" : sym;
        } else if (sym.length === 6 && !sym.match(/\d/)) {
            const type = this.assetTypes.find(at => at.name.toLowerCase().includes("forex") || at.code.toLowerCase().includes("fx"));
            typeId = type?.id || this.assetTypes[0]?.id || "";
        } else if (sym.length >= 5 && (sym.endsWith("11") || sym.endsWith("3") || sym.endsWith("4"))) {
            const type = this.assetTypes.find(at => at.name.toLowerCase().includes("ação") || at.code.toLowerCase().includes("stk"));
            typeId = type?.id || this.assetTypes[0]?.id || "";
        } else {
            typeId = this.assetTypes[0]?.id || "";
        }

        let pv = 1.0;
        if (sym.startsWith("WDO") || sym.startsWith("DOL")) pv = 10.0;
        else if (sym.startsWith("WIN") || sym.startsWith("IND")) pv = 0.20;
        else if (sym.startsWith("BIT")) pv = 0.1;

        this.addAsset({
            symbol: sym,
            name: `${name} (Auto)`,
            asset_type_id: typeId,
            point_value: pv,
            default_fee_id: ""
        });
    }

    // Currencies
    async syncExchangeRates() {
        const existingCodes = this.currencies.map(c => c.code.toUpperCase()).filter(code => code !== "BRL");
        if (existingCodes.length === 0) return;
        const pairs = existingCodes.map(code => `${code}-BRL`).join(",");
        const baseUrl = this.userProfile.currency_api_url || "https://economia.awesomeapi.com.br/last/";
        const url = `${baseUrl}${pairs}`;

        try {
            const response = await fetch(url);
            if (!response.ok) throw new Error(`API Error: ${response.statusText}`);
            const data = await response.json();
            let updatedCount = 0;
            this.currencies = this.currencies.map(currency => {
                const code = currency.code.toUpperCase();
                const pairKey = `${code}BRL`;
                if (data[pairKey]) {
                    updatedCount++;
                    return { ...currency, exchange_rate: parseFloat(data[pairKey].bid) };
                }
                return currency;
            });
            if (updatedCount > 0) {
                await this.saveCurrencies();
                return { success: true, count: updatedCount };
            }
        } catch (error) {
            console.error("[SettingsStore] Failed to sync exchange rates:", error);
            return { success: false, error: String(error) };
        }
    }

    addCurrency(item: Omit<Currency, "id">) {
        this.currencies.push({ ...item, id: crypto.randomUUID() });
        this.saveCurrencies();
    }
    updateCurrency(id: string, item: Partial<Currency>) {
        this.currencies = this.currencies.map(c => c.id === id ? { ...c, ...item } : c);
        this.saveCurrencies();
    }
    async deleteCurrency(id: string): Promise<{ success: boolean; error?: string }> {
        if (this.accounts.some(a => a.currency === id || a.currency === this.currencies.find(c => c.id === id)?.code)) {
            return { success: false, error: "Esta Moeda está associada a contas existentes." };
        }
        await invoke("delete_currency", { id });
        this.currencies = this.currencies.filter(c => c.id !== id);
        return { success: true };
    }

    getMarketAssets(marketId: string | null): Asset[] {
        if (!marketId) return this.assets;
        const assetTypesInMarket = this.assetTypes
            .filter(at => at.market_id === marketId)
            .map(at => at.id);
        return this.assets.filter(a => assetTypesInMarket.includes(a.asset_type_id));
    }

    getCurrencySymbol(code: string): string {
        const currency = this.currencies.find(c => c.code.toUpperCase() === (code || "BRL").toUpperCase());
        return currency?.symbol || "R$";
    }

    // Accounts
    addAccount(item: Omit<Account, "id">) {
        this.accounts.push({ ...item, id: crypto.randomUUID() });
        this.saveAccounts();
    }
    updateAccount(id: string, item: Partial<Account>) {
        this.accounts = this.accounts.map(a => a.id === id ? { ...a, ...item } : a);
        this.saveAccounts();
    }
    async deleteAccount(id: string): Promise<{ success: boolean; error?: string }> {
        try {
            await invoke("delete_account", { id });
            this.accounts = this.accounts.filter(a => a.id !== id);
            this.cashTransactions = this.cashTransactions.filter(t => t.account_id !== id);
            return { success: true };
        } catch (e) {
            return { success: false, error: String(e) };
        }
    }

    async deduplicateAccounts() {
        const seenNicks = new Set<string>();
        const toKeep: Account[] = [];
        for (const acc of this.accounts) {
            if (!seenNicks.has(acc.nickname)) {
                seenNicks.add(acc.nickname);
                toKeep.push(acc);
            } else {
                await invoke("delete_account", { id: acc.id }).catch(e => console.error(e));
            }
        }
        this.accounts = toKeep;
    }

    // Fees
    addFeeProfile(item: Omit<FeeProfile, "id">) {
        this.fees.push({ ...item, id: crypto.randomUUID() });
        this.saveFees();
    }
    updateFeeProfile(id: string, item: Partial<FeeProfile>) {
        this.fees = this.fees.map(f => f.id === id ? { ...f, ...item } : f);
        this.saveFees();
    }
    async deleteFeeProfile(id: string): Promise<{ success: boolean; error?: string }> {
        if (this.assets.some(a => a.default_fee_id === id)) return { success: false, error: "Este Perfil de Taxas é usado por Ativos." };
        await invoke("delete_fee", { id });
        this.fees = this.fees.filter(f => f.id !== id);
        return { success: true };
    }

    // Strategies
    addStrategy(item: Omit<Strategy, "id">) {
        this.strategies.push({ ...item, id: crypto.randomUUID() });
        this.saveStrategies();
    }
    updateStrategy(id: string, item: Partial<Strategy>) {
        this.strategies = this.strategies.map(s => s.id === id ? { ...s, ...item } : s);
        this.saveStrategies();
    }
    async deleteStrategy(id: string): Promise<{ success: boolean; error?: string }> {
        await invoke("delete_strategy", { id });
        this.strategies = this.strategies.filter(s => s.id !== id);
        return { success: true };
    }

    // Risk Profiles
    addRiskProfile(item: Omit<RiskProfile, "id">) {
        this.riskProfiles.push({ ...item, id: crypto.randomUUID() });
        this.saveRiskProfiles();
    }
    updateRiskProfile(id: string, item: Partial<RiskProfile>) {
        this.riskProfiles = this.riskProfiles.map(r => r.id === id ? { ...r, ...item } : r);
        this.saveRiskProfiles();
    }
    async deleteRiskProfile(id: string): Promise<{ success: boolean; error?: string }> {
        await invoke("delete_risk_profile", { id });
        this.riskProfiles = this.riskProfiles.filter(r => r.id !== id);
        return { success: true };
    }
    updateRiskProfilePhase(id: string, newPhaseIndex: number) {
        this.riskProfiles = this.riskProfiles.map(r => r.id === id ? { ...r, current_phase_index: newPhaseIndex } : r);
        this.saveRiskProfiles();
    }

    // Modalities
    addModality(item: Omit<Modality, "id">) {
        this.modalities.push({ ...item, id: crypto.randomUUID() });
        this.saveModalities();
    }
    updateModality(id: string, item: Partial<Modality>) {
        this.modalities = this.modalities.map(m => m.id === id ? { ...m, ...item } : m);
        this.saveModalities();
    }
    async deleteModality(id: string): Promise<{ success: boolean; error?: string }> {
        await invoke("delete_modality", { id });
        this.modalities = this.modalities.filter(m => m.id !== id);
        return { success: true };
    }

    async addTaxRule(item: Omit<TaxRule, "id">) {
        const id = crypto.randomUUID();
        const rule = { ...item, id };
        await invoke("save_tax_rule", { rule: $state.snapshot(rule) });
        this.taxRules.push(rule);
    }

    async updateTaxRule(id: string, item: Partial<TaxRule>) {
        const rule = this.taxRules.find(r => r.id === id);
        if (rule) {
            const updated = { ...rule, ...item };
            await invoke("save_tax_rule", { rule: $state.snapshot(updated) });
            this.taxRules = this.taxRules.map(r => r.id === id ? updated : r);
        }
    }
    async deleteTaxRule(id: string): Promise<{ success: boolean; error?: string }> {
        // Check usage in mappings or profiles (entries)
        if (this.taxMappings.some(m => m.tax_rule_id === id)) return { success: false, error: "Esta regra é usada em Mapeamentos antigos." };
        if (this.taxProfileEntries.some(e => e.tax_rule_id === id)) return { success: false, error: "Esta regra é usada em Perfis Fiscais." };

        try {
            await invoke("delete_tax_rule", { id });
            this.taxRules = this.taxRules.filter(r => r.id !== id);
            return { success: true };
        } catch (e) {
            return { success: false, error: String(e) };
        }
    }

    // Fiscal Module (Tax Profiles - New Option B)
    async addTaxProfile(item: Omit<TaxProfile, "id">) {
        const id = crypto.randomUUID();
        const profile = { ...item, id };
        await invoke("save_tax_profile", { profile: $state.snapshot(profile) });
        this.taxProfiles.push(profile);
        return id;
    }

    async updateTaxProfile(id: string, item: Partial<TaxProfile>) {
        const profile = this.taxProfiles.find(p => p.id === id);
        if (profile) {
            const updated = { ...profile, ...item };
            await invoke("save_tax_profile", { profile: $state.snapshot(updated) });
            this.taxProfiles = this.taxProfiles.map(p => p.id === id ? updated : p);
        }
    }

    async deleteTaxProfile(id: string): Promise<{ success: boolean; error?: string }> {
        if (this.assetTypes.some(a => a.tax_profile_id === id)) {
            return { success: false, error: "Este Perfil é usado por Tipos de Ativos." };
        }
        try {
            await invoke("delete_tax_profile", { id });
            this.taxProfiles = this.taxProfiles.filter(p => p.id !== id);
            // Also remove local entries for this profile
            this.taxProfileEntries = this.taxProfileEntries.filter(e => e.tax_profile_id !== id);
            return { success: true };
        } catch (e) {
            return { success: false, error: String(e) };
        }
    }

    // Fiscal Module (Tax Profile Entries)
    async addTaxProfileEntry(item: Omit<TaxProfileEntry, "id">) {
        const id = crypto.randomUUID();
        const entry = { ...item, id };
        await invoke("save_tax_profile_entry", { entry: $state.snapshot(entry) });
        this.taxProfileEntries.push(entry);
    }

    async deleteTaxProfileEntry(id: string) {
        try {
            await invoke("delete_tax_profile_entry", { id });
            this.taxProfileEntries = this.taxProfileEntries.filter(e => e.id !== id);
        } catch (e) {
            console.error("Failed to delete profile entry", e);
        }
    }

    getEntriesForProfile(profileId: string) {
        return this.taxProfileEntries.filter(e => e.tax_profile_id === profileId);
    }


    // Fiscal Module (Mappings)
    async addTaxMapping(item: Omit<TaxMapping, "id">) {
        const id = crypto.randomUUID();
        const mapping = { ...item, id };
        await invoke("save_tax_mapping", { mapping: $state.snapshot(mapping) });
        this.taxMappings.push(mapping);
    }

    async updateTaxMapping(id: string, item: Partial<TaxMapping>) {
        const mapping = this.taxMappings.find(m => m.id === id);
        if (mapping) {
            const updated = { ...mapping, ...item };
            await invoke("save_tax_mapping", { mapping: $state.snapshot(updated) });
            this.taxMappings = this.taxMappings.map(m => m.id === id ? updated : m);
        }
    }

    async deleteTaxMapping(id: string) {
        try {
            await invoke("delete_tax_mapping", { id });
            this.taxMappings = this.taxMappings.filter(m => m.id !== id);
        } catch (e) {
            console.error("Error deleting tax mapping", e);
        }
    }

    // Cash Transactions
    async addCashTransaction(item: Omit<CashTransaction, "id">) {
        try {
            const id = crypto.randomUUID();
            const transaction = { ...item, id };
            await invoke("save_cash_transaction", { transaction: $state.snapshot(transaction) });
            this.cashTransactions.push(transaction);

            const account = this.accounts.find(a => a.id === item.account_id);
            if (account) {
                const newBalance = account.balance + item.amount;
                await this.updateAccount(account.id, { balance: newBalance });
            }
            return { success: true, id };
        } catch (e: any) {
            console.error("[SettingsStore] Error saving transaction:", e);
            return { success: false, error: e.toString() };
        }
    }

    async removeCashTransaction(id: string) {
        try {
            const tx = this.cashTransactions.find(t => t.id === id);
            if (tx) {
                await invoke("delete_cash_transaction", { id });
                this.cashTransactions = this.cashTransactions.filter(t => t.id !== id);
                const account = this.accounts.find(a => a.id === tx.account_id);
                if (account) {
                    const newBalance = account.balance - tx.amount;
                    await this.updateAccount(account.id, { balance: newBalance });
                }
            }
            return { success: true };
        } catch (e: any) {
            console.error("[SettingsStore] Error deleting transaction:", e);
            return { success: false, error: e.toString() };
        }
    }

    async transferFunds(options: {
        fromAccountId: string,
        toAccountId: string,
        amountParams: {
            sourceAmount: number,
            fee: number,
            destAmount: number
        },
        date: string,
        description: string
    }) {
        try {
            const { fromAccountId, toAccountId, amountParams, date, description } = options;

            // 1. Withdraw from source (Total = sourceAmount)
            const res1 = await this.addCashTransaction({
                date,
                amount: -amountParams.sourceAmount,
                type: "Withdraw",
                description: description || "Transferência (Saída)",
                account_id: fromAccountId
            });
            if (!res1.success) throw new Error(res1.error);

            // 2. Deposit to destination (Net = destAmount)
            const res2 = await this.addCashTransaction({
                date,
                amount: amountParams.destAmount,
                type: "Deposit",
                description: description || "Transferência (Entrada)",
                account_id: toAccountId
            });
            if (!res2.success) throw new Error(res2.error);

            return { success: true };
        } catch (e: any) {
            console.error("[SettingsStore] Error transferring funds:", e);
            return { success: false, error: e.toString() };
        }
    }

    // User Profile
    updateUserProfile(data: Partial<UserProfile>) {
        this.userProfile = { ...this.userProfile, ...data };
        this.saveUserProfile();
        if (typeof window !== "undefined" && data.language) {
            localStorage.setItem("locale", data.language);
        }
        if (data.license_key !== undefined || data.name || data.cpf || data.birth_date) {
            this.refreshLicenseStatus();
        }
    }

    // Psychology / Emotional States
    addEmotionalState(item: Omit<EmotionalState, "id">) {
        this.emotionalStates.push({ ...item, id: crypto.randomUUID() });
        this.saveEmotionalStates();
    }
    updateEmotionalState(id: string, item: Partial<EmotionalState>) {
        this.emotionalStates = this.emotionalStates.map(e => e.id === id ? { ...e, ...item } : e);
        this.saveEmotionalStates();
    }
    async deleteEmotionalState(id: string): Promise<{ success: boolean; error?: string }> {
        await invoke("delete_emotional_state", { id });
        this.emotionalStates = this.emotionalStates.filter(e => e.id !== id);
        return { success: true };
    }

    // Tags
    addTag(item: Omit<Tag, "id">) {
        this.tags.push({ ...item, id: crypto.randomUUID() });
        this.saveTags();
    }
    updateTag(id: string, item: Partial<Tag>) {
        this.tags = this.tags.map(t => t.id === id ? { ...t, ...item } : t);
        this.saveTags();
    }
    async deleteTag(id: string): Promise<{ success: boolean; error?: string }> {
        await invoke("delete_tag", { id });
        this.tags = this.tags.filter(t => t.id !== id);
        return { success: true };
    }

    // Indicators
    addIndicator(item: Omit<Indicator, "id">) {
        this.indicators.push({ ...item, id: crypto.randomUUID() });
        this.saveIndicators();
    }
    updateIndicator(id: string, item: Partial<Indicator>) {
        this.indicators = this.indicators.map(i => i.id === id ? { ...i, ...item } : i);
        this.saveIndicators();
    }
    async deleteIndicator(id: string): Promise<{ success: boolean; error?: string }> {
        await invoke("delete_indicator", { id });
        this.indicators = this.indicators.filter(i => i.id !== id);
        return { success: true };
    }

    // Timeframes
    addTimeframe(item: Omit<Timeframe, "id">) {
        this.timeframes.push({ ...item, id: crypto.randomUUID() });
        this.saveTimeframes();
    }
    updateTimeframe(id: string, item: Partial<Timeframe>) {
        this.timeframes = this.timeframes.map(t => t.id === id ? { ...t, ...item } : t);
        this.saveTimeframes();
    }
    async deleteTimeframe(id: string): Promise<{ success: boolean; error?: string }> {
        await invoke("delete_timeframe", { id });
        this.timeframes = this.timeframes.filter(t => t.id !== id);
        return { success: true };
    }

    // Chart Types
    addChartType(item: Omit<ChartType, "id">) {
        this.chartTypes.push({ ...item, id: crypto.randomUUID() });
        this.saveChartTypes();
    }
    updateChartType(id: string, item: Partial<ChartType>) {
        this.chartTypes = this.chartTypes.map(c => c.id === id ? { ...c, ...item } : c);
        this.saveChartTypes();
    }
    async deleteChartType(id: string): Promise<{ success: boolean; error?: string }> {
        await invoke("delete_chart_type", { id });
        this.chartTypes = this.chartTypes.filter(c => c.id !== id);
        return { success: true };
    }

    // API & RTD
    addApiConfig(item: Omit<ApiConfig, "id">) {
        this.apiConfigs.push({ ...item, id: crypto.randomUUID() });
        this.saveApiConfigs();
    }
    updateApiConfig(id: string, item: Partial<ApiConfig>) {
        this.apiConfigs = this.apiConfigs.map(c => c.id === id ? { ...c, ...item } : c);
        this.saveApiConfigs();
    }
    async deleteApiConfig(id: string): Promise<{ success: boolean; error?: string }> {
        this.apiConfigs = this.apiConfigs.filter(c => c.id !== id);
        this.saveApiConfigs();
        return { success: true };
    }

    saveServiceBindings(psychology: string, marketData: string) {
        this.psychologyApiId = psychology;
        this.marketDataApiId = marketData;
        if (typeof window !== "undefined") {
            localStorage.setItem("service_api_bindings", JSON.stringify({ psychology, market_data: marketData }));
        }
    }

    setRtdEnabled(enabled: boolean) {
        this.rtdEnabled = enabled;
        if (typeof window !== "undefined") localStorage.setItem("rtd_enabled", enabled.toString());
    }

    setRtdExcelPath(path: string) {
        this.rtdExcelPath = path;
        if (typeof window !== "undefined") localStorage.setItem("rtd_excel_path", path);
    }

    // Journaling
    async addJournalEntry(item: Omit<JournalEntry, "id">) {
        const id = crypto.randomUUID();
        const entry = { ...item, id };
        this.journalEntries.push(entry);
        try {
            await invoke("save_journal_entry", { entry: $state.snapshot(entry) });
        } catch (e) {
            console.error("[SettingsStore] Error saving journal entry:", e);
        }
    }

    async removeJournalEntry(id: string) {
        try {
            await invoke("delete_journal_entry", { id });
            this.journalEntries = this.journalEntries.filter(j => j.id !== id);
        } catch (e) {
            console.error("[SettingsStore] Error deleting journal entry:", e);
        }
    }

    updateJournalEntry(id: string, item: Partial<JournalEntry>) {
        this.journalEntries = this.journalEntries.map(j => j.id === id ? { ...j, ...item } : j);
        const entry = this.journalEntries.find(j => j.id === id);
        if (entry) {
            try {
                invoke("save_journal_entry", { entry: $state.snapshot(entry) });
            } catch (e) {
                console.error("[SettingsStore] Error updating journal entry:", e);
            }
        }
    }


    // Debug / Seed
    seedDatabase() {
        if (this.strategies.length === 0) {
            this.addStrategy({
                name: "Setup Padrão (Seed)",
                description: "Estratégia gerada automaticamente.",
                market_ids: ["m1"],
                timeframes: ["5m"],
                asset_types: ["Futuros"],
                indicators: ["VWAP"],
                specific_assets: ["WIN"],
                entry_criteria: "Cross",
                exit_criteria: "Stop",
                management_criteria: "None",
                has_partial: false,
                partial_description: "",
                images: []
            });
        }
    }

    clearDatabase() {
        this.markets = []; this.assetTypes = []; this.assets = []; this.accounts = [];
        this.fees = []; this.strategies = []; this.riskProfiles = []; this.modalities = [];
        this.emotionalStates = []; this.tags = []; this.indicators = []; this.timeframes = [];
        this.chartTypes = [];
    }

    login(email: string, pass: string) { return true; }
    logout() { console.log("User logged out"); }
}

export const settingsStore = new SettingsStore();
