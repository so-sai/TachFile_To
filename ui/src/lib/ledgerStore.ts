import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface LedgerEntry {
    id: string;
    source_path: string;
    checksum: string;
    pages_processed: number;
    extraction_engine: string;
    extraction_version: string;
    processing_time_ms: number;
    status: string;
    error_message: string | null;
    metadata_json: string;
    created_at: string;
}

interface LedgerState {
    entries: LedgerEntry[];
    isLoading: boolean;
    fetchEntries: (limit?: number) => Promise<void>;
}

export const useLedgerStore = create<LedgerState>((set) => ({
    entries: [],
    isLoading: false,
    fetchEntries: async (limit = 50) => {
        set({ isLoading: true });
        try {
            const entries = await invoke<LedgerEntry[]>('get_ledger_entries', { limit });
            set({ entries, isLoading: false });
        } catch (error) {
            console.error("Failed to fetch ledger entries:", error);
            set({ isLoading: false });
        }
    }
}));
