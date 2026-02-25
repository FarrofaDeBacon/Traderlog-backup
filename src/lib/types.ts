
export type TradingSession = {
    start_time: string;  // "HH:MM" format
    end_time: string;    // "HH:MM" format
};

export type Market = {
    id: string;
    code: string;
    name: string;
    timezone: string;
    trading_days: number[];
    trading_sessions: TradingSession[];
};

export type AssetType = {
    id: string;
    name: string;
    code: string;
    market_id: string;
    default_fee_id?: string;
    tax_profile_id?: string; // New field
    unit_label: string;
    result_type: "points" | "currency";
};

// ... (other types remain)

export type TaxRule = {
    id: string;
    name: string; // e.g., "Swing Trade Ações", "Day Trade Geral"
    tax_rate: number; // Percentage (e.g., 20.0, 15.0)
    withholding_rate: number; // "Dedo-duro" (e.g., 1.0, 0.005)
    exemption_threshold: number; // 0 for none, 20000 for Stocks
    basis: string; // "NetProfit" | "GrossProfit"
    cumulative_losses: boolean; // Can offset past losses? (True for most)
    trade_type: string; // "DayTrade" | "SwingTrade"
    withholding_basis: string; // "Profit" | "SalesVolume"
    revenue_code: string; // e.g., "6015", "3317"
};

export type TaxProfile = {
    id: string;
    name: string;
};

export type TaxProfileEntry = {
    id: string;
    tax_profile_id: string;
    modality_id: string;
    tax_rule_id: string;
};

export type TaxMapping = {
    id: string;
    asset_type_id: string; // FK to AssetType
    modality_id: string; // FK to Modality ("Day Trade", "Swing Trade")
    tax_rule_id: string; // FK to TaxRule
};

export type Asset = {
    id: string;
    symbol: string;
    name: string;
    asset_type_id: string;
    point_value: number;
    default_fee_id?: string;
    tax_profile_id?: string; // New field
};

export type Currency = {
    id: string;
    code: string;
    symbol: string;
    name: string;
    exchange_rate: number;
};

export type Account = {
    id: string;
    nickname: string;
    account_type: "Real" | "Demo" | "Prop";
    broker: string;
    account_number: string;
    currency: string;
    balance: number;
    custom_logo: string | null;
};

export type JournalEntry = {
    id: string;
    date: string;
    content: string;
    emotional_state_id: string | null;
    intensity: number;
    followed_plan?: boolean;
    risk_accepted?: boolean;
    market_context?: string;
    daily_score?: number;
};

export type Trade = {
    id: string;
    date: string; // YYYY-MM-DD
    asset_symbol: string;
    asset_type_id: string;
    strategy_id: string;
    account_id: string;
    result: number; // In Account Currency
    quantity: number;
    direction: "Buy" | "Sell";
    entry_price: number;
    exit_price: number | null;
    exit_date: string | null;
    fee_total: number;
    notes: string;

    // Technical Context
    timeframe: string;
    volatility: string;

    // Entry Psychology
    entry_emotional_state_id: string | null;
    entry_emotional_state_name: string | null;

    // Closing Context
    exit_reason: string | null;
    exit_emotional_state_id: string | null;
    exit_emotional_state_name: string | null;

    // Deep Analysis
    entry_rationale: string;
    confirmation_signals: string;
    market_context: string;
    relevant_news: string;
    psychology_analysis_during: string;

    followed_plan: boolean;
    what_worked: string;
    mistakes_improvements: string;
    lessons_learned: string;

    images: string[];
    partial_exits: any[];
    modality_id: string;
    stop_loss: number | null;
    take_profit: number | null;
    intensity: number;
};

export type Strategy = {
    id: string;
    name: string;
    description: string;
    market_ids: string[];
    timeframes: string[];
    asset_types: string[];
    indicators: string[];
    specific_assets: string[];
    entry_criteria: string;
    exit_criteria: string;
    management_criteria: string;
    has_partial: boolean;
    partial_description: string;
    images: { path: string, description: string }[];
};

export type UserProfile = {
    id: string;
    name: string;
    email: string;
    phone: string;
    cpf: string;
    theme: "light" | "dark" | "system";
    language: string;
    timezone: string;
    main_currency: string;
    avatar: string | null;
    convert_all_to_main: boolean;
    onboarding_completed: boolean;
    currency_api_url: string;
    birth_date: string | null;
    trial_start_date: string | null;
    license_key: string | null;
    utc_offset: number;
};

export type EmotionalState = {
    id: string;
    name: string;
    impact: 'Positive' | 'Negative' | 'Neutral';
    description: string;
    potential_impact: string;
    weight: number;
};

export type Tag = {
    id: string;
    name: string;
    color: string;
};

export type Indicator = {
    id: string;
    name: string;
    category?: 'Trend' | 'Oscillator' | 'Volume' | 'Other';
    plot_type?: 'Overlay' | 'SubWindow';
    default_color?: string;
    usage_description?: string;
    parameters?: { key: string, value: string }[];
};

export type Timeframe = {
    id: string;
    name: string;
    value: string;
};

export type ChartType = {
    id: string;
    name: string;
    base_type?: 'TimeBased' | 'Renko' | 'Range';
    parameter?: string;
};

export type ApiConfig = {
    id: string;
    provider: string; // openai, custom, etc.
    api_key: string;
    enabled: boolean;
    daily_limit: number;
    requests_today: number;
    extra_config: string;
};

export type CashTransaction = {
    id: string;
    date: string; // YYYY-MM-DD
    amount: number;
    type: "Deposit" | "Withdraw" | "Adjustment" | "DailyResult";
    description: string;
    account_id: string;
    trade_ids?: string[];
    category?: string;
    system_linked?: boolean;
};

export type Modality = {
    id: string;
    name: string;
    description?: string;
};

export type FeeCustomItem = {
    id: string;
    name: string;
    value: number;
    type: "fixed" | "percentage";
    category: "cost" | "tax";
};

export type FeeProfile = {
    id: string;
    name: string;
    broker: string;
    fixed_fee: number;
    percentage_fee: number;
    exchange_fee: number;
    iss: number;
    currency_spread: number;
    withholding_tax: number;
    income_tax_rate: number;
    custom_items: FeeCustomItem[];
    tax_rule_id?: string;
    notes: string;
};

export type TaxAppraisal = {
    id: string;
    name: string;
    broker: string;
    tax_rate: number;
    tax_due: number;
    withheld_tax: number;
    withholding_credit_used: number;
    withholding_credit_remaining: number;
    tax_payable: number;
    tax_accumulated: number;
    total_payable: number;
    is_exempt: boolean;
    calculation_date: string;
    status: "Pending" | "Paid" | "Ok";
    trade_ids: string[];
};

export type TaxLoss = {
    id?: string;
    trade_type: string;
    amount: number;
    origin_date: string;
    balance: number;
};

export type GrowthPhase = {
    id: string;
    name: string;
    description: string;
    max_lots: number;
    max_daily_loss: number;
    progression_rules: {
        condition: "profit_target" | "days_positive" | "consistency_days";
        value: number;
    }[];
    regression_rules: {
        condition: "drawdown_limit" | "max_daily_loss_streak";
        value: number;
    }[];
};

export type RiskProfile = {
    id: string;
    name: string;
    max_daily_loss: number;
    daily_target: number;
    max_risk_per_trade_percent: number;
    max_trades_per_day: number;
    min_risk_reward: number;
    lock_on_loss: boolean;
    account_type_applicability: 'All' | 'Prop' | 'Real' | 'Demo';
    growth_plan_enabled: boolean;
    current_phase_index: number;
    growth_phases: GrowthPhase[];
    psychological_coupling_enabled: boolean;
    outlier_regression_enabled: boolean;
    sniper_mode_enabled: boolean;
    sniper_mode_selectivity: number;
    psychological_lookback_count: number;
    outlier_lookback_count: number;
    psychological_threshold: number;
    lot_reduction_multiplier: number;
    psychological_search_strategy: 'Strict' | 'Sequence';
};



export type TaxPayment = {
    id: string;
    period: string; // "YYYY-MM"
    tax_due: number; // Calculated original tax
    fine: number; // Multa
    interest: number; // Juros
    total_paid: number;
    payment_date: string | null;
    status: "Pending" | "Paid";
    notes: string;
};
