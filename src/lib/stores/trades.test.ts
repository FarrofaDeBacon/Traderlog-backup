import { describe, it, expect, vi, beforeEach } from 'vitest';
import { tradesStore } from './trades.svelte';

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

// Mock Settings Store dependency
vi.mock('./settings.svelte', () => ({
    settingsStore: {
        loadCashTransactions: vi.fn(),
        accounts: []
    }
}));

// Mock Utility
vi.mock('$lib/utils', () => ({
    getLocalDatePart: (date: string) => date.split('T')[0]
}));

describe('TradesStore Unit Tests', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        tradesStore.trades = [];
    });

    it('should calculate daily results correctly for a specific account', () => {
        const mockAccounts = [
            { id: 'acc1', nickname: 'Real Account', currency: 'BRL', balance: 0 }
        ];

        // Mock trades
        tradesStore.trades = [
            {
                id: 't1',
                account_id: 'acc1',
                result: 150.0,
                date: '2024-03-02T10:00:00Z',
                exit_date: '2024-03-02T10:30:00Z',
                exit_price: 15.0
            },
            {
                id: 't2',
                account_id: 'acc1',
                result: -50.0,
                date: '2024-03-02T11:00:00Z',
                exit_date: '2024-03-02T11:30:00Z',
                exit_price: 14.5
            },
            {
                id: 't3',
                account_id: 'acc2', // Different account
                result: 500.0,
                date: '2024-03-02T10:00:00Z',
                exit_date: '2024-03-02T10:30:00Z',
                exit_price: 20.0
            }
        ] as any;

        const results = tradesStore.getDailyResultByAccount('2024-03-02', mockAccounts as any);
        expect(results[0].result).toBe(100.0);
        expect(results[0].trades_count).toBe(2);
    });

    it('should count closed trades for a specific date', () => {
        tradesStore.trades = [
            { id: 't1', exit_price: 10, date: '2024-03-02' },
            { id: 't2', exit_price: 20, date: '2024-03-02' },
            { id: 't3', exit_price: null, date: '2024-03-02' }, // Open trade
            { id: 't4', exit_price: 30, date: '2024-03-03' }  // Different day
        ] as any;

        expect(tradesStore.getTradesCountForDate('2024-03-02')).toBe(2);
    });

    it('should convert trade results using exchange rates', () => {
        const mockAccounts = [
            { id: 'acc_usd', nickname: 'USD Account', currency: 'USD' }
        ];
        const mockCurrencies = [
            { code: 'USD', exchange_rate: 5.4 }
        ];
        const trade = { account_id: 'acc_usd', result: 100.0 } as any;

        const converted = tradesStore.getConvertedTradeResult(trade, mockAccounts as any, mockCurrencies as any);
        expect(converted).toBe(540.0);
    });

    it('should calculate strategy stats correctly', () => {
        const mockAccounts = [
            { id: 'acc1', currency: 'BRL' }
        ] as any;
        const mockCurrencies = [] as any;
        const mockProfile = { main_currency: 'BRL' } as any;

        tradesStore.trades = [
            { strategy_id: 's1', account_id: 'acc1', result: 100 }, // Win
            { strategy_id: 's1', account_id: 'acc1', result: 200 }, // Win
            { strategy_id: 's1', account_id: 'acc1', result: -100 }, // Loss
            { strategy_id: 's1', account_id: 'acc1', result: 0 },    // Neutral
        ] as any;

        const stats = tradesStore.getStrategyStats('s1', mockAccounts, mockCurrencies, mockProfile);

        expect(stats.total_trades).toBe(4);
        expect(stats.win_rate).toBe(50); // 2/4
        expect(stats.total_profit).toBe(200); // 100+200-100
        expect(stats.profit_factor).toBe(3); // (100+200) / 100
        expect(stats.payoff).toBe(1.5); // AvgWin(150) / AvgLoss(100)
    });

    it('should handle division by zero in stats (no losses)', () => {
        tradesStore.trades = [
            { strategy_id: 's1', account_id: 'acc1', result: 100 }
        ] as any;
        const stats = tradesStore.getStrategyStats('s1', [{ id: 'acc1' }] as any, [], { main_currency: 'BRL' } as any);
        expect(stats.profit_factor).toBe(99.99); // Capped by law/logic
    });

    it('should handle robust currency conversion (pt-BR formats)', () => {
        const mockAccounts = [{ id: 'acc1', currency: 'BRL' }] as any;
        const mockCurrencies = [] as any;

        // Standard string with comma
        expect(tradesStore.getConvertedTradeResult(
            { account_id: 'acc1', result: "R$ 1.250,50" } as any,
            mockAccounts,
            mockCurrencies
        )).toBe(1250.5);

        // Negative with symbol
        expect(tradesStore.getConvertedTradeResult(
            { account_id: 'acc1', result: "- R$ 500,25" } as any,
            mockAccounts,
            mockCurrencies
        )).toBe(-500.25);

        // Plain number
        expect(tradesStore.getConvertedTradeResult(
            { account_id: 'acc1', result: 1000 } as any,
            mockAccounts,
            mockCurrencies
        )).toBe(1000);
    });
});
