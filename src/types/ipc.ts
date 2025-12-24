/**
 * IPC Protocol Types for TachFileTo Frontend
 * 
 * This file defines TypeScript types that match the Rust/Python protocol.
 * Any changes here must be reflected in:
 * - src-tauri/src/ipc/protocol.rs (Rust)
 * - backend/app/protocol.py (Python)
 */

// ==================== ENUMS ====================

export enum Priority {
    Immediate = "immediate",   // User clicked, needs <500ms
    Normal = "normal",         // User hovered, needs <2s
    Background = "background"  // Prefetch, can wait
}

export enum ErrorType {
    FileNotFound = "file_not_found",
    PageOutOfRange = "page_out_of_range",
    MemoryExhausted = "memory_exhausted",
    TimeoutExceeded = "timeout_exceeded",
    ParsingFailed = "parsing_failed",
    WorkerUnavailable = "worker_unavailable"
}

// ==================== REQUEST TYPES ====================

export interface EvidenceRequest {
    requestId: string;
    filePath: string;
    pageIndex: number;
    bbox: [number, number, number, number]; // [x, y, width, height]
    dpi: number;
    priority: Priority;
}

// ==================== RESPONSE TYPES ====================

export interface EvidenceSuccess {
    status: "success";
    requestId: string;
    dataBase64: string;
    mimeType: string;
    dimensions: [number, number]; // [width, height]
    isCacheHit: boolean;
}

export interface EvidencePending {
    status: "pending";
    requestId: string;
    queuePosition: number;
    estimatedWaitMs: number;
}

export interface EvidenceError {
    status: "failed";
    requestId: string;
    error: {
        type: ErrorType;
        message: string;
    };
    retryAfterMs?: number;
}

export type EvidenceResponse = EvidenceSuccess | EvidencePending | EvidenceError;

// ==================== HEALTH CHECK ====================

export interface HealthReport {
    status: "healthy" | "degraded" | "critical";
    metrics: {
        totalRequests: number;
        cacheHitRate: number;
        avgResponseTimeMs: number;
        memoryUsageMb: number;
        queueDepth: number;
        errorRate: number;
    };
    recommendations: string[];
}

// ==================== CACHE TYPES ====================

export interface CacheStats {
    layer1: {
        itemCount: number;
        sizeMb: number;
        hitRate: number;
    };
    layer2: {
        itemCount: number;
        sizeMb: number;
        hitRate: number;
    };
    layer3: {
        itemCount: number;
        sizeMb: number;
        oldestAccessDays: number;
    };
}

// ==================== TAURI INVOKE HELPERS ====================

import { invoke } from "@tauri-apps/api/tauri";

/**
 * Request evidence extraction from a PDF
 */
export async function extractEvidence(request: EvidenceRequest): Promise<EvidenceResponse> {
    return await invoke<EvidenceResponse>("extract_evidence", { request });
}

/**
 * Get current health status of the evidence system
 */
export async function getEvidenceHealth(): Promise<HealthReport> {
    return await invoke<HealthReport>("get_evidence_health");
}

/**
 * Clear all evidence caches
 */
export async function clearEvidenceCache(): Promise<void> {
    return await invoke("clear_evidence_cache");
}

/**
 * Force restart the Python worker
 */
export async function restartEvidenceWorker(): Promise<boolean> {
    return await invoke<boolean>("restart_evidence_worker");
}

// ==================== UTILITY FUNCTIONS ====================

/**
 * Generate a unique request ID
 */
export function generateRequestId(): string {
    return crypto.randomUUID();
}

/**
 * Check if response is successful
 */
export function isSuccess(response: EvidenceResponse): response is EvidenceSuccess {
    return response.status === "success";
}

/**
 * Check if response is pending
 */
export function isPending(response: EvidenceResponse): response is EvidencePending {
    return response.status === "pending";
}

/**
 * Check if response is an error
 */
export function isError(response: EvidenceResponse): response is EvidenceError {
    return response.status === "failed";
}
