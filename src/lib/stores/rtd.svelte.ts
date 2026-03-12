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

export interface RTDTradeEvent {
    quote: RTDQuote;
    type: 'new' | 'partial_entry' | 'partial_exit';
    isPartial: boolean;
}

// Callback type for trade detection
export type TradeExecutionCallback = (event: RTDTradeEvent) => void;

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
    private _pendingDetections: RTDTradeEvent[] = [];

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
            // console.log("[RTDStore] 'rtd-update' received:", event.payload.substring(0, 50) + "...");
            this.parseCSV(event.payload);
        });
    }

    private triggerTradeCallback(sym: string, avgPrice: number, type: 'new' | 'partial_entry' | 'partial_exit', isPartial: boolean, sheet?: string) {
        // PRICE GUARD: Ignore 0.00 prices which are usually noise or uninitialized data
        if (avgPrice <= 0) {
            log(`Ignoring detection for ${sym} due to zero price.`);
            return;
        }

        const quote = this._quotesBuffer[sym] || {
            symbol: sym,
            last: avgPrice,
            open: 0, high: 0, low: 0, close: 0, volume: 0, trades: 0,
            sheet: sheet
        };

        const event: RTDTradeEvent = {
            quote,
            type,
            isPartial
        };

        this._pendingDetections.push(event);

        if (this._detectionDebounceTimer) clearTimeout(this._detectionDebounceTimer);

        this._detectionDebounceTimer = setTimeout(() => {
            const uniqueDetections = new Map<string, RTDTradeEvent>();
            this._pendingDetections.forEach(d => uniqueDetections.set(d.quote.symbol, d));

            uniqueDetections.forEach(d => {
                this.onTradeExecutedCallbacks.forEach(cb => cb(d));
            });

            this._pendingDetections = [];
            this._detectionDebounceTimer = null;
        }, 500);
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

                const prevTradeCount = this.previousTrades[rawSym];
                if (prevTradeCount !== undefined && trades > prevTradeCount) {
                    log(`Trade count increase detected for ${rawSym}: ${prevTradeCount} -> ${trades}`);
                    const isPartial = (this.previousPositions[rawSym] || 0) > 0;
                    this.triggerTradeCallback(rawSym, last, isPartial ? 'partial_entry' : 'new', isPartial, sheet);
                } else if (prevTradeCount === undefined) {
                    log(`Initializing trade count for ${rawSym}: ${trades}`);
                }

                this.previousTrades[rawSym] = trades;
                this._quotesBuffer[rawSym] = newQuote;

                // --- SYMBOL NORMALIZATION ---
                if (rawSym.startsWith('WIN') && rawSym.length > 3) {
                    this._quotesBuffer['WIN'] = { ...newQuote, symbol: 'WIN' };
                    this._quotesBuffer['WINFUT'] = { ...newQuote, symbol: 'WINFUT' };
                    this.previousTrades['WIN'] = trades;
                    this.previousTrades['WINFUT'] = trades;
                } else if (rawSym.startsWith('WDO') && rawSym.length > 3) {
                    this._quotesBuffer['WDO'] = { ...newQuote, symbol: 'WDO' };
                    this._quotesBuffer['WDOFUT'] = { ...newQuote, symbol: 'WDOFUT' };
                    this.previousTrades['WDO'] = trades;
                    this.previousTrades['WDOFUT'] = trades;
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
                        if (qty !== prevQty) {
                            log(`Position change detected for ${sym}: ${prevQty} -> ${qty}`);
                            const type = qty > prevQty ? 'partial_entry' : 'partial_exit';
                            this.triggerTradeCallback(sym, avgPrice, type, true, sheet);
                        }
                    } else {
                        // INITIALIZATION GUARD: Don't trigger on first load to avoid ghost trades
                        log(`Initializing position for ${sym}: ${qty}`);
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
