import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export type FileStatusLabel = 'Clean' | 'Tainted' | 'Rejected';
export type VerdictLabel = 'Admissible' | 'Inadmissible';

export interface FileStatus {
    name: string;
    status: FileStatusLabel;
    timestamp: string;
}

export interface CellVerdict {
    cell_id: string;
    value: string | null;
    verdict: VerdictLabel;
    reason?: string;
}

export interface EvidenceData {
    image_base64: string;
    metadata: string;
}

export interface DiscrepancySummary {
    consistent: number;
    inconsistent: number;
    requires_review: number;
}

interface TruthState {
    // Data
    files: FileStatus[];
    activeFile: string | null;
    cells: CellVerdict[];
    selectedCellId: string | null;
    evidenceCache: Record<string, EvidenceData>;
    summary: DiscrepancySummary | null;

    // Loading states
    isFilesLoading: boolean;
    isTableLoading: boolean;
    isEvidenceLoading: boolean;

    // Actions
    fetchFiles: () => Promise<void>;
    selectFile: (fileName: string) => Promise<void>;
    selectCell: (cellId: string) => Promise<void>;
    refreshSummary: () => Promise<void>;
}

export const useTruthStore = create<TruthState>((set, get) => ({
    files: [],
    activeFile: null,
    cells: [],
    selectedCellId: null,
    evidenceCache: {},
    summary: null,

    isFilesLoading: false,
    isTableLoading: false,
    isEvidenceLoading: false,

    fetchFiles: async () => {
        set({ isFilesLoading: true });
        try {
            const files = await invoke<FileStatus[]>('get_file_ledger');
            set({ files, isFilesLoading: false });
        } catch (err) {
            console.error('Failed to fetch file ledger:', err);
            set({ isFilesLoading: false });
        }
    },

    selectFile: async (fileName: string) => {
        set({ activeFile: fileName, isTableLoading: true, cells: [], selectedCellId: null });
        try {
            const cells = await invoke<CellVerdict[]>('get_table_truth', { fileId: fileName });
            set({ cells, isTableLoading: false });
        } catch (err) {
            console.error('Failed to fetch table truth:', err);
            set({ isTableLoading: false });
        }
    },

    selectCell: async (cellId: string) => {
        set({ selectedCellId: cellId });

        // Zero-latency optimization: Check cache first
        const cache = get().evidenceCache;
        if (cache[cellId]) return;

        set({ isEvidenceLoading: true });
        try {
            const evidence = await invoke<EvidenceData>('get_evidence', { cellId: cellId });
            set((state) => ({
                evidenceCache: { ...state.evidenceCache, [cellId]: evidence },
                isEvidenceLoading: false
            }));
        } catch (err) {
            console.error('Failed to fetch evidence:', err);
            set({ isEvidenceLoading: false });
        }
    },

    refreshSummary: async () => {
        try {
            const summary = await invoke<DiscrepancySummary>('get_discrepancy');
            set({ summary });
        } catch (err) {
            console.error('Failed to fetch discrepancy summary:', err);
        }
    }
}));
