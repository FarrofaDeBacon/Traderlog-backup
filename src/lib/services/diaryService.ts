export interface ChecklistEntry {
    date: string; // YYYY-MM-DD
    items: any[]; // The checklist state
    news: any[]; // Detected news
    riskLimit: string; // The input value
    timestamp: number;
}

const STORAGE_PREFIX = "traderlog:diary:";

export const diaryService = {
    // Save or Update today's entry
    saveEntry(date: string, data: Omit<ChecklistEntry, "date" | "timestamp">) {
        const entry: ChecklistEntry = {
            date,
            timestamp: Date.now(),
            ...data
        };
        localStorage.setItem(`${STORAGE_PREFIX}${date}`, JSON.stringify(entry));

        // Also update the index of days
        const index = this.getIndex();
        if (!index.includes(date)) {
            index.push(date);
            index.sort().reverse(); // Newest first
            localStorage.setItem(`${STORAGE_PREFIX}index`, JSON.stringify(index));
        }

        console.log("Diary entry saved:", entry);
    },

    // Get specific entry
    getEntry(date: string): ChecklistEntry | null {
        const item = localStorage.getItem(`${STORAGE_PREFIX}${date}`);
        if (!item) return null;
        try {
            return JSON.parse(item);
        } catch {
            return null;
        }
    },

    // List all recorded dates
    getIndex(): string[] {
        const raw = localStorage.getItem(`${STORAGE_PREFIX}index`);
        if (!raw) return [];
        try {
            return JSON.parse(raw);
        } catch {
            return [];
        }
    },

    // Get all entries (for future table/filter)
    getAllEntries(): ChecklistEntry[] {
        const index = this.getIndex();
        return index.map(date => this.getEntry(date)).filter(Boolean) as ChecklistEntry[];
    }
};
