import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { sessionManager } from './sessionManager';

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
    row_idx: number;
    col_idx: number;
    source_text: string;
}

export interface EncodingCandidate {
    mode: 'Unicode' | 'Vni' | 'Tcvn3' | 'Auto';
    text: string;
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
    encodingCandidates: EncodingCandidate[];

    // Forensic Error Tracking (Mission 028)
    lastError: {
        message: string;
        code?: string;
        source: 'IPC' | 'UI';
        timestamp: string;
    } | null;

    // Loading states
    isFilesLoading: boolean;
    isTableLoading: boolean;
    isEvidenceLoading: boolean;

    // Actions
    fetchFiles: () => Promise<void>;
    selectFile: (fileName: string) => Promise<void>;
    selectCell: (cellId: string) => Promise<void>;
    refreshSummary: () => Promise<void>;
    fetchEncodingCandidates: (text: string) => Promise<void>;
    applyEncodingRepair: (rowIdx: number, colIdx: number, original: string, selected: EncodingCandidate) => Promise<void>;
    clearError: () => void;
}

// 🛡️ MISSION 028: Load Session on Initialization
const initialSession = sessionManager.load();

export const useTruthStore = create<TruthState>((set, get) => ({
    files: [],
    activeFile: initialSession?.activeFile || null,
    cells: [],
    selectedCellId: initialSession?.selectedCellId || null,
    evidenceCache: {},
    summary: null,
    encodingCandidates: [],
    lastError: null,

    isFilesLoading: false,
    isTableLoading: false,
    isEvidenceLoading: false,

    clearError: () => set({ lastError: null }),

    fetchFiles: async () => {
        set({ isFilesLoading: true, lastError: null });
        try {
            const files = await invoke<FileStatus[]>('get_file_ledger');
            set({ files, isFilesLoading: false });

            // If we have an activeFile from session, trigger its load
            const currentActive = get().activeFile;
            if (currentActive && files.some(f => f.name === currentActive)) {
                get().selectFile(currentActive);
            }
        } catch (err) {
            set({
                isFilesLoading: false,
                lastError: {
                    message: `IPC_ERROR: get_file_ledger - ${err}`,
                    source: 'IPC',
                    timestamp: new Date().toISOString()
                }
            });
        }
    },

    selectFile: async (fileName: string) => {
        set({ activeFile: fileName, isTableLoading: true, cells: [], selectedCellId: null, lastError: null });

        // 💾 PERSISTENCE_SYNC
        sessionManager.save({ activeFile: fileName, selectedCellId: null });

        try {
            const cells = await invoke<CellVerdict[]>('get_table_truth', { fileId: fileName });
            set({ cells, isTableLoading: false });

            // If we have a selectedCellId from session, trigger its load
            const currentSelected = initialSession?.selectedCellId;
            if (currentSelected && cells.some(c => c.cell_id === currentSelected)) {
                get().selectCell(currentSelected);
            }
        } catch (err) {
            set({
                isTableLoading: false,
                lastError: {
                    message: `IPC_ERROR: get_table_truth - ${err}`,
                    source: 'IPC',
                    timestamp: new Date().toISOString()
                }
            });
        }
    },

    selectCell: async (cellId: string) => {
        set({ selectedCellId: cellId, lastError: null });

        // 💾 PERSISTENCE_SYNC
        sessionManager.save({ selectedCellId: cellId });

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
            set({
                isEvidenceLoading: false,
                lastError: {
                    message: `IPC_ERROR: get_evidence - ${err}`,
                    source: 'IPC',
                    timestamp: new Date().toISOString()
                }
            });
        }
    },

    refreshSummary: async () => {
        try {
            const summary = await invoke<DiscrepancySummary>('get_discrepancy');
            set({ summary });
        } catch (err) {
            console.error('Failed to fetch discrepancy summary:', err);
        }
    },

    fetchEncodingCandidates: async (text: string) => {
        set({ encodingCandidates: [] });
        try {
            const candidates = await invoke<EncodingCandidate[]>('get_encoding_candidates', { text });
            set({ encodingCandidates: candidates });
        } catch (err) {
            console.error('Failed to fetch encoding candidates:', err);
        }
    },

    applyEncodingRepair: async (rowIdx: number, colIdx: number, original: string, selected: EncodingCandidate) => {
        const { activeFile } = get();
        if (!activeFile) return;

        const repair = {
            row_idx: rowIdx,
            col_idx: colIdx,
            old_value: { Text: original },
            new_value: { Text: selected.text },
            reason: `Human selected ${selected.mode} interpretation`
        };

        try {
            // Wait, we need the original TableTruth to apply repairs?
            // Actually, apply_table_repairs should probably be more flexible or we need a way to get the current table.
            // For now, let's assume we can invoke a command to apply this repair.

            // Re-fetch the table after repair to update UI
            await invoke('apply_table_repairs_to_active', { repair });
            get().selectFile(activeFile);
        } catch (err) {
            set({
                lastError: {
                    message: `REPAIR_ERROR: ${err}`,
                    source: 'IPC',
                    timestamp: new Date().toISOString()
                }
            });
        }
    }
}));
