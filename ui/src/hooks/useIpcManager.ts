import { invoke } from "@tauri-apps/api/core";

/**
 * IPC Manager Hook - Bridge between React UI and Rust Core
 * Provides type-safe wrappers for all Tauri commands
 */

export interface EvidenceRequest {
    filePath: string;
    page: number;
    bbox: [number, number, number, number]; // [x, y, width, height]
    dpi?: number;
    outputFormat?: "jpeg" | "png";
    quality?: number;
}

export interface EvidenceResponse {
    imageBase64: string;
    dimensions: [number, number];
    format: string;
    metadata: {
        extractionTimeMs: number;
        cacheStatus: "memory" | "disk" | "miss";
    };
}

export interface HandshakeResponse {
    workerPid: number;
    doclingVersion: string;
    pythonVersion: string;
    capabilitiesSupported: string[];
    maxMemoryMb: number;
    status: string;
}

export const useIpcManager = () => {
    /**
     * Convert legacy Vietnamese text (TCVN3/VNI) to Unicode
     */
    const convertLegacyText = async (text: string): Promise<string> => {
        try {
            return await invoke<string>("convert_legacy_text", { text });
        } catch (error) {
            console.error("IPC Error (convert_legacy_text):", error);
            throw error;
        }
    };

    /**
     * Fetch evidence image from PDF
     */
    const fetchEvidence = async (
        request: EvidenceRequest
    ): Promise<EvidenceResponse> => {
        try {
            return await invoke<EvidenceResponse>("get_evidence", {
                filePath: request.filePath,
                page: request.page,
                bbox: request.bbox,
                dpi: request.dpi || 150,
                outputFormat: request.outputFormat || "jpeg",
                quality: request.quality || 85,
            });
        } catch (error) {
            console.error("IPC Error (get_evidence):", error);
            throw error;
        }
    };

    /**
     * Perform handshake with Python worker
     */
    const handshake = async (): Promise<HandshakeResponse> => {
        try {
            return await invoke<HandshakeResponse>("handshake");
        } catch (error) {
            console.error("IPC Error (handshake):", error);
            throw error;
        }
    };

    return {
        convertLegacyText,
        fetchEvidence,
        handshake,
    };
};
