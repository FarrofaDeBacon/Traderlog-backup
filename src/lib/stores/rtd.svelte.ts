import { listen } from '@tauri-apps/api/event';

export interface RTDQuote {
    symbol: string;
    last: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
    trades: number;
    sheet?: string; // Tab name from Excel for account affinity
}

// Global logger for RTD events
const log = (msg: string, ...args: any[]) => console.log(`[RTDStore] ${msg}`, ...args);

export interface RTDBook {
    id: string;
    bid: number;
    ask: number;
}

export interface RTDPlayer {
    broker: string;
    saldo: number;
}

// Callback type for trade detection
export type TradeExecutionCallback = (quote: RTDQuote) => void;

class RTDStore {
    quotes = $state<Record<string, RTDQuote>>({});
    books = $state<Record<string, RTDBook>>({});
    players = $state<RTDPlayer[]>([]);
    lastUpdate = $state<Date | null>(null);
    symbols = $state<string[]>([]);
    previousPositions = $state<Record<string, number>>({});

    // Track previous trade counts to detect executions
    private previousTrades: Record<string, number> = {};
    private onTradeExecutedCallbacks: TradeExecutionCallback[] = [];

    // Internal buffers for throttling
    private _quotesBuffer: Record<string, RTDQuote> = {};
    private _booksBuffer: Record<string, RTDBook> = {};
    private _playersBuffer: RTDPlayer[] = [];
    private _isThrottling = false;
    private _throttleMs = 1000; // Update UI every 1s (reduced from 500ms to avoid UI lag)
    private _detectionDebounceTimer: any = null;
    private _pendingDetections: RTDQuote[] = [];

    constructor() {
        if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
            this.initListener();
        }
    }

    onTradeExecuted(cb: TradeExecutionCallback) {
        this.onTradeExecutedCallbacks.push(cb);
    }

    private async initListener() {
        console.log("[RTDStore] Initializing Tauri event listener for 'rtd-update'...");
        await listen<string>('rtd-update', (event) => {
            this.parseCSV(event.payload);
        });
    }

    private triggerTradeCallback(sym: string, avgPrice: number, sheet?: string) {
        const quote = this._quotesBuffer[sym] || {
            symbol: sym,
            last: avgPrice,
            open: 0, high: 0, low: 0, close: 0, volume: 0, trades: 0,
            sheet: sheet
        };

        // Debounce detections: if multiple detections happen within 500ms, 
        // only trigger once for the latest per symbol or just batch them.
        // For now, let's do a simple batch/debounce to avoid UI spam.
        this._pendingDetections.push(quote);

        if (this._detectionDebounceTimer) clearTimeout(this._detectionDebounceTimer);

        this._detectionDebounceTimer = setTimeout(() => {
            // Take only the last detection for actually alerting or just one alert
            // if many symbols detected at exact same time.
            const uniqueDetections = new Map<string, RTDQuote>();
            this._pendingDetections.forEach(d => uniqueDetections.set(d.symbol, d));

            uniqueDetections.forEach(d => {
                this.onTradeExecutedCallbacks.forEach(cb => cb(d));
            });

            this._pendingDetections = [];
            this._detectionDebounceTimer = null;
        }, 500); // 500ms group window
    }

    private parseCSV(content: string) {
        if (!content) return;
        const lines = content.split('\n');

        // Reset players buffer for each event to avoid accumulation
        this._playersBuffer = [];

        // Skip header
        for (let i = 1; i < lines.length; i++) {
            const line = lines[i].trim();
            if (!line) continue;

            const parts = line.split(';');
            const type = parts[0];

            if (type === 'QUOTE') {
                const rawSym = parts[1];
                const trades = parseFloat(parts[8]) || 0;
                const sheet = parts[11];
                const last = parseFloat(parts[2]) || 0;

                const newQuote: RTDQuote = {
                    symbol: rawSym,
                    last: last,
                    open: parseFloat(parts[3]) || 0,
                    high: parseFloat(parts[4]) || 0,
                    low: parseFloat(parts[5]) || 0,
                    close: parseFloat(parts[6]) || 0,
                    volume: parseFloat(parts[7]) || 0,
                    trades: trades,
                    sheet: sheet
                };

                this.previousTrades[rawSym] = trades;
                this._quotesBuffer[rawSym] = newQuote;

                // --- SYMBOL NORMALIZATION ---
                if (rawSym.startsWith('WIN') && rawSym.length > 3) {
                    this._quotesBuffer['WIN'] = { ...newQuote, symbol: 'WIN' };
                    this._quotesBuffer['WINFUT'] = { ...newQuote, symbol: 'WINFUT' };
                } else if (rawSym.startsWith('WDO') && rawSym.length > 3) {
                    this._quotesBuffer['WDO'] = { ...newQuote, symbol: 'WDO' };
                    this._quotesBuffer['WDOFUT'] = { ...newQuote, symbol: 'WDOFUT' };
                }

            } else if (type === 'POS') {
                const rawSym = parts[1];
                const qty = Math.abs(parseFloat(parts[2]) || 0);
                const avgPrice = parseFloat(parts[3]) || 0;
                const sheet = parts[11];

                const symsToUpdate = [rawSym];
                if (rawSym.startsWith('WIN') && rawSym.length > 3) symsToUpdate.push('WIN', 'WINFUT');
                if (rawSym.startsWith('WDO') && rawSym.length > 3) symsToUpdate.push('WDO', 'WDOFUT');

                symsToUpdate.forEach(sym => {
                    const prevQty = this.previousPositions[sym];
                    if (prevQty !== undefined) {
                        // Trigger on any increase (Entry/Scale-in) 
                        // or on full exit (qty === 0)
                        // Decrease > 0 is a partial exit, also worth notifying for journaling
                        if (qty !== prevQty) {
                            log(`Position change detected for ${sym}: ${prevQty} -> ${qty}`);
                            this.triggerTradeCallback(sym, avgPrice, sheet);
                        }
                    } else if (qty > 0) {
                        // First time seeing this position
                        this.triggerTradeCallback(sym, avgPrice, sheet);
                    }
                    this.previousPositions[sym] = qty;
                });

            } else if (type === 'BOOK') {
                const id = parts[1];
                this._booksBuffer[id] = {
                    id: id,
                    bid: parseFloat(parts[9]) || 0,
                    ask: parseFloat(parts[10]) || 0,
                };
            } else if (type === 'PLAYER') {
                this._playersBuffer.push({
                    broker: parts[1],
                    saldo: parseFloat(parts[2]) || 0,
                });
            }
        }

        this.scheduleSync();
    }

    /**
     * Throttled sync to reactive state
     */
    private scheduleSync() {
        if (this._isThrottling) return;

        this._isThrottling = true;
        setTimeout(() => {
            this.syncBufferToState();
            this._isThrottling = false;
        }, this._throttleMs);
    }

    private syncBufferToState() {
        // SURGICAL UPDATE: Iterate buffer and update individual keys in state
        let quotesChanged = false;
        for (const [sym, quote] of Object.entries(this._quotesBuffer)) {
            if (!this.quotes[sym] || this.quotes[sym].last !== quote.last) {
                this.quotes[sym] = quote;
                quotesChanged = true;
            }
        }

        for (const [id, book] of Object.entries(this._booksBuffer)) {
            if (!this.books[id] || this.books[id].bid !== book.bid || this.books[id].ask !== book.ask) {
                this.books[id] = book;
            }
        }

        if (this._playersBuffer.length > 0) {
            this.players = [...this._playersBuffer].sort((a, b) => Math.abs(b.saldo) - Math.abs(a.saldo));
            // Buffer is already cleared at the start of parseCSV
        }

        const currentSymbols = Object.keys(this.quotes);
        if (currentSymbols.length !== this.symbols.length) {
            this.symbols = currentSymbols;
        }

        if (quotesChanged) {
            this.lastUpdate = new Date();
        }
    }

    // Derived values for easier UI access
    get winQuote() { return this.quotes['WIN'] || this.quotes['WINFUT']; }
    get wdoQuote() { return this.quotes['WDO'] || this.quotes['WDOFUT']; }
    get winBook() { return this.books['BOOK_WIN']; }
    get wdoBook() { return this.books['BOOK_WDO']; }
}

export const rtdStore = new RTDStore();
