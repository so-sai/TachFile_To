// ui/src/lib/types.ts
// Canonical TypeScript types — must mirror Rust IPC structs exactly.
// Source of truth: SYSTEM_ARCHITECTURE.md Section 3.5

export type ProcessError =
    | 'FileTooLarge'
    | 'OcrFailed'
    | 'UnsupportedFormat'
    | 'UserCancelled'
    | 'IoError'
    | 'EnginePanic';

export interface DocumentSummary {
    id: string;
    sourcePath: string;
    totalPages: number;
    hasOcr: boolean;
}

export type IpcDeltaKind = 'Added' | 'Removed' | 'Modified';

export interface IpcDelta {
    nodeId: string;
    kind: IpcDeltaKind;
    location: string;
    oldValue?: string;
    newValue?: string;
}

export interface IpcDiffReport {
    isIdentical: boolean;
    totalDeltas: number;
    deltas: IpcDelta[];
}

// App state machine phases — CTO approved phase set
export type AppPhase =
    | 'idle'
    | 'processing'
    | 'ready'
    | 'exporting'
    | 'comparing'
    | 'error';

export type AppMode = 'clean' | 'compare';

export interface ProgressInfo {
    current: number;
    total: number;
    label: string;
}
