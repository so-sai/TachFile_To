// ui/src/lib/state.svelte.ts
// V1.0 App State Machine — CTO approved phases
// RULE: Zero business logic. This file manages transitions only.

import type { AppMode, AppPhase, DocumentSummary, IpcDiffReport, ProcessError, ProgressInfo } from './types';

export class AppState {
    // ─── Current mode ─────────────────────────────────────────────────────────
    mode = $state<AppMode>('clean');

    // ─── State machine phase ───────────────────────────────────────────────────
    phase = $state<AppPhase>('idle');

    // ─── Session file registry (in-memory only, not persisted) ────────────────
    processedFiles = $state<DocumentSummary[]>([]);
    activeSummaryId = $state<string | null>(null);

    // ─── Progress (streaming) ─────────────────────────────────────────────────
    progress = $state<ProgressInfo>({ current: 0, total: 0, label: '' });

    // ─── Results ──────────────────────────────────────────────────────────────
    markdown = $state<string>('');
    diffReport = $state<IpcDiffReport | null>(null);

    // ─── Compare mode file slots ───────────────────────────────────────────────
    compareFileA = $state<DocumentSummary | null>(null);
    compareFileB = $state<DocumentSummary | null>(null);

    // ─── Error state ──────────────────────────────────────────────────────────
    error = $state<ProcessError | null>(null);

    // ─── Derived ──────────────────────────────────────────────────────────────
    activeSummary = $derived(
        this.processedFiles.find(f => f.id === this.activeSummaryId) ?? null
    );

    // ─── Transitions ──────────────────────────────────────────────────────────

    setMode(m: AppMode) {
        this.mode = m;
        this.reset();
    }

    startProcessing(label: string) {
        this.phase = 'processing';
        this.error = null;
        this.progress = { current: 0, total: 100, label };
    }

    updateProgress(current: number, total: number, label: string) {
        this.progress = { current, total, label };
    }

    finishProcessing(summary: DocumentSummary, md: string) {
        this.processedFiles.push(summary);
        this.activeSummaryId = summary.id;
        this.markdown = md;
        this.phase = 'ready';
        this.progress = { current: 100, total: 100, label: '' };
    }

    startExporting() {
        this.phase = 'exporting';
    }

    finishExporting() {
        this.phase = 'ready';
    }

    startComparing() {
        this.phase = 'comparing';
        this.diffReport = null;
    }

    finishComparing(report: IpcDiffReport) {
        this.diffReport = report;
        this.phase = 'ready';
    }

    setError(err: ProcessError) {
        this.error = err;
        this.phase = 'error';
    }

    setCompareSlot(slot: 'a' | 'b', summary: DocumentSummary) {
        if (slot === 'a') this.compareFileA = summary;
        else this.compareFileB = summary;
    }

    reset() {
        this.phase = 'idle';
        this.error = null;
        this.markdown = '';
        this.diffReport = null;
        this.compareFileA = null;
        this.compareFileB = null;
        this.progress = { current: 0, total: 0, label: '' };
    }
}

export const appState = new AppState();
